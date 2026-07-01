use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::creds::api_key::RunninghubApiKey;
use crate::error::RunninghubError;

const BASE_URL: &str = "https://www.runninghub.ai";
const POLL_INTERVAL_MS: u64 = 2000;

/// Timeout for image generation tasks (usually resolve within seconds).
pub const POLL_MAX_SECONDS_IMAGE: u64 = 300;

/// Timeout for video generation tasks, which routinely take several minutes.
/// Reusing the image timeout here previously caused RunningHub Grok Video
/// requests to time out client-side while the job was still rendering.
pub const POLL_MAX_SECONDS_VIDEO: u64 = 900;

#[derive(Debug, Deserialize)]
struct QueryResponse {
  status: Option<String>,
  #[serde(rename = "failedReason")]
  failed_reason: Option<String>,
  results: Option<Vec<TaskResult>>,
}

#[derive(Debug, Deserialize)]
struct TaskResult {
  url: Option<String>,
  #[serde(rename = "fileUrl")]
  file_url: Option<String>,
}

/// Poll RunningHub until the task completes, then return the result URL.
pub async fn poll_task(
  api_key: &RunninghubApiKey,
  task_id: &str,
  max_seconds: u64,
) -> Result<String, RunninghubError> {
  let client = reqwest::Client::new();
  let url = format!("{}/openapi/v2/query", BASE_URL);
  let body = serde_json::json!({ "taskId": task_id });

  let mut elapsed_secs = 0u64;

  loop {
    if elapsed_secs >= max_seconds {
      return Err(RunninghubError::PollingTimeout { seconds: max_seconds });
    }

    sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    elapsed_secs += POLL_INTERVAL_MS / 1000;

    let response = client
      .post(&url)
      .header("Authorization", format!("Bearer {}", api_key.as_str()))
      .header("Content-Type", "application/json")
      .json(&body)
      .send()
      .await?;

    let query: QueryResponse = response.json().await?;

    match query.status.as_deref() {
      Some("SUCCESS") => {
        let results = query.results.unwrap_or_default();
        let result_url = results
          .into_iter()
          .find_map(|r| r.url.or(r.file_url))
          .ok_or(RunninghubError::NoResultUrl)?;
        return Ok(result_url);
      }
      Some("FAILED") => {
        return Err(RunninghubError::TaskFailed {
          reason: query.failed_reason.unwrap_or_else(|| "unknown".to_string()),
        });
      }
      _ => {
        // Still running — keep polling
        log::debug!("RunningHub task {} still pending", task_id);
      }
    }
  }
}
