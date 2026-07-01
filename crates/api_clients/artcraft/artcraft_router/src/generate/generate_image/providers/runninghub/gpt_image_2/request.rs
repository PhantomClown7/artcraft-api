use runninghub_client::requests::image::gpt_image_2::image_to_image::GptImage2ImageToImageRequest;
use runninghub_client::requests::image::gpt_image_2::text_to_image::GptImage2TextToImageRequest;

use crate::client::router_runninghub_client::RouterRunninghubClient;
use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_response::{
  GenerateImageResponse, RunninghubImageResponsePayload,
};

#[derive(Clone, Debug)]
pub enum RunninghubGptImage2RequestState {
  TextToImage(GptImage2TextToImageRequest),
  EditImage(GptImage2ImageToImageRequest),
}

impl RunninghubGptImage2RequestState {
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
