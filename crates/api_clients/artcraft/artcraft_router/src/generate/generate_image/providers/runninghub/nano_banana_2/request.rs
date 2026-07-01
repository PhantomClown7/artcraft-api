use runninghub_client::requests::image::nano_banana_2::image_to_image::NanaBanana2ImageToImageRequest;
use runninghub_client::requests::image::nano_banana_2::text_to_image::NanoBanana2TextToImageRequest;

use crate::client::router_runninghub_client::RouterRunninghubClient;
use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_response::{
  GenerateImageResponse, RunninghubImageResponsePayload,
};

#[derive(Clone, Debug)]
pub enum RunninghubNanoBanana2RequestState {
  TextToImage(NanoBanana2TextToImageRequest),
  EditImage(NanaBanana2ImageToImageRequest),
}

impl RunninghubNanoBanana2RequestState {
  pub async fn send(&self, client: &RouterRunninghubClient) -> Result<GenerateImageResponse, ArtcraftRouterError> {
    let image_url = match self {
      Self::TextToImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("RunningHub error: {:?}", e)))?,
      Self::EditImage(request) => request.send(&client.api_key).await
        .map_err(|e| ArtcraftRouterError::InvalidInput(format!("RunningHub error: {:?}", e)))?,
    };
    Ok(GenerateImageResponse::Runninghub(RunninghubImageResponsePayload {
      task_id: String::new(),
      image_url,
    }))
  }
}
