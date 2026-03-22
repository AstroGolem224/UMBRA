use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::commands::config::CustomAgentConfig;
use crate::errors::AppError;
use crate::state::AppState;
use crate::types::{Agent, AgentStatus, AgentTask};

const DEFAULT_AGENT_IDS: &[&str] = &["forge", "prism", "jim"];

/// Default agent roster — used to seed the registry on startup.
pub fn default_agents() -> Vec<Agent> {
    let now = Utc::now().to_rfc3339();
    vec![
        Agent {
            id: "forge".into(),
            name: "Forge".into(),
            role: "Web / Code Agent".into(),
            status: AgentStatus::Offline,
            allowed_tools: vec![
                "vscode".into(),
                "git".into(),
                "github".into(),
                "filesystem".into(),
                "shell".into(),
                "browser".into(),
            ],
            skills: vec![
                "TypeScript".into(),
                "Vue 3".into(),
                "Rust".into(),
                "FastAPI".into(),
                "Git".into(),
                "Architecture".into(),
                "TailwindCSS".into(),
            ],
            last_seen: now.clone(),
            active_task_id: None,
        },
        Agent {
            id: "prism".into(),
            name: "Prism".into(),
            role: "Godot / Game Dev Agent".into(),
            status: AgentStatus::Offline,
            allowed_tools: vec!["godot".into(), "git".into(), "filesystem".into()],
            skills: vec![
                "GDScript".into(),
                "Godot 4".into(),
                "Scene Design".into(),
                "Physics".into(),
                "Animation".into(),
                "2D".into(),
            ],
            last_seen: now.clone(),
            active_task_id: None,
        },
        Agent {
            id: "jim".into(),
            name: "Jim".into(),
            role: "Ops / Automation Agent".into(),
            status: AgentStatus::Offline,
            allowed_tools: vec![
                "shell".into(),
                "powershell".into(),
                "filesystem".into(),
                "cron".into(),
                "pm".into(),
            ],
            skills: vec![
                "Windows Automation".into(),
                "PowerShell".into(),
                "CI/CD".into(),
                "Observability".into(),
                "Task Orchestration".into(),
                "Release Ops".into(),
            ],
            last_seen: now.clone(),
            active_task_id: None,
        },
    ]
}

/// Returns all agents from the registry, marking stale agents as offline.
#[tauri::command]
pub async fn get_agents(state: State<'_, AppState>) -> std::result::Result<Vec<Agent>, AppError> {
    let mut agents_map = state.agent_registry.agents.write().await;

    for agent in agents_map.values_mut() {
        if agent.is_stale() && agent.status != AgentStatus::Offline {
            agent.status = AgentStatus::Offline;
        }
    }

    let mut agents: Vec<Agent> = agents_map.values().cloned().collect();
    // Stable ordering: default roster first, then others alphabetically
    let order = ["forge", "prism", "jim"];
    agents.sort_by_key(|a| {
        let pos = order.iter().position(|&id| id == a.id).unwrap_or(usize::MAX);
        (pos, a.name.clone())
    });

    Ok(agents)
}

/// Convert a CustomAgentConfig into a live Agent (offline by default).
pub fn custom_agent_to_agent(c: &CustomAgentConfig) -> Agent {
    Agent {
        id: c.id.clone(),
        name: c.name.clone(),
        role: c.role.clone(),
        status: AgentStatus::Offline,
        allowed_tools: c.allowed_tools.clone(),
        skills: c.skills.clone(),
        last_seen: Utc::now().to_rfc3339(),
        active_task_id: None,
    }
}

/// Add a new custom agent. Persists to config, adds to live registry.
#[tauri::command]
pub async fn add_agent(
    agent: CustomAgentConfig,
    state: State<'_, AppState>,
) -> std::result::Result<Agent, AppError> {
    // Validate: id must not collide with defaults or existing custom agents
    if DEFAULT_AGENT_IDS.contains(&agent.id.as_str()) {
        return Err(AppError::Other(format!("ID '{}' is reserved for a built-in agent", agent.id)));
    }

    // Persist in config
    let live_agent = custom_agent_to_agent(&agent);
    {
        let mut cfg = state.config.write().await;
        // Remove any existing entry with same id first (upsert)
        cfg.custom_agents.retain(|a| a.id != agent.id);
        cfg.custom_agents.push(agent);
        let json = serde_json::to_string_pretty(&*cfg)?;
        drop(cfg);
        tokio::fs::write(&state.config_path, json).await?;
    }

    // Add to live registry
    {
        let mut agents = state.agent_registry.agents.write().await;
        agents.insert(live_agent.id.clone(), live_agent.clone());
    }

    Ok(live_agent)
}

/// Remove an agent. Removes from config (if custom) and live registry.
#[tauri::command]
pub async fn remove_agent(
    id: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {

    // Remove from config
    {
        let mut cfg = state.config.write().await;
        cfg.custom_agents.retain(|a| a.id != id);
        let json = serde_json::to_string_pretty(&*cfg)?;
        drop(cfg);
        tokio::fs::write(&state.config_path, json).await?;
    }

    // Remove from live registry
    {
        let mut agents = state.agent_registry.agents.write().await;
        agents.remove(&id);
    }

    Ok(())
}

/// Push a task into an agent's pending queue. The agent receives it on next heartbeat.
#[tauri::command]
pub async fn push_agent_task(
    agent_id: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let task = AgentTask {
        id: Uuid::new_v4().to_string(),
        title,
        description,
        priority: priority.unwrap_or_else(|| "medium".into()),
    };

    let mut queues = state.agent_registry.task_queues.write().await;
    queues.entry(agent_id).or_default().push_back(task);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agents_has_three_entries() {
        assert_eq!(default_agents().len(), 3);
    }

    #[test]
    fn all_default_agents_offline() {
        for agent in default_agents() {
            assert_eq!(agent.status, AgentStatus::Offline);
        }
    }
}
