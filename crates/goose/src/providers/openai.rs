use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use super::base::{ConfigKey, Provider, ProviderMetadata, ProviderUsage, Usage};
use super::embedding::{EmbeddingCapable, EmbeddingRequest, EmbeddingResponse};
use super::errors::ProviderError;
use super::formats::openai::{create_request, get_usage, response_to_message};
use super::utils::{emit_debug_trace, get_model, handle_response_openai_compat, ImageFormat};
use crate::message::Message;
use crate::model::ModelConfig;
use mcp_core::tool::Tool;

pub const OPEN_AI_DEFAULT_MODEL: &str = "gpt-4o";
pub const OPEN_AI_KNOWN_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "gpt-3.5-turbo",
    "o1",
    "o3",
    "o4-mini",
];

pub const OPEN_AI_DOC_URL: &str = "https://platform.openai.com/docs/models";

#[derive(Debug, serde::Serialize)]
pub struct OpenAiProvider {
    #[serde(skip)]
    client: Client,
    host: String,
    base_path: String,
    api_key: String,
    organization: Option<String>,
    project: Option<String>,
    model: ModelConfig,
    custom_headers: Option<HashMap<String, String>>,
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        let model = ModelConfig::new(OpenAiProvider::metadata().default_model);
        OpenAiProvider::from_env(model).expect("Failed to initialize OpenAI provider")
    }
}

