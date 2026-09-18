use crate::providers::{configured_provider, ProviderRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command, sync::Arc};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub workspace: PathBuf,
    pub provider: String,
    pub max_file_bytes: u64,
    pub max_output_bytes: usize,
    pub allow_commands: bool,
}

impl AgentConfig {
    pub fn from_env(workspace: PathBuf) -> Self {
        Self {
            workspace,
            provider: std::env::var("OMEGACLAW_PROVIDER").unwrap_or_else(|_| "offline".into()),
            max_file_bytes: 1_000_000,
            max_output_bytes: 20_000,
            allow_commands: std::env::var("OMEGACLAW_ALLOW_COMMANDS")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    pub event: String,
    pub input_hash: String,
    pub output_hash: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    pub decision: String,
    pub severity: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub next_tools: Vec<String>,
    pub confidence: f32,
    pub requires_human: bool,
}

#[derive(Debug, Clone)]
pub struct OmegaClawAgent {
    pub config: AgentConfig,
    client: reqwest::Client,
    audit: Arc<tokio::sync::Mutex<Vec<AuditEvent>>>,
}

impl OmegaClawAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            audit: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    pub async fn analyze_path(
        &self,
        relative_path: &str,
        question: &str,
    ) -> anyhow::Result<AgentDecision> {
        let path = self.safe_path(relative_path)?;
        let metadata = fs::metadata(&path).await?;
        if metadata.len() > self.config.max_file_bytes {
            anyhow::bail!("input exceeds max_file_bytes")
        }
        let source = fs::read_to_string(&path).await?;
        let evidence = vec![format!(
            "read_file:{} bytes={} sha256={}",
            relative_path,
            source.len(),
            hash(&source)
        )];
        let input = format!(
            "Question: {question}\nFile: {relative_path}\nContent:\n{}",
            truncate(&source, self.config.max_output_bytes)
        );
        let decision = if self.config.provider == "offline" {
            self.offline_decision(&source, evidence.clone())
        } else {
            self.provider_decision(input.clone(), evidence.clone())
                .await?
        };
        self.record("analyze_path", &input, &decision).await;
        Ok(decision)
    }
    pub async fn inspect_repo(&self) -> anyhow::Result<Value> {
        let mut files = Vec::new();
        let mut stack = vec![self.config.workspace.clone()];
        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let p = entry.path();
                let name = p.file_name().and_then(|v| v.to_str()).unwrap_or_default();
                if name == ".git" || name == "target" || name == "node_modules" {
                    continue;
                }
                if p.is_dir() {
                    stack.push(p);
                } else if let Ok(m) = entry.metadata().await {
                    files.push(json!({"path": p.strip_prefix(&self.config.workspace).unwrap_or(&p), "bytes": m.len()}));
                }
            }
        }
        let result =
            json!({"workspace": self.config.workspace, "files": files, "count": files.len()});
        self.record("inspect_repo", &result, &result).await;
        Ok(result)
    }
    pub async fn run_safe_check(&self, check: &str) -> anyhow::Result<Value> {
        if !self.config.allow_commands {
            anyhow::bail!("command execution disabled; set OMEGACLAW_ALLOW_COMMANDS=true in a trusted workspace")
        }
        let (program, args): (&str, &[&str]) = match check {
            "rust-test" => ("cargo", &["test"]),
            "rust-format" => ("cargo", &["fmt", "--all", "--", "--check"]),
            "frontend-build" => ("npm", &["run", "build"]),
            _ => anyhow::bail!("unsupported check: {check}"),
        };
        let output = Command::new(program)
            .args(args)
            .current_dir(&self.config.workspace)
            .output()?;
        let stdout = truncate(
            &String::from_utf8_lossy(&output.stdout),
            self.config.max_output_bytes,
        );
        let stderr = truncate(
            &String::from_utf8_lossy(&output.stderr),
            self.config.max_output_bytes,
        );
        let result = json!({"check":check,"success":output.status.success(),"stdout":stdout,"stderr":stderr,"exit_code":output.status.code()});
        self.record("run_safe_check", &result, &result).await;
        Ok(result)
    }
    fn safe_path(&self, relative: &str) -> anyhow::Result<PathBuf> {
        let candidate = self.config.workspace.join(relative);
        let canonical_root = self.config.workspace.canonicalize()?;
        let canonical = candidate.canonicalize()?;
        if !canonical.starts_with(&canonical_root) {
            anyhow::bail!("path escapes workspace")
        }
        Ok(canonical)
    }
    async fn provider_decision(
        &self,
        input: String,
        _evidence: Vec<String>,
    ) -> anyhow::Result<AgentDecision> {
        let provider = configured_provider(self.client.clone(), &self.config.provider)?;
        let schema = json!({"type":"object","properties":{"decision":{"type":"string","enum":["abstain","investigate","escalate"]},"severity":{"type":"string","enum":["informational","low","medium","high","critical"]},"summary":{"type":"string"},"evidence":{"type":"array","items":{"type":"string"}},"assumptions":{"type":"array","items":{"type":"string"}},"next_tools":{"type":"array","items":{"type":"string"}},"confidence":{"type":"number","minimum":0,"maximum":1},"requires_human":{"type":"boolean"}},"required":["decision","severity","summary","evidence","assumptions","next_tools","confidence","requires_human"]});
        let response = provider.decide(ProviderRequest { system: "You are OmegaClaw, a defensive Web3 security developer agent. Never exploit live systems, never request keys, never invent evidence, and abstain when the file is insufficient. Separate code-pattern confidence from impact. Return only the requested JSON.".into(), input, response_schema: schema }).await?;
        Ok(serde_json::from_value(response.structured)?)
    }
    fn offline_decision(&self, source: &str, evidence: Vec<String>) -> AgentDecision {
        let keywords = ["delegatecall", "selfdestruct", "tx.origin", "ecrecover"];
        let hits: Vec<_> = keywords
            .iter()
            .filter(|k| source.contains(**k))
            .map(|k| format!("pattern:{k}"))
            .collect();
        let has_hits = !hits.is_empty();
        AgentDecision {
            decision: if has_hits { "escalate" } else { "investigate" }.into(),
            severity: if has_hits { "medium" } else { "informational" }.into(),
            summary: if has_hits {
                "High-signal code patterns require human validation and exploitability analysis."
            } else {
                "No configured high-signal patterns found; run protocol-specific checks."
            }
            .into(),
            evidence: [evidence, hits].concat(),
            assumptions: vec![
                "Offline mode does not establish exploitability or economic impact".into(),
            ],
            next_tools: vec!["rust-test".into(), "rust-format".into()],
            confidence: if has_hits { 0.55 } else { 0.2 },
            requires_human: true,
        }
    }
    async fn record<I: Serialize, O: Serialize>(&self, event: &str, input: &I, output: &O) {
        let input_json = serde_json::to_string(input).unwrap_or_default();
        let output_json = serde_json::to_string(output).unwrap_or_default();
        self.audit.lock().await.push(AuditEvent {
            id: Uuid::new_v4(),
            at: Utc::now(),
            event: event.into(),
            input_hash: hash(&input_json),
            output_hash: hash(&output_json),
            metadata: json!({"provider":self.config.provider,"workspace":self.config.workspace}),
        });
    }
    pub async fn audit_events(&self) -> Vec<AuditEvent> {
        self.audit.lock().await.clone()
    }
}
fn hash(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    hex::encode(h.finalize())
}
fn truncate(input: &str, max: usize) -> String {
    input.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn offline_agent_is_human_gated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.sol");
        fs::write(
            &path,
            "contract X { function f() external { selfdestruct(payable(msg.sender)); } }",
        )
        .await
        .unwrap();
        let agent = OmegaClawAgent::new(AgentConfig::from_env(dir.path().to_path_buf()));
        let d = agent
            .analyze_path("x.sol", "review defensively")
            .await
            .unwrap();
        assert_eq!(d.decision, "escalate");
        assert!(d.requires_human);
        assert_eq!(agent.audit_events().await.len(), 1);
    }
}
