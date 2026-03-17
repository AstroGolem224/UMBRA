use tauri::{AppHandle, Emitter};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::state::AppState;

pub async fn start_scheduler(app: AppHandle, state: AppState) {
    let sched = match JobScheduler::new().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[cron] Failed to create scheduler: {e}");
            return;
        }
    };

    // PM Tool polling job — clones are cheap (Arc inside)
    let app_clone = app.clone();
    let state_clone = state.clone();
    let job = Job::new_async("every 30 seconds", move |_uuid, _lock| {
        let app = app_clone.clone();
        let state = state_clone.clone();
        Box::pin(async move {
            poll_pm_tool(&app, &state).await;
        })
    });

    match job {
        Ok(j) => {
            if let Err(e) = sched.add(j).await {
                eprintln!("[cron] Failed to add PM poll job: {e}");
            }
        }
        Err(e) => eprintln!("[cron] Failed to create PM poll job: {e}"),
    }

    if let Err(e) = sched.start().await {
        eprintln!("[cron] Failed to start scheduler: {e}");
    }
}

async fn poll_pm_tool(app: &AppHandle, state: &AppState) {
    let cfg = state.config.read().await;
    let url = format!("{}/api/tasks", cfg.pm_tool_url);
    drop(cfg);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let _ = app.emit("tasks-updated", json);
            }
        }
        Ok(resp) => {
            eprintln!("[cron] PM tool returned {}", resp.status());
        }
        Err(e) => {
            eprintln!("[cron] PM tool unreachable: {e}");
        }
    }
}
