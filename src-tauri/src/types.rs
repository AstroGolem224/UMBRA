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
    /// Returns true if the agent hasn't been seen in over 30 minutes.
    pub fn is_stale(&self) -> bool {
        let Ok(last) = chrono::DateTime::parse_from_rfc3339(&self.last_seen) else {
            return false;
        };
        let age = Utc::now().signed_duration_since(last.with_timezone(&Utc));
        age.num_seconds() > 1800 // 30 minutes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePreset {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub writable: bool,
    #[serde(default)]
    pub allowed_providers: Vec<String>,
    #[serde(default)]
    pub allowed_agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaPreset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DispatchMode {
    Chat,
    Task,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DispatchStatus {
    Draft,
    Queued,
    Working,
    Done,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcomeStatus {
    Succeeded,
    Blocked,
    NeedsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRun {
    pub id: String,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub source_message_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub mode: DispatchMode,
    pub agent_id: String,
    pub provider_id: String,
    pub workspace_id: String,
    #[serde(default)]
    pub pm_task_id: Option<String>,
    pub prompt: String,
    pub persona_id: Option<String>,
    #[serde(default)]
    pub outcome_status: Option<RunOutcomeStatus>,
    pub status: DispatchStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunEventType {
    UserMessage,
    System,
    Stdout,
    Stderr,
    AgentMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub id: String,
    pub run_id: String,
    #[serde(rename = "type")]
    pub event_type: RunEventType,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunEventPage {
    pub items: Vec<RunEvent>,
    #[serde(default)]
    pub next_before: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunArtifactKind {
    Summary,
    File,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMessageKind {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsChannel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub workspace_id: String,
    #[serde(default)]
    pub default_agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsChannelMessage {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub parent_message_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub author_label: Option<String>,
    pub kind: ChannelMessageKind,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsChannelMessagePage {
    pub items: Vec<OpsChannelMessage>,
    #[serde(default)]
    pub next_before: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateOpsChannelInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub workspace_id: String,
    #[serde(default)]
    pub default_agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendOpsChannelMessageInput {
    pub channel_id: String,
    #[serde(default)]
    pub parent_message_id: Option<String>,
    pub body: String,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpsJobStatus {
    Open,
    Running,
    Blocked,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsJob {
    pub id: String,
    pub channel_id: String,
    pub source_message_id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub agent_id: String,
    pub workspace_id: String,
    #[serde(default)]
    pub pm_task_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    pub status: OpsJobStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateOpsJobInput {
    pub channel_id: String,
    pub source_message_id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub agent_id: String,
    pub workspace_id: String,
    #[serde(default)]
    pub pm_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RouteApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsRouteApproval {
    pub id: String,
    pub channel_id: String,
    pub message_id: String,
    pub agent_id: String,
    pub workspace_id: String,
    pub reason: String,
    pub status: RouteApprovalStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub target_agent_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub enabled: bool,
    pub requires_human_gate: bool,
    #[serde(default)]
    pub last_triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveOpsRuleInput {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub target_agent_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub enabled: bool,
    pub requires_human_gate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpsSessionState {
    Active,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsSessionTemplate {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub agent_ids: Vec<String>,
    pub auto_advance: bool,
    pub requires_human_gate: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveOpsSessionTemplateInput {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub workspace_id: String,
    pub agent_ids: Vec<String>,
    pub auto_advance: bool,
    pub requires_human_gate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpsSession {
    pub id: String,
    pub channel_id: String,
    pub template_id: String,
    pub state: OpsSessionState,
    pub current_turn_index: usize,
    #[serde(default)]
    pub awaiting_human_gate: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StartOpsSessionInput {
    pub channel_id: String,
    pub template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunArtifact {
    pub id: String,
    pub run_id: String,
    pub kind: RunArtifactKind,
    pub label: String,
    pub value: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateDispatchRunInput {
    pub mode: DispatchMode,
    pub agent_id: String,
    pub workspace_id: String,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub source_message_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub pm_task_id: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub continue_from_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppendRunEventInput {
    pub run_id: String,
    #[serde(rename = "type")]
    pub event_type: RunEventType,
    pub body: String,
}
