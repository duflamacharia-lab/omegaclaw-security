use crate::providers::{configured_provider, ProviderRequest};
use crate::tool_runner::{ToolRequest, ToolRunner, ToolRunnerConfig, WorkerMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs as stdfs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub workspace: PathBuf,
    pub provider: String,
    pub knowledge_root: PathBuf,
    pub max_file_bytes: u64,
    pub max_output_bytes: usize,
    pub allow_commands: bool,
}

impl AgentConfig {
    pub fn from_env(workspace: PathBuf) -> Self {
        Self {
            knowledge_root: std::env::var("OMEGACLAW_KNOWLEDGE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| workspace.join("data/kazamadono")),
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
        self.validate_knowledge_prerequisites()?;
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
        self.validate_knowledge_prerequisites()?;
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
        self.validate_knowledge_prerequisites()?;
        let tool = match check {
            "rust-test" => "cargo-test",
            "rust-format" => "cargo-format",
            "frontend-build" => "frontend-build",
            other => anyhow::bail!("unsupported check: {other}"),
        };
        let runner = ToolRunner::new(ToolRunnerConfig {
            workspace: self.config.workspace.clone(),
            max_output_bytes: self.config.max_output_bytes,
            timeout_seconds: 300,
            allow_commands: self.config.allow_commands,
            mode: if std::env::var("OMEGACLAW_WORKER_MODE")
                .map(|v| v.eq_ignore_ascii_case("docker"))
                .unwrap_or(false)
            {
                WorkerMode::Docker
            } else {
                WorkerMode::Native
            },
            docker_network: std::env::var("OMEGACLAW_DOCKER_NETWORK")
                .unwrap_or_else(|_| "none".into()),
        });
        let result = serde_json::to_value(
            runner
                .run(ToolRequest {
                    tool: tool.into(),
                    args: vec![],
                })
                .await?,
        )?;
        self.record("run_safe_check", &result, &result).await;
        Ok(result)
    }

    pub fn validate_knowledge_prerequisites(&self) -> anyhow::Result<Value> {
        let root = &self.config.knowledge_root;
        let catalog = read_json(root.join("catalog.manifest.json"))?;
        let candidates = read_json(root.join("defensive-candidates.manifest.json"))?;
        let videos = read_json(root.join("video-links.manifest.json"))?;
        let transcripts = read_json(root.join("transcripts/transcripts.manifest.json"))?;
        let catalog_hash = catalog
            .get("catalog_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("catalog manifest is missing catalog_sha256"))?;
        if catalog_hash.len() != 64 || !catalog_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("catalog_sha256 must be a 64-character hexadecimal digest")
        }
        for (name, manifest) in [
            ("defensive candidates", &candidates),
            ("video links", &videos),
        ] {
            if manifest
                .get("source_catalog_sha256")
                .and_then(Value::as_str)
                != Some(catalog_hash)
            {
                anyhow::bail!("{name} manifest does not match catalog hash")
            }
        }
        let resources = catalog
            .get("resources")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("catalog resources are missing"))?;
        if resources.is_empty() {
            anyhow::bail!("catalog contains no resources")
        }
        let transcript_records = transcripts
            .get("records")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("transcript records are missing"))?;
        let mut retrieved = 0usize;
        for record in transcript_records {
            if record.get("status").and_then(Value::as_str) == Some("transcript_retrieved") {
                retrieved += 1;
                let relative = record
                    .get("transcript")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("retrieved transcript has no path"))?;
                let transcript_path = self.config.workspace.join(relative);
                if !transcript_path.starts_with(&self.config.workspace)
                    || !transcript_path.is_file()
                {
                    anyhow::bail!(
                        "retrieved transcript is missing or escapes workspace: {relative}"
                    )
                }
                let expected = record
                    .get("transcript_sha256")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("retrieved transcript has no hash"))?;
                let actual = crate::manifest::sha256_file(&transcript_path)?;
                if expected != actual {
                    anyhow::bail!("transcript hash mismatch: {relative}")
                }
                if record.get("content_status").and_then(Value::as_str) == Some("admitted_evidence")
                {
                    anyhow::bail!("transcripts require review before admission")
                }
            }
        }
        Ok(json!({
            "ok": true,
            "catalog_sha256": catalog_hash,
            "resources": resources.len(),
            "defensive_candidates": candidates.get("resources").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "video_links": videos.get("resources").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "retrieved_transcripts": retrieved,
            "admission": "metadata_and_quarantine_only"
        }))
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

fn read_json(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let bytes = stdfs::read(path)
        .map_err(|error| anyhow::anyhow!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| anyhow::anyhow!("invalid JSON {}: {error}", path.display()))
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
        create_knowledge_fixture(dir.path());
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

    fn create_knowledge_fixture(root: &Path) {
        let knowledge = root.join("data/kazamadono/transcripts");
        stdfs::create_dir_all(&knowledge).unwrap();
        stdfs::write(
            knowledge.join("sample.txt"),
            "reviewable fixture transcript\n",
        )
        .unwrap();
        let hash = crate::manifest::sha256_file(knowledge.join("sample.txt")).unwrap();
        let catalog_hash = "a".repeat(64);
        stdfs::write(
            root.join("data/kazamadono/catalog.manifest.json"),
            json!({"catalog_sha256":catalog_hash,"resources":[{"id":"1"}]}).to_string(),
        )
        .unwrap();
        stdfs::write(
            root.join("data/kazamadono/defensive-candidates.manifest.json"),
            json!({"source_catalog_sha256":catalog_hash,"resources":[]}).to_string(),
        )
        .unwrap();
        stdfs::write(
            root.join("data/kazamadono/video-links.manifest.json"),
            json!({"source_catalog_sha256":catalog_hash,"resources":[]}).to_string(),
        )
        .unwrap();
        stdfs::write(
            root.join("data/kazamadono/transcripts/transcripts.manifest.json"),
            json!({"records":[{"status":"transcript_retrieved","transcript":"data/kazamadono/transcripts/sample.txt","transcript_sha256":hash,"content_status":"quarantine_review"}]}).to_string(),
        )
        .unwrap();
    }
}
