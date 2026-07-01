use serde::{Deserialize, Serialize};

use crate::creds::api_key::RunninghubApiKey;
use crate::error::RunninghubError;
use crate::polling::poll_task::poll_task;

const BASE_URL: &str = "https://www.runninghub.ai";
const ENDPOINT: &str = "/openapi/v2/rhart-image-n-g31-flash/image-to-image";

#[derive(Debug, Clone)]
pub struct NanaBanana2ImageToImageRequest {
  pub prompt: String,
  pub image_urls: Vec<String>,
  pub resolution: Option<String>,
  pub aspect_ratio: Option<String>,
}

#[derive(Debug, Serialize)]
struct RawRequest {
  prompt: String,
  image_urls: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  resolution: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  aspect_ratio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EnqueueResponse {
  #[serde(rename = "taskId")]
  task_id: Option<String>,
  message: Option<String>,
}

impl NanaBanana2ImageToImageRequest {
  pub async fn send(&self, api_key: &RunninghubApiKey) -> Result<String, RunninghubError> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", BASE_URL, ENDPOINT);

    let raw = RawRequest {
      prompt: self.prompt.clone(),
      image_urls: self.image_urls.clone(),
      resolution: self.resolution.clone(),
      aspect_ratio: self.aspect_ratio.clone(),
    };

    let enqueue_response: EnqueueResponse = client
      .post(&url)
      .header("Authorization", format!("Bearer {}", api_key.as_str()))
      .header("Content-Type", "application/json")
      .json(&raw)
      .send()
      .await?
      .json()
      .await?;

    let task_id = enqueue_response.task_id.ok_or_else(|| {
      RunninghubError::ApiError {
        message: enqueue_response.message.unwrap_or_else(|| "no task_id returned".to_string()),
      }
    })?;

    log::info!("RunningHub NanaBanana2 image-to-image task enqueued: {}", task_id);
    poll_task(api_key, &task_id).await
  }
}
