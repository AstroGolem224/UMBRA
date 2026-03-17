use chrono::Utc;
use tauri::{AppHandle, Emitter, State};
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

use crate::commands::config::CronJobConfig;
use crate::errors::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_cron_jobs(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<CronJobConfig>, AppError> {
    Ok(state.config.read().await.cron_jobs.clone())
}

#[tauri::command]
pub async fn create_cron_job(
    name: String,
    schedule: String,
    command: String,
    state: State<'_, AppState>,
) -> std::result::Result<CronJobConfig, AppError> {
    let job = CronJobConfig {
        id: Uuid::new_v4().to_string(),
        name,
        schedule,
        command,
        enabled: true,
        last_run: None,
        last_status: "pending".into(),
        last_output: None,
    };

    let json = {
        let mut cfg = state.config.write().await;
        cfg.cron_jobs.push(job.clone());
        serde_json::to_string_pretty(&*cfg)?
    };
    persist(&state.config_path, &json).await?;
    Ok(job)
}

#[tauri::command]
pub async fn toggle_cron_job(
    id: String,
    state: State<'_, AppState>,
) -> std::result::Result<bool, AppError> {
    let (json, new_state) = {
        let mut cfg = state.config.write().await;
        let job = cfg
            .cron_jobs
            .iter_mut()
            .find(|j| j.id == id)
            .ok_or_else(|| AppError::Other(format!("cron job {id} not found")))?;
        job.enabled = !job.enabled;
        let s = job.enabled;
        (serde_json::to_string_pretty(&*cfg)?, s)
    };
    persist(&state.config_path, &json).await?;
    Ok(new_state)
}

#[tauri::command]
pub async fn delete_cron_job(
    id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let json = {
        let mut cfg = state.config.write().await;
        cfg.cron_jobs.retain(|j| j.id != id);
        serde_json::to_string_pretty(&*cfg)?
    };
    persist(&state.config_path, &json).await?;
    Ok(())
}

#[tauri::command]
pub async fn run_cron_job_now(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<String, AppError> {
    let cmd_str = {
        let cfg = state.config.read().await;
        cfg.cron_jobs
            .iter()
            .find(|j| j.id == id)
            .map(|j| j.command.clone())
            .ok_or_else(|| AppError::Other(format!("cron job {id} not found")))?
    };

    let output = TokioCommand::new("cmd")
        .args(["/C", &cmd_str])
        .output()
        .await
        .map_err(AppError::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let ok = output.status.success();
    let status = if ok { "ok" } else { "error" };
    let combined = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        format!("{}\n{}", stdout.trim(), stderr.trim())
    };

    let now = Utc::now().to_rfc3339();
    let json = {
        let mut cfg = state.config.write().await;
        if let Some(job) = cfg.cron_jobs.iter_mut().find(|j| j.id == id) {
            job.last_run = Some(now);
            job.last_status = status.into();
            job.last_output = Some(combined.clone());
        }
        serde_json::to_string_pretty(&*cfg)?
    };
    persist(&state.config_path, &json).await?;

    let _ = app.emit(
        "cron-job-ran",
        serde_json::json!({ "id": id, "status": status, "output": combined }),
    );

    Ok(combined)
}

async fn persist(path: &std::path::Path, json: &str) -> std::result::Result<(), AppError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, json).await?;
    Ok(())
}
