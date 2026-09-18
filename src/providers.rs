use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub system: String,
    pub input: String,
    pub response_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub provider: String,
    pub model: String,
    pub text: String,
    pub structured: Value,
    pub request_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider is not configured")]
    NotConfigured,
    #[error("provider returned HTTP {status}: {body}")]
    Http { status: StatusCode, body: String },
    #[error("provider request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("provider returned invalid structured output: {0}")]
    InvalidOutput(String),
}

#[async_trait]
pub trait ReasoningProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> &str;
    async fn decide(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError>;
}

#[derive(Clone)]
pub struct GeminiProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: Client,
}
#[derive(Clone)]
pub struct DifyProvider {
    pub base_url: String,
    pub api_key: String,
    pub workflow_id: String,
    pub client: Client,
}
#[derive(Clone)]
pub struct HuggingFaceProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: Client,
}

impl GeminiProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            api_key: std::env::var("GEMINI_API_KEY").ok()?,
            model: std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.8-flash".into()),
            base_url: std::env::var("GEMINI_API_URL").unwrap_or_else(|_| {
                "https://generativelanguage.googleapis.com/v1beta/interactions".into()
            }),
            client,
        })
    }
}

#[async_trait]
impl ReasoningProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }
    fn model(&self) -> &str {
        &self.model
    }
    async fn decide(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let body = json!({
            "model": self.model,
            "system_instruction": request.system,
            "input": request.input,
            "response_format": { "type": "text", "mime_type": "application/json", "schema": request.response_schema }
        });
        let response = self
            .client
            .post(&self.base_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let raw: Value = response.json().await?;
        if !status.is_success() {
            return Err(ProviderError::Http {
                status,
                body: raw.to_string(),
            });
        }
        let text = raw
            .get("output_text")
            .and_then(Value::as_str)
            .or_else(|| raw.pointer("/outputs/0/text").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        if text.is_empty() {
            return Err(ProviderError::InvalidOutput("missing output_text".into()));
        }
        let structured = serde_json::from_str(&text)
            .map_err(|e| ProviderError::InvalidOutput(format!("{e}: {text}")))?;
        Ok(ProviderResponse {
            provider: self.name().into(),
            model: self.model.clone(),
            text,
            structured,
            request_id: raw.get("id").and_then(Value::as_str).map(str::to_string),
        })
    }
}

impl DifyProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            base_url: std::env::var("DIFY_API_URL").ok()?,
            api_key: std::env::var("DIFY_API_KEY").ok()?,
            workflow_id: std::env::var("DIFY_WORKFLOW_ID")
                .unwrap_or_else(|_| "omegaclaw-security-review".into()),
            client,
        })
    }
}

#[async_trait]
impl ReasoningProvider for DifyProvider {
    fn name(&self) -> &'static str {
        "dify"
    }
    fn model(&self) -> &str {
        &self.workflow_id
    }
    async fn decide(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let url = format!("{}/workflows/run", self.base_url.trim_end_matches('/'));
        let body = json!({ "inputs": { "system": request.system, "input": request.input, "response_schema": request.response_schema }, "response_mode": "blocking", "user": "omegaclaw-agent" });
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let raw: Value = response.json().await?;
        if !status.is_success() {
            return Err(ProviderError::Http {
                status,
                body: raw.to_string(),
            });
        }
        let output = raw.pointer("/data/outputs").cloned().unwrap_or(raw.clone());
        let text = output
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| output.to_string());
        let structured = serde_json::from_str(&text).unwrap_or(output);
        Ok(ProviderResponse {
            provider: self.name().into(),
            model: self.workflow_id.clone(),
            text,
            structured,
            request_id: raw
                .get("workflow_run_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

impl HuggingFaceProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            api_key: std::env::var("HUGGINGFACE_API_TOKEN").ok()?,
            model: std::env::var("HUGGINGFACE_MODEL")
                .unwrap_or_else(|_| "build-small-hackathon/OpenMythos".into()),
            base_url: std::env::var("HUGGINGFACE_API_URL")
                .unwrap_or_else(|_| "https://router.huggingface.co/v1/chat/completions".into()),
            client,
        })
    }
}

#[async_trait]
impl ReasoningProvider for HuggingFaceProvider {
    fn name(&self) -> &'static str {
        "huggingface"
    }
    fn model(&self) -> &str {
        &self.model
    }
    async fn decide(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        match self.decide_with_model(&self.model, &request).await {
            Ok(response) => Ok(response),
            Err(primary_error) => {
                let fallback = std::env::var("HUGGINGFACE_FALLBACK_MODEL")
                    .unwrap_or_else(|_| "Qwen/Qwen2.5-Coder-7B-Instruct".into());
                if fallback == self.model {
                    return Err(primary_error);
                }
                self.decide_with_model(&fallback, &request)
                    .await
                    .or(Err(primary_error))
            }
        }
    }
}

impl HuggingFaceProvider {
    async fn decide_with_model(
        &self,
        model: &str,
        request: &ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let body = json!({ "model": model, "messages": [{"role":"system","content":request.system},{"role":"user","content":request.input}], "response_format": {"type":"json_object"}, "temperature": 0 });
        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let raw: Value = response.json().await?;
        if !status.is_success() {
            return Err(ProviderError::Http {
                status,
                body: raw.to_string(),
            });
        }
        let text = raw
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let structured =
            serde_json::from_str(&text).map_err(|e| ProviderError::InvalidOutput(e.to_string()))?;
        Ok(ProviderResponse {
            provider: self.name().into(),
            model: model.into(),
            text,
            structured,
            request_id: raw.get("id").and_then(Value::as_str).map(str::to_string),
        })
    }
}

pub fn configured_provider(
    client: Client,
    name: &str,
) -> Result<Box<dyn ReasoningProvider>, ProviderError> {
    match name {
        "gemini" => GeminiProvider::from_env(client)
            .map(|p| Box::new(p) as Box<dyn ReasoningProvider>)
            .ok_or(ProviderError::NotConfigured),
        "dify" => DifyProvider::from_env(client)
            .map(|p| Box::new(p) as Box<dyn ReasoningProvider>)
            .ok_or(ProviderError::NotConfigured),
        "huggingface" => HuggingFaceProvider::from_env(client)
            .map(|p| Box::new(p) as Box<dyn ReasoningProvider>)
            .ok_or(ProviderError::NotConfigured),
        _ => Err(ProviderError::NotConfigured),
    }
}
