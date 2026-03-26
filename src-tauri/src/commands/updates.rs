use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State, Url};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub configured: bool,
    pub update_available: bool,
    pub current_version: String,
    pub version: Option<String>,
}

fn collect_endpoints(raw: &str) -> Result<Vec<Url>, AppError> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| Url::parse(line).map_err(|error| AppError::Other(error.to_string())))
        .collect()
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, AppState>,
    pending_update: State<'_, PendingUpdate>,
) -> Result<UpdateCheckResult, AppError> {
    let config = state.config.read().await.clone();
    let endpoints = collect_endpoints(&config.updater_endpoint)?;
    let public_key = config.updater_public_key.trim().to_string();
    let current_version = app.package_info().version.to_string();

    if endpoints.is_empty() || public_key.is_empty() {
        if let Ok(mut guard) = pending_update.0.lock() {
            *guard = None;
        }
        return Ok(UpdateCheckResult {
            configured: false,
            update_available: false,
            current_version,
            version: None,
        });
    }

    let update = app
        .updater_builder()
        .endpoints(endpoints)
        .map_err(|error| AppError::Other(error.to_string()))?
        .pubkey(public_key)
        .build()
        .map_err(|error| AppError::Other(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::Other(error.to_string()))?;

    let result = UpdateCheckResult {
        configured: true,
        update_available: update.is_some(),
        current_version,
        version: update.as_ref().map(|item| item.version.clone()),
    };

    match pending_update.0.lock() {
        Ok(mut guard) => *guard = update,
        Err(_) => return Err(AppError::Other("failed to acquire update lock".into())),
    }

    Ok(result)
}

#[tauri::command]
pub async fn install_pending_update(
    pending_update: State<'_, PendingUpdate>,
) -> Result<bool, AppError> {
    let update = match pending_update.0.lock() {
        Ok(mut guard) => guard.take(),
        Err(_) => return Err(AppError::Other("failed to acquire update lock".into())),
    };
    let Some(update) = update else {
        return Ok(false);
    };

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| AppError::Other(error.to_string()))?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_runtime_endpoints_from_multiline_config() {
        let endpoints =
            collect_endpoints("https://a.example/latest.json\n\n https://b.example/latest.json ")
                .unwrap();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(
            endpoints[0],
            Url::parse("https://a.example/latest.json").unwrap()
        );
        assert_eq!(
            endpoints[1],
            Url::parse("https://b.example/latest.json").unwrap()
        );
    }

    #[test]
    fn rejects_invalid_endpoint_urls() {
        let error = collect_endpoints("not-a-url").unwrap_err().to_string();
        assert!(error.contains("relative URL without a base") || error.contains("invalid"));
    }
}
