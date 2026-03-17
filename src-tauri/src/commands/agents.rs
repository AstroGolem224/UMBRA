use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub allowed_tools: Vec<String>,
    pub skills: Vec<String>,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Online,
    Idle,
    Offline,
    Error,
}

fn default_agents() -> Vec<Agent> {
    let now = Utc::now().to_rfc3339();
    vec![
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
        },
        Agent {
            id: "forge".into(),
            name: "Forge".into(),
            role: "Web / Code Agent".into(),
            status: AgentStatus::Online,
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
        },
        Agent {
            id: "jim".into(),
            name: "Jim".into(),
            role: "Master Dev / Architecture Agent".into(),
            status: AgentStatus::Idle,
            allowed_tools: vec![
                "vscode".into(),
                "git".into(),
                "github".into(),
                "filesystem".into(),
                "browser".into(),
                "notebooklm".into(),
            ],
            skills: vec![
                "Architecture".into(),
                "UI/UX".into(),
                "System Design".into(),
                "Code Review".into(),
                "Planning".into(),
            ],
            last_seen: now,
        },
    ]
}

#[tauri::command]
pub async fn get_agents(_state: State<'_, AppState>) -> std::result::Result<Vec<Agent>, AppError> {
    // In future: read from agents.md in vault or check heartbeat endpoints
    Ok(default_agents())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agents_has_three_entries() {
        let agents = default_agents();
        assert_eq!(agents.len(), 3);
    }

    #[test]
    fn forge_is_online_by_default() {
        let agents = default_agents();
        let forge = agents.iter().find(|a| a.id == "forge").unwrap();
        assert_eq!(forge.status, AgentStatus::Online);
    }
}
