use apiyi_client::requests::image::nano_banana_2::image_to_image::NanaBanana2ImageToImageRequest;
use apiyi_client::requests::image::nano_banana_2::text_to_image::NanaBanana2TextToImageRequest;

use crate::client::router_apiyi_nano_banana_client::RouterApiyiNanaBananaClient;
use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_response::{
  ApiyiImageResponsePayload, GenerateImageResponse,
};

#[derive(Clone, Debug)]
pub enum ApiyiNanaBanana2RequestState {
  TextToImage(NanaBanana2TextToImageRequest),
  EditImage(NanaBanana2ImageToImageRequest),
}

impl ApiyiNanaBanana2RequestState {
  pub async fn send(&self, client: &RouterApiyiNanaBananaClient) -> Result<GenerateImageResponse, ArtcraftRouterError> {
    let image_base64 = match self {
      Self::TextToImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("Apiyi error: {:?}", e)))?,
      Self::EditImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("Apiyi error: {:?}", e)))?,
    };
    Ok(GenerateImageResponse::Apiyi(ApiyiImageResponsePayload { image_base64 }))
  }
}
