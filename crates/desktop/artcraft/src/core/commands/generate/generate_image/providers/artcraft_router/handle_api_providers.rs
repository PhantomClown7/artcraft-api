use apiyi_client::creds::api_key::ApiyiApiKey;
use apiyi_client::requests::image::gpt_image_2_vip::image_to_image::GptImage2VipImageToImageRequest;
use apiyi_client::requests::image::gpt_image_2_vip::text_to_image::GptImage2VipTextToImageRequest;
use apiyi_client::requests::image::nano_banana_2::image_to_image::NanaBanana2ImageToImageRequest;
use apiyi_client::requests::image::nano_banana_2::text_to_image::NanaBanana2TextToImageRequest;
use artcraft_client::endpoints::media_files::upload_image_media_file_from_bytes::{upload_image_media_file_from_bytes, ImageType, UploadImageBytesArgs};
use artcraft_client::endpoints::prompts::create_prompt::create_prompt;
use artcraft_client::utils::api_host::ApiHost;
use artcraft_router::api::image_list_ref::ImageListRef;
use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_provider::RouterProvider;
use artcraft_router::api::router_resolution::RouterResolution;
use artcraft_router::client::generation_mode_mismatch_strategy::GenerationModeMismatchStrategy;
use artcraft_router::client::request_mismatch_mitigation_strategy::RequestMismatchMitigationStrategy;
use artcraft_router::client::router_client::RouterClient;
use artcraft_router::client::router_fal_client::RouterFalClient;
use artcraft_router::client::router_runninghub_client::RouterRunninghubClient;
use artcraft_router::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use artcraft_router::generate::generate_image::generate_image_response::GenerateImageResponse;
use artcraft_router::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use artcraft_router::generate::generate_image::providers::runninghub::aspect_ratio::plan_runninghub_aspect_ratio;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use tokens::tokens::media_files::MediaFileToken;
use tokens::tokens::prompts::PromptToken;

use crate::core::api_adapters::models::image::tauri_image_model_to_generation_model::tauri_image_model_to_generation_model;
use crate::core::api_adapters::models::image::tauri_image_model_to_router_model::tauri_image_model_to_router_model;
use crate::core::commands::enqueue::generate_error::{GenerateError, MissingCredentialsReason};
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::commands::generate::common::router_image_request_to_artcraft_prompt::router_image_request_to_artcraft_prompt;
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::convert_enums_to_router::{convert_aspect_ratio, convert_quality, convert_resolution};
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::map_media_files_to_urls::map_media_file_tokens_to_cdn_urls;
use crate::core::commands::generate::generate_image::tauri_generate_image_request::TauriGenerateImageRequest;
use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;
use crate::core::providers::credentials::payload::provider_credential_payload::ProviderCredentialPayload;
use crate::core::providers::credentials::provider_credential_key::ProviderCredentialKey;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

/// Handle image generation for providers that authenticate via API key.
pub async fn handle_api_key_provider(
  request: &TauriGenerateImageRequest,
  provider: GenerationProvider,
  api_key: &str,
  app_env_configs: &AppEnvConfigs,
  storyteller_creds_manager: &StorytellerCredentialManager,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  match provider {
    GenerationProvider::Fal => {
      handle_fal(request, api_key, app_env_configs, storyteller_creds_manager).await
    }
    GenerationProvider::Runninghub => {
      handle_runninghub(request, api_key, app_env_configs, storyteller_creds_manager).await
    }
    _ => {
      Err(GenerateError::NotYetImplemented(
        format!("API key provider {:?} is not yet supported via the router path", provider),
      ))
    }
  }
}

// ── FAL ──

