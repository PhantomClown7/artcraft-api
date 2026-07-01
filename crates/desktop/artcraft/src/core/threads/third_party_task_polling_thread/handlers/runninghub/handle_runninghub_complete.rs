use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::data_dir::trait_data_subdir::DataSubdir;
use crate::core::state::task_database::TaskDatabase;
use crate::core::threads::third_party_task_polling_thread::events::notify_frontend_of_completion::{
  notify_frontend_of_completion, CompletionData,
};
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;
use artcraft_client::endpoints::media_files::get_media_file::get_media_file;
use artcraft_client::endpoints::media_files::upload_image_media_file_from_file::{
  upload_image_media_file_from_file, UploadImageFromFileArgs,
};
use artcraft_client::endpoints::media_files::upload_video_media_file_from_file::{
  upload_video_media_file_from_file, UploadVideoFromFileArgs,
};
use artcraft_api_defs::utils::media_links_to_thumbnail_template::media_links_to_thumbnail_template;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_media_file_class::TaskMediaFileClass;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use sqlite_tasks::queries::update_successful_task_status_with_metadata::{
  update_successful_task_status_with_metadata, UpdateSuccessfulTaskArgs,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tauri::AppHandle;
use tokens::tokens::media_files::MediaFileToken;
use uuid_utils::uuid::generate_random_uuid;

pub async fn handle_runninghub_complete(
  app_handle: &AppHandle,
  app_env_configs: &AppEnvConfigs,
  app_data_root: &AppDataRoot,
  task_database: &TaskDatabase,
  storyteller_creds_manager: &StorytellerCredentialManager,
  task: &Task,
) {
  info!("[RunninghubComplete] Handling completed task {}", task.id.as_str());

  let result = handle_runninghub_complete_inner(
    app_handle,
    app_env_configs,
    app_data_root,
    task_database,
    storyteller_creds_manager,
    task,
  ).await;

  if let Err(err) = result {
    error!("[RunninghubComplete] Failed to handle task {}: {:?}", task.id.as_str(), err);
  }
}

async fn handle_runninghub_complete_inner(
  app_handle: &AppHandle,
  app_env_configs: &AppEnvConfigs,
  app_data_root: &AppDataRoot,
  task_database: &TaskDatabase,
  storyteller_creds_manager: &StorytellerCredentialManager,
  task: &Task,
) -> Result<(), Box<dyn std::error::Error>> {
  let media_url = match &task.queue_response_url {
    Some(url) if !url.is_empty() => url.clone(),
    _ => {
      warn!("[RunninghubComplete] Task {} has no queue_response_url, skipping", task.id.as_str());
      return Ok(());
    }
  };

  let creds = storyteller_creds_manager.get_credentials()?
    .ok_or("No Storyteller credentials available")?;

  let is_video = task.task_type == TaskType::VideoGeneration;
  let media_class = if is_video { TaskMediaFileClass::Video } else { TaskMediaFileClass::Image };

  info!("[RunninghubComplete] Downloading {} from: {}", if is_video { "video" } else { "image" }, media_url);
  let download_path = download_file(&media_url, app_data_root, is_video).await?;

  info!("[RunninghubComplete] Uploading to CDN...");
  let media_file_token: MediaFileToken = if is_video {
    let upload_result = upload_video_media_file_from_file(UploadVideoFromFileArgs {
      api_host: &app_env_configs.storyteller_host,
      maybe_creds: Some(&creds),
      path: &download_path,
      maybe_prompt_token: None,
      maybe_generation_provider: Some(GenerationProvider::Runninghub),
    }).await?;
    upload_result.media_file_token
  } else {
    let upload_result = upload_image_media_file_from_file(UploadImageFromFileArgs {
      api_host: &app_env_configs.storyteller_host,
      maybe_creds: Some(&creds),
      path: &download_path,
      is_intermediate_system_file: false,
      maybe_prompt_token: None,
      maybe_batch_token: None,
      maybe_generation_provider: Some(GenerationProvider::Runninghub),
    }).await?;
    upload_result.media_file_token
  };

  info!("[RunninghubComplete] Uploaded as media file: {:?}", media_file_token);

  let mut maybe_cdn_url = None;
  let mut maybe_cdn_url_str = None;
  let mut maybe_thumbnail_url_template = None;

  match get_media_file(&app_env_configs.storyteller_host, &media_file_token).await {
    Ok(response) => {
      maybe_cdn_url = Some(response.media_file.media_links.cdn_url.clone());
      maybe_cdn_url_str = Some(response.media_file.media_links.cdn_url.to_string());
      maybe_thumbnail_url_template = media_links_to_thumbnail_template(&response.media_file.media_links)
        .map(|s| s.to_string());
    }
    Err(err) => {
      error!("[RunninghubComplete] Failed to look up media file after upload: {:?}", err);
    }
  }

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs {
    db: task_database.get_connection(),
    task_id: &task.id,
    maybe_batch_token: None,
    maybe_primary_media_file_token: Some(&media_file_token),
    maybe_primary_media_file_class: Some(media_class),
    maybe_primary_media_file_cdn_url: maybe_cdn_url_str.as_deref(),
    maybe_primary_media_file_thumbnail_url_template: maybe_thumbnail_url_template.as_deref(),
  }).await?;

  if updated {
    let completion = CompletionData {
      primary_media_file_token: media_file_token,
      maybe_cdn_url,
      maybe_thumbnail_url_template,
      maybe_batch_token: None,
      media_class,
    };

    notify_frontend_of_completion(
      app_handle,
      &app_env_configs.storyteller_host,
      Some(&creds),
      task,
      &completion,
    ).await;
  }

  info!("[RunninghubComplete] Task {} fully handled", task.id.as_str());
  Ok(())
}

async fn download_file(
  url: &str,
  app_data_root: &AppDataRoot,
  is_video: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
  let response = reqwest::get(url).await?;
  let bytes = response.bytes().await?;

  let fallback_extension = if is_video { "mp4" } else { "png" };
  let extension = url_utils::download_extension::extract_download_extension_from_url::extract_download_extension_from_url_str(url)
    .map(|ext| ext.as_extension_without_period())
    .unwrap_or(fallback_extension);

  let tempdir = app_data_root.temp_dir().path();
  let filename = format!("runninghub_{}.{}", generate_random_uuid(), extension);
  let download_path = tempdir.join(filename);

  let mut file = File::create(&download_path)?;
  file.write_all(&bytes)?;

  Ok(download_path)
}
