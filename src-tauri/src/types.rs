use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Online,
    Working,
    Idle,
    Offline,
    Error,
}

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
    pub active_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentCronJob {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub job: String,
    pub timing: String,
    pub recurrence: String,
    pub timezone: Option<String>,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub last_status: String,
    pub notes: Option<String>,
    pub source: Option<String>,
    pub command: Option<String>,
    pub updated_at: String,
}

impl Agent {
    /// Returns true if the agent hasn't been seen in over 90 seconds.
    pub fn is_stale(&self) -> bool {
        let Ok(last) = chrono::DateTime::parse_from_rfc3339(&self.last_seen) else {
            return false;
        };
        let age = Utc::now().signed_duration_since(last.with_timezone(&Utc));
        age.num_seconds() > 1800 // 30 minutes
    }
}