async fn handle_fal(
  request: &TauriGenerateImageRequest,
  api_key: &str,
  app_env_configs: &AppEnvConfigs,
  storyteller_creds_manager: &StorytellerCredentialManager,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let tauri_model = request.model.ok_or(GenerateError::no_model_specified())?;
  let api_host = &app_env_configs.storyteller_host;

  let router_model = tauri_image_model_to_router_model(tauri_model)
    .ok_or(GenerateError::NotYetImplemented(
      format!("Model {:?} is not supported via the FAL router path", tauri_model),
    ))?;

  let image_inputs = resolve_image_inputs(request, api_host).await?;

  let router_request = GenerateImageRequestBuilder {
    model: router_model,
    provider: RouterProvider::Fal,
    prompt: request.prompt.clone(),
    image_inputs,
    resolution: request.resolution.map(convert_resolution),
    aspect_ratio: request.aspect_ratio.map(convert_aspect_ratio),
    quality: request.quality.map(convert_quality),
    image_batch_count: request.batch_size.map(|n| n as u16),
    horizontal_angle: request.adjust_horizontal_angle,
    vertical_angle: request.adjust_vertical_angle,
    zoom: request.adjust_zoom,
    request_mismatch_mitigation_strategy: RequestMismatchMitigationStrategy::PayMoreUpgrade,
    generation_mode_mismatch_strategy: Some(GenerationModeMismatchStrategy::GenerateAnyway),
    idempotency_token: None,
  };

  let maybe_prompt_token = create_prompt_record(
    &router_request,
    api_host,
    storyteller_creds_manager,
  ).await;

  let fal_client = RouterFalClient::new_polling_only_from_raw_key(api_key);
  let client = RouterClient::Fal(fal_client);

  info!("Building FAL image generation plan: model={:?}", router_model);

  let request = match router_request.build2() {
    Ok(ImageGenerationDraftOrRequest::Request(request)) => request,
    Ok(ImageGenerationDraftOrRequest::Draft(draft)) => {
      warn!("Fal is trying to send draft request: {:?}", draft);
      return Err(GenerateError::NotYetImplemented("Fal should not be sending draft requests".to_string()));
    },
    Err(err) => {
      warn!("Could not use FAL: {:?}", err);
      return Err(GenerateError::NotYetImplemented("Error Message: TODO".to_string()));
    }
  };

  info!("Executing FAL image generation. Request: {:?}", request);

  match request.send_request(&client).await {
    Ok(response) => {
      build_task_enqueue_success_fal(tauri_model, response, maybe_prompt_token)
    },
    Err(err) => {
      warn!("Fal image generation failed: {:?}", err);
      Err(err.into())
    }
  }
}

// ── RunningHub ──

async fn handle_runninghub(
  request: &TauriGenerateImageRequest,
  api_key: &str,
  app_env_configs: &AppEnvConfigs,
  storyteller_creds_manager: &StorytellerCredentialManager,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let tauri_model = request.model.ok_or(GenerateError::no_model_specified())?;
  let api_host = &app_env_configs.storyteller_host;

  let router_model = tauri_image_model_to_router_model(tauri_model)
    .ok_or_else(|| GenerateError::NotYetImplemented(
      format!("Model {:?} is not supported via the RunningHub router path", tauri_model),
    ))?;

  let image_inputs = resolve_image_inputs(request, api_host).await?;

  let router_request = GenerateImageRequestBuilder {
    model: router_model,
    provider: RouterProvider::Runninghub,
    prompt: request.prompt.clone(),
    image_inputs,
    resolution: request.resolution.map(convert_resolution),
    aspect_ratio: request.aspect_ratio.map(convert_aspect_ratio),
    quality: request.quality.map(convert_quality),
    image_batch_count: request.batch_size.map(|n| n as u16),
    horizontal_angle: request.adjust_horizontal_angle,
    vertical_angle: request.adjust_vertical_angle,
    zoom: request.adjust_zoom,
    request_mismatch_mitigation_strategy: RequestMismatchMitigationStrategy::PayMoreUpgrade,
    generation_mode_mismatch_strategy: Some(GenerationModeMismatchStrategy::GenerateAnyway),
    idempotency_token: None,
  };

  let maybe_prompt_token = create_prompt_record(
    &router_request,
    api_host,
    storyteller_creds_manager,
  ).await;

  let runninghub_client = RouterRunninghubClient::new_from_raw_key(api_key);
  let client = RouterClient::Runninghub(runninghub_client);

  info!("Building RunningHub image generation plan: model={:?}", router_model);

  let built_request = match router_request.build2() {
    Ok(ImageGenerationDraftOrRequest::Request(r)) => r,
    Ok(ImageGenerationDraftOrRequest::Draft(draft)) => {
      warn!("RunningHub received draft request: {:?}", draft);
      return Err(GenerateError::NotYetImplemented("RunningHub should not send draft requests".to_string()));
    },
    Err(err) => {
      warn!("Could not build RunningHub request: {:?}", err);
      return Err(err.into());
    }
  };

  info!("Executing RunningHub image generation (synchronous poll)...");

  match built_request.send_request(&client).await {
    Ok(response) => {
      build_task_enqueue_success_runninghub(tauri_model, response, maybe_prompt_token)
    },
    Err(err) => {
      warn!("RunningHub image generation failed: {:?}", err);
      Err(err.into())
    }
  }
}

