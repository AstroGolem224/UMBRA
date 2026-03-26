use std::path::Path;
use std::process::Stdio;

use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

use crate::commands::ops_room::{
    emit_channel_message_added, emit_channel_updated, emit_job_updated, emit_session_updated,
};
use crate::errors::{AppError, Result};
use crate::state::AppState;
use crate::types::{
    AppendRunEventInput, DispatchRun, DispatchStatus, RunArtifact, RunArtifactKind, RunEvent,
    RunEventType, RunOutcomeStatus, WorkspacePreset,
};
use crate::workbench::adapters::{build_provider_plans, ProviderExecutionPlan, ProviderOutputMode};

pub fn spawn_run(app: AppHandle, state: AppState, run_id: String) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = execute_run(app.clone(), state.clone(), run_id.clone()).await {
            let _ = mark_run_failed(&app, &state, &run_id, &error.to_string()).await;
        }
    });
}

pub async fn kill_run_process(state: &AppState, run_id: &str) -> Result<Option<String>> {
    let Some(pid) = state.active_run_registry.remove(run_id).await else {
        return Ok(None);
    };

    terminate_process(pid).await?;
    Ok(Some(format!(
        "termination requested for provider process pid {pid}"
    )))
}

async fn execute_run(app: AppHandle, state: AppState, run_id: String) -> Result<()> {
    let run = state
        .workbench_store
        .get_run(&run_id)?
        .ok_or_else(|| AppError::TargetNotFound(run_id.clone()))?;

    if is_terminal_status(&run.status) {
        return Ok(());
    }

    let workspace = resolve_workspace(&state, &run.workspace_id).await?;
    let workspace_path = std::fs::canonicalize(&workspace.root_path)?;
    let configured_command = resolve_provider_command(&state, &run.provider_id).await;
    let effective_prompt = resolve_effective_prompt(&state, &run).await;
    let plans = build_provider_plans(&run.provider_id, configured_command, &effective_prompt);
    if plans.is_empty() {
        return Err(AppError::Other(format!(
            "provider {} is not wired yet",
            run.provider_id
        )));
    }

    let working_run = state
        .workbench_store
        .set_run_status(&run.id, DispatchStatus::Working)?;
    emit_run_updated(&app, &working_run);

    append_event(
        &app,
        &state,
        &run.id,
        RunEventType::System,
        format!(
            "starting {} run in workspace {} via {}{}",
            run.provider_id,
            workspace.name,
            workspace_path.display(),
            run.persona_id
                .as_ref()
                .map(|persona_id| format!(" with persona {persona_id}"))
                .unwrap_or_default()
        ),
    )?;

    execute_provider_plans(&app, &state, &run, &plans, &workspace_path).await
}

async fn execute_provider_plans(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    plans: &[ProviderExecutionPlan],
    workspace_path: &Path,
) -> Result<()> {
    let mut startup_errors = Vec::new();

    for plan in plans {
        append_event(
            app,
            state,
            &run.id,
            RunEventType::System,
            format!("launch attempt via {}", plan.label),
        )?;
        match spawn_and_stream_provider(app, state, run, plan, workspace_path).await {
            Ok(result) => {
                return finalize_provider_run(app, state, run, result, workspace_path).await;
            }
            Err(AppError::Io(error)) if is_retryable_spawn_error(&error) => {
                let failure = format!("{} failed to launch: {}", plan.label, error);
                startup_errors.push(failure.clone());
                append_event(app, state, &run.id, RunEventType::System, failure)?;
            }
            Err(error) => return Err(error),
        }
    }

    Err(AppError::Other(format!(
        "{} command was not launchable in the current runtime. {}.\n{}",
        run.provider_id,
        provider_launch_help(&run.provider_id),
        startup_errors.join("\n")
    )))
}

