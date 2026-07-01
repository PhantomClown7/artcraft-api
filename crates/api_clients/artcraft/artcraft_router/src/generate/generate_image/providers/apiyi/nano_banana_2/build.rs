use apiyi_client::requests::image::nano_banana_2::image_to_image::NanaBanana2ImageToImageRequest;
use apiyi_client::requests::image::nano_banana_2::text_to_image::NanaBanana2TextToImageRequest;

use crate::errors::artcraft_router_error::ArtcraftRouterError;
use crate::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use crate::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use crate::generate::generate_image::image_generation_request::ImageGenerationRequest;
use crate::generate::generate_image::providers::apiyi::nano_banana_2::request::ApiyiNanaBanana2RequestState;

pub fn build_apiyi_nano_banana_2(
  builder: GenerateImageRequestBuilder,
) -> Result<ImageGenerationDraftOrRequest, ArtcraftRouterError> {
  let prompt = builder.prompt.clone().unwrap_or_default();
  let image_size = None::<String>; // TODO: map resolution
  let aspect_ratio = builder.aspect_ratio.map(|ar| format!("{:?}", ar));
  let image_urls = match &builder.image_inputs {
    Some(crate::api::image_list_ref::ImageListRef::Urls(urls)) => urls.clone(),
    _ => vec![],
  };

  let state = if image_urls.is_empty() {
    ApiyiNanaBanana2RequestState::TextToImage(NanaBanana2TextToImageRequest {
      prompt,
      image_size,
      aspect_ratio,
    })
  } else {
    // For image-to-image, Apiyi Nano Banana uses base64, but we receive URLs from the router.
    // We pass empty base64 list here; the desktop handler will need to fetch & encode the images.
    ApiyiNanaBanana2RequestState::EditImage(NanaBanana2ImageToImageRequest {
      prompt,
      image_base64_list: vec![],
      image_size,
      aspect_ratio,
    })
  };

  Ok(ImageGenerationDraftOrRequest::Request(
    ImageGenerationRequest::ApiyiNanaBanana2(state),
  ))
}
