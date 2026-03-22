mod commands;
mod cron;
mod errors;
mod state;
mod types;
mod uap;

use std::path::PathBuf;
use tauri::{include_image, Manager};
use tauri_plugin_window_state::Builder as WindowStateBuilder;
#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

use commands::agents::{add_agent, custom_agent_to_agent, default_agents, get_agents, push_agent_task, remove_agent};
use commands::config::{get_config, load_config, save_config};
use commands::cron::{create_cron_job, delete_cron_job, list_agent_cron_jobs, list_cron_jobs, run_cron_job_now, toggle_cron_job};
use commands::github::{get_github_repos, list_user_repos};
use commands::integrations::{add_pm_comment, create_pm_task, get_pm_columns, get_pm_projects, get_pm_tasks, move_pm_task, reorder_pm_tasks, update_pm_task};
use commands::launcher::{launch_target, open_github, open_github_url, open_local_repo_folder, open_local_repo_terminal};
use commands::notes::{delete_note, list_notes, save_note};
use commands::plugins::{get_obsidian_stats, get_tmlite_tasks, list_skills};
use commands::updates::{check_for_updates, install_pending_update, PendingUpdate};
use state::AppState;
use uap::start_uap_server;

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
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            let path = config_path();
            let config = tauri::async_runtime::block_on(load_config(&path));
            let uap_port = config.uap_port;
            let uap_token = config.uap_token.clone();
            let state = AppState::new(config, path);
            app.manage(PendingUpdate::default());

            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(include_image!("icons/icon.png"));
                let _ = apply_mica(&window, None);
            }

            // Seed registry: built-in defaults + any persisted custom agents.
            tauri::async_runtime::block_on(async {
                let custom = state.config.read().await.custom_agents.clone();
                let mut agents = state.agent_registry.agents.write().await;
                for agent in default_agents() {
                    agents.insert(agent.id.clone(), agent);
                }
                for c in &custom {
                    agents.insert(c.id.clone(), custom_agent_to_agent(c));
                }
            });

            let state_for_cron = state.clone();
            let state_for_uap = state.clone();
            app.manage(state);

            let app_handle = app.handle().clone();

            // Cron scheduler
            tauri::async_runtime::spawn(async move {
                cron::start_scheduler(app_handle, state_for_cron).await;
            });

            // UAP heartbeat server
            let app_handle_uap = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_uap_server(
                    app_handle_uap,
                    state_for_uap.agent_registry,
                    uap_token,
                    uap_port,
                )
                .await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_agents,
            add_agent,
            remove_agent,
            push_agent_task,
            list_notes,
            save_note,
            delete_note,
            launch_target,
            open_github,
            open_github_url,
            open_local_repo_folder,
            open_local_repo_terminal,
            get_pm_tasks,
            get_pm_projects,
            get_pm_columns,
            create_pm_task,
            move_pm_task,
            reorder_pm_tasks,
            update_pm_task,
            add_pm_comment,
            list_cron_jobs,
            list_agent_cron_jobs,
            create_cron_job,
            toggle_cron_job,
            delete_cron_job,
            run_cron_job_now,
            get_github_repos,
            list_user_repos,
            get_obsidian_stats,
            get_tmlite_tasks,
            list_skills,
            check_for_updates,
            install_pending_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UMBRA");
}
