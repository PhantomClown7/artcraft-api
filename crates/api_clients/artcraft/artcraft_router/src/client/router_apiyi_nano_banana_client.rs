use apiyi_client::creds::api_key::ApiyiApiKey;

pub struct RouterApiyiNanaBananaClient {
  pub(crate) api_key: ApiyiApiKey,
}

impl RouterApiyiNanaBananaClient {
  pub fn new_from_raw_key(api_key: &str) -> Self {
    Self { api_key: ApiyiApiKey::from_str(api_key) }
  }
}
