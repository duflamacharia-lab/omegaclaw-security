use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{fs, process::Command, time::timeout};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerMode {
    Native,
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRunnerConfig {
    pub workspace: PathBuf,
    pub max_output_bytes: usize,
    pub timeout_seconds: u64,
    pub allow_commands: bool,
    pub mode: WorkerMode,
    pub docker_network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub args: Vec<String>,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub input_sha256: String,
    pub execution_mode: WorkerMode,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("command execution is disabled")]
    Disabled,
    #[error("unsupported tool: {0}")]
    Unsupported(String),
    #[error("tool arguments are not permitted")]
    InvalidArguments,
    #[error("workspace error: {0}")]
    Workspace(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct ToolRunner {
    config: ToolRunnerConfig,
}

impl ToolRunner {
    pub fn new(config: ToolRunnerConfig) -> Self {
        Self { config }
    }
    pub async fn run(&self, request: ToolRequest) -> Result<ToolResult, ToolError> {
        if !self.config.allow_commands {
            return Err(ToolError::Disabled);
        }
        let (program, base_args, docker_image) = match request.tool.as_str() {
            "cargo-test" => ("cargo", vec!["test".into(), "--all-targets".into()], None),
            "cargo-format" => (
                "cargo",
                vec!["fmt".into(), "--all".into(), "--".into(), "--check".into()],
                None,
            ),
            "frontend-build" => ("npm", vec!["run".into(), "build".into()], None),
            "slither" => (
                "slither",
                Vec::new(),
                Some("trailofbits/eth-security-toolbox"),
            ),
            "forge-test" => (
                "forge",
                vec!["test".into()],
                Some("ghcr.io/foundry-rs/foundry:latest"),
            ),
            "echidna" => (
                "echidna",
                Vec::new(),
                Some("ghcr.io/crytic/echidna/echidna:latest"),
            ),
            other => return Err(ToolError::Unsupported(other.into())),
        };
        if request.args.iter().any(|arg| {
            arg.starts_with('-')
                || arg.contains(';')
                || arg.contains("&&")
                || arg.contains('|')
                || arg.contains('$')
        }) {
            return Err(ToolError::InvalidArguments);
        }
        let mut args = base_args;
        args.extend(request.args.clone());
        let input_sha256 = sha256(
            &serde_json::to_string(&json!({"tool":request.tool,"args":args})).unwrap_or_default(),
        );
        let (program, final_args, mode): (String, Vec<String>, WorkerMode) =
            match (&self.config.mode, docker_image) {
                (WorkerMode::Docker, Some(image)) => {
                    let mut docker_args = vec![
                        "run".into(),
                        "--rm".into(),
                        "--network".into(),
                        self.config.docker_network.clone(),
                        "--read-only".into(),
                        "--cap-drop=ALL".into(),
                        "--security-opt".into(),
                        "no-new-privileges".into(),
                        "-v".into(),
                        format!("{}:/src:ro", self.config.workspace.display()),
                        image.into(),
                        "bash".into(),
                        "-lc".into(),
                    ];
                    let command = format!("cd /src && {} {}", program, shell_join(&args));
                    docker_args.push(command);
                    ("docker".into(), docker_args, WorkerMode::Docker)
                }
                _ => (program.to_string(), args.clone(), WorkerMode::Native),
            };
        let child = Command::new(&program)
            .args(&final_args)
            .current_dir(&self.config.workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let result = match timeout(
            Duration::from_secs(self.config.timeout_seconds),
            child.wait_with_output(),
        )
        .await
        {
            Ok(output) => {
                let output = output?;
                ToolResult {
                    tool: request.tool,
                    args,
                    success: output.status.success(),
                    exit_code: output.status.code(),
                    stdout: truncate(
                        &String::from_utf8_lossy(&output.stdout),
                        self.config.max_output_bytes,
                    ),
                    stderr: truncate(
                        &String::from_utf8_lossy(&output.stderr),
                        self.config.max_output_bytes,
                    ),
                    timed_out: false,
                    input_sha256,
                    execution_mode: mode,
                }
            }
            Err(_) => ToolResult {
                tool: request.tool,
                args,
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: "tool execution timed out".into(),
                timed_out: true,
                input_sha256,
                execution_mode: mode,
            },
        };
        Ok(result)
    }
    pub async fn read_workspace_file(
        &self,
        relative_path: &str,
        max_bytes: u64,
    ) -> Result<serde_json::Value, ToolError> {
        let root = self.config.workspace.canonicalize()?;
        let candidate = self.config.workspace.join(relative_path);
        let path = candidate.canonicalize()?;
        if !path.starts_with(&root) {
            return Err(ToolError::InvalidArguments);
        }
        let metadata = fs::metadata(&path).await?;
        if metadata.len() > max_bytes {
            return Err(ToolError::InvalidArguments);
        }
        let content = fs::read_to_string(&path).await?;
        Ok(
            json!({"path":relative_path,"bytes":content.len(),"sha256":sha256(&content),"content":content}),
        )
    }
}
fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ")
}
fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}
fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config(allow: bool) -> ToolRunnerConfig {
        ToolRunnerConfig {
            workspace: std::env::current_dir().unwrap(),
            max_output_bytes: 1000,
            timeout_seconds: 1,
            allow_commands: allow,
            mode: WorkerMode::Native,
            docker_network: "none".into(),
        }
    }
    #[tokio::test]
    async fn disabled_runner_fails_closed() {
        assert!(matches!(
            ToolRunner::new(config(false))
                .run(ToolRequest {
                    tool: "cargo-test".into(),
                    args: vec![]
                })
                .await,
            Err(ToolError::Disabled)
        ));
    }
    #[tokio::test]
    async fn injection_is_rejected() {
        assert!(matches!(
            ToolRunner::new(config(true))
                .run(ToolRequest {
                    tool: "slither".into(),
                    args: vec!["; rm -rf /".into()]
                })
                .await,
            Err(ToolError::InvalidArguments)
        ));
    }
    #[test]
    fn shell_join_quotes() {
        assert_eq!(shell_join(&["x y".into()]), "'x y'");
    }
}
