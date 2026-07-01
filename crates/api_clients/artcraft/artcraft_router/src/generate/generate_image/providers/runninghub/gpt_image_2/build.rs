use runninghub_client::requests::image::gpt_image_2::image_to_image::GptImage2ImageToImageRequest;
use runninghub_client::requests::image::gpt_image_2::text_to_image::GptImage2TextToImageRequest;

use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use crate::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use crate::generate::generate_image::image_generation_request::ImageGenerationRequest;
use crate::generate::generate_image::providers::runninghub::gpt_image_2::request::RunninghubGptImage2RequestState;

pub fn build_runninghub_gpt_image_2(
  builder: GenerateImageRequestBuilder,
) -> Result<ImageGenerationDraftOrRequest, ArtcraftRouterError> {
  let prompt = builder.prompt.clone().unwrap_or_default();
  let resolution = builder.resolution.map(|r| format!("{:?}", r).to_lowercase());
  let aspect_ratio = builder.aspect_ratio.map(|ar| format!("{:?}", ar));
  let image_urls = match &builder.image_inputs {
    Some(crate::api::image_list_ref::ImageListRef::Urls(urls)) => urls.clone(),
    _ => vec![],
  };

  let state = if image_urls.is_empty() {
    RunninghubGptImage2RequestState::TextToImage(GptImage2TextToImageRequest {
      prompt,
      resolution,
      aspect_ratio,
    })
  } else {
    RunninghubGptImage2RequestState::EditImage(GptImage2ImageToImageRequest {
      prompt,
      image_urls,
      resolution,
      aspect_ratio,
    })
  };

  Ok(ImageGenerationDraftOrRequest::Request(
    ImageGenerationRequest::RunninghubGptImage2(state),
  ))
}