impl OpenAiProvider {
    pub fn from_env(model: ModelConfig) -> Result<Self> {
        let config = crate::config::Config::global();
        let api_key: String = config.get_secret("OPENAI_API_KEY")?;
        let host: String = config
            .get_param("OPENAI_HOST")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());
        let base_path: String = config
            .get_param("OPENAI_BASE_PATH")
            .unwrap_or_else(|_| "v1/chat/completions".to_string());
        let organization: Option<String> = config.get_param("OPENAI_ORGANIZATION").ok();
        let project: Option<String> = config.get_param("OPENAI_PROJECT").ok();
        let custom_headers: Option<HashMap<String, String>> = config
            .get_secret("OPENAI_CUSTOM_HEADERS")
            .or_else(|_| config.get_param("OPENAI_CUSTOM_HEADERS"))
            .ok()
            .map(parse_custom_headers);
        let timeout_secs: u64 = config.get_param("OPENAI_TIMEOUT").unwrap_or(600);
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            host,
            base_path,
            api_key,
            organization,
            project,
            model,
            custom_headers,
        })
    }

    /// Helper function to add OpenAI-specific headers to a request
    fn add_headers(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // Add organization header if present
        if let Some(org) = &self.organization {
            request = request.header("OpenAI-Organization", org);
        }

        // Add project header if present
        if let Some(project) = &self.project {
            request = request.header("OpenAI-Project", project);
        }

        // Add custom headers if present
        if let Some(custom_headers) = &self.custom_headers {
            for (key, value) in custom_headers {
                request = request.header(key, value);
            }
        }

        request
    }

    async fn post(&self, payload: &Value) -> Result<Value, ProviderError> {
        let base_url = url::Url::parse(&self.host)
            .map_err(|e| ProviderError::RequestFailed(format!("Invalid base URL: {e}")))?;
        let url = base_url.join(&self.base_path).map_err(|e| {
            ProviderError::RequestFailed(format!("Failed to construct endpoint URL: {e}"))
        })?;

        let request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key));

        let request = self.add_headers(request);

        let response = request.json(&payload).send().await?;

        handle_response_openai_compat(response).await
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            "openai",
            "OpenAI",
            "GPT-4 and other OpenAI models, including OpenAI compatible ones",
            OPEN_AI_DEFAULT_MODEL,
            OPEN_AI_KNOWN_MODELS.to_vec(),
            OPEN_AI_DOC_URL,
            vec![
                ConfigKey::new("OPENAI_API_KEY", true, true, None),
                ConfigKey::new("OPENAI_HOST", true, false, Some("https://api.openai.com")),
                ConfigKey::new("OPENAI_BASE_PATH", true, false, Some("v1/chat/completions")),
                ConfigKey::new("OPENAI_ORGANIZATION", false, false, None),
                ConfigKey::new("OPENAI_PROJECT", false, false, None),
                ConfigKey::new("OPENAI_CUSTOM_HEADERS", false, true, None),
                ConfigKey::new("OPENAI_TIMEOUT", false, false, Some("600")),
            ],
        )
    }

    fn get_model_config(&self) -> ModelConfig {
        self.model.clone()
    }

    #[tracing::instrument(
        skip(self, system, messages, tools),
        fields(model_config, input, output, input_tokens, output_tokens, total_tokens)
    )]
    async fn complete(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<(Message, ProviderUsage), ProviderError> {
        let payload = create_request(&self.model, system, messages, tools, &ImageFormat::OpenAi)?;

        // Make request
        let response = self.post(&payload).await?;

        // Parse response
        let message = response_to_message(&response)?;
        let usage = match get_usage(&response) {
            Ok(usage) => usage,
            Err(ProviderError::UsageError(e)) => {
                tracing::debug!("Failed to get usage data: {}", e);
                Usage::default()
            }
            Err(e) => return Err(e),
        };
        let model = get_model(&response);
        emit_debug_trace(&self.model, &payload, &response, &usage);
        Ok((message, ProviderUsage::new(model, usage)))
    }

    /// Fetch supported models from OpenAI; returns Err on any failure, Ok(None) if no data
    async fn fetch_supported_models_async(&self) -> Result<Option<Vec<String>>, ProviderError> {
        // List available models via OpenAI API
        let base_url =
            url::Url::parse(&self.host).map_err(|e| ProviderError::RequestFailed(e.to_string()))?;
        let url = base_url
            .join("v1/models")
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;
        let mut request = self.client.get(url).bearer_auth(&self.api_key);
        if let Some(org) = &self.organization {
            request = request.header("OpenAI-Organization", org);
        }
        if let Some(project) = &self.project {
            request = request.header("OpenAI-Project", project);
        }
        if let Some(headers) = &self.custom_headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }
        let response = request.send().await?;
        let json: serde_json::Value = response.json().await?;
        if let Some(err_obj) = json.get("error") {
            let msg = err_obj
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            return Err(ProviderError::Authentication(msg.to_string()));
        }
        let data = json.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
            ProviderError::UsageError("Missing data field in JSON response".into())
        })?;
        let mut models: Vec<String> = data
            .iter()
            .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(str::to_string))
            .collect();
        models.sort();
        Ok(Some(models))
    }

    fn supports_embeddings(&self) -> bool {
        true
    }

    async fn create_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, ProviderError> {
        EmbeddingCapable::create_embeddings(self, texts)
            .await
            .map_err(|e| ProviderError::ExecutionError(e.to_string()))
    }
}

fn parse_custom_headers(s: String) -> HashMap<String, String> {
    s.split(',')
        .filter_map(|header| {
            let mut parts = header.splitn(2, '=');
            let key = parts.next().map(|s| s.trim().to_string())?;
            let value = parts.next().map(|s| s.trim().to_string())?;
            Some((key, value))
        })
        .collect()
}

#[async_trait]
impl EmbeddingCapable for OpenAiProvider {
    async fn create_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Get embedding model from env var or use default
        let embedding_model = std::env::var("GOOSE_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string());

        let request = EmbeddingRequest {
            input: texts,
            model: embedding_model,
        };

        // Construct embeddings endpoint URL
        let base_url =
            url::Url::parse(&self.host).map_err(|e| anyhow::anyhow!("Invalid base URL: {e}"))?;
        let url = base_url
            .join("v1/embeddings")
            .map_err(|e| anyhow::anyhow!("Failed to construct embeddings URL: {e}"))?;

        let req = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request);

        let req = self.add_headers(req);

        let response = req
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send embedding request: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Embedding API error: {}", error_text));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse embedding response: {e}"))?;

        Ok(embedding_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }
}
