mod commands;
pub mod credentials;
mod cron;
mod errors;
mod state;
mod types;
mod uap;
mod workbench;

use std::path::PathBuf;
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{include_image, AppHandle, Emitter, Manager, Runtime, Window, WindowEvent};
use tauri_plugin_window_state::Builder as WindowStateBuilder;
#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

use commands::agents::{
    add_agent, custom_agent_to_agent, default_agents, get_agents, push_agent_task, remove_agent,
};
use commands::config::{get_config, load_config, save_config};
use commands::cron::{
    create_cron_job, delete_cron_job, list_agent_cron_jobs, list_cron_jobs, run_cron_job_now,
    toggle_cron_job,
};
use commands::github::{get_github_repos, list_user_repos};
use commands::integrations::{
    add_pm_comment, create_pm_task, get_pm_columns, get_pm_projects, get_pm_tasks, move_pm_task,
    reorder_pm_tasks, update_pm_task,
};
use commands::launcher::{
    launch_target, open_github, open_github_url, open_local_repo_folder, open_local_repo_terminal,
};
use commands::notes::{delete_note, list_notes, save_note, save_note_attachment};
use commands::ops_room::{
    advance_ops_session, create_ops_channel, create_ops_job, list_ops_channel_messages,
    list_ops_channel_messages_page, list_ops_channels, list_ops_jobs, list_ops_route_approvals,
    list_ops_rules, list_ops_session_templates, list_ops_sessions, pause_ops_session,
    resolve_ops_route_approval, save_ops_rule, save_ops_session_template,
    send_ops_channel_message, start_ops_session,
};
use commands::plugins::{create_tmlite_task, get_obsidian_stats, get_tmlite_tasks, list_skills};
use commands::updates::{check_for_updates, install_pending_update, PendingUpdate};
use commands::workbench::{
    append_run_event, bootstrap_provider_workspace, cancel_dispatch_run, create_dispatch_run,
    check_provider_setup, get_dispatch_run, get_provider_auth_state, get_provider_env_template,
    list_channel_dispatch_runs, list_dispatch_runs, list_run_artifacts, list_run_events, list_run_events_page,
    list_workspace_presets, probe_provider_command, recover_incomplete_runs,
    recover_incomplete_runs_internal, retry_dispatch_run, smoke_test_provider_command,
    reveal_run_path, open_workspace_folder, open_workspace_terminal,
};
use state::AppState;
use types::{Agent, AgentStatus};
use uap::{start_uap_server, AgentRegistry};

fn config_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(app_data).join("umbra").join("config.json")
}

