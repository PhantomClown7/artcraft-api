use serde::{Deserialize, Serialize};

use crate::creds::api_key::RunninghubApiKey;
use crate::error::RunninghubError;
use crate::polling::poll_task::{poll_task, POLL_MAX_SECONDS_IMAGE};

const BASE_URL: &str = "https://www.runninghub.ai";
const ENDPOINT: &str = "/openapi/v2/rhart-image-n-g31-flash/text-to-image";

#[derive(Debug, Clone)]
pub struct NanoBanana2TextToImageRequest {
  pub prompt: String,
  pub resolution: Option<String>,
  pub aspect_ratio: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RawRequest {
  prompt: String,
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

impl NanoBanana2TextToImageRequest {
  pub async fn send(&self, api_key: &RunninghubApiKey) -> Result<String, RunninghubError> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", BASE_URL, ENDPOINT);

    let raw = RawRequest {
      prompt: self.prompt.clone(),
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

    log::info!("RunningHub NanaBanana2 text-to-image task enqueued: {}", task_id);
    poll_task(api_key, &task_id, POLL_MAX_SECONDS_IMAGE).await
  }
}