async fn spawn_and_stream_provider(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    plan: &ProviderExecutionPlan,
    workspace_path: &Path,
) -> Result<ProviderExecutionResult> {
    let mut command = Command::new(&plan.program);
    command
        .args(&plan.args)
        .current_dir(workspace_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("UMBRA_RUN_ID", &run.id)
        .env("UMBRA_AGENT_ID", &run.agent_id)
        .env("UMBRA_WORKSPACE_ID", &run.workspace_id)
        .env("UMBRA_PROVIDER_ID", &run.provider_id);

    let mut child = command.spawn().map_err(AppError::Io)?;
    if let Some(pid) = child.id() {
        state
            .active_run_registry
            .register(run.id.clone(), pid)
            .await;
    }
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Other(format!("{} did not expose stdout", plan.label)))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Other(format!("{} did not expose stderr", plan.label)))?;

    let stdout_task = {
        let app = app.clone();
        let state = state.clone();
        let run_id = run.id.clone();
        let output_mode = plan.output_mode.clone();
        tauri::async_runtime::spawn(async move {
            stream_reader(
                app,
                state,
                run_id,
                stdout,
                output_mode,
                StreamSource::Stdout,
            )
            .await
        })
    };

    let stderr_task = {
        let app = app.clone();
        let state = state.clone();
        let run_id = run.id.clone();
        tauri::async_runtime::spawn(async move {
            stream_reader(
                app,
                state,
                run_id,
                stderr,
                ProviderOutputMode::PlainTextAssistant,
                StreamSource::Stderr,
            )
            .await
        })
    };

    let status = child.wait().await.map_err(AppError::Io)?;
    let _ = state.active_run_registry.remove(&run.id).await;
    let stdout_summary = stdout_task
        .await
        .map_err(|error| AppError::Other(format!("stdout task failed: {error}")))??;
    let stderr_summary = stderr_task
        .await
        .map_err(|error| AppError::Other(format!("stderr task failed: {error}")))??;

    Ok(ProviderExecutionResult {
        plan: plan.clone(),
        success: status.success(),
        exit_code: status.code(),
        stdout_summary,
        stderr_summary,
    })
}

async fn stream_reader<R: AsyncRead + Unpin>(
    app: AppHandle,
    state: AppState,
    run_id: String,
    reader: R,
    output_mode: ProviderOutputMode,
    source: StreamSource,
) -> Result<StreamSummary> {
    let mut lines = BufReader::new(reader).lines();
    let mut summary = StreamSummary::default();

    while let Some(line) = lines.next_line().await.map_err(AppError::Io)? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if run_is_cancelled(&state, &run_id)? {
            break;
        }

        let (event_type, body) = classify_stream_line(trimmed, &output_mode, source);
        append_event(&app, &state, &run_id, event_type.clone(), body.clone())?;
        summary.record(&event_type, &body);
    }

    Ok(summary)
}

fn classify_stream_line(
    line: &str,
    output_mode: &ProviderOutputMode,
    source: StreamSource,
) -> (RunEventType, String) {
    match source {
        StreamSource::Stderr => (RunEventType::Stderr, line.to_string()),
        StreamSource::Stdout => match output_mode {
            ProviderOutputMode::PlainTextAssistant => {
                (RunEventType::AgentMessage, line.to_string())
            }
            ProviderOutputMode::StreamJson => extract_stream_json_text(line)
                .map(|text| (RunEventType::AgentMessage, text))
                .unwrap_or_else(|| (RunEventType::Stdout, line.to_string())),
        },
    }
}

fn extract_stream_json_text(line: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(line).ok()?;

    if let Some(result) = value.get("result").and_then(Value::as_str) {
        return trim_to_option(result);
    }

    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return trim_to_option(text);
    }

    if let Some(delta_text) = value
        .get("delta")
        .and_then(|delta| delta.get("text"))
        .and_then(Value::as_str)
    {
        return trim_to_option(delta_text);
    }

    if let Some(message) = value.get("message") {
        if let Some(text) = extract_text_from_value(message) {
            return Some(text);
        }
    }

    if let Some(content) = value.get("content") {
        if let Some(text) = extract_text_from_value(content) {
            return Some(text);
        }
    }

    None
}