const TRAY_ID: &str = "umbra-tray";
const TRAY_HEALTH_SUMMARY: &str = "tray-health-summary";
const TRAY_HEALTH_COUNTS: &str = "tray-health-counts";
const TRAY_SHOW: &str = "tray-show";
const TRAY_HIDE: &str = "tray-hide";
const TRAY_SYNC_PM: &str = "tray-sync-pm";
const TRAY_QUIT: &str = "tray-quit";

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayHealthSnapshot {
    level: TrayHealthLevel,
    summary: String,
    counts: String,
    tooltip: String,
    agent_rows: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayHealthLevel {
    Healthy,
    Attention,
    Offline,
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn close_to_tray_enabled<R: Runtime>(window: &Window<R>) -> bool {
    let state = window.state::<AppState>();
    tauri::async_runtime::block_on(async { state.config.read().await.close_to_tray })
}

fn tray_status_label(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Online => "online",
        AgentStatus::Working => "working",
        AgentStatus::Idle => "idle",
        AgentStatus::Offline => "offline",
        AgentStatus::Error => "error",
    }
}

pub(crate) fn normalized_agent_status(agent: &Agent) -> AgentStatus {
    if agent.is_stale() && agent.status != AgentStatus::Offline {
        AgentStatus::Offline
    } else {
        agent.status.clone()
    }
}

fn default_agent_order(id: &str) -> usize {
    match id {
        "forge" => 0,
        "prism" => 1,
        _ => usize::MAX,
    }
}

fn tray_health_snapshot_from_agents(agents: &[Agent]) -> TrayHealthSnapshot {
    let mut working = 0usize;
    let mut online = 0usize;
    let mut idle = 0usize;
    let mut offline = 0usize;
    let mut errors = 0usize;

    let mut sorted_agents = agents.to_vec();
    sorted_agents.sort_by_key(|agent| (default_agent_order(&agent.id), agent.name.clone()));

    for agent in &sorted_agents {
        match normalized_agent_status(agent) {
            AgentStatus::Working => working += 1,
            AgentStatus::Online => online += 1,
            AgentStatus::Idle => idle += 1,
            AgentStatus::Offline => offline += 1,
            AgentStatus::Error => errors += 1,
        }
    }

    let mut agent_rows = if sorted_agents.is_empty() {
        vec!["agents: none registered".into()]
    } else {
        sorted_agents
            .iter()
            .take(6)
            .map(|agent| {
                let status = normalized_agent_status(agent);
                format!("{}: {}", agent.name, tray_status_label(&status))
            })
            .collect::<Vec<_>>()
    };

    if sorted_agents.len() > 6 {
        let hidden = sorted_agents.len() - 6;
        agent_rows.push(format!("... and {hidden} more"));
    }

    finalize_tray_snapshot(
        agent_rows,
        working,
        online,
        idle,
        offline,
        errors,
        sorted_agents.len(),
    )
}

fn finalize_tray_snapshot(
    agent_rows: Vec<String>,
    working: usize,
    online: usize,
    idle: usize,
    offline: usize,
    errors: usize,
    total_agents: usize,
) -> TrayHealthSnapshot {
    let active = working + online + idle;
    let level = if total_agents == 0 || active == 0 {
        TrayHealthLevel::Offline
    } else if errors > 0 || offline > 0 {
        TrayHealthLevel::Attention
    } else {
        TrayHealthLevel::Healthy
    };
    let fleet = match level {
        TrayHealthLevel::Healthy => "healthy",
        TrayHealthLevel::Attention => "attention",
        TrayHealthLevel::Offline => "offline",
    };

    TrayHealthSnapshot {
        level,
        summary: format!("fleet health: {fleet}"),
        counts: format!(
            "active {active}/{total_agents} | working {working} | online {online} | idle {idle} | offline {offline} | errors {errors}"
        ),
        tooltip: format!(
            "UMBRA | {fleet} | active {active}/{total_agents} | working {working} | offline {offline} | errors {errors}"
        ),
        agent_rows,
    }
}

fn tray_icon_for_level(level: TrayHealthLevel) -> tauri::image::Image<'static> {
    match level {
        TrayHealthLevel::Healthy => include_image!("icons/tray-healthy.png"),
        TrayHealthLevel::Attention => include_image!("icons/tray-attention.png"),
        TrayHealthLevel::Offline => include_image!("icons/tray-offline.png"),
    }
}

async fn collect_tray_health_snapshot(registry: &AgentRegistry) -> TrayHealthSnapshot {
    let agents = registry.agents.read().await;
    let cloned_agents = agents.values().cloned().collect::<Vec<_>>();
    tray_health_snapshot_from_agents(&cloned_agents)
}

fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &TrayHealthSnapshot,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let summary = MenuItemBuilder::with_id(TRAY_HEALTH_SUMMARY, &snapshot.summary)
        .enabled(false)
        .build(app)?;
    let counts = MenuItemBuilder::with_id(TRAY_HEALTH_COUNTS, &snapshot.counts)
        .enabled(false)
        .build(app)?;

    let mut builder = MenuBuilder::new(app)
        .item(&summary)
        .item(&counts)
        .separator();

    for (index, row) in snapshot.agent_rows.iter().enumerate() {
        let row_item = MenuItemBuilder::with_id(format!("tray-agent-{index}"), row)
            .enabled(false)
            .build(app)?;
        builder = builder.item(&row_item);
    }

    builder
        .separator()
        .text(TRAY_SHOW, "show umbra")
        .text(TRAY_HIDE, "hide window")
        .separator()
        .text(TRAY_SYNC_PM, "sync pm now")
        .separator()
        .text(TRAY_QUIT, "quit")
        .build()
}

fn apply_tray_snapshot<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &TrayHealthSnapshot,
) -> tauri::Result<()> {
    let menu = build_tray_menu(app, snapshot)?;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(snapshot.tooltip.as_str()))?;
        tray.set_icon(Some(tray_icon_for_level(snapshot.level)))?;
        return Ok(());
    }

    let _ = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(&snapshot.tooltip)
        .icon(tray_icon_for_level(snapshot.level))
        .build(app)?;

    Ok(())
}

