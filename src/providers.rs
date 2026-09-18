use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub task: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub provider: String,
    pub output: String,
    pub confidence: Option<f32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider is not configured")]
    NotConfigured,
    #[error("provider request failed: {0}")]
    Request(String),
}

pub trait ReasoningProvider {
    fn name(&self) -> &'static str;
}

pub struct DifyProvider {
    pub base_url: String,
    pub api_key: String,
    pub client: Client,
}
pub struct GeminiProvider {
    pub api_key: String,
    pub model: String,
    pub client: Client,
}
pub struct HuggingFaceProvider {
    pub api_key: String,
    pub model: String,
    pub client: Client,
}

impl ReasoningProvider for DifyProvider {
    fn name(&self) -> &'static str {
        "dify"
    }
}
impl ReasoningProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }
}
impl ReasoningProvider for HuggingFaceProvider {
    fn name(&self) -> &'static str {
        "huggingface"
    }
}

impl DifyProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            base_url: std::env::var("DIFY_API_URL").ok()?,
            api_key: std::env::var("DIFY_API_KEY").ok()?,
            client,
        })
    }
}
impl GeminiProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            api_key: std::env::var("GEMINI_API_KEY").ok()?,
            model: std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".into()),
            client,
        })
    }
}
impl HuggingFaceProvider {
    pub fn from_env(client: Client) -> Option<Self> {
        Some(Self {
            api_key: std::env::var("HUGGINGFACE_API_TOKEN").ok()?,
            model: std::env::var("HUGGINGFACE_MODEL")
                .unwrap_or_else(|_| "mistralai/Mistral-7B-Instruct-v0.3".into()),
            client,
        })
    }
}

// Network calls are intentionally not performed by the MVP API. The adapter boundary
// keeps provider credentials server-side and lets us add audited request/response
// recording, tenant budgets, retries, and redaction before enabling inference.
