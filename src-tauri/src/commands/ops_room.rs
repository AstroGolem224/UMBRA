use std::collections::HashSet;

use tauri::{AppHandle, Emitter, State};

use crate::commands::workbench::{derive_provider_id, validate_create_input};
use crate::errors::{AppError, Result};
use crate::state::AppState;
use crate::types::{
    AppendRunEventInput, ChannelMessageKind, CreateDispatchRunInput, CreateOpsChannelInput,
    CreateOpsJobInput, DispatchMode, DispatchRun, OpsChannel, OpsChannelMessage,
    OpsChannelMessagePage, OpsJob, OpsJobStatus, OpsRouteApproval, OpsRule, OpsSession,
    OpsSessionState, OpsSessionTemplate, RouteApprovalStatus, SaveOpsRuleInput,
    SaveOpsSessionTemplateInput, SendOpsChannelMessageInput, StartOpsSessionInput,
};
use crate::workbench::runner;

#[tauri::command]
pub async fn list_ops_channels(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsChannel>, AppError> {
    state.workbench_store.list_channels().map_err(AppError::from)
}

#[tauri::command]
pub async fn create_ops_channel(
    input: CreateOpsChannelInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsChannel, AppError> {
    validate_channel_input(&input, &state).await?;
    let channel = state
        .workbench_store
        .create_channel(input)
        .map_err(AppError::from)?;
    emit_channel_created(&app, &channel);
    Ok(channel)
}

#[tauri::command]
pub async fn list_ops_channel_messages(
    channel_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsChannelMessage>, AppError> {
    state
        .workbench_store
        .list_channel_messages(&channel_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_ops_channel_messages_page(
    channel_id: String,
    before: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> std::result::Result<OpsChannelMessagePage, AppError> {
    state
        .workbench_store
        .list_channel_messages_page(&channel_id, before.as_deref(), limit.unwrap_or(100))
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn send_ops_channel_message(
    input: SendOpsChannelMessageInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsChannelMessage, AppError> {
    let channel = state
        .workbench_store
        .get_channel(&input.channel_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("channel {}", input.channel_id)))?;
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::Other("message body cannot be empty".into()));
    }

    let (updated_channel, message) = state
        .workbench_store
        .append_channel_message(
            &channel.id,
            input.parent_message_id,
            None,
            None,
            None,
            input.agent_id.clone(),
            Some("you".into()),
            ChannelMessageKind::User,
            body,
        )
        .map_err(AppError::from)?;
    emit_channel_updated(&app, &updated_channel);
    emit_channel_message_added(&app, &message);

    route_channel_message(&app, &state, &updated_channel, &message, None).await?;
    Ok(message)
}

#[tauri::command]
pub async fn list_ops_jobs(
    channel_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsJob>, AppError> {
    state
        .workbench_store
        .list_jobs(&channel_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn create_ops_job(
    input: CreateOpsJobInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsJob, AppError> {
    let channel = state
        .workbench_store
        .get_channel(&input.channel_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("channel {}", input.channel_id)))?;
    let source_message = state
        .workbench_store
        .list_channel_messages(&input.channel_id)
        .map_err(AppError::from)?
        .into_iter()
        .find(|message| message.id == input.source_message_id)
        .ok_or_else(|| AppError::TargetNotFound(format!("message {}", input.source_message_id)))?;

    if channel.workspace_id != input.workspace_id {
        return Err(AppError::TargetNotAllowed(
            "job workspace must match the channel workspace".into(),
        ));
    }

    let job = state
        .workbench_store
        .create_job(input.clone())
        .map_err(AppError::from)?;
    emit_job_updated(&app, &job);

    append_system_message(
        &app,
        &state,
        &channel.id,
        Some(source_message.id.clone()),
        None,
        Some(job.id.clone()),
        None,
        format!("job created: {}", job.title),
    )?;

    let run = spawn_ops_run(
        &app,
        &state,
        CreateDispatchRunInput {
            mode: DispatchMode::Task,
            agent_id: job.agent_id.clone(),
            workspace_id: job.workspace_id.clone(),
            channel_id: Some(channel.id.clone()),
            source_message_id: Some(source_message.id.clone()),
            job_id: Some(job.id.clone()),
            session_id: None,
            pm_task_id: job.pm_task_id.clone(),
            prompt: if job.summary.trim().is_empty() {
                job.title.clone()
            } else {
                format!("{}\n\n{}", job.title, job.summary)
            },
            persona_id: None,
            continue_from_run_id: None,
        },
    )
    .await?;
    let updated_job = state
        .workbench_store
        .set_job_run(&job.id, &run.id, OpsJobStatus::Running)
        .map_err(AppError::from)?;
    emit_job_updated(&app, &updated_job);

    append_system_message(
        &app,
        &state,
        &channel.id,
        Some(source_message.id),
        Some(run.id),
        Some(updated_job.id.clone()),
        None,
        format!("job dispatched to @{}", updated_job.agent_id),
    )?;

    Ok(updated_job)
}

#[tauri::command]
pub async fn list_ops_route_approvals(
    channel_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsRouteApproval>, AppError> {
    state
        .workbench_store
        .list_route_approvals(&channel_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn resolve_ops_route_approval(
    approval_id: String,
    approved: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsRouteApproval, AppError> {
    state
        .workbench_store
        .get_route_approval(&approval_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("route approval {}", approval_id)))?;
    let new_status = if approved {
        RouteApprovalStatus::Approved
    } else {
        RouteApprovalStatus::Rejected
    };
    let approval = state
        .workbench_store
        .resolve_route_approval(&approval_id, new_status)
        .map_err(AppError::from)?;
    emit_route_approval_updated(&app, &approval);

    if approved {
        let channel = state
            .workbench_store
            .get_channel(&approval.channel_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::TargetNotFound(format!("channel {}", approval.channel_id)))?;
        let messages = state
            .workbench_store
            .list_channel_messages(&approval.channel_id)
            .map_err(AppError::from)?;
        let source_message = messages
            .iter()
            .find(|message| message.id == approval.message_id)
            .ok_or_else(|| AppError::TargetNotFound(format!("message {}", approval.message_id)))?;

        append_system_message(
            &app,
            &state,
            &approval.channel_id,
            Some(source_message.id.clone()),
            None,
            None,
            None,
            format!("approval granted for @{}", approval.agent_id),
        )?;
        dispatch_channel_run(
            &app,
            &state,
            &channel,
            source_message,
            &approval.agent_id,
            None,
            None,
            None,
            DispatchMode::Chat,
        )
        .await?;
    } else {
        append_system_message(
            &app,
            &state,
            &approval.channel_id,
            Some(approval.message_id.clone()),
            None,
            None,
            None,
            format!("approval rejected for @{}", approval.agent_id),
        )?;
    }

    Ok(approval)
}

#[tauri::command]
pub async fn list_ops_rules(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsRule>, AppError> {
    state.workbench_store.list_rules().map_err(AppError::from)
}

#[tauri::command]
pub async fn save_ops_rule(
    input: SaveOpsRuleInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsRule, AppError> {
    let rule = state.workbench_store.save_rule(input).map_err(AppError::from)?;
    emit_rule_updated(&app, &rule);
    Ok(rule)
}

#[tauri::command]
pub async fn list_ops_session_templates(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsSessionTemplate>, AppError> {
    state
        .workbench_store
        .list_session_templates()
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn save_ops_session_template(
    input: SaveOpsSessionTemplateInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsSessionTemplate, AppError> {
    if input.agent_ids.is_empty() {
        return Err(AppError::Other(
            "session templates need at least one agent".into(),
        ));
    }
    let template = state
        .workbench_store
        .save_session_template(input)
        .map_err(AppError::from)?;
    emit_session_template_updated(&app, &template);
    Ok(template)
}

#[tauri::command]
pub async fn list_ops_sessions(
    channel_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<OpsSession>, AppError> {
    state
        .workbench_store
        .list_sessions(&channel_id)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn start_ops_session(
    input: StartOpsSessionInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsSession, AppError> {
    let channel = state
        .workbench_store
        .get_channel(&input.channel_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("channel {}", input.channel_id)))?;
    let template = state
        .workbench_store
        .get_session_template(&input.template_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("session template {}", input.template_id)))?;
    if template.workspace_id != channel.workspace_id {
        return Err(AppError::TargetNotAllowed(
            "session template workspace must match channel workspace".into(),
        ));
    }

    let session = state
        .workbench_store
        .start_session(input)
        .map_err(AppError::from)?;
    emit_session_updated(&app, &session);
    if let Some(agent_id) = current_session_agent(&template, session.current_turn_index) {
        append_system_message(
            &app,
            &state,
            &channel.id,
            None,
            None,
            None,
            Some(session.id.clone()),
            format!("session started. current turn: @{}", agent_id),
        )?;
    }
    Ok(session)
}

#[tauri::command]
pub async fn advance_ops_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsSession, AppError> {
    let session = state
        .workbench_store
        .get_session(&session_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("session {}", session_id)))?;
    let template = state
        .workbench_store
        .get_session_template(&session.template_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::TargetNotFound(format!("session template {}", session.template_id)))?;
    let next_index = next_session_turn_index(&template, session.current_turn_index);
    let updated = state
        .workbench_store
        .set_session_turn(&session.id, next_index, false)
        .map_err(AppError::from)?;
    emit_session_updated(&app, &updated);
    if let Some(agent_id) = current_session_agent(&template, updated.current_turn_index) {
        append_system_message(
            &app,
            &state,
            &updated.channel_id,
            None,
            None,
            None,
            Some(updated.id.clone()),
            format!("session advanced. current turn: @{}", agent_id),
        )?;
    }
    Ok(updated)
}

#[tauri::command]
pub async fn pause_ops_session(
    session_id: String,
    paused: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<OpsSession, AppError> {
    let updated = state
        .workbench_store
        .update_session_state(
            &session_id,
            if paused {
                OpsSessionState::Paused
            } else {
                OpsSessionState::Active
            },
        )
        .map_err(AppError::from)?;
    emit_session_updated(&app, &updated);
    Ok(updated)
}

async fn route_channel_message(
    app: &AppHandle,
    state: &State<'_, AppState>,
    channel: &OpsChannel,
    message: &OpsChannelMessage,
    linked_job_id: Option<String>,
) -> Result<()> {
    let rules = state.workbench_store.list_rules()?;
    let active_session = state.workbench_store.get_active_session_for_channel(&channel.id)?;
    let session_template = if let Some(session) = &active_session {
        state.workbench_store.get_session_template(&session.template_id)?
    } else {
        None
    };

    let mut targets = resolve_message_targets(
        state,
        channel,
        message,
        active_session.as_ref(),
        session_template.as_ref(),
    )
    .await?;
    targets.sort();
    targets.dedup();

    for agent_id in targets {
        let matching_rules = matching_rules(&rules, &agent_id, &channel.workspace_id, &message.body);
        for rule in &matching_rules {
            let _ = state.workbench_store.set_rule_triggered(&rule.id);
        }
        let requires_human_gate = matching_rules.iter().any(|rule| rule.requires_human_gate)
            || session_template
                .as_ref()
                .map(|template| template.requires_human_gate)
                .unwrap_or(false);

        if requires_human_gate {
            let approval = state.workbench_store.create_route_approval(
                &channel.id,
                &message.id,
                &agent_id,
                &channel.workspace_id,
                "matching route rule requires human approval",
            )?;
            emit_route_approval_updated(app, &approval);
            append_system_message(
                app,
                state,
                &channel.id,
                Some(message.id.clone()),
                None,
                linked_job_id.clone(),
                active_session.as_ref().map(|session| session.id.clone()),
                format!("approval required before routing to @{}", agent_id),
            )?;
            continue;
        }

        dispatch_channel_run(
            app,
            state,
            channel,
            message,
            &agent_id,
            linked_job_id.clone(),
            active_session.as_ref(),
            session_template.as_ref(),
            if linked_job_id.is_some() {
                DispatchMode::Task
            } else {
                DispatchMode::Chat
            },
        )
        .await?;
    }

    Ok(())
}

async fn resolve_message_targets(
    state: &State<'_, AppState>,
    channel: &OpsChannel,
    message: &OpsChannelMessage,
    active_session: Option<&OpsSession>,
    session_template: Option<&OpsSessionTemplate>,
) -> Result<Vec<String>> {
    if let Some(agent_id) = message.agent_id.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        return Ok(vec![agent_id.to_string()]);
    }

    let mentions = extract_mentions(&message.body);
    if mentions.contains("all") {
        let agents = state.agent_registry.agents.read().await;
        return Ok(agents
            .values()
            .filter(|agent| !matches!(crate::normalized_agent_status(agent), crate::types::AgentStatus::Offline))
            .map(|agent| agent.id.clone())
            .collect());
    }

    if !mentions.is_empty() {
        return Ok(mentions.into_iter().collect());
    }

    if let (Some(session), Some(template)) = (active_session, session_template) {
        if matches!(session.state, OpsSessionState::Active) {
            if let Some(agent_id) = current_session_agent(template, session.current_turn_index) {
                return Ok(vec![agent_id.to_string()]);
            }
        }
    }

    if let Some(agent_id) = &channel.default_agent_id {
        return Ok(vec![agent_id.clone()]);
    }

    Ok(Vec::new())
}

async fn dispatch_channel_run(
    app: &AppHandle,
    state: &State<'_, AppState>,
    channel: &OpsChannel,
    message: &OpsChannelMessage,
    agent_id: &str,
    job_id: Option<String>,
    session: Option<&OpsSession>,
    session_template: Option<&OpsSessionTemplate>,
    mode: DispatchMode,
) -> Result<DispatchRun> {
    let run = spawn_ops_run(
        app,
        state,
        CreateDispatchRunInput {
            mode,
            agent_id: agent_id.to_string(),
            workspace_id: channel.workspace_id.clone(),
            channel_id: Some(channel.id.clone()),
            source_message_id: Some(message.id.clone()),
            job_id: job_id.clone(),
            session_id: session.map(|entry| entry.id.clone()),
            pm_task_id: None,
            prompt: message.body.clone(),
            persona_id: None,
            continue_from_run_id: None,
        },
    )
    .await?;

    append_system_message(
        app,
        state,
        &channel.id,
        Some(message.id.clone()),
        Some(run.id.clone()),
        job_id.clone(),
        session.map(|entry| entry.id.clone()),
        format!("dispatched to @{}", agent_id),
    )?;

    if let (Some(job_id), DispatchMode::Task) = (job_id.clone(), run.mode.clone()) {
        let job = state
            .workbench_store
            .set_job_run(&job_id, &run.id, OpsJobStatus::Running)?;
        emit_job_updated(app, &job);
    }

    if let (Some(session), Some(template)) = (session, session_template) {
        if template.auto_advance {
            let next_index = next_session_turn_index(template, session.current_turn_index);
            let updated_session = state
                .workbench_store
                .set_session_turn(&session.id, next_index, false)?;
            emit_session_updated(app, &updated_session);
            if let Some(next_agent_id) = current_session_agent(template, updated_session.current_turn_index) {
                append_system_message(
                    app,
                    state,
                    &channel.id,
                    Some(message.id.clone()),
                    Some(run.id.clone()),
                    job_id,
                    Some(updated_session.id.clone()),
                    format!("next turn: @{}", next_agent_id),
                )?;
            }
        }
    }

    Ok(run)
}

async fn spawn_ops_run(
    app: &AppHandle,
    state: &State<'_, AppState>,
    input: CreateDispatchRunInput,
) -> Result<DispatchRun> {
    let validated = validate_create_input(&input, state).await?;
    let continue_from_run_id = validated.continue_from_run_id.clone();
    let provider_id = derive_provider_id(&validated.agent_id);
    let (run, first_event) = state.workbench_store.create_run(validated, provider_id)?;
    let _ = app.emit("workbench:run-created", &run);
    let _ = app.emit("workbench:event-added", &first_event);
    if let Some(parent_run_id) = continue_from_run_id {
        let (_, event) = state.workbench_store.append_event(AppendRunEventInput {
            run_id: run.id.clone(),
            event_type: crate::types::RunEventType::System,
            body: format!("continued from run {parent_run_id}"),
        })?;
        let _ = app.emit("workbench:event-added", &event);
    }
    runner::spawn_run(app.clone(), state.inner().clone(), run.id.clone());
    Ok(run)
}

fn matching_rules<'a>(
    rules: &'a [OpsRule],
    agent_id: &str,
    workspace_id: &str,
    body: &str,
) -> Vec<&'a OpsRule> {
    let normalized_body = body.to_lowercase();
    rules
        .iter()
        .filter(|rule| rule.enabled)
        .filter(|rule| {
            rule.target_agent_id
                .as_deref()
                .map(|value| value == agent_id)
                .unwrap_or(true)
        })
        .filter(|rule| {
            rule.workspace_id
                .as_deref()
                .map(|value| value == workspace_id)
                .unwrap_or(true)
        })
        .filter(|rule| {
            let pattern = rule.pattern.trim().to_lowercase();
            pattern == "*" || normalized_body.contains(&pattern)
        })
        .collect()
}

fn extract_mentions(body: &str) -> HashSet<String> {
    body.split_whitespace()
        .filter_map(|segment| segment.strip_prefix('@'))
        .map(|segment| {
            segment
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
                .to_lowercase()
        })
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn current_session_agent<'a>(
    template: &'a OpsSessionTemplate,
    current_turn_index: usize,
) -> Option<&'a str> {
    if template.agent_ids.is_empty() {
        return None;
    }
    template
        .agent_ids
        .get(current_turn_index % template.agent_ids.len())
        .map(String::as_str)
}

fn next_session_turn_index(template: &OpsSessionTemplate, current_turn_index: usize) -> usize {
    if template.agent_ids.is_empty() {
        0
    } else {
        (current_turn_index + 1) % template.agent_ids.len()
    }
}

async fn validate_channel_input(
    input: &CreateOpsChannelInput,
    state: &State<'_, AppState>,
) -> Result<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Other("channel name cannot be empty".into()));
    }

    let config = state.config.read().await;
    if !config
        .workspace_presets
        .iter()
        .any(|workspace| workspace.id == input.workspace_id)
    {
        return Err(AppError::TargetNotFound(format!(
            "workspace {}",
            input.workspace_id
        )));
    }
    Ok(())
}

fn append_system_message(
    app: &AppHandle,
    state: &State<'_, AppState>,
    channel_id: &str,
    parent_message_id: Option<String>,
    run_id: Option<String>,
    job_id: Option<String>,
    session_id: Option<String>,
    body: String,
) -> Result<()> {
    let (channel, message) = state.workbench_store.append_channel_message(
        channel_id,
        parent_message_id,
        run_id,
        job_id,
        session_id,
        None,
        Some("umbra".into()),
        ChannelMessageKind::System,
        body,
    )?;
    emit_channel_updated(app, &channel);
    emit_channel_message_added(app, &message);
    Ok(())
}

pub(crate) fn emit_channel_created(app: &AppHandle, channel: &OpsChannel) {
    let _ = app.emit("ops:channel-created", channel);
}

pub(crate) fn emit_channel_updated(app: &AppHandle, channel: &OpsChannel) {
    let _ = app.emit("ops:channel-updated", channel);
}

pub(crate) fn emit_channel_message_added(app: &AppHandle, message: &OpsChannelMessage) {
    let _ = app.emit("ops:message-added", message);
}

pub(crate) fn emit_job_updated(app: &AppHandle, job: &OpsJob) {
    let _ = app.emit("ops:job-updated", job);
}

pub(crate) fn emit_route_approval_updated(app: &AppHandle, approval: &OpsRouteApproval) {
    let _ = app.emit("ops:approval-updated", approval);
}

pub(crate) fn emit_rule_updated(app: &AppHandle, rule: &OpsRule) {
    let _ = app.emit("ops:rule-updated", rule);
}

pub(crate) fn emit_session_template_updated(app: &AppHandle, template: &OpsSessionTemplate) {
    let _ = app.emit("ops:session-template-updated", template);
}

pub(crate) fn emit_session_updated(app: &AppHandle, session: &OpsSession) {
    let _ = app.emit("ops:session-updated", session);
}
