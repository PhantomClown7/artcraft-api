use crate::client::multi_router_client::MultiRouterClient;
use crate::client::router_apiyi_gpt_image_2_client::RouterApiyiGptImage2Client;
use crate::client::router_apiyi_nano_banana_client::RouterApiyiNanaBananaClient;
use crate::client::router_artcraft_client::RouterArtcraftClient;
use crate::client::router_fal_client::RouterFalClient;
use crate::client::router_gmicloud_client::RouterGmiCloudClient;
use crate::client::router_grok_api_client::RouterGrokApiClient;
use crate::client::router_runninghub_client::RouterRunninghubClient;
use crate::client::router_seedance2pro_client::RouterSeedance2ProClient;
use crate::errors::client_error::{ClientError, ClientType};

pub enum RouterClient {
  ApiyiGptImage2(RouterApiyiGptImage2Client),
  ApiyiNanaBanana(RouterApiyiNanaBananaClient),
  Artcraft(RouterArtcraftClient),
  Fal(RouterFalClient),
  GmiCloud(RouterGmiCloudClient),
  GrokApi(RouterGrokApiClient),
  Multi(MultiRouterClient),
  Runninghub(RouterRunninghubClient),
  Seedance2Pro(RouterSeedance2ProClient),
}

impl RouterClient {
  pub fn get_artcraft_client_ref(&self) -> Result<&RouterArtcraftClient, ClientError> {
    match self {
      RouterClient::Artcraft(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_artcraft_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::Artcraft)),
    }
  }

  pub fn get_fal_client_ref(&self) -> Result<&RouterFalClient, ClientError> {
    match self {
      RouterClient::Fal(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_fal_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::Fal)),
    }
  }

  pub fn get_gmicloud_client_ref(&self) -> Result<&RouterGmiCloudClient, ClientError> {
    match self {
      RouterClient::GmiCloud(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_gmicloud_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::GmiCloud)),
    }
  }

  pub fn get_grok_api_client_ref(&self) -> Result<&RouterGrokApiClient, ClientError> {
    match self {
      RouterClient::GrokApi(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_grok_api_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::GrokApi)),
    }
  }

  pub fn get_runninghub_client_ref(&self) -> Result<&RouterRunninghubClient, ClientError> {
    match self {
      RouterClient::Runninghub(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_runninghub_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::Runninghub)),
    }
  }

  pub fn get_apiyi_nano_banana_client_ref(&self) -> Result<&RouterApiyiNanaBananaClient, ClientError> {
    match self {
      RouterClient::ApiyiNanaBanana(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_apiyi_nano_banana_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::ApiyiNanaBanana)),
    }
  }

  pub fn get_apiyi_gpt_image_2_client_ref(&self) -> Result<&RouterApiyiGptImage2Client, ClientError> {
    match self {
      RouterClient::ApiyiGptImage2(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_apiyi_gpt_image_2_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::ApiyiGptImage2)),
    }
  }

  pub fn get_seedance2pro_client_ref(&self) -> Result<&RouterSeedance2ProClient, ClientError> {
    match self {
      RouterClient::Seedance2Pro(client) => Ok(client),
      RouterClient::Multi(multi) => multi.get_seedance2pro_client_ref(),
      _ => Err(ClientError::ClientNotConfigured(ClientType::Seedance2Pro)),
    }
  }
}