// ── Apiyi ──

/// Handle Apiyi image generation. Apiyi uses model-specific API keys, so
/// this function loads the appropriate key from the credential cache.
pub async fn handle_apiyi(
  request: &TauriGenerateImageRequest,
  credential_cache: &ProviderCredentialLoadingCache,
  app_env_configs: &AppEnvConfigs,
  storyteller_creds_manager: &StorytellerCredentialManager,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let tauri_model = request.model.ok_or(GenerateError::no_model_specified())?;
  let api_host = &app_env_configs.storyteller_host;

  let credential_key = match tauri_model {
    TauriImageModel::ApiyiNanaBanana2 => ProviderCredentialKey::ApiyiNanoBananaApiKey,
    TauriImageModel::ApiyiGptImage2Vip => ProviderCredentialKey::ApiyiGptImage2ApiKey,
    _ => return Err(GenerateError::NotYetImplemented(
      format!("Apiyi model {:?} is not supported", tauri_model)
    )),
  };

  let payload = credential_cache.get_credentials(credential_key)
    .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Failed to load Apiyi credentials: {:?}", e)))?
    .ok_or_else(|| GenerateError::MissingCredentials(MissingCredentialsReason::NeedsApiyiApiKey))?;

  let raw_key = match payload {
    ProviderCredentialPayload::ApiKey(key) => key.as_str().to_string(),
    ProviderCredentialPayload::WebLogin(_) => {
      return Err(GenerateError::MissingCredentials(MissingCredentialsReason::NeedsApiyiApiKey));
    }
  };

  let api_key = ApiyiApiKey::from_str(&raw_key);
  let prompt = request.prompt.clone().unwrap_or_default();
  let image_urls = get_image_urls_from_request(request, api_host).await?;
  let aspect_ratio = request.aspect_ratio.map(convert_aspect_ratio);
  let resolution = request.resolution.map(convert_resolution);

  info!("Executing Apiyi image generation: model={:?}, image_inputs={}", tauri_model, image_urls.len());

  let image_base64 = match tauri_model {
    TauriImageModel::ApiyiNanaBanana2 => {
      call_apiyi_nano_banana_2(&api_key, &prompt, &image_urls, aspect_ratio, resolution).await?
    }
    TauriImageModel::ApiyiGptImage2Vip => {
      call_apiyi_gpt_image_2_vip(&api_key, &prompt, &image_urls, aspect_ratio, resolution).await?
    }
    _ => unreachable!(),
  };

  // Decode base64 and upload to Artcraft CDN
  let bytes = BASE64_STANDARD.decode(&image_base64)
    .map_err(|e| GenerateError::DecodeError(e))?;

  let creds = storyteller_creds_manager.get_credentials()
    .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Credential read error: {:?}", e)))?
    .ok_or_else(|| GenerateError::needs_storyteller_credentials())?;

  let upload_result = upload_image_media_file_from_bytes(UploadImageBytesArgs {
    api_host,
    maybe_creds: Some(&creds),
    image_bytes: bytes,
    image_type: ImageType::Png,
    is_intermediate_system_file: false,
    maybe_generation_provider: Some(GenerationProvider::Apiyi),
  }).await
    .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("CDN upload failed: {:?}", e)))?;

  info!("Apiyi image uploaded to CDN: {:?}", upload_result.media_file_token);

  let generation_model = tauri_image_model_to_generation_model(tauri_model);

  Ok(TaskEnqueueSuccess {
    task_type: TaskType::ImageGeneration,
    model: Some(generation_model),
    provider: GenerationProvider::Apiyi,
    provider_job_id: Some(upload_result.media_file_token.as_str().to_string()),
    maybe_queue_status_url: None,
    maybe_queue_response_url: None,
    maybe_prompt_token: None,
  })
}

