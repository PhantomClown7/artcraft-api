use crate::client::multi_router_client::MultiRouterClient;
use crate::client::router_apiyi_gpt_image_2_client::RouterApiyiGptImage2Client;
use crate::client::router_apiyi_nano_banana_client::RouterApiyiNanaBananaClient;
use crate::client::router_artcraft_client::RouterArtcraftClient;
use crate::client::router_fal_client::RouterFalClient;
use crate::client::router_gmicloud_client::RouterGmiCloudClient;
use crate::client::router_grok_api_client::RouterGrokApiClient;
use crate::client::router_runninghub_client::RouterRunninghubClient;
use crate::client::router_seedance2pro_client::RouterSeedance2ProClient;

pub struct MultiRouterClientBuilder {
  apiyi_gpt_image_2_client: Option<RouterApiyiGptImage2Client>,
  apiyi_nano_banana_client: Option<RouterApiyiNanaBananaClient>,
  artcraft_client: Option<RouterArtcraftClient>,
  fal_client: Option<RouterFalClient>,
  gmicloud_client: Option<RouterGmiCloudClient>,
  grok_api_client: Option<RouterGrokApiClient>,
  runninghub_client: Option<RouterRunninghubClient>,
  seedance2pro_client: Option<RouterSeedance2ProClient>,
}

impl MultiRouterClientBuilder {
  pub fn new() -> Self {
    Self {
      apiyi_gpt_image_2_client: None,
      apiyi_nano_banana_client: None,
      artcraft_client: None,
      fal_client: None,
      gmicloud_client: None,
      grok_api_client: None,
      runninghub_client: None,
      seedance2pro_client: None,
    }
  }

  pub fn set_apiyi_gpt_image_2_client(mut self, client: RouterApiyiGptImage2Client) -> Self {
    self.apiyi_gpt_image_2_client = Some(client);
    self
  }

  pub fn set_apiyi_nano_banana_client(mut self, client: RouterApiyiNanaBananaClient) -> Self {
    self.apiyi_nano_banana_client = Some(client);
    self
  }

  pub fn set_artcraft_client(mut self, client: RouterArtcraftClient) -> Self {
    self.artcraft_client = Some(client);
    self
  }

  pub fn set_fal_client(mut self, client: RouterFalClient) -> Self {
    self.fal_client = Some(client);
    self
  }

  pub fn set_gmicloud_client(mut self, client: RouterGmiCloudClient) -> Self {
    self.gmicloud_client = Some(client);
    self
  }

  pub fn set_grok_api_client(mut self, client: RouterGrokApiClient) -> Self {
    self.grok_api_client = Some(client);
    self
  }

  pub fn set_runninghub_client(mut self, client: RouterRunninghubClient) -> Self {
    self.runninghub_client = Some(client);
    self
  }

  pub fn set_seedance2pro_client(mut self, client: RouterSeedance2ProClient) -> Self {
    self.seedance2pro_client = Some(client);
    self
  }

  pub fn build(self) -> MultiRouterClient {
    MultiRouterClient {
      apiyi_gpt_image_2_client: self.apiyi_gpt_image_2_client,
      apiyi_nano_banana_client: self.apiyi_nano_banana_client,
      artcraft_client: self.artcraft_client,
      fal_client: self.fal_client,
      gmicloud_client: self.gmicloud_client,
      grok_api_client: self.grok_api_client,
      runninghub_client: self.runninghub_client,
      seedance2pro_client: self.seedance2pro_client,
    }
  }
}
