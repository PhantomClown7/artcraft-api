use runninghub_client::requests::image::nano_banana_2::image_to_image::NanaBanana2ImageToImageRequest;
use runninghub_client::requests::image::nano_banana_2::text_to_image::NanoBanana2TextToImageRequest;

use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use crate::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use crate::generate::generate_image::image_generation_request::ImageGenerationRequest;
use crate::generate::generate_image::providers::runninghub::aspect_ratio::plan_runninghub_aspect_ratio;
use crate::generate::generate_image::providers::runninghub::resolution::plan_runninghub_resolution;
use crate::generate::generate_image::providers::runninghub::nano_banana_2::request::RunninghubNanoBanana2RequestState;

pub fn build_runninghub_nano_banana_2(
  builder: GenerateImageRequestBuilder,
) -> Result<ImageGenerationDraftOrRequest, ArtcraftRouterError> {
  let prompt = builder.prompt.clone().unwrap_or_default();
  let resolution = plan_runninghub_resolution(builder.resolution);
  // Nano Banana 2's RunningHub channel supports 14 aspect ratios but, unlike
  // the GPT Image 2.0 channel, does not include "9:21" - fall back to the
  // API's own default rather than sending an aspect ratio it will reject.
  let aspect_ratio = plan_runninghub_aspect_ratio(builder.aspect_ratio)
    .filter(|ar| ar != "9:21");
  let image_urls = match &builder.image_inputs {
    Some(crate::api::image_list_ref::ImageListRef::Urls(urls)) => urls.clone(),
    _ => vec![],
  };

  let state = if image_urls.is_empty() {
    RunninghubNanoBanana2RequestState::TextToImage(NanoBanana2TextToImageRequest {
      prompt,
      resolution,
      aspect_ratio,
    })
  } else {
    RunninghubNanoBanana2RequestState::EditImage(NanaBanana2ImageToImageRequest {
      prompt,
      image_urls,
      resolution,
      aspect_ratio,
    })
  };

  Ok(ImageGenerationDraftOrRequest::Request(
    ImageGenerationRequest::RunninghubNanoBanana2(state),
  ))
}