fn extract_text_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => trim_to_option(text),
        Value::Array(items) => {
            let mut parts = Vec::new();
            for item in items {
                if let Some(text) = extract_text_from_value(item) {
                    parts.push(text);
                }
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(Value::as_str) {
                return trim_to_option(text);
            }
            if let Some(text) = map.get("content").and_then(extract_text_from_value) {
                return Some(text);
            }
            if let Some(text) = map.get("message").and_then(extract_text_from_value) {
                return Some(text);
            }
            None
        }
        _ => None,
    }
}

fn trim_to_option(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

async fn finalize_provider_run(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    result: ProviderExecutionResult,
    workspace_path: &Path,
) -> Result<()> {
    let contract = parse_result_contract(&result.stdout_summary);

    if run_is_cancelled(state, &run.id)? {
        append_event(
            app,
            state,
            &run.id,
            RunEventType::System,
            format!(
                "run was cancelled before {} returned; further provider output was ignored",
                result.plan.label
            ),
        )?;
        return Ok(());
    }

    if result.success {
        append_event(
            app,
            state,
            &run.id,
            RunEventType::System,
            format_process_success(&result.plan.label, result.exit_code),
        )?;
        replace_artifacts(
            app,
            state,
            &run.id,
            build_run_artifacts(state, &run.id, &result, workspace_path, contract.as_ref()).await,
        )?;

        if result.stdout_summary.agent_messages == 0 {
            append_event(
                app,
                state,
                &run.id,
                RunEventType::System,
                format!("{} completed without agent output", result.plan.label),
            )?;
        }

        if result.stderr_summary.stderr_lines > 0 {
            append_event(
                app,
                state,
                &run.id,
                RunEventType::System,
                format!(
                    "{} completed with {} stderr event(s)",
                    result.plan.label, result.stderr_summary.stderr_lines
                ),
            )?;
        }

        append_event(
            app,
            state,
            &run.id,
            RunEventType::System,
            format!(
                "{} completed with outcome {}",
                result.plan.label,
                format_outcome_status(&
                    contract
                        .as_ref()
                        .and_then(|value| value.outcome_status())
                        .unwrap_or_else(|| infer_outcome_status(&result.stdout_summary))
                )
            ),
        )?;
        append_system_channel_message(
            app,
            state,
            run,
            format!(
                "{} completed with outcome {}",
                result.plan.label,
                format_outcome_status(
                    &contract
                        .as_ref()
                        .and_then(|value| value.outcome_status())
                        .unwrap_or_else(|| infer_outcome_status(&result.stdout_summary))
                )
            ),
        )?;

        let updated = state
            .workbench_store
            .complete_run(
                &run.id,
                DispatchStatus::Done,
                Some(
                    contract
                        .as_ref()
                        .and_then(|value| value.outcome_status())
                        .unwrap_or_else(|| infer_outcome_status(&result.stdout_summary)),
                ),
            )?;
        emit_run_updated(app, &updated);
        sync_linked_job_and_session(
            app,
            state,
            &updated,
            contract
                .as_ref()
                .and_then(|value| value.outcome_status())
                .unwrap_or_else(|| infer_outcome_status(&result.stdout_summary)),
        )?;
        return Ok(());
    }

    replace_artifacts(
        app,
        state,
        &run.id,
        build_run_artifacts(state, &run.id, &result, workspace_path, contract.as_ref()).await,
    )?;
    append_event(
        app,
        state,
        &run.id,
        RunEventType::System,
        format_process_failure(&result.plan.label, result.exit_code),
    )?;
    append_system_channel_message(
        app,
        state,
        run,
        format_process_failure(&result.plan.label, result.exit_code),
    )?;
    let updated = state
        .workbench_store
        .set_run_status(&run.id, DispatchStatus::Error)?;
    emit_run_updated(app, &updated);
    sync_linked_job_failure(app, state, &updated)?;
    Ok(())
}

async fn mark_run_failed(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    message: &str,
) -> Result<()> {
    if run_is_cancelled(state, run_id)? {
        return Ok(());
    }

    append_event(
        app,
        state,
        run_id,
        RunEventType::System,
        message.to_string(),
    )?;
    let updated = state
        .workbench_store
        .set_run_status(run_id, DispatchStatus::Error)?;
    emit_run_updated(app, &updated);
    append_system_channel_message(app, state, &updated, message.to_string())?;
    sync_linked_job_failure(app, state, &updated)?;
    Ok(())
}

async fn resolve_workspace(state: &AppState, workspace_id: &str) -> Result<WorkspacePreset> {
    let config = state.config.read().await;
    let workspace = config
        .workspace_presets
        .iter()
        .find(|preset| preset.id == workspace_id)
        .cloned()
        .ok_or_else(|| AppError::TargetNotFound(format!("workspace {}", workspace_id)))?;
    let workspace_path = std::fs::canonicalize(&workspace.root_path)?;
    ensure_workspace_granted(&workspace_path, &config.workspace_grant_roots)?;
    Ok(workspace)
}

async fn resolve_effective_prompt(state: &AppState, run: &DispatchRun) -> String {
    let base_prompt = if let Some(persona_id) = &run.persona_id {
        let config = state.config.read().await;
        if let Some(persona) = config
            .persona_presets
            .iter()
            .find(|preset| preset.id == *persona_id)
        {
            format!(
                "{}\n\nuser task:\n{}",
                persona.prompt.trim(),
                run.prompt.trim()
            )
        } else {
            run.prompt.clone()
        }
    } else {
        run.prompt.clone()
    };

    format!(
        "{base_prompt}\n\nfinal response contract for umbra:\n- finish with exactly one line that starts with `UMBRA_RESULT_JSON:`.\n- after that prefix, emit valid minified json only.\n- use this shape: {{\"outcome\":\"succeeded|blocked|needs_input\",\"summary\":\"short delivery summary\",\"files\":[\"relative/path\"],\"tests\":[\"short test result\"],\"pmComment\":\"optional short pm update\"}}.\n- if you are blocked or need clarification, set the matching outcome and explain it in `summary`.\n- keep the rest of your reply human-readable above that final line."
    )
}

async fn resolve_provider_command(state: &AppState, provider_id: &str) -> Option<String> {
    let config = state.config.read().await;
    let configured = config
        .provider_commands
        .iter()
        .find(|entry| entry.provider_id == provider_id)
        .map(|entry| entry.command.trim().to_string())
        .filter(|value| !value.is_empty());
    drop(config);

    configured.or_else(|| {
        std::env::var(env_command_key(provider_id))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn env_command_key(provider_id: &str) -> String {
    format!("UMBRA_{}_COMMAND", provider_id.trim().to_uppercase())
}

fn run_is_cancelled(state: &AppState, run_id: &str) -> Result<bool> {
    Ok(matches!(
        state.workbench_store.get_run(run_id)?.map(|run| run.status),
        Some(DispatchStatus::Cancelled)
    ))
}

fn append_event(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    event_type: RunEventType,
    body: String,
) -> Result<()> {
    let body = body.trim().to_string();
    if body.is_empty() {
        return Ok(());
    }

    let (run, event) = state.workbench_store.append_event(AppendRunEventInput {
        run_id: run_id.to_string(),
        event_type,
        body,
    })?;
    emit_run_updated(app, &run);
    emit_event_added(app, &event);
    if matches!(event.event_type, RunEventType::AgentMessage) {
        mirror_agent_event_to_channel(app, state, &run, &event)?;
    }
    Ok(())
}

fn mirror_agent_event_to_channel(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    event: &RunEvent,
) -> Result<()> {
    let Some(channel_id) = &run.channel_id else {
        return Ok(());
    };
    let (channel, message) = state.workbench_store.append_channel_message(
        channel_id,
        run.source_message_id.clone(),
        Some(run.id.clone()),
        run.job_id.clone(),
        run.session_id.clone(),
        Some(run.agent_id.clone()),
        Some(run.agent_id.clone()),
        crate::types::ChannelMessageKind::Agent,
        event.body.clone(),
    )?;
    emit_channel_updated(app, &channel);
    emit_channel_message_added(app, &message);
    Ok(())
}

fn append_system_channel_message(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    body: String,
) -> Result<()> {
    let Some(channel_id) = &run.channel_id else {
        return Ok(());
    };
    let (channel, message) = state.workbench_store.append_channel_message(
        channel_id,
        run.source_message_id.clone(),
        Some(run.id.clone()),
        run.job_id.clone(),
        run.session_id.clone(),
        None,
        Some("umbra".into()),
        crate::types::ChannelMessageKind::System,
        body,
    )?;
    emit_channel_updated(app, &channel);
    emit_channel_message_added(app, &message);
    Ok(())
}

fn sync_linked_job_and_session(
    app: &AppHandle,
    state: &AppState,
    run: &DispatchRun,
    outcome_status: RunOutcomeStatus,
) -> Result<()> {
    if let Some(job_id) = &run.job_id {
        let status = match outcome_status {
            RunOutcomeStatus::Succeeded => crate::types::OpsJobStatus::Done,
            RunOutcomeStatus::Blocked | RunOutcomeStatus::NeedsInput => {
                crate::types::OpsJobStatus::Blocked
            }
        };
        let job = state.workbench_store.set_job_status(job_id, status)?;
        emit_job_updated(app, &job);
    }

    if let Some(session_id) = &run.session_id {
        if let Some(session) = state.workbench_store.get_session(session_id)? {
            emit_session_updated(app, &session);
        }
    }

    Ok(())
}

fn sync_linked_job_failure(app: &AppHandle, state: &AppState, run: &DispatchRun) -> Result<()> {
    if let Some(job_id) = &run.job_id {
        let job = state
            .workbench_store
            .set_job_status(job_id, crate::types::OpsJobStatus::Blocked)?;
        emit_job_updated(app, &job);
    }
    if let Some(session_id) = &run.session_id {
        if let Some(session) = state.workbench_store.get_session(session_id)? {
            emit_session_updated(app, &session);
        }
    }
    Ok(())
}

fn emit_run_updated(app: &AppHandle, run: &DispatchRun) {
    let _ = app.emit("workbench:run-updated", run);
}

fn emit_event_added(app: &AppHandle, event: &RunEvent) {
    let _ = app.emit("workbench:event-added", event);
}

fn emit_artifacts_replaced(app: &AppHandle, run_id: &str, artifacts: &[RunArtifact]) {
    let _ = app.emit(
        "workbench:artifacts-replaced",
        serde_json::json!({
            "runId": run_id,
            "artifacts": artifacts,
        }),
    );
}

fn is_terminal_status(status: &DispatchStatus) -> bool {
    matches!(
        status,
        DispatchStatus::Done | DispatchStatus::Error | DispatchStatus::Cancelled
    )
}

fn is_retryable_spawn_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
    )
}

fn provider_launch_help(provider_id: &str) -> &'static str {
    match provider_id {
        "codex" => {
            "configure a launchable Codex path in Settings > workbench providers, set UMBRA_CODEX_COMMAND, or use a working WSL bridge"
        }
        "claude" => {
            "configure a launchable Claude Code path in Settings > workbench providers or set UMBRA_CLAUDE_COMMAND"
        }
        "gemini" => {
            "configure a launchable Gemini CLI path in Settings > workbench providers or set UMBRA_GEMINI_COMMAND"
        }
        _ => "configure a valid provider command in Settings > workbench providers",
    }
}

#[cfg(target_os = "windows")]
async fn terminate_process(pid: u32) -> Result<()> {
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(AppError::Io)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(AppError::Other(if stderr.is_empty() {
        format!("failed to terminate process {pid}")
    } else {
        format!("failed to terminate process {pid}: {stderr}")
    }))
}

#[cfg(not(target_os = "windows"))]
async fn terminate_process(pid: u32) -> Result<()> {
    let output = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(AppError::Io)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(AppError::Other(if stderr.is_empty() {
        format!("failed to terminate process {pid}")
    } else {
        format!("failed to terminate process {pid}: {stderr}")
    }))
}

fn format_process_failure(plan_label: &str, exit_code: Option<i32>) -> String {
    let code = exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".into());
    format!("{plan_label} exited with code {code}")
}

fn format_process_success(plan_label: &str, exit_code: Option<i32>) -> String {
    match exit_code {
        Some(code) => format!("{plan_label} exited successfully with code {code}"),
        None => format!("{plan_label} exited successfully"),
    }
}

fn replace_artifacts(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    artifacts: Vec<(RunArtifactKind, String, String)>,
) -> Result<()> {
    let stored = state.workbench_store.replace_artifacts(run_id, artifacts)?;
    emit_artifacts_replaced(app, run_id, &stored);
    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedResultContract {
    outcome: Option<String>,
    summary: String,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    pm_comment: Option<String>,
}

impl ParsedResultContract {
    fn outcome_status(&self) -> Option<RunOutcomeStatus> {
        match self.outcome.as_deref() {
            Some("blocked") => Some(RunOutcomeStatus::Blocked),
            Some("needs_input") => Some(RunOutcomeStatus::NeedsInput),
            Some("succeeded") => Some(RunOutcomeStatus::Succeeded),
            _ => None,
        }
    }
}

fn parse_result_contract(summary: &StreamSummary) -> Option<ParsedResultContract> {
    summary
        .agent_output_lines
        .iter()
        .rev()
        .find_map(|line| {
            let payload = line
                .trim()
                .strip_prefix("UMBRA_RESULT_JSON:")
                .map(str::trim)?;
            serde_json::from_str::<ParsedResultContract>(payload).ok()
        })
}

fn infer_outcome_status(summary: &StreamSummary) -> RunOutcomeStatus {
    let transcript = summary
        .agent_output_lines
        .join("\n")
        .to_lowercase();

    if transcript.contains("needs input")
        || transcript.contains("need input")
        || transcript.contains("need clarification")
        || transcript.contains("need your input")
    {
        return RunOutcomeStatus::NeedsInput;
    }

    if transcript.contains("blocked")
        || transcript.contains("cannot continue")
        || transcript.contains("can't continue")
        || transcript.contains("unable to proceed")
    {
        return RunOutcomeStatus::Blocked;
    }

    RunOutcomeStatus::Succeeded
}

fn format_outcome_status(status: &RunOutcomeStatus) -> &'static str {
    match status {
        RunOutcomeStatus::Succeeded => "succeeded",
        RunOutcomeStatus::Blocked => "blocked",
        RunOutcomeStatus::NeedsInput => "needs_input",
    }
}

fn build_result_summary(
    result: &ProviderExecutionResult,
    contract: Option<&ParsedResultContract>,
    outcome_status: &RunOutcomeStatus,
) -> String {
    if let Some(summary) = contract.and_then(|value| truncate_artifact_value(&value.summary, 1200)) {
        return format!(
            "{} ({})",
            summary,
            format_outcome_status(outcome_status)
        );
    }

    if result.success {
        format!(
            "{} finished successfully (exit code {}, outcome {})",
            result.plan.label,
            result.exit_code.unwrap_or(0),
            format_outcome_status(outcome_status)
        )
    } else {
        format_process_failure(&result.plan.label, result.exit_code)
    }
}

async fn build_run_artifacts(
    state: &AppState,
    run_id: &str,
    result: &ProviderExecutionResult,
    workspace_path: &Path,
    contract: Option<&ParsedResultContract>,
) -> Vec<(RunArtifactKind, String, String)> {
    let outcome_status = contract
        .and_then(|value| value.outcome_status())
        .unwrap_or_else(|| infer_outcome_status(&result.stdout_summary));
    let mut artifacts = vec![
        (
            RunArtifactKind::Summary,
            "result".into(),
            build_result_summary(result, contract, &outcome_status),
        ),
        (
            RunArtifactKind::Summary,
            "exit summary".into(),
            format!(
                "agent messages: {} | stdout events: {} | stderr events: {}",
                result.stdout_summary.agent_messages,
                result.stdout_summary.stdout_lines,
                result.stderr_summary.stderr_lines
            ),
        ),
    ];

    if let Some(contract) = contract {
        if let Some(summary) = truncate_artifact_value(&contract.summary, 1200) {
            artifacts.push((RunArtifactKind::Summary, "agent summary".into(), summary));
        }

        artifacts.extend(
            contract
                .files
                .iter()
                .take(25)
                .map(|path| (RunArtifactKind::File, path.clone(), "reported by provider".into())),
        );

        artifacts.extend(
            contract
                .tests
                .iter()
                .take(8)
                .filter_map(|value| {
                    truncate_artifact_value(value, 500)
                        .map(|text| (RunArtifactKind::Test, "provider test result".into(), text))
                }),
        );

        if let Some(pm_comment) = contract
            .pm_comment
            .as_ref()
            .and_then(|value| truncate_artifact_value(value, 800))
        {
            artifacts.push((RunArtifactKind::Summary, "pm comment".into(), pm_comment));
        }
    }

    let changed_files = detect_changed_files(workspace_path).await;
    if !changed_files.is_empty() {
        artifacts.push((
            RunArtifactKind::Summary,
            "changed files".into(),
            format!("{} tracked file(s) changed", changed_files.len()),
        ));
        artifacts.extend(
            changed_files
                .into_iter()
                .take(25)
                .map(|(path, status)| (RunArtifactKind::File, path, status)),
        );
    }

    if let Some(diff_preview) = detect_diff_preview(workspace_path).await {
        artifacts.push((RunArtifactKind::Summary, "diff preview".into(), diff_preview));
    }

    artifacts.extend(detect_test_artifacts(state, run_id));

    artifacts
}

async fn detect_changed_files(workspace_path: &Path) -> Vec<(String, String)> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;

    let Ok(output) = output else {
        return vec![];
    };
    if !output.status.success() {
        return vec![];
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_git_status_line)
        .collect()
}

