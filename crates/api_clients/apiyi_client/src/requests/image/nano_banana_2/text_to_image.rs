use serde::{Deserialize, Serialize};

use crate::creds::api_key::ApiyiApiKey;
use crate::error::ApiyiError;

const ENDPOINT: &str =
  "https://api.apiyi.com/v1beta/models/gemini-3.1-flash-image-preview:generateContent";

#[derive(Debug, Clone)]
pub struct NanaBanana2TextToImageRequest {
  pub prompt: String,
  pub image_size: Option<String>,
  pub aspect_ratio: Option<String>,
}

#[derive(Debug, Serialize)]
struct ContentPart {
  text: String,
}

#[derive(Debug, Serialize)]
struct Content {
  parts: Vec<ContentPart>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
  response_modalities: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  image_size: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  aspect_ratio: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RawRequest {
  contents: Vec<Content>,
  generation_config: GenerationConfig,
}

#[derive(Debug, Deserialize)]
struct InlineData {
  data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePart {
  inline_data: Option<InlineData>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
  parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
  content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct RawResponse {
  candidates: Vec<Candidate>,
}

impl NanaBanana2TextToImageRequest {
  pub async fn send(&self, api_key: &ApiyiApiKey) -> Result<String, ApiyiError> {
    let client = reqwest::Client::new();

    let raw = RawRequest {
      contents: vec![Content {
        parts: vec![ContentPart { text: self.prompt.clone() }],
      }],
      generation_config: GenerationConfig {
        response_modalities: vec!["IMAGE".to_string()],
        image_size: self.image_size.clone(),
        aspect_ratio: self.aspect_ratio.clone(),
      },
    };

    let response = client
      .post(ENDPOINT)
      .header("Authorization", format!("Bearer {}", api_key.as_str()))
      .header("Content-Type", "application/json")
      .json(&raw)
      .send()
      .await?;

    let raw_response: RawResponse = response.json().await?;

    raw_response
      .candidates
      .into_iter()
      .find_map(|c| {
        c.content.parts.into_iter().find_map(|p| {
          p.inline_data.map(|d| d.data)
        })
      })
      .ok_or(ApiyiError::NoImageData)
  }
}
