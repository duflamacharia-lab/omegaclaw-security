use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("MCP server is not configured")]
    NotConfigured,
    #[error("MCP server URL is not allowed")]
    ServerNotAllowed,
    #[error("MCP tool is not allowlisted: {0}")]
    ToolNotAllowed(String),
    #[error("MCP returned HTTP {0}")]
    Http(StatusCode),
    #[error("MCP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("MCP protocol error: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCall {
    pub tool: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Clone)]
pub struct McpClient {
    client: Client,
    url: String,
    token: Option<String>,
    allowed_tools: Vec<String>,
}

impl McpClient {
    pub fn from_env(client: Client) -> Result<Self, McpError> {
        let url = env::var("OMEGACLAW_MCP_URL").map_err(|_| McpError::NotConfigured)?;
        if !is_allowed_url(&url) {
            return Err(McpError::ServerNotAllowed);
        }
        let allowed_tools = env::var("OMEGACLAW_MCP_ALLOWED_TOOLS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        Ok(Self {
            client,
            url,
            token: env::var("OMEGACLAW_MCP_TOKEN").ok(),
            allowed_tools,
        })
    }

    pub fn configured() -> bool {
        env::var("OMEGACLAW_MCP_URL")
            .map(|u| is_allowed_url(&u))
            .unwrap_or(false)
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let raw = self.request("tools/list", json!({})).await?;
        let tools = raw
            .pointer("/result/tools")
            .cloned()
            .unwrap_or_else(|| json!([]));
        serde_json::from_value(tools).map_err(|e| McpError::Protocol(e.to_string()))
    }

    pub async fn call(&self, call: McpCall) -> Result<Value, McpError> {
        if !self.allowed_tools.iter().any(|name| name == &call.tool) {
            return Err(McpError::ToolNotAllowed(call.tool));
        }
        let raw = self
            .request(
                "tools/call",
                json!({"name": call.tool, "arguments": call.arguments}),
            )
            .await?;
        if let Some(error) = raw.get("error") {
            return Err(McpError::Protocol(error.to_string()));
        }
        Ok(raw.pointer("/result").cloned().unwrap_or(raw))
    }

    async fn request(&self, method: &str, params: Value) -> Result<Value, McpError> {
        let request = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
        let mut builder = self
            .client
            .post(&self.url)
            .timeout(Duration::from_secs(15))
            .json(&request);
        if let Some(token) = &self.token {
            builder = builder.bearer_auth(token);
        }
        let response = builder.send().await?;
        let status = response.status();
        let raw: Value = response.json().await?;
        if !status.is_success() {
            return Err(McpError::Http(status));
        }
        if raw.get("error").is_some() {
            return Err(McpError::Protocol(raw["error"].to_string()));
        }
        Ok(raw)
    }
}

fn is_allowed_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("https://")
        || lower.starts_with("http://127.0.0.1")
        || lower.starts_with("http://localhost"))
        && !lower.contains("@")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_embedded_credentials_and_unknown_schemes() {
        assert!(!is_allowed_url("ftp://host/mcp"));
        assert!(!is_allowed_url("https://user:pass@host/mcp"));
    }
    #[test]
    fn permits_tls_or_local_fixture() {
        assert!(is_allowed_url("https://mcp.example.test"));
        assert!(is_allowed_url("http://127.0.0.1:9000/mcp"));
    }
}
