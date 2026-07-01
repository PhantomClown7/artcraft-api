use crate::client::router_apiyi_gpt_image_2_client::RouterApiyiGptImage2Client;
use crate::client::router_apiyi_nano_banana_client::RouterApiyiNanaBananaClient;
use crate::client::router_artcraft_client::RouterArtcraftClient;
use crate::client::router_fal_client::RouterFalClient;
use crate::client::router_gmicloud_client::RouterGmiCloudClient;
use crate::client::router_grok_api_client::RouterGrokApiClient;
use crate::client::router_runninghub_client::RouterRunninghubClient;
use crate::client::router_seedance2pro_client::RouterSeedance2ProClient;
use crate::errors::client_error::{ClientError, ClientType};

pub struct MultiRouterClient {
  pub(crate) apiyi_gpt_image_2_client: Option<RouterApiyiGptImage2Client>,
  pub(crate) apiyi_nano_banana_client: Option<RouterApiyiNanaBananaClient>,
  pub(crate) artcraft_client: Option<RouterArtcraftClient>,
  pub(crate) fal_client: Option<RouterFalClient>,
  pub(crate) gmicloud_client: Option<RouterGmiCloudClient>,
  pub(crate) grok_api_client: Option<RouterGrokApiClient>,
  pub(crate) runninghub_client: Option<RouterRunninghubClient>,
  pub(crate) seedance2pro_client: Option<RouterSeedance2ProClient>,
}

impl MultiRouterClient {
  pub fn get_apiyi_gpt_image_2_client_ref(&self) -> Result<&RouterApiyiGptImage2Client, ClientError> {
    self.apiyi_gpt_image_2_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::ApiyiGptImage2))
  }

  pub fn get_apiyi_nano_banana_client_ref(&self) -> Result<&RouterApiyiNanaBananaClient, ClientError> {
    self.apiyi_nano_banana_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::ApiyiNanaBanana))
  }

  pub fn get_artcraft_client_ref(&self) -> Result<&RouterArtcraftClient, ClientError> {
    self.artcraft_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::Artcraft))
  }

  pub fn get_fal_client_ref(&self) -> Result<&RouterFalClient, ClientError> {
    self.fal_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::Fal))
  }

  pub fn get_gmicloud_client_ref(&self) -> Result<&RouterGmiCloudClient, ClientError> {
    self.gmicloud_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::GmiCloud))
  }

  pub fn get_grok_api_client_ref(&self) -> Result<&RouterGrokApiClient, ClientError> {
    self.grok_api_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::GrokApi))
  }

  pub fn get_runninghub_client_ref(&self) -> Result<&RouterRunninghubClient, ClientError> {
    self.runninghub_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::Runninghub))
  }

  pub fn get_seedance2pro_client_ref(&self) -> Result<&RouterSeedance2ProClient, ClientError> {
    self.seedance2pro_client.as_ref()
      .ok_or(ClientError::ClientNotConfigured(ClientType::Seedance2Pro))
  }
}
