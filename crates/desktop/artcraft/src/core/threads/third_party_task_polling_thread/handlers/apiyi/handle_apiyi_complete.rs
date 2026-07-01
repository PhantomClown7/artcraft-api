use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::task_database::TaskDatabase;
use crate::core::threads::third_party_task_polling_thread::events::notify_frontend_of_completion::{
  notify_frontend_of_completion, CompletionData,
};
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;
use artcraft_api_defs::utils::media_links_to_thumbnail_template::media_links_to_thumbnail_template;
use artcraft_client::endpoints::media_files::get_media_file::get_media_file;
use enums::tauri::tasks::task_media_file_class::TaskMediaFileClass;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use sqlite_tasks::queries::update_successful_task_status_with_metadata::{
  update_successful_task_status_with_metadata, UpdateSuccessfulTaskArgs,
};
use tauri::AppHandle;
use tokens::tokens::media_files::MediaFileToken;

/// Handle an Apiyi task that has already been completed synchronously.
///
/// The image was already uploaded to CDN during the generate command.
/// The `provider_job_id` holds the media file token. This handler looks up
/// the CDN URL and marks the task as complete so the frontend is notified.
pub async fn handle_apiyi_complete(
  app_handle: &AppHandle,
  app_env_configs: &AppEnvConfigs,
  task_database: &TaskDatabase,
  storyteller_creds_manager: &StorytellerCredentialManager,
  task: &Task,
) {
  info!("[ApiyiComplete] Handling completed task {}", task.id.as_str());

  let result = handle_apiyi_complete_inner(
    app_handle,
    app_env_configs,
    task_database,
    storyteller_creds_manager,
    task,
  ).await;

  if let Err(err) = result {
    error!("[ApiyiComplete] Failed to handle task {}: {:?}", task.id.as_str(), err);
  }
}

async fn handle_apiyi_complete_inner(
  app_handle: &AppHandle,
  app_env_configs: &AppEnvConfigs,
  task_database: &TaskDatabase,
  storyteller_creds_manager: &StorytellerCredentialManager,
  task: &Task,
) -> Result<(), Box<dyn std::error::Error>> {
  let job_id = match &task.provider_job_id {
    Some(id) if !id.is_empty() => id.clone(),
    _ => {
      warn!("[ApiyiComplete] Task {} has no provider_job_id (media file token), skipping", task.id.as_str());
      return Ok(());
    }
  };

  let media_file_token = MediaFileToken::new_from_str(&job_id);

  let creds = storyteller_creds_manager.get_credentials()?
    .ok_or("No Storyteller credentials available")?;

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
      error!("[ApiyiComplete] Failed to look up media file {}: {:?}", job_id, err);
      return Err(Box::new(err));
    }
  }

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs {
    db: task_database.get_connection(),
    task_id: &task.id,
    maybe_batch_token: None,
    maybe_primary_media_file_token: Some(&media_file_token),
    maybe_primary_media_file_class: Some(TaskMediaFileClass::Image),
    maybe_primary_media_file_cdn_url: maybe_cdn_url_str.as_deref(),
    maybe_primary_media_file_thumbnail_url_template: maybe_thumbnail_url_template.as_deref(),
  }).await?;

  if updated {
    let completion = CompletionData {
      primary_media_file_token: media_file_token,
      maybe_cdn_url,
      maybe_thumbnail_url_template,
      maybe_batch_token: None,
      media_class: TaskMediaFileClass::Image,
    };

    notify_frontend_of_completion(
      app_handle,
      &app_env_configs.storyteller_host,
      Some(&creds),
      task,
      &completion,
    ).await;
  }

  info!("[ApiyiComplete] Task {} fully handled", task.id.as_str());
  Ok(())
}