// ── Apiyi model helpers ──

async fn call_apiyi_nano_banana_2(
  api_key: &ApiyiApiKey,
  prompt: &str,
  image_urls: &[String],
  aspect_ratio: Option<RouterAspectRatio>,
  resolution: Option<RouterResolution>,
) -> Result<String, GenerateError> {
  let image_size = apiyi_nano_banana_2_image_size(resolution);
  let aspect_ratio = apiyi_nano_banana_2_aspect_ratio(aspect_ratio);

  if image_urls.is_empty() {
    let req = NanaBanana2TextToImageRequest {
      prompt: prompt.to_string(),
      image_size,
      aspect_ratio,
    };
    req.send(api_key).await
      .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Apiyi NanaBanana2 error: {:?}", e)))
  } else {
    let base64_list = download_and_encode_images(image_urls).await?;
    let req = NanaBanana2ImageToImageRequest {
      prompt: prompt.to_string(),
      image_base64_list: base64_list,
      image_size,
      aspect_ratio,
    };
    req.send(api_key).await
      .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Apiyi NanaBanana2 image-to-image error: {:?}", e)))
  }
}

async fn call_apiyi_gpt_image_2_vip(
  api_key: &ApiyiApiKey,
  prompt: &str,
  image_urls: &[String],
  aspect_ratio: Option<RouterAspectRatio>,
  resolution: Option<RouterResolution>,
) -> Result<String, GenerateError> {
  let size = apiyi_gpt_image_2_vip_size(aspect_ratio, resolution);

  if image_urls.is_empty() {
    let req = GptImage2VipTextToImageRequest {
      prompt: prompt.to_string(),
      size,
    };
    req.send(api_key).await
      .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Apiyi GptImage2Vip error: {:?}", e)))
  } else {
    // Use the first image for GPT Image 2 VIP edits
    let bytes = download_image_bytes(&image_urls[0]).await?;
    let req = GptImage2VipImageToImageRequest {
      prompt: prompt.to_string(),
      image_bytes: bytes,
      image_filename: "input.png".to_string(),
      size,
    };
    req.send(api_key).await
      .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Apiyi GptImage2Vip image-to-image error: {:?}", e)))
  }
}

/// Apiyi's Nano Banana 2 channel (Gemini 3.1 Flash) supports the same 14
/// aspect ratios as RunningHub's Nano Banana 2 channel for the same
/// underlying model, minus `9:21`.
fn apiyi_nano_banana_2_aspect_ratio(ar: Option<RouterAspectRatio>) -> Option<String> {
  plan_runninghub_aspect_ratio(ar).filter(|ar| ar != "9:21")
}

/// Maps a router resolution to Apiyi's `imageConfig.imageSize` values:
/// `"512"`, `"1K"`, `"2K"`, or `"4K"`.
fn apiyi_nano_banana_2_image_size(res: Option<RouterResolution>) -> Option<String> {
  let s = match res? {
    RouterResolution::HalfK => "512",
    RouterResolution::OneK => "1K",
    RouterResolution::TwoK => "2K",
    RouterResolution::ThreeK => "2K",
    RouterResolution::FourK => "4K",
    RouterResolution::FourEightyP => "1K",
    RouterResolution::SevenTwentyP => "1K",
    RouterResolution::TenEightyP => "2K",
  };
  Some(s.to_string())
}

