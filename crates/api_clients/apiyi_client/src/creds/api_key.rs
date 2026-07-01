#[derive(Clone)]
pub struct ApiyiApiKey(pub String);

impl ApiyiApiKey {
  pub fn from_str(api_key: &str) -> Self {
    Self(api_key.trim().to_string())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}