pub(crate) async fn sync_tray_with_agent_registry<R: Runtime>(
    app: &AppHandle<R>,
    registry: &AgentRegistry,
) -> tauri::Result<()> {
    let snapshot = collect_tray_health_snapshot(registry).await;
    apply_tray_snapshot(app, &snapshot)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(WindowStateBuilder::default().build())
        .on_menu_event(|app, event| {
            let id = event.id();
            if id == TRAY_SHOW {
                show_main_window(app);
            } else if id == TRAY_HIDE {
                hide_main_window(app);
            } else if id == TRAY_SYNC_PM {
                let _ = app.emit(TRAY_SYNC_PM, ());
                show_main_window(app);
            } else if id == TRAY_QUIT {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|app, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_main_window(app),
            _ => {}
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                if close_to_tray_enabled(window) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            let path = config_path();
            let config = tauri::async_runtime::block_on(load_config(&path));
            let uap_port = config.uap_port;
            let state = AppState::new(config, path)?;
            let state_for_seed = state.clone();
            let state_for_cron = state.clone();
            let state_for_uap = state.clone();
            app.manage(PendingUpdate::default());
            app.manage(state.clone());

            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(include_image!("icons/icon.png"));
                let _ = apply_mica(&window, None);
            }

            // Seed registry: built-in defaults + any persisted custom agents.
            tauri::async_runtime::block_on(async {
                let custom = state_for_seed.config.read().await.custom_agents.clone();
                let mut agents = state_for_seed.agent_registry.agents.write().await;
                for agent in default_agents() {
                    agents.insert(agent.id.clone(), agent);
                }
                for c in &custom {
                    agents.insert(c.id.clone(), custom_agent_to_agent(c));
                }
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(sync_tray_with_agent_registry(
                &app_handle,
                &state_for_seed.agent_registry,
            ))?;

            let tray_app_handle = app.handle().clone();
            let tray_registry = state_for_seed.agent_registry.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let _ = sync_tray_with_agent_registry(&tray_app_handle, &tray_registry).await;
                }
            });

            recover_incomplete_runs_internal(&app.handle().clone(), &state_for_seed)?;

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
                    state_for_uap.config.clone(),
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
            save_note_attachment,
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
            create_tmlite_task,
            list_skills,
            check_for_updates,
            install_pending_update,
            list_workspace_presets,
            list_ops_channels,
            create_ops_channel,
            list_ops_channel_messages,
            list_ops_channel_messages_page,
            send_ops_channel_message,
            list_ops_jobs,
            create_ops_job,
            list_ops_route_approvals,
            resolve_ops_route_approval,
            list_ops_rules,
            save_ops_rule,
            list_ops_session_templates,
            save_ops_session_template,
            list_ops_sessions,
            start_ops_session,
            advance_ops_session,
            pause_ops_session,
            probe_provider_command,
            smoke_test_provider_command,
            get_provider_auth_state,
            get_provider_env_template,
            check_provider_setup,
            bootstrap_provider_workspace,
            create_dispatch_run,
            list_channel_dispatch_runs,
            list_dispatch_runs,
            get_dispatch_run,
            list_run_events,
            list_run_events_page,
            list_run_artifacts,
            reveal_run_path,
            open_workspace_folder,
            open_workspace_terminal,
            cancel_dispatch_run,
            retry_dispatch_run,
            recover_incomplete_runs,
            append_run_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UMBRA");
}

#[cfg(test)]
mod tests {
    use chrono::{Duration as ChronoDuration, Utc};

    use super::tray_health_snapshot_from_agents;
    use crate::types::{Agent, AgentStatus};
    use crate::TrayHealthLevel;

    fn agent(id: &str, name: &str, status: AgentStatus, minutes_ago: i64) -> Agent {
        Agent {
            id: id.into(),
            name: name.into(),
            role: "agent".into(),
            status,
            allowed_tools: vec![],
            skills: vec![],
            last_seen: (Utc::now() - ChronoDuration::minutes(minutes_ago)).to_rfc3339(),
            active_task_id: None,
        }
    }

    #[test]
    fn tray_health_snapshot_flags_attention_when_agents_are_offline() {
        let snapshot = tray_health_snapshot_from_agents(&[
            agent("forge", "Forge", AgentStatus::Working, 1),
            agent("prism", "Prism", AgentStatus::Offline, 1),
        ]);

        assert_eq!(snapshot.level, TrayHealthLevel::Attention);
        assert_eq!(snapshot.summary, "fleet health: attention");
        assert!(snapshot.counts.contains("working 1"));
        assert!(snapshot.counts.contains("offline 1"));
        assert!(snapshot
            .agent_rows
            .iter()
            .any(|row| row == "Forge: working"));
        assert!(snapshot
            .agent_rows
            .iter()
            .any(|row| row == "Prism: offline"));
    }

    #[test]
    fn tray_health_snapshot_marks_stale_agents_offline() {
        let snapshot =
            tray_health_snapshot_from_agents(&[agent("forge", "Forge", AgentStatus::Online, 40)]);

        assert_eq!(snapshot.level, TrayHealthLevel::Offline);
        assert_eq!(snapshot.summary, "fleet health: offline");
        assert!(snapshot.counts.contains("active 0/1"));
        assert!(snapshot
            .agent_rows
            .iter()
            .any(|row| row == "Forge: offline"));
    }
}