/// Apiyi's GPT Image 2 VIP has no `aspectRatio`/`resolution` fields - it
/// takes an explicit `"WIDTHxHEIGHT"` `size` from a fixed table of 30 values
/// (10 aspect ratios x 3 quality tiers). Ratios/tiers outside that table
/// return `None`, which omits the field and lets the API use `"auto"`.
fn apiyi_gpt_image_2_vip_size(ar: Option<RouterAspectRatio>, res: Option<RouterResolution>) -> Option<String> {
  let ratio_key = match ar? {
    RouterAspectRatio::Square | RouterAspectRatio::SquareHd => "1:1",
    RouterAspectRatio::TallTwoByThree | RouterAspectRatio::Tall => "2:3",
    RouterAspectRatio::WideThreeByTwo | RouterAspectRatio::Wide => "3:2",
    RouterAspectRatio::TallThreeByFour => "3:4",
    RouterAspectRatio::WideFourByThree => "4:3",
    RouterAspectRatio::TallFourByFive => "4:5",
    RouterAspectRatio::WideFiveByFour => "5:4",
    RouterAspectRatio::TallNineBySixteen => "9:16",
    RouterAspectRatio::WideSixteenByNine => "16:9",
    RouterAspectRatio::WideTwentyOneByNine => "21:9",
    RouterAspectRatio::Auto
    | RouterAspectRatio::TallNineByTwentyOne
    | RouterAspectRatio::Auto2k
    | RouterAspectRatio::Auto3k
    | RouterAspectRatio::Auto4k => return None,
  };

  let tier = match res {
    Some(RouterResolution::FourK) => "4k",
    Some(RouterResolution::ThreeK) | Some(RouterResolution::TwoK) | Some(RouterResolution::TenEightyP) => "2k",
    _ => "1k",
  };

  let size = match (ratio_key, tier) {
    ("1:1", "1k") => "1280x1280",
    ("1:1", "2k") => "2048x2048",
    ("1:1", "4k") => "2880x2880",
    ("2:3", "1k") => "848x1280",
    ("2:3", "2k") => "1360x2048",
    ("2:3", "4k") => "2336x3520",
    ("3:2", "1k") => "1280x848",
    ("3:2", "2k") => "2048x1360",
    ("3:2", "4k") => "3520x2336",
    ("3:4", "1k") => "960x1280",
    ("3:4", "2k") => "1536x2048",
    ("3:4", "4k") => "2480x3312",
    ("4:3", "1k") => "1280x960",
    ("4:3", "2k") => "2048x1536",
    ("4:3", "4k") => "3312x2480",
    ("4:5", "1k") => "1024x1280",
    ("4:5", "2k") => "1632x2048",
    ("4:5", "4k") => "2560x3216",
    ("5:4", "1k") => "1280x1024",
    ("5:4", "2k") => "2048x1632",
    ("5:4", "4k") => "3216x2560",
    ("9:16", "1k") => "720x1280",
    ("9:16", "2k") => "1152x2048",
    ("9:16", "4k") => "2160x3840",
    ("16:9", "1k") => "1280x720",
    ("16:9", "2k") => "2048x1152",
    ("16:9", "4k") => "3840x2160",
    ("21:9", "1k") => "1280x544",
    ("21:9", "2k") => "2048x864",
    ("21:9", "4k") => "3840x1632",
    _ => return None,
  };

  Some(size.to_string())
}

// ── Shared helpers ──

async fn resolve_image_inputs(
  request: &TauriGenerateImageRequest,
  api_host: &ApiHost,
) -> Result<Option<ImageListRef>, GenerateError> {
  let mut tokens = Vec::new();

  if let Some(canvas_token) = &request.canvas_image_media_token {
    tokens.push(canvas_token.clone());
  }
  if let Some(scene_token) = &request.scene_image_media_token {
    tokens.push(scene_token.clone());
  }
  if let Some(media_tokens) = &request.image_media_tokens {
    tokens.extend(media_tokens.clone());
  }

  if tokens.is_empty() {
    return Ok(None);
  }

  let urls = map_media_file_tokens_to_cdn_urls(&tokens, api_host).await?;
  Ok(Some(ImageListRef::Urls(urls)))
}

