mod commands;
mod cron;
mod errors;
mod state;

use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_window_state::Builder as WindowStateBuilder;

use commands::agents::get_agents;
use commands::config::{get_config, load_config, save_config};
use commands::cron::{create_cron_job, delete_cron_job, list_cron_jobs, run_cron_job_now, toggle_cron_job};
use commands::integrations::get_pm_tasks;
use commands::launcher::{launch_target, open_github};
use commands::notes::{delete_note, list_notes, save_note};
use state::AppState;

fn config_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(app_data).join("umbra").join("config.json")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(WindowStateBuilder::default().build())
        .setup(|app| {
            let path = config_path();
            let config = tauri::async_runtime::block_on(load_config(&path));
            let state = AppState::new(config, path);

            // Clone before moving into Tauri — both share the same Arc<RwLock<AppConfig>>
            let state_for_cron = state.clone();
            app.manage(state);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                cron::start_scheduler(app_handle, state_for_cron).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_agents,
            list_notes,
            save_note,
            delete_note,
            launch_target,
            open_github,
            get_pm_tasks,
            list_cron_jobs,
            create_cron_job,
            toggle_cron_job,
            delete_cron_job,
            run_cron_job_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UMBRA");
}
