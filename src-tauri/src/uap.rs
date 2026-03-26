/// UAP — UMBRA Agent Protocol
/// Lightweight axum HTTP server (127.0.0.1:8765) for agent heartbeats and task dispatch.
///
/// Endpoints:
///   POST /api/agents/:id/heartbeat  — agent registers / updates status
///   GET  /api/agents/:id/tasks      — agent polls for pending tasks
///
/// Auth: X-Agent-Token header must match the configured token for the addressed agent id.
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::commands::config::AppConfig;
use crate::types::{Agent, AgentCronJob, AgentStatus, AgentTask};

// ── Registry ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AgentRegistry {
    pub agents: Arc<RwLock<HashMap<String, Agent>>>,
    pub task_queues: Arc<RwLock<HashMap<String, VecDeque<AgentTask>>>>,
    pub cron_jobs: Arc<RwLock<HashMap<String, Vec<AgentCronJob>>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queues: Arc::new(RwLock::new(HashMap::new())),
            cron_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// ── Axum State ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct UapState {
    registry: AgentRegistry,
    config: Arc<RwLock<AppConfig>>,
    app_handle: AppHandle,
}

// ── Request / Response shapes ────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatBody {
    name: String,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    status: Option<AgentStatus>,
    #[serde(default)]
    allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    skills: Option<Vec<String>>,
    #[serde(default)]
    active_task_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatResponse {
    ok: bool,
    pending_tasks: Vec<AgentTask>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CronJobsBody {
    #[serde(default)]
    agent_name: Option<String>,
    #[serde(default)]
    jobs: Vec<CronJobSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CronJobSnapshot {
    #[serde(default)]
    id: Option<String>,
    job: String,
    timing: String,
    recurrence: String,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default = "bool_true")]
    enabled: bool,
    #[serde(default)]
    last_run: Option<String>,
    #[serde(default)]
    next_run: Option<String>,
    #[serde(default = "default_cron_status")]
    last_status: String,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    command: Option<String>,
}

fn bool_true() -> bool {
    true
}
fn default_cron_status() -> String {
    "scheduled".into()
}

// ── Auth helper ──────────────────────────────────────────────────────────────

async fn check_token(headers: &HeaderMap, agent_id: &str, config: &Arc<RwLock<AppConfig>>) -> bool {
    let provided = headers
        .get("x-agent-token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let Some(provided) = provided else {
        return false;
    };

    let config = config.read().await;
    config
        .agent_auth_tokens
        .get(agent_id)
        .map(|expected| expected == provided)
        .unwrap_or(false)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn handle_heartbeat(
    Path(agent_id): Path<String>,
    State(state): State<UapState>,
    headers: HeaderMap,
    Json(body): Json<HeartbeatBody>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !check_token(&headers, &agent_id, &state.config).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid token"})),
        );
    }

    let now = Utc::now().to_rfc3339();

    // Upsert agent record
    let updated_agent = {
        let mut agents = state.registry.agents.write().await;
        let record = agents.entry(agent_id.clone()).or_insert_with(|| Agent {
            id: agent_id.clone(),
            name: body.name.clone(),
            role: String::new(),
            status: AgentStatus::Online,
            allowed_tools: vec![],
            skills: vec![],
            last_seen: now.clone(),
            active_task_id: None,
        });
        record.name = body.name;
        record.status = body.status.unwrap_or(AgentStatus::Online);
        record.last_seen = now;
        if let Some(role) = body.role {
            record.role = role;
        }
        if let Some(tools) = body.allowed_tools {
            record.allowed_tools = tools;
        }
        if let Some(skills) = body.skills {
            record.skills = skills;
        }
        record.active_task_id = body.active_task_id;
        record.clone()
    };

    // Emit full agent record to frontend
    let _ = state
        .app_handle
        .emit("agent-status-changed", &updated_agent);
    let _ = crate::sync_tray_with_agent_registry(&state.app_handle, &state.registry).await;

    // Drain pending tasks for this agent
    let tasks: Vec<AgentTask> = {
        let mut queues = state.registry.task_queues.write().await;
        queues
            .get_mut(&agent_id)
            .map(|q| q.drain(..).collect())
            .unwrap_or_default()
    };

    (
        StatusCode::OK,
        Json(
            serde_json::to_value(HeartbeatResponse {
                ok: true,
                pending_tasks: tasks,
            })
            .unwrap(),
        ),
    )
}

async fn handle_get_tasks(
    Path(agent_id): Path<String>,
    State(state): State<UapState>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    if !check_token(&headers, &agent_id, &state.config).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid token"})),
        );
    }

    let tasks: Vec<AgentTask> = {
        let mut queues = state.registry.task_queues.write().await;
        queues
            .get_mut(&agent_id)
            .map(|q| q.drain(..).collect())
            .unwrap_or_default()
    };

    (StatusCode::OK, Json(serde_json::to_value(tasks).unwrap()))
}

