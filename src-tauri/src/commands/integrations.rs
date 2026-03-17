use std::time::Duration;
use tauri::State;

use crate::errors::AppError;
use crate::state::AppState;

/// Manual one-shot fetch of PM Tool tasks.
/// The cron scheduler polls automatically every 30s and emits `tasks-updated`.
/// This command is for the SYNC button in TasksView.
#[tauri::command]
pub async fn get_pm_tasks(
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/tasks", cfg.pm_tool_url)
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .unwrap_or_default();

    let resp = client.get(&url).send().await?;
    let tasks = resp.json::<serde_json::Value>().await?;
    Ok(tasks)
}
