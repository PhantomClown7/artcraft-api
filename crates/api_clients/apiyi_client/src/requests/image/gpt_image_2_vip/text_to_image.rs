use serde::{Deserialize, Serialize};

use crate::creds::api_key::ApiyiApiKey;
use crate::error::ApiyiError;

const ENDPOINT: &str = "https://api.apiyi.com/v1/images/generations";

#[derive(Debug, Clone)]
pub struct GptImage2VipTextToImageRequest {
  pub prompt: String,
  pub size: Option<String>,
}

#[derive(Debug, Serialize)]
struct RawRequest {
  model: String,
  prompt: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  size: Option<String>,
  response_format: String,
}

#[derive(Debug, Deserialize)]
struct ImageData {
  b64_json: Option<String>,
  url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawResponse {
  data: Option<Vec<ImageData>>,
  error: Option<serde_json::Value>,
}

impl GptImage2VipTextToImageRequest {
  pub async fn send(&self, api_key: &ApiyiApiKey) -> Result<String, ApiyiError> {
    let client = reqwest::Client::new();

    let raw = RawRequest {
      model: "gpt-image-2-vip".to_string(),
      prompt: self.prompt.clone(),
      size: self.size.clone(),
      response_format: "b64_json".to_string(),
    };

    let response = client
      .post(ENDPOINT)
      .header("Authorization", format!("Bearer {}", api_key.as_str()))
      .header("Content-Type", "application/json")
      .json(&raw)
      .send()
      .await?;

    let raw_response: RawResponse = response.json().await?;

    if let Some(err) = raw_response.error {
      return Err(ApiyiError::ApiError {
        message: err.to_string(),
      });
    }

    raw_response
      .data
      .and_then(|d| d.into_iter().next())
      .and_then(|item| item.b64_json)
      .ok_or(ApiyiError::NoImageData)
  }
}
