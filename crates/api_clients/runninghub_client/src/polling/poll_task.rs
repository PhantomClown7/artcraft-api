use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::creds::api_key::RunninghubApiKey;
use crate::error::RunninghubError;

const BASE_URL: &str = "https://www.runninghub.ai";
const POLL_INTERVAL_MS: u64 = 2000;
const POLL_MAX_SECONDS: u64 = 300;

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
) -> Result<String, RunninghubError> {
  let client = reqwest::Client::new();
  let url = format!("{}/openapi/v2/query", BASE_URL);
  let body = serde_json::json!({ "taskId": task_id });

  let elapsed_secs = std::cell::Cell::new(0u64);

  loop {
    if elapsed_secs.get() >= POLL_MAX_SECONDS {
      return Err(RunninghubError::PollingTimeout { seconds: POLL_MAX_SECONDS });
    }

    sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    elapsed_secs.set(elapsed_secs.get() + POLL_INTERVAL_MS / 1000);

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
