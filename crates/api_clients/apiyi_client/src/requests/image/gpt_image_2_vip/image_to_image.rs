use serde::Deserialize;

use crate::creds::api_key::ApiyiApiKey;
use crate::error::ApiyiError;

const ENDPOINT: &str = "https://api.apiyi.com/v1/images/edits";

#[derive(Debug, Clone)]
pub struct GptImage2VipImageToImageRequest {
  pub prompt: String,
  pub image_bytes: Vec<u8>,
  pub image_filename: String,
  pub size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageData {
  b64_json: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawResponse {
  data: Option<Vec<ImageData>>,
  error: Option<serde_json::Value>,
}

impl GptImage2VipImageToImageRequest {
  pub async fn send(&self, api_key: &ApiyiApiKey) -> Result<String, ApiyiError> {
    let client = reqwest::Client::new();

    let image_part = reqwest::multipart::Part::bytes(self.image_bytes.clone())
      .file_name(self.image_filename.clone())
      .mime_str("image/png")
      .unwrap();

    let mut form = reqwest::multipart::Form::new()
      .text("model", "gpt-image-2-vip")
      .text("prompt", self.prompt.clone())
      .part("image", image_part);

    if let Some(size) = &self.size {
      form = form.text("size", size.clone());
    }

    let response = client
      .post(ENDPOINT)
      .header("Authorization", format!("Bearer {}", api_key.as_str()))
      .multipart(form)
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
