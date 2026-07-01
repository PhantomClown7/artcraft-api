use apiyi_client::requests::image::gpt_image_2_vip::image_to_image::GptImage2VipImageToImageRequest;
use apiyi_client::requests::image::gpt_image_2_vip::text_to_image::GptImage2VipTextToImageRequest;

use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use crate::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use crate::generate::generate_image::image_generation_request::ImageGenerationRequest;
use crate::generate::generate_image::providers::apiyi::gpt_image_2_vip::request::ApiyiGptImage2VipRequestState;

pub fn build_apiyi_gpt_image_2_vip(
  builder: GenerateImageRequestBuilder,
) -> Result<ImageGenerationDraftOrRequest, ArtcraftRouterError> {
  let prompt = builder.prompt.clone().unwrap_or_default();
  let size = None::<String>; // TODO: map resolution to size string
  let image_urls = match &builder.image_inputs {
    Some(crate::api::image_list_ref::ImageListRef::Urls(urls)) => urls.clone(),
    _ => vec![],
  };

  let state = if image_urls.is_empty() {
    ApiyiGptImage2VipRequestState::TextToImage(GptImage2VipTextToImageRequest {
      prompt,
      size,
    })
  } else {
    // For image-to-image, we need image bytes. Pass empty for now; desktop handler fetches + converts.
    ApiyiGptImage2VipRequestState::EditImage(GptImage2VipImageToImageRequest {
      prompt,
      image_bytes: vec![],
      image_filename: "input.png".to_string(),
      size,
    })
  };

  Ok(ImageGenerationDraftOrRequest::Request(
    ImageGenerationRequest::ApiyiGptImage2Vip(state),
  ))
}
