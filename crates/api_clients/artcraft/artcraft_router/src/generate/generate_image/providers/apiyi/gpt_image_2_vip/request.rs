use apiyi_client::requests::image::gpt_image_2_vip::image_to_image::GptImage2VipImageToImageRequest;
use apiyi_client::requests::image::gpt_image_2_vip::text_to_image::GptImage2VipTextToImageRequest;

use crate::client::router_apiyi_gpt_image_2_client::RouterApiyiGptImage2Client;
use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_response::{
  ApiyiImageResponsePayload, GenerateImageResponse,
};

#[derive(Clone, Debug)]
pub enum ApiyiGptImage2VipRequestState {
  TextToImage(GptImage2VipTextToImageRequest),
  EditImage(GptImage2VipImageToImageRequest),
}

impl ApiyiGptImage2VipRequestState {
  pub async fn send(&self, client: &RouterApiyiGptImage2Client) -> Result<GenerateImageResponse, ArtcraftRouterError> {
    let image_base64 = match self {
      Self::TextToImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("Apiyi error: {:?}", e)))?,
      Self::EditImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("Apiyi error: {:?}", e)))?,
    };
    Ok(GenerateImageResponse::Apiyi(ApiyiImageResponsePayload { image_base64 }))
  }
}
