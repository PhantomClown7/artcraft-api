#[derive(Debug, thiserror::Error)]
pub enum ApiyiError {
  #[error("HTTP request failed: {0}")]
  RequestError(#[from] reqwest::Error),

  #[error("JSON error: {0}")]
  JsonError(#[from] serde_json::Error),

  #[error("No image data in response")]
  NoImageData,

  #[error("API error: {message}")]
  ApiError { message: String },
}