async fn handle_upsert_cron_jobs(
    Path(agent_id): Path<String>,
    State(state): State<UapState>,
    headers: HeaderMap,
    Json(body): Json<CronJobsBody>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !check_token(&headers, &agent_id, &state.config).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid token"})),
        );
    }

    let now = Utc::now().to_rfc3339();
    let agent_name = resolve_agent_name(&state.registry, &agent_id, body.agent_name.clone()).await;
    let jobs = materialize_cron_jobs(&agent_id, &agent_name, body.jobs, &now);

    {
        let mut registry_jobs = state.registry.cron_jobs.write().await;
        registry_jobs.insert(agent_id.clone(), jobs);
    }

    let all_jobs = collect_all_cron_jobs(&state.registry).await;
    let _ = state.app_handle.emit("agent-cron-updated", &all_jobs);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "agentId": agent_id,
            "jobCount": all_jobs.len(),
        })),
    )
}

async fn resolve_agent_name(
    registry: &AgentRegistry,
    agent_id: &str,
    preferred_name: Option<String>,
) -> String {
    if let Some(name) = preferred_name {
        let mut agents = registry.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.name = name.clone();
        }
        return name;
    }

    let agents = registry.agents.read().await;
    agents
        .get(agent_id)
        .map(|agent| agent.name.clone())
        .unwrap_or_else(|| agent_id.to_string())
}

fn materialize_cron_jobs(
    agent_id: &str,
    agent_name: &str,
    jobs: Vec<CronJobSnapshot>,
    now: &str,
) -> Vec<AgentCronJob> {
    jobs.into_iter()
        .map(|job| AgentCronJob {
            id: job
                .id
                .unwrap_or_else(|| fallback_cron_job_id(agent_id, &job.job)),
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            job: job.job,
            timing: job.timing,
            recurrence: job.recurrence,
            timezone: job.timezone,
            enabled: job.enabled,
            last_run: job.last_run,
            next_run: job.next_run,
            last_status: job.last_status,
            notes: job.notes,
            source: job.source,
            command: job.command,
            updated_at: now.to_string(),
        })
        .collect()
}

fn fallback_cron_job_id(agent_id: &str, job: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in job.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        format!("{agent_id}-job")
    } else {
        format!("{agent_id}-{slug}")
    }
}

pub async fn collect_all_cron_jobs(registry: &AgentRegistry) -> Vec<AgentCronJob> {
    let jobs = registry.cron_jobs.read().await;
    let mut flattened: Vec<AgentCronJob> = jobs.values().flatten().cloned().collect();
    flattened.sort_by(|a, b| {
        let a_next = a.next_run.as_deref().unwrap_or("9999");
        let b_next = b.next_run.as_deref().unwrap_or("9999");
        a_next
            .cmp(b_next)
            .then_with(|| a.agent_name.cmp(&b.agent_name))
            .then_with(|| a.job.cmp(&b.job))
    });
    flattened
}

// ── Server entry point ───────────────────────────────────────────────────────

pub async fn start_uap_server(
    app_handle: AppHandle,
    registry: AgentRegistry,
    config: Arc<RwLock<AppConfig>>,
    port: u16,
) {
    let state = UapState {
        registry,
        config,
        app_handle,
    };

    let app = Router::new()
        .route("/api/agents/{id}/heartbeat", post(handle_heartbeat))
        .route("/api/agents/{id}/tasks", get(handle_get_tasks))
        .route("/api/agents/{id}/cron-jobs", post(handle_upsert_cron_jobs))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port); // accepts localhost + Tailscale
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[UAP] Failed to bind {addr}: {e}");
            return;
        }
    };

    eprintln!("[UAP] Listening on http://{addr}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[UAP] Server error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{fallback_cron_job_id, materialize_cron_jobs, CronJobSnapshot};

    #[test]
    fn fallback_cron_job_id_is_stable_and_sanitized() {
        assert_eq!(
            fallback_cron_job_id("forge", "Daily Build @ 09:00"),
            "forge-daily-build-09-00"
        );
    }

    #[test]
    fn materialize_cron_jobs_preserves_explicit_ids() {
        let jobs = materialize_cron_jobs(
            "forge",
            "Forge",
            vec![CronJobSnapshot {
                id: Some("job-1".into()),
                job: "Daily Build".into(),
                timing: "09:00".into(),
                recurrence: "weekdays".into(),
                timezone: Some("Europe/Berlin".into()),
                enabled: true,
                last_run: None,
                next_run: Some("2026-03-21T09:00:00Z".into()),
                last_status: "ok".into(),
                notes: Some("publishes summary".into()),
                source: Some("agent".into()),
                command: None,
            }],
            "2026-03-20T13:00:00Z",
        );

        assert_eq!(jobs[0].id, "job-1");
        assert_eq!(jobs[0].agent_name, "Forge");
        assert_eq!(jobs[0].last_status, "ok");
    }
}