async fn get_image_urls_from_request(
  request: &TauriGenerateImageRequest,
  api_host: &ApiHost,
) -> Result<Vec<String>, GenerateError> {
  match resolve_image_inputs(request, api_host).await? {
    Some(ImageListRef::Urls(urls)) => Ok(urls),
    _ => Ok(vec![]),
  }
}

async fn download_image_bytes(url: &str) -> Result<Vec<u8>, GenerateError> {
  let response = reqwest::get(url).await
    .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Download failed for {}: {:?}", url, e)))?;
  let bytes = response.bytes().await
    .map_err(|e| GenerateError::AnyhowError(anyhow::anyhow!("Read bytes failed: {:?}", e)))?;
  Ok(bytes.to_vec())
}

async fn download_and_encode_images(
  urls: &[String],
) -> Result<Vec<(String, String)>, GenerateError> {
  let mut result = Vec::with_capacity(urls.len());
  for url in urls {
    let bytes = download_image_bytes(url).await?;
    let encoded = BASE64_STANDARD.encode(&bytes);
    result.push(("image/png".to_string(), encoded));
  }
  Ok(result)
}

/// Create a prompt record in the Artcraft backend before sending the generation request.
/// Fails open: if prompt creation fails, we log and return None rather than blocking generation.
async fn create_prompt_record(
  router_request: &GenerateImageRequestBuilder,
  api_host: &ApiHost,
  storyteller_creds_manager: &StorytellerCredentialManager,
) -> Option<PromptToken> {
  let creds = match storyteller_creds_manager.get_credentials() {
    Ok(Some(creds)) => creds,
    _ => {
      warn!("[Router] No Storyteller credentials available, skipping prompt creation");
      return None;
    }
  };

  let prompt_request = router_image_request_to_artcraft_prompt(router_request);

  match create_prompt(api_host, Some(&creds), prompt_request).await {
    Ok(response) => {
      info!("[Router] Created prompt: {:?}", response.prompt_token);
      Some(response.prompt_token)
    }
    Err(err) => {
      error!("[Router] Failed to create prompt (continuing anyway): {:?}", err);
      None
    }
  }
}

fn build_task_enqueue_success_fal(
  tauri_model: TauriImageModel,
  response: GenerateImageResponse,
  maybe_prompt_token: Option<PromptToken>,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let fal_payload = response.get_fal_payload()
    .ok_or(GenerateError::ResponseHadNoJobTokens)?;

  let provider_job_id = fal_payload.request_id
    .or(fal_payload.gateway_request_id)
    .ok_or(GenerateError::ResponseHadNoJobTokens)?;

  let generation_model = tauri_image_model_to_generation_model(tauri_model);

  Ok(TaskEnqueueSuccess {
    task_type: TaskType::ImageGeneration,
    model: Some(generation_model),
    provider: GenerationProvider::Fal,
    provider_job_id: Some(provider_job_id),
    maybe_queue_status_url: fal_payload.maybe_status_url,
    maybe_queue_response_url: fal_payload.maybe_response_url,
    maybe_prompt_token,
  })
}

fn build_task_enqueue_success_runninghub(
  tauri_model: TauriImageModel,
  response: GenerateImageResponse,
  maybe_prompt_token: Option<PromptToken>,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let runninghub_payload = response.get_runninghub_payload()
    .ok_or(GenerateError::ResponseHadNoJobTokens)?;

  let generation_model = tauri_image_model_to_generation_model(tauri_model);

  // Store the final image URL in queue_response_url so the polling thread can download it.
  Ok(TaskEnqueueSuccess {
    task_type: TaskType::ImageGeneration,
    model: Some(generation_model),
    provider: GenerationProvider::Runninghub,
    provider_job_id: Some(runninghub_payload.task_id.clone()),
    maybe_queue_status_url: None,
    maybe_queue_response_url: Some(runninghub_payload.image_url.clone()),
    maybe_prompt_token,
  })
}