async fn detect_diff_preview(workspace_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["diff", "--no-ext-diff", "--unified=0", "--", "."])
        .current_dir(workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    truncate_artifact_value(&String::from_utf8_lossy(&output.stdout), 2500)
}

fn detect_test_artifacts(state: &AppState, run_id: &str) -> Vec<(RunArtifactKind, String, String)> {
    let Ok(events) = state.workbench_store.list_events(run_id) else {
        return vec![];
    };

    let mut artifacts = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for event in events {
        let normalized = event.body.trim();
        if !looks_like_test_signal(normalized) {
            continue;
        }

        let dedupe_key = normalized.to_lowercase();
        if !seen.insert(dedupe_key) {
            continue;
        }

        let label = if normalized.to_lowercase().contains("fail") {
            "test signal (failure)"
        } else if normalized.to_lowercase().contains("pass") {
            "test signal (success)"
        } else {
            "test signal"
        };
        artifacts.push((
            RunArtifactKind::Test,
            label.into(),
            truncate_artifact_value(normalized, 500).unwrap_or_else(|| normalized.to_string()),
        ));
        if artifacts.len() >= 6 {
            break;
        }
    }

    artifacts
}

fn looks_like_test_signal(value: &str) -> bool {
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }

    let markers = [
        "cargo test",
        "npm test",
        "pnpm test",
        "yarn test",
        "pytest",
        "vitest",
        "jest",
        "tests passed",
        "test passed",
        "tests failed",
        "test failed",
        "failing test",
        "passed",
        "failed",
    ];

    markers.iter().any(|marker| normalized.contains(marker))
}

