use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tauri::{AppHandle, Emitter, State};
use tokio::fs;
use tokio::process::Command;

use crate::errors::{AppError, Result};
use crate::state::AppState;
use crate::types::{
    AppendRunEventInput, CreateDispatchRunInput, DispatchRun, RunArtifact, RunEvent,
    RunEventPage, WorkspacePreset,
};
use crate::workbench::runner;

#[tauri::command]
pub async fn list_workspace_presets(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<WorkspacePreset>, AppError> {
    Ok(state.config.read().await.workspace_presets.clone())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProbeResult {
    provider_id: String,
    command: String,
    launchable: bool,
    exit_code: Option<i32>,
    summary: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthState {
    provider_id: String,
    agent_ids: Vec<String>,
    provisioned_count: usize,
    summary: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBootstrapResult {
    provider_id: String,
    workspace_id: String,
    files: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSetupChecklistItem {
    key: String,
    label: String,
    ready: bool,
    detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSetupChecklistResult {
    provider_id: String,
    workspace_id: String,
    items: Vec<ProviderSetupChecklistItem>,
    summary: String,
}

#[tauri::command]
pub async fn probe_provider_command(
    provider_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<ProviderProbeResult, AppError> {
    let command = resolve_provider_command(&state, &provider_id).await?;
    let probe_args = provider_probe_args(&provider_id);
    let summary_prefix = format!("{command} {}", probe_args.join(" "));

    let output = match tokio::time::timeout(
        Duration::from_secs(8),
        Command::new(&command)
            .args(&probe_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    {
        Ok(result) => result.map_err(AppError::Io)?,
        Err(_) => {
            return Ok(ProviderProbeResult {
                provider_id,
                command,
                launchable: false,
                exit_code: None,
                summary: format!("{summary_prefix} timed out"),
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let summary = if !stdout.is_empty() {
        stdout.lines().next().unwrap_or_default().to_string()
    } else if !stderr.is_empty() {
        stderr.lines().next().unwrap_or_default().to_string()
    } else if output.status.success() {
        format!("{summary_prefix} launched successfully")
    } else {
        format!("{summary_prefix} exited without output")
    };

    Ok(ProviderProbeResult {
        provider_id,
        command,
        launchable: output.status.success() || !summary.is_empty(),
        exit_code: output.status.code(),
        summary,
    })
}

#[tauri::command]
pub async fn smoke_test_provider_command(
    provider_id: String,
    workspace_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<ProviderProbeResult, AppError> {
    let command = resolve_provider_command(&state, &provider_id).await?;
    let (_, workspace_path) = resolve_workspace_with_path(&state, &workspace_id).await?;
    let args = provider_smoke_args(&provider_id);
    let summary_prefix = format!("{command} {}", args.join(" "));

    let output = match tokio::time::timeout(
        Duration::from_secs(10),
        Command::new(&command)
            .args(&args)
            .current_dir(&workspace_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    {
        Ok(result) => result.map_err(AppError::Io)?,
        Err(_) => {
            return Ok(ProviderProbeResult {
                provider_id,
                command,
                launchable: false,
                exit_code: None,
                summary: format!(
                    "{summary_prefix} timed out inside {}",
                    workspace_path.display()
                ),
            });
        }
    };

    Ok(ProviderProbeResult {
        provider_id,
        command,
        launchable: output.status.success(),
        exit_code: output.status.code(),
        summary: summarize_provider_output(&summary_prefix, &output, Some(&workspace_path)),
    })
}

#[tauri::command]
pub async fn get_provider_auth_state(
    provider_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<ProviderAuthState, AppError> {
    let config = state.config.read().await;
    let agent_ids = provider_agent_ids(&config, &provider_id);
    let provisioned = agent_ids
        .iter()
        .filter(|agent_id| config.agent_auth_tokens.contains_key(*agent_id))
        .count();
    let summary = if agent_ids.is_empty() {
        format!("no agents mapped to provider {provider_id} yet")
    } else {
        format!(
            "{provisioned}/{} agent token(s) provisioned for {provider_id}: {}",
            agent_ids.len(),
            agent_ids.join(", ")
        )
    };

    Ok(ProviderAuthState {
        provider_id,
        agent_ids,
        provisioned_count: provisioned,
        summary,
    })
}

#[tauri::command]
pub async fn get_provider_env_template(
    provider_id: String,
    agent_id: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<String, AppError> {
    let config = state.config.read().await;
    build_worker_env_template(&config, &provider_id, agent_id.as_deref())
}

#[tauri::command]
pub async fn check_provider_setup(
    provider_id: String,
    workspace_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<ProviderSetupChecklistResult, AppError> {
    let config = state.config.read().await;
    let command = config
        .provider_commands
        .iter()
        .find(|entry| entry.provider_id == provider_id)
        .map(|entry| entry.command.trim().to_string())
        .filter(|value| !value.is_empty());

    let auth_state = {
        let agent_ids = provider_agent_ids(&config, &provider_id);
        let provisioned = agent_ids
            .iter()
            .filter(|agent_id| config.agent_auth_tokens.contains_key(*agent_id))
            .count();
        (agent_ids, provisioned)
    };
    drop(config);

    let workspace_check = resolve_workspace_with_path(&state, &workspace_id).await;
    let (instruction_path, env_path) = if let Ok((_, workspace_path)) = &workspace_check {
        let instruction_path = provider_instruction_file(&provider_id)
            .map(|(file_name, _)| workspace_path.join(file_name));
        let env_path = workspace_path.join(".umbra").join("worker.env.example");
        (instruction_path, Some(env_path))
    } else {
        (None, None)
    };

    let mut items = vec![
        ProviderSetupChecklistItem {
            key: "command".into(),
            label: "provider command configured".into(),
            ready: command.is_some(),
            detail: command.unwrap_or_else(|| "set a command in settings or via env override".into()),
        },
        ProviderSetupChecklistItem {
            key: "auth".into(),
            label: "per-agent auth provisioned".into(),
            ready: auth_state.1 > 0,
            detail: if auth_state.0.is_empty() {
                format!("no agents currently map to provider {provider_id}")
            } else {
                format!(
                    "{}/{} token(s) ready for {}",
                    auth_state.1,
                    auth_state.0.len(),
                    auth_state.0.join(", ")
                )
            },
        },
        ProviderSetupChecklistItem {
            key: "workspace".into(),
            label: "workspace inside grant roots".into(),
            ready: workspace_check.is_ok(),
            detail: match &workspace_check {
                Ok((workspace, _)) => format!("{} is allowed", workspace.name),
                Err(error) => error.to_string(),
            },
        },
    ];

    if let Some(path) = instruction_path {
        items.push(ProviderSetupChecklistItem {
            key: "instructions".into(),
            label: "provider instruction file present".into(),
            ready: path.exists(),
            detail: if path.exists() {
                format!("found {}", path.display())
            } else {
                format!("missing {}", path.display())
            },
        });
    }

    if let Some(path) = env_path {
        items.push(ProviderSetupChecklistItem {
            key: "workerEnv".into(),
            label: "worker env template present".into(),
            ready: path.exists(),
            detail: if path.exists() {
                format!("found {}", path.display())
            } else {
                format!("missing {}", path.display())
            },
        });
    }

    let ready_count = items.iter().filter(|item| item.ready).count();
    Ok(ProviderSetupChecklistResult {
        provider_id,
        workspace_id,
        summary: format!("{ready_count}/{} checklist item(s) ready", items.len()),
        items,
    })
}

#[tauri::command]
pub async fn bootstrap_provider_workspace(
    provider_id: String,
    workspace_id: String,
    agent_id: Option<String>,
    overwrite: Option<bool>,
    state: State<'_, AppState>,
) -> std::result::Result<WorkspaceBootstrapResult, AppError> {
    let (_, workspace_path) = resolve_workspace_with_path(&state, &workspace_id).await?;
    let mut written = Vec::new();
    let overwrite = overwrite.unwrap_or(false);

    if let Some((file_name, content)) = provider_instruction_file(&provider_id) {
        let file_path = workspace_path.join(file_name);
        write_bootstrap_file(&file_path, content, overwrite).await?;
        written.push(file_path.display().to_string());
    }

    let env_content = {
        let config = state.config.read().await;
        build_worker_env_template(&config, &provider_id, agent_id.as_deref())?
    };
    let env_path = workspace_path.join(".umbra").join("worker.env.example");
    write_bootstrap_file(&env_path, &env_content, overwrite).await?;
    written.push(env_path.display().to_string());

    Ok(WorkspaceBootstrapResult {
        provider_id,
        workspace_id,
        summary: if overwrite {
            format!("refreshed {} bootstrap file(s)", written.len())
        } else {
            format!("wrote {} bootstrap file(s)", written.len())
        },
        files: written,
    })
}

#[tauri::command]
pub async fn create_dispatch_run(
    input: CreateDispatchRunInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<DispatchRun, AppError> {
    create_run_with_events(input, &state, &app).await
}

#[tauri::command]
pub async fn list_dispatch_runs(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<DispatchRun>, AppError> {
    state.workbench_store.list_runs().map_err(AppError::from)
}

#[tauri::command]
pub async fn list_channel_dispatch_runs(
    channel_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<DispatchRun>, AppError> {
    state
        .workbench_store
        .list_runs_for_channel(&channel_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_dispatch_run(
    run_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Option<DispatchRun>, AppError> {
    state
        .workbench_store
        .get_run(&run_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_run_events(
    run_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<RunEvent>, AppError> {
    state
        .workbench_store
        .list_events(&run_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_run_events_page(
    run_id: String,
    before: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> std::result::Result<RunEventPage, AppError> {
    state
        .workbench_store
        .list_events_page(&run_id, before.as_deref(), limit.unwrap_or(80))
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_run_artifacts(
    run_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<RunArtifact>, AppError> {
    state
        .workbench_store
        .list_artifacts(&run_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn reveal_run_path(
    run_id: String,
    relative_path: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let run = state
        .workbench_store
        .get_run(&run_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("run {}", run_id)))?;
    let (_, workspace_path) = resolve_workspace_with_path(&state, &run.workspace_id).await?;
    let target = relative_path
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| workspace_path.join(value))
        .unwrap_or_else(|| workspace_path.clone());
    let canonical_target = if target.exists() {
        std::fs::canonicalize(&target)?
    } else {
        target
    };

    if canonical_target != workspace_path && !canonical_target.starts_with(&workspace_path) {
        return Err(AppError::TargetNotAllowed(format!(
            "path {} is outside workspace {}",
            canonical_target.display(),
            workspace_path.display()
        )));
    }

    reveal_path(&canonical_target).await
}

#[tauri::command]
pub async fn open_workspace_folder(
    workspace_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let (_, workspace_path) = resolve_workspace_with_path(&state, &workspace_id).await?;
    reveal_path(&workspace_path).await
}

#[tauri::command]
pub async fn open_workspace_terminal(
    workspace_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let (_, workspace_path) = resolve_workspace_with_path(&state, &workspace_id).await?;
    open_terminal_at(&workspace_path).await
}

#[tauri::command]
pub async fn cancel_dispatch_run(
    run_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<DispatchRun, AppError> {
    let (run, event) = state
        .workbench_store
        .cancel_run(&run_id)
        .map_err(AppError::from)?;

    let _ = app.emit("workbench:run-updated", &run);
    let _ = app.emit("workbench:event-added", &event);

    match runner::kill_run_process(state.inner(), &run_id).await {
        Ok(Some(message)) => {
            let (updated_run, kill_event) = state
                .workbench_store
                .append_event(AppendRunEventInput {
                    run_id: run_id.clone(),
                    event_type: crate::types::RunEventType::System,
                    body: message,
                })
                .map_err(AppError::from)?;
            let _ = app.emit("workbench:run-updated", &updated_run);
            let _ = app.emit("workbench:event-added", &kill_event);
        }
        Ok(None) => {}
        Err(error) => {
            let (updated_run, kill_event) = state
                .workbench_store
                .append_event(AppendRunEventInput {
                    run_id: run_id.clone(),
                    event_type: crate::types::RunEventType::System,
                    body: format!("run marked cancelled, but provider termination failed: {error}"),
                })
                .map_err(AppError::from)?;
            let _ = app.emit("workbench:run-updated", &updated_run);
            let _ = app.emit("workbench:event-added", &kill_event);
        }
    }

    Ok(run)
}

#[tauri::command]
pub async fn retry_dispatch_run(
    run_id: String,
    prompt: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<DispatchRun, AppError> {
    let source_run = state
        .workbench_store
        .get_run(&run_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("run {}", run_id)))?;

    let input = CreateDispatchRunInput {
        mode: source_run.mode.clone(),
        agent_id: source_run.agent_id.clone(),
        workspace_id: source_run.workspace_id.clone(),
        channel_id: source_run.channel_id.clone(),
        source_message_id: source_run.source_message_id.clone(),
        job_id: source_run.job_id.clone(),
        session_id: source_run.session_id.clone(),
        pm_task_id: source_run.pm_task_id.clone(),
        prompt: prompt
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| source_run.prompt.clone()),
        persona_id: source_run.persona_id.clone(),
        continue_from_run_id: Some(source_run.id.clone()),
    };

    let run = create_run_with_events(input, &state, &app).await?;
    let (updated_source, retry_event) = state
        .workbench_store
        .append_event(AppendRunEventInput {
            run_id: source_run.id,
            event_type: crate::types::RunEventType::System,
            body: format!("retry dispatched as run {}", run.id),
        })
        .map_err(AppError::from)?;
    let _ = app.emit("workbench:run-updated", &updated_source);
    let _ = app.emit("workbench:event-added", &retry_event);
    Ok(run)
}

#[tauri::command]
pub async fn append_run_event(
    input: AppendRunEventInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<RunEvent, AppError> {
    if input.body.trim().is_empty() {
        return Err(AppError::Other("event body cannot be empty".into()));
    }

    let (run, event) = state
        .workbench_store
        .append_event(input)
        .map_err(AppError::from)?;

    let _ = app.emit("workbench:run-updated", &run);
    let _ = app.emit("workbench:event-added", &event);
    Ok(event)
}

#[tauri::command]
pub async fn recover_incomplete_runs(
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<Vec<DispatchRun>, AppError> {
    recover_incomplete_runs_internal(&app, state.inner()).map_err(AppError::from)
}

pub(crate) fn recover_incomplete_runs_internal(
    app: &AppHandle,
    state: &AppState,
) -> Result<Vec<DispatchRun>> {
    let recovered = state.workbench_store.recover_incomplete_runs()?;
    for run in &recovered {
        let _ = app.emit("workbench:run-updated", run);
        if let Some(event) = state.workbench_store.list_events(&run.id)?.last().cloned() {
            let _ = app.emit("workbench:event-added", &event);
        }
    }
    Ok(recovered)
}

async fn create_run_with_events(
    input: CreateDispatchRunInput,
    state: &State<'_, AppState>,
    app: &AppHandle,
) -> std::result::Result<DispatchRun, AppError> {
    let validated = validate_create_input(&input, state).await?;
    let continue_from_run_id = validated.continue_from_run_id.clone();
    let provider_id = {
        let cfg = state.config.read().await;
        derive_provider_id_with_map(&validated.agent_id, Some(&cfg.agent_provider_map))
    };
    let (run, first_event) = state
        .workbench_store
        .create_run(validated, provider_id)
        .map_err(AppError::from)?;

    let _ = app.emit("workbench:run-created", &run);
    let _ = app.emit("workbench:event-added", &first_event);
    if let Some(parent_run_id) = continue_from_run_id {
        let (_, event) = state
            .workbench_store
            .append_event(AppendRunEventInput {
                run_id: run.id.clone(),
                event_type: crate::types::RunEventType::System,
                body: format!("continued from run {parent_run_id}"),
            })
            .map_err(AppError::from)?;
        let _ = app.emit("workbench:event-added", &event);
    }
    runner::spawn_run(app.clone(), state.inner().clone(), run.id.clone());
    Ok(run)
}

pub(crate) async fn validate_create_input(
    input: &CreateDispatchRunInput,
    state: &State<'_, AppState>,
) -> Result<CreateDispatchRunInput> {
    let prompt = input.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::Other("prompt cannot be empty".into()));
    }

    let agent = {
        let agents = state.agent_registry.agents.read().await;
        agents.get(&input.agent_id).cloned()
    }
    .ok_or_else(|| AppError::TargetNotFound(format!("agent {}", input.agent_id)))?;

    if matches!(
        crate::normalized_agent_status(&agent),
        crate::types::AgentStatus::Offline
    ) {
        return Err(AppError::Other(format!("agent {} is offline", agent.name)));
    }

    let config = state.config.read().await;
    let workspace = config
        .workspace_presets
        .iter()
        .find(|preset| preset.id == input.workspace_id)
        .cloned()
        .ok_or_else(|| AppError::TargetNotFound(format!("workspace {}", input.workspace_id)))?;

    if !workspace.allowed_agents.is_empty()
        && !workspace
            .allowed_agents
            .iter()
            .any(|id| id == &input.agent_id)
    {
        return Err(AppError::TargetNotAllowed(format!(
            "agent {} is not allowed in workspace {}",
            input.agent_id, workspace.name
        )));
    }

    let provider_id = {
        let cfg = state.config.read().await;
        derive_provider_id_with_map(&input.agent_id, Some(&cfg.agent_provider_map))
    };
    if !workspace.allowed_providers.is_empty()
        && !workspace
            .allowed_providers
            .iter()
            .any(|provider| provider == &provider_id)
    {
        return Err(AppError::TargetNotAllowed(format!(
            "provider {provider_id} is not allowed in workspace {}",
            workspace.name
        )));
    }

    if matches!(input.mode, crate::types::DispatchMode::Task) && !workspace.writable {
        return Err(AppError::TargetNotAllowed(format!(
            "workspace {} is read-only",
            workspace.name
        )));
    }

    let root_path = Path::new(&workspace.root_path);
    if !root_path.exists() {
        return Err(AppError::TargetNotFound(workspace.root_path));
    }

    if !root_path.is_dir() {
        return Err(AppError::TargetNotAllowed(format!(
            "workspace path is not a directory: {}",
            workspace.root_path
        )));
    }

    let canonical_root = std::fs::canonicalize(root_path)?;
    ensure_workspace_granted(&canonical_root, &config.workspace_grant_roots)?;

    let persona_id = input
        .persona_id
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(persona_id) = &persona_id {
        if !config
            .persona_presets
            .iter()
            .any(|preset| preset.id == *persona_id)
        {
            return Err(AppError::TargetNotFound(format!("persona {}", persona_id)));
        }
    }

    let pm_task_id = input
        .pm_task_id
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let continue_from_run_id = input
        .continue_from_run_id
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(parent_run_id) = &continue_from_run_id {
        let source_run = state
            .workbench_store
            .get_run(parent_run_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("run {}", parent_run_id)))?;
        if source_run.agent_id != input.agent_id.trim() {
            return Err(AppError::TargetNotAllowed(format!(
                "continued run must keep the same agent as source run {}",
                parent_run_id
            )));
        }
        if source_run.workspace_id != input.workspace_id.trim() {
            return Err(AppError::TargetNotAllowed(format!(
                "continued run must keep the same workspace as source run {}",
                parent_run_id
            )));
        }
    }

    Ok(CreateDispatchRunInput {
        mode: input.mode.clone(),
        agent_id: input.agent_id.trim().to_string(),
        workspace_id: input.workspace_id.trim().to_string(),
        channel_id: input.channel_id.clone(),
        source_message_id: input.source_message_id.clone(),
        job_id: input.job_id.clone(),
        session_id: input.session_id.clone(),
        pm_task_id,
        prompt,
        persona_id,
        continue_from_run_id,
    })
}

pub(crate) fn derive_provider_id(agent_id: &str) -> String {
    derive_provider_id_with_map(agent_id, None)
}

pub(crate) fn derive_provider_id_with_map(
    agent_id: &str,
    agent_provider_map: Option<&HashMap<String, String>>,
) -> String {
    let normalized = agent_id.trim().to_lowercase();
    // 1. Check explicit mapping first
    if let Some(map) = agent_provider_map {
        if let Some(provider) = map.get(&normalized) {
            return provider.clone();
        }
    }
    // 2. Fallback to prefix heuristic
    if normalized.starts_with("codex") {
        "codex".into()
    } else if normalized.starts_with("claude") {
        "claude".into()
    } else if normalized.starts_with("gemini") {
        "gemini".into()
    } else if normalized.starts_with("kimi") {
        "kimi".into()
    } else {
        "custom".into()
    }
}

async fn resolve_provider_command(
    state: &State<'_, AppState>,
    provider_id: &str,
) -> Result<String> {
    let config = state.config.read().await;
    config
        .provider_commands
        .iter()
        .find(|entry| entry.provider_id == provider_id)
        .map(|entry| entry.command.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var(format!("UMBRA_{}_COMMAND", provider_id.to_uppercase()))
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .ok_or_else(|| AppError::TargetNotFound(format!("provider command {}", provider_id)))
}

fn provider_probe_args(provider_id: &str) -> Vec<String> {
    match provider_id {
        "codex" | "claude" | "gemini" | "kimi" => vec!["--help".into()],
        _ => vec!["--help".into()],
    }
}

fn provider_smoke_args(provider_id: &str) -> Vec<String> {
    match provider_id {
        "codex" | "claude" | "gemini" | "kimi" => vec!["--version".into()],
        _ => vec!["--version".into()],
    }
}

async fn resolve_workspace_with_path(
    state: &State<'_, AppState>,
    workspace_id: &str,
) -> Result<(WorkspacePreset, PathBuf)> {
    let config = state.config.read().await;
    let workspace = config
        .workspace_presets
        .iter()
        .find(|preset| preset.id == workspace_id)
        .cloned()
        .ok_or_else(|| AppError::TargetNotFound(format!("workspace {}", workspace_id)))?;

    let root_path = Path::new(&workspace.root_path);
    if !root_path.exists() {
        return Err(AppError::TargetNotFound(workspace.root_path));
    }
    if !root_path.is_dir() {
        return Err(AppError::TargetNotAllowed(format!(
            "workspace path is not a directory: {}",
            workspace.root_path
        )));
    }

    let canonical_root = std::fs::canonicalize(root_path)?;
    ensure_workspace_granted(&canonical_root, &config.workspace_grant_roots)?;
    Ok((workspace, canonical_root))
}

fn ensure_workspace_granted(workspace_path: &Path, grant_roots: &[String]) -> Result<()> {
    if grant_roots.is_empty() {
        return Err(AppError::TargetNotAllowed(
            "workspace grant roots are not configured".into(),
        ));
    }

    for root in grant_roots {
        let Ok(canonical_root) = std::fs::canonicalize(root) else {
            continue;
        };
        if workspace_path == canonical_root || workspace_path.starts_with(&canonical_root) {
            return Ok(());
        }
    }

    Err(AppError::TargetNotAllowed(format!(
        "workspace {} is outside the configured workspace grant roots",
        workspace_path.display()
    )))
}

fn provider_agent_ids(config: &crate::commands::config::AppConfig, provider_id: &str) -> Vec<String> {
    let mut ids = vec!["forge".to_string(), "prism".to_string()];
    ids.extend(config.custom_agents.iter().map(|agent| agent.id.clone()));
    ids.into_iter()
        .filter(|agent_id| derive_provider_id_with_map(agent_id, Some(&config.agent_provider_map)) == provider_id)
        .collect()
}

fn provider_instruction_file(provider_id: &str) -> Option<(&'static str, &'static str)> {
    match provider_id {
        "codex" => Some(("AGENTS.md", include_str!("../../../templates/AGENTS.codex.md"))),
        "claude" => Some(("CLAUDE.md", include_str!("../../../templates/CLAUDE.md"))),
        "gemini" => Some(("GEMINI.md", include_str!("../../../templates/GEMINI.md"))),
        _ => None,
    }
}

fn build_worker_env_template(
    config: &crate::commands::config::AppConfig,
    provider_id: &str,
    agent_id: Option<&str>,
) -> Result<String> {
    let agent_id = agent_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| provider_agent_ids(config, provider_id).into_iter().next())
        .ok_or_else(|| AppError::TargetNotFound(format!("agent for provider {}", provider_id)))?;
    let token = config
        .agent_auth_tokens
        .get(&agent_id)
        .cloned()
        .ok_or_else(|| AppError::TargetNotFound(format!("agent token {}", agent_id)))?;

    Ok(format!(
        "UMBRA_AGENT_ID={agent_id}\nUMBRA_PROVIDER_ID={provider_id}\nUMBRA_UAP_HOST={}\nUMBRA_UAP_PORT={}\nUMBRA_AGENT_TOKEN={token}\n",
        config.uap_advertise_host,
        config.uap_port
    ))
}

async fn write_bootstrap_file(path: &Path, content: &str, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(AppError::TargetNotAllowed(format!(
            "bootstrap file already exists: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(path, content).await?;
    Ok(())
}

fn summarize_provider_output(
    summary_prefix: &str,
    output: &std::process::Output,
    workspace_path: Option<&Path>,
) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let body = if !stdout.is_empty() {
        stdout.lines().next().unwrap_or_default().to_string()
    } else if !stderr.is_empty() {
        stderr.lines().next().unwrap_or_default().to_string()
    } else if output.status.success() {
        "completed successfully".into()
    } else {
        "exited without output".into()
    };

    if let Some(workspace_path) = workspace_path {
        format!("{summary_prefix} in {}: {body}", workspace_path.display())
    } else {
        body
    }
}

#[cfg(target_os = "windows")]
async fn reveal_path(path: &Path) -> Result<()> {
    Command::new("explorer")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(target_os = "macos")]
async fn reveal_path(path: &Path) -> Result<()> {
    Command::new("open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
async fn reveal_path(path: &Path) -> Result<()> {
    Command::new("xdg-open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(target_os = "windows")]
async fn open_terminal_at(path: &Path) -> Result<()> {
    let escaped = path.display().to_string().replace('\'', "''");
    Command::new("powershell")
        .args([
            "-NoExit",
            "-Command",
            &format!("Set-Location -LiteralPath '{escaped}'"),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(target_os = "macos")]
async fn open_terminal_at(path: &Path) -> Result<()> {
    Command::new("open")
        .args(["-a", "Terminal"])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
async fn open_terminal_at(path: &Path) -> Result<()> {
    Command::new("x-terminal-emulator")
        .arg("--working-directory")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}
