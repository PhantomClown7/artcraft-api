use runninghub_client::creds::api_key::RunninghubApiKey;

pub struct RouterRunninghubClient {
  pub(crate) api_key: RunninghubApiKey,
}

impl RouterRunninghubClient {
  pub fn new_from_raw_key(api_key: &str) -> Self {
    Self { api_key: RunninghubApiKey::from_str(api_key) }
  }
}