fn truncate_artifact_value(value: &str, max_chars: usize) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut shortened = trimmed.chars().take(max_chars).collect::<String>();
    if trimmed.chars().count() > max_chars {
        shortened.push_str("\n...");
    }
    Some(shortened)
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

fn parse_git_status_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_end();
    if trimmed.len() < 4 {
        return None;
    }

    let status = trimmed.get(0..2)?.trim().to_string();
    let path = trimmed.get(3..)?.trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some((
            path,
            if status.is_empty() {
                "??".into()
            } else {
                status
            },
        ))
    }
}

#[derive(Debug)]
struct ProviderExecutionResult {
    plan: ProviderExecutionPlan,
    success: bool,
    exit_code: Option<i32>,
    stdout_summary: StreamSummary,
    stderr_summary: StreamSummary,
}

#[derive(Debug, Default)]
struct StreamSummary {
    agent_messages: usize,
    stdout_lines: usize,
    stderr_lines: usize,
    agent_output_lines: Vec<String>,
    stdout_output_lines: Vec<String>,
    stderr_output_lines: Vec<String>,
}

impl StreamSummary {
    fn record(&mut self, event_type: &RunEventType, body: &str) {
        match event_type {
            RunEventType::AgentMessage => {
                self.agent_messages += 1;
                self.agent_output_lines.push(body.to_string());
            }
            RunEventType::Stdout => {
                self.stdout_lines += 1;
                self.stdout_output_lines.push(body.to_string());
            }
            RunEventType::Stderr => {
                self.stderr_lines += 1;
                self.stderr_output_lines.push(body.to_string());
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StreamSource {
    Stdout,
    Stderr,
}

#[cfg(test)]
mod tests {
    use super::{
        classify_stream_line, env_command_key, extract_stream_json_text, looks_like_test_signal,
        parse_git_status_line, truncate_artifact_value, StreamSource,
    };
    use crate::types::RunEventType;
    use crate::workbench::adapters::ProviderOutputMode;

    #[test]
    fn stream_json_text_prefers_result_field() {
        let line = r#"{"type":"result","result":"all tests passed"}"#;
        assert_eq!(
            extract_stream_json_text(line).as_deref(),
            Some("all tests passed")
        );
    }

    #[test]
    fn stream_json_text_reads_message_content() {
        let line = r#"{"message":{"content":[{"type":"text","text":"patched 2 files"}]}}"#;
        assert_eq!(
            extract_stream_json_text(line).as_deref(),
            Some("patched 2 files")
        );
    }

    #[test]
    fn plain_text_stdout_maps_to_agent_message() {
        let (event_type, body) = classify_stream_line(
            "done",
            &ProviderOutputMode::PlainTextAssistant,
            StreamSource::Stdout,
        );
        assert_eq!(event_type, RunEventType::AgentMessage);
        assert_eq!(body, "done");
    }

    #[test]
    fn stderr_maps_to_stderr_events() {
        let (event_type, body) = classify_stream_line(
            "boom",
            &ProviderOutputMode::StreamJson,
            StreamSource::Stderr,
        );
        assert_eq!(event_type, RunEventType::Stderr);
        assert_eq!(body, "boom");
    }

    #[test]
    fn provider_env_command_key_is_provider_specific() {
        assert_eq!(env_command_key("codex"), "UMBRA_CODEX_COMMAND");
        assert_eq!(env_command_key("claude"), "UMBRA_CLAUDE_COMMAND");
    }

    #[test]
    fn parses_git_status_output() {
        assert_eq!(
            parse_git_status_line(" M src/main.rs"),
            Some(("src/main.rs".into(), "M".into()))
        );
        assert_eq!(
            parse_git_status_line("?? docs/new.md"),
            Some(("docs/new.md".into(), "??".into()))
        );
    }

    #[test]
    fn test_signal_detection_catches_common_runner_output() {
        assert!(looks_like_test_signal("cargo test --package umbra"));
        assert!(looks_like_test_signal("2 tests failed"));
        assert!(!looks_like_test_signal("patched 2 files"));
    }

    #[test]
    fn artifact_truncation_keeps_signal_short() {
        let value = "a".repeat(600);
        let shortened = truncate_artifact_value(&value, 32).unwrap();
        assert!(shortened.len() < 40);
        assert!(shortened.ends_with("..."));
    }
}
