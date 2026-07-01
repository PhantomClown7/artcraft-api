use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum RunninghubError {
  #[error("HTTP request failed: {0}")]
  RequestError(#[from] reqwest::Error),

  #[error("JSON error: {0}")]
  JsonError(#[from] serde_json::Error),

  #[error("Task failed: {reason}")]
  TaskFailed { reason: String },

  #[error("Polling timed out after {seconds}s")]
  PollingTimeout { seconds: u64 },

  #[error("No result URL in response")]
  NoResultUrl,

  #[error("API error: {message}")]
  ApiError { message: String },
}

#[derive(Debug, Deserialize)]
pub struct RunninghubApiError {
  pub message: Option<String>,
  pub code: Option<i64>,
}
