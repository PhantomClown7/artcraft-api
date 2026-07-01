use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info};
use runninghub_client::creds::api_key::RunninghubApiKey;
use runninghub_client::requests::video::grok_video::image_to_video::GrokVideoImageToVideoRequest;
use runninghub_client::requests::video::grok_video::text_to_video::GrokVideoTextToVideoRequest;

use crate::core::commands::enqueue::generate_error::{GenerateError, MissingCredentialsReason};
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::map_media_files_to_urls::map_media_file_tokens_to_cdn_urls;
use crate::core::commands::generate::generate_video::request::TauriGenerateVideoRequest;
use crate::core::events::generation_events::common::GenerationModel;
use crate::core::providers::credentials::payload::provider_credential_payload::ProviderCredentialPayload;
use crate::core::providers::credentials::provider_credential_key::ProviderCredentialKey;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;

pub async fn handle_runninghub_grok_video(
  request: &TauriGenerateVideoRequest,
  credential_cache: &ProviderCredentialLoadingCache,
  app_env_configs: &AppEnvConfigs,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let api_key = load_api_key(credential_cache)?;

  let prompt = request.prompt.clone().unwrap_or_default();
  // Grok Video's aspectRatio/resolution/duration are all required fields with
  // no documented server-side default, so always supply a value here rather
  // than omitting the field.
  let aspect_ratio = plan_grok_video_aspect_ratio(request.aspect_ratio);
  let resolution = plan_grok_video_resolution(request.resolution);
  let duration = request.duration_seconds.map(|d| d as u32).unwrap_or(6);

  let video_url = if let Some(image_token) = &request.start_frame_image_media_token {
    info!("[RunninghubGrokVideo] Image-to-video mode");

    let image_urls = map_media_file_tokens_to_cdn_urls(
      &[image_token.clone()],
      &app_env_configs.storyteller_host,
    ).await?;

    let result = GrokVideoImageToVideoRequest {
      prompt,
      image_urls,
      aspect_ratio: Some(aspect_ratio.to_string()),
      resolution: Some(resolution.to_string()),
      duration: Some(duration),
    }.send(&api_key).await;

    match result {
      Ok(url) => url,
      Err(err) => {
        error!("[RunninghubGrokVideo] Image-to-video failed: {:?}", err);
        return Err(GenerateError::AnyhowError(anyhow::anyhow!("{:?}", err)));
      }
    }
  } else {
    info!("[RunninghubGrokVideo] Text-to-video mode");

    let result = GrokVideoTextToVideoRequest {
      prompt,
      aspect_ratio: Some(aspect_ratio.to_string()),
      resolution: Some(resolution.to_string()),
      duration: Some(duration),
    }.send(&api_key).await;

    match result {
      Ok(url) => url,
      Err(err) => {
        error!("[RunninghubGrokVideo] Text-to-video failed: {:?}", err);
        return Err(GenerateError::AnyhowError(anyhow::anyhow!("{:?}", err)));
      }
    }
  };

  info!("[RunninghubGrokVideo] Video ready at: {}", video_url);

  Ok(TaskEnqueueSuccess {
    provider: GenerationProvider::Runninghub,
    model: Some(GenerationModel::GrokVideo),
    task_type: TaskType::VideoGeneration,
    provider_job_id: None,
    maybe_queue_status_url: None,
    maybe_queue_response_url: Some(video_url),
    maybe_prompt_token: None,
  })
}

fn load_api_key(
  credential_cache: &ProviderCredentialLoadingCache,
) -> Result<RunninghubApiKey, GenerateError> {
  let payload = credential_cache
    .get_credentials(ProviderCredentialKey::RunninghubApiKey)
    .map_err(|err| GenerateError::AnyhowError(anyhow::anyhow!("Failed to load credentials: {:?}", err)))?
    .ok_or(GenerateError::MissingCredentials(MissingCredentialsReason::NeedsRunninghubApiKey))?;

  match payload {
    ProviderCredentialPayload::ApiKey(data) => Ok(RunninghubApiKey::from_str(data.as_str())),
    _ => Err(GenerateError::AnyhowError(anyhow::anyhow!(
      "RunningHub credential is not an API key"
    ))),
  }
}

/// Grok Video only supports 5 aspect ratios (unlike RunningHub's image
/// endpoints, which support up to 15) and the field is required, so this
/// always returns a value - snapping unsupported ratios to the closest match.
fn plan_grok_video_aspect_ratio(ar: Option<RouterAspectRatio>) -> &'static str {
  match ar {
    Some(RouterAspectRatio::Square) | Some(RouterAspectRatio::SquareHd) => "1:1",
    Some(RouterAspectRatio::WideSixteenByNine) => "16:9",
    Some(RouterAspectRatio::TallNineBySixteen) => "9:16",
    Some(RouterAspectRatio::WideThreeByTwo)
    | Some(RouterAspectRatio::Wide)
    | Some(RouterAspectRatio::WideFourByThree)
    | Some(RouterAspectRatio::WideFiveByFour)
    | Some(RouterAspectRatio::WideTwentyOneByNine) => "3:2",
    Some(RouterAspectRatio::TallTwoByThree)
    | Some(RouterAspectRatio::Tall)
    | Some(RouterAspectRatio::TallThreeByFour)
    | Some(RouterAspectRatio::TallFourByFive)
    | Some(RouterAspectRatio::TallNineByTwentyOne) => "2:3",
    Some(RouterAspectRatio::Auto)
    | Some(RouterAspectRatio::Auto2k)
    | Some(RouterAspectRatio::Auto3k)
    | Some(RouterAspectRatio::Auto4k)
    | None => "16:9",
  }
}

/// Grok Video only supports "480p"/"720p" and the field is required, so this
/// always returns a value rather than an `Option`.
fn plan_grok_video_resolution(res: Option<RouterResolution>) -> &'static str {
  match res {
    Some(RouterResolution::HalfK) | Some(RouterResolution::FourEightyP) => "480p",
    _ => "720p",
  }
}
