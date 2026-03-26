use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::fs;
use uuid::Uuid;

use crate::errors::AppError;
use crate::state::AppState;
use crate::types::{PersonaPreset, WorkspacePreset};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTarget {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GithubTarget {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CronJobConfig {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub command: String,
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub last_run: Option<String>,
    #[serde(default = "default_last_status")]
    pub last_status: String,
    #[serde(default)]
    pub last_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCommandConfig {
    pub provider_id: String,
    pub command: String,
}

fn bool_true() -> bool {
    true
}

fn default_last_status() -> String {
    "pending".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_close_to_tray")]
    pub close_to_tray: bool,
    #[serde(default = "default_vault_path")]
    pub vault_path: String,
    #[serde(default = "default_notes_subdir")]
    pub notes_subdir: String,
    #[serde(default = "default_repo_root_path")]
    pub repo_root_path: String,
    #[serde(default)]
    pub workspace_presets: Vec<WorkspacePreset>,
    #[serde(default)]
    pub workspace_grant_roots: Vec<String>,
    #[serde(default)]
    pub default_workspace_id: Option<String>,
    #[serde(default = "default_persona_presets")]
    pub persona_presets: Vec<PersonaPreset>,
    #[serde(default = "default_provider_commands")]
    pub provider_commands: Vec<ProviderCommandConfig>,
    #[serde(default)]
    pub launch_targets: Vec<LaunchTarget>,
    #[serde(default)]
    pub github_targets: Vec<GithubTarget>,
    #[serde(default = "default_pm_url")]
    pub pm_tool_url: String,
    #[serde(default = "default_pm_dashboard_url")]
    pub pm_tool_dashboard_url: String,
    #[serde(default = "default_poll_seconds")]
    pub pm_tool_poll_seconds: u64,
    #[serde(default = "default_updater_endpoint")]
    pub updater_endpoint: String,
    #[serde(default = "default_updater_public_key")]
    pub updater_public_key: String,
    #[serde(default)]
    pub auto_check_for_updates: bool,
    #[serde(default = "default_uap_advertise_host")]
    pub uap_advertise_host: String,
    #[serde(default = "default_uap_port")]
    pub uap_port: u16,
    #[serde(default = "default_uap_token")]
    pub uap_token: String,
    #[serde(default)]
    pub cron_jobs: Vec<CronJobConfig>,
    #[serde(default)]
    pub github_pat: Option<String>,
    #[serde(default)]
    pub task_lane_prefs: HashMap<String, bool>,
    #[serde(default)]
    pub agent_notes: HashMap<String, AgentNote>,
    #[serde(default)]
    pub agent_auth_tokens: HashMap<String, String>,
    #[serde(default)]
    pub custom_agents: Vec<CustomAgentConfig>,
    /// Explicit agent-id → provider-id mapping. Overrides the name-prefix heuristic.
    /// Example: { "forge": "claude", "prism": "gemini" }
    #[serde(default = "default_agent_provider_map")]
    pub agent_provider_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentNote {
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

fn default_theme() -> String {
    "ember".into()
}

fn default_close_to_tray() -> bool {
    true
}

fn default_vault_path() -> String {
    String::new()
}

fn default_notes_subdir() -> String {
    "UMBRA_Notes".into()
}

fn default_repo_root_path() -> String {
    String::new()
}

fn default_pm_url() -> String {
    String::new()
}

fn default_pm_dashboard_url() -> String {
    String::new()
}

fn default_poll_seconds() -> u64 {
    30
}

fn default_updater_endpoint() -> String {
    String::new()
}

fn default_updater_public_key() -> String {
    String::new()
}

fn default_uap_advertise_host() -> String {
    "127.0.0.1".into()
}

fn default_uap_port() -> u16 {
    8765
}

fn default_uap_token() -> String {
    Uuid::new_v4().to_string()
}

fn default_agent_provider_map() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("forge".into(), "claude".into());
    m.insert("prism".into(), "gemini".into());
    m
}

impl AppConfig {
    fn normalize(mut self) -> Self {
        if !matches!(self.theme.as_str(), "ember" | "neon" | "light") {
            self.theme = default_theme();
        }

        self.vault_path = self.vault_path.trim().to_string();
        self.notes_subdir = normalize_or_default(self.notes_subdir, default_notes_subdir());
        self.repo_root_path = self.repo_root_path.trim().to_string();
        self.workspace_presets = self
            .workspace_presets
            .into_iter()
            .filter_map(|mut preset| {
                preset.id = normalize_or_default(preset.id, Uuid::new_v4().to_string());
                preset.name = preset.name.trim().to_string();
                preset.root_path = preset.root_path.trim().to_string();
                preset.allowed_providers = preset
                    .allowed_providers
                    .into_iter()
                    .filter_map(trim_to_option)
                    .collect();
                preset.allowed_agents = preset
                    .allowed_agents
                    .into_iter()
                    .filter_map(trim_to_option)
                    .collect();
                if preset.name.is_empty() || preset.root_path.is_empty() {
                    None
                } else {
                    Some(preset)
                }
            })
            .collect();
        self.workspace_grant_roots =
            normalize_workspace_grant_roots(self.workspace_grant_roots, &self.workspace_presets);
        self.default_workspace_id = self
            .default_workspace_id
            .and_then(|value| trim_to_option(value))
            .filter(|workspace_id| {
                self.workspace_presets
                    .iter()
                    .any(|preset| preset.id == *workspace_id)
            });
        self.persona_presets = normalize_persona_presets(self.persona_presets);
        self.provider_commands = normalize_provider_commands(self.provider_commands);
        self.pm_tool_url = normalize_url(&self.pm_tool_url);
        self.pm_tool_dashboard_url = normalize_url(&self.pm_tool_dashboard_url);
        self.pm_tool_poll_seconds = self.pm_tool_poll_seconds.clamp(5, 300);
        self.updater_endpoint = normalize_endpoint_list(self.updater_endpoint);
        self.updater_public_key = self.updater_public_key.trim().to_string();
        self.uap_advertise_host =
            normalize_or_default(self.uap_advertise_host, default_uap_advertise_host());
        self.uap_port = clamp_uap_port(self.uap_port);
        self.uap_token = normalize_or_default(self.uap_token, default_uap_token());

        self.github_pat = self
            .github_pat
            .and_then(|token| trim_to_option(token.trim().to_string()));

        self.launch_targets = self
            .launch_targets
            .into_iter()
            .filter_map(|mut target| {
                target.id = normalize_or_default(target.id, Uuid::new_v4().to_string());
                target.name = target.name.trim().to_string();
                target.path = target.path.trim().to_string();
                target.icon = target.icon.and_then(|icon| trim_to_option(icon));
                if target.name.is_empty() || target.path.is_empty() {
                    None
                } else {
                    Some(target)
                }
            })
            .collect();

        self.github_targets = self
            .github_targets
            .into_iter()
            .filter_map(|mut target| {
                target.id = normalize_or_default(target.id, Uuid::new_v4().to_string());
                target.name = target.name.trim().to_string();
                target.owner = target.owner.trim().to_string();
                target.repo = target.repo.trim().to_string();
                if target.name.is_empty() || target.owner.is_empty() || target.repo.is_empty() {
                    None
                } else {
                    Some(target)
                }
            })
            .collect();

        self.task_lane_prefs.retain(|kind, _| {
            matches!(kind.as_str(), "backlog" | "in_progress" | "review" | "done")
        });

        self.agent_notes = self
            .agent_notes
            .into_iter()
            .filter_map(|(agent_id, mut note)| {
                let id = agent_id.trim().to_string();
                if id.is_empty() {
                    return None;
                }
                note.notes = note.notes.trim().to_string();
                note.link = note.link.trim().to_string();
                Some((id, note))
            })
            .collect();

        self.custom_agents = self
            .custom_agents
            .into_iter()
            .filter_map(|mut agent| {
                agent.id = normalize_or_default(agent.id, Uuid::new_v4().to_string());
                agent.name = agent.name.trim().to_string();
                agent.role = agent.role.trim().to_string();
                agent.skills = agent
                    .skills
                    .into_iter()
                    .filter_map(|skill| trim_to_option(skill))
                    .collect();
                agent.allowed_tools = agent
                    .allowed_tools
                    .into_iter()
                    .filter_map(|tool| trim_to_option(tool))
                    .collect();
                if agent.name.is_empty() || agent.role.is_empty() {
                    None
                } else {
                    Some(agent)
                }
            })
            .collect();
        self.agent_auth_tokens =
            normalize_agent_auth_tokens(self.agent_auth_tokens, &self.custom_agents);

        self
    }
}

fn normalize_or_default(value: String, fallback: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed.to_string()
    }
}

fn trim_to_option(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn default_persona_presets() -> Vec<PersonaPreset> {
    vec![
        PersonaPreset {
            id: "implementer".into(),
            name: "implementer".into(),
            description: "prefer shipping code, tests, and a concise result summary.".into(),
            prompt: "you are the implementation persona. prefer concrete code changes over discussion. finish with changed files, checks, and blockers.".into(),
        },
        PersonaPreset {
            id: "reviewer".into(),
            name: "reviewer".into(),
            description: "prioritize bugs, regressions, missing tests, and trust boundaries.".into(),
            prompt: "you are the review persona. prioritize findings, regressions, missing tests, trust boundaries, and operational risk over summaries.".into(),
        },
        PersonaPreset {
            id: "architect".into(),
            name: "architect".into(),
            description: "plan with clear tradeoffs, rollout phases, and failure modes.".into(),
            prompt: "you are the architecture persona. respond with a concrete plan, tradeoffs, rollout order, and critical failure modes before implementation details.".into(),
        },
    ]
}

fn default_agent_ids() -> Vec<String> {
    vec!["forge".into(), "prism".into()]
}

fn normalize_agent_auth_tokens(
    existing: HashMap<String, String>,
    custom_agents: &[CustomAgentConfig],
) -> HashMap<String, String> {
    let mut valid_ids = default_agent_ids();
    valid_ids.extend(custom_agents.iter().map(|agent| agent.id.clone()));

    let mut normalized = HashMap::new();
    for agent_id in valid_ids {
        let token = existing
            .get(&agent_id)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        normalized.insert(agent_id, token);
    }

    normalized
}

fn normalize_workspace_grant_roots(
    existing: Vec<String>,
    workspace_presets: &[WorkspacePreset],
) -> Vec<String> {
    let mut roots = existing
        .into_iter()
        .filter_map(trim_to_option)
        .collect::<Vec<_>>();
    roots.extend(
        workspace_presets
            .iter()
            .filter_map(|preset| trim_to_option(preset.root_path.clone())),
    );
    roots.sort_by_key(|value| value.to_lowercase());
    roots.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    roots
}

fn normalize_persona_presets(value: Vec<PersonaPreset>) -> Vec<PersonaPreset> {
    let presets = value
        .into_iter()
        .filter_map(|mut preset| {
            preset.id = normalize_or_default(preset.id, Uuid::new_v4().to_string());
            preset.name = preset.name.trim().to_string();
            preset.description = preset.description.trim().to_string();
            preset.prompt = preset.prompt.trim().to_string();
            if preset.name.is_empty() || preset.prompt.is_empty() {
                None
            } else {
                Some(preset)
            }
        })
        .collect::<Vec<_>>();

    if presets.is_empty() {
        return default_persona_presets();
    }

    presets
}

fn default_provider_commands() -> Vec<ProviderCommandConfig> {
    vec![
        ProviderCommandConfig {
            provider_id: "codex".into(),
            command: "codex".into(),
        },
        ProviderCommandConfig {
            provider_id: "claude".into(),
            command: "claude".into(),
        },
        ProviderCommandConfig {
            provider_id: "gemini".into(),
            command: "gemini".into(),
        },
        ProviderCommandConfig {
            provider_id: "kimi".into(),
            command: "kimi".into(),
        },
    ]
}

fn normalize_provider_commands(value: Vec<ProviderCommandConfig>) -> Vec<ProviderCommandConfig> {
    let mut by_provider = HashMap::<String, String>::new();
    for entry in value {
        let provider_id = entry.provider_id.trim().to_lowercase();
        let command = entry.command.trim().to_string();
        if provider_id.is_empty() || command.is_empty() {
            continue;
        }
        by_provider.insert(provider_id, command);
    }

    let defaults = default_provider_commands();
    let mut normalized = defaults
        .iter()
        .map(|default| ProviderCommandConfig {
            provider_id: default.provider_id.clone(),
            command: by_provider
                .remove(&default.provider_id)
                .unwrap_or_else(|| default.command.clone()),
        })
        .collect::<Vec<_>>();

    let mut extras = by_provider
        .into_iter()
        .map(|(provider_id, command)| ProviderCommandConfig {
            provider_id,
            command,
        })
        .collect::<Vec<_>>();
    extras.sort_by(|left, right| left.provider_id.cmp(&right.provider_id));
    normalized.extend(extras);
    normalized
}

fn normalize_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn normalize_endpoint_list(value: String) -> String {
    value
        .split(|c| matches!(c, '\n' | ',' | ';'))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.trim_end_matches('/'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn clamp_uap_port(port: u16) -> u16 {
    if port == 0 {
        default_uap_port()
    } else {
        port
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            close_to_tray: default_close_to_tray(),
            vault_path: default_vault_path(),
            notes_subdir: default_notes_subdir(),
            repo_root_path: default_repo_root_path(),
            workspace_presets: vec![],
            workspace_grant_roots: vec![],
            default_workspace_id: None,
            persona_presets: default_persona_presets(),
            provider_commands: default_provider_commands(),
            launch_targets: vec![],
            github_targets: vec![],
            pm_tool_url: default_pm_url(),
            pm_tool_dashboard_url: default_pm_dashboard_url(),
            pm_tool_poll_seconds: default_poll_seconds(),
            updater_endpoint: default_updater_endpoint(),
            updater_public_key: default_updater_public_key(),
            auto_check_for_updates: false,
            uap_advertise_host: default_uap_advertise_host(),
            uap_port: default_uap_port(),
            uap_token: default_uap_token(),
            cron_jobs: vec![],
            github_pat: None,
            task_lane_prefs: HashMap::new(),
            agent_notes: HashMap::new(),
            agent_auth_tokens: normalize_agent_auth_tokens(HashMap::new(), &[]),
            custom_agents: vec![],
            agent_provider_map: default_agent_provider_map(),
        }
    }
}

async fn persist_config(path: &std::path::Path, config: &AppConfig) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, json).await;
    }
}

pub async fn load_config(path: &std::path::Path) -> AppConfig {
    let mut config = match fs::read_to_string(path).await {
        Ok(s) => match serde_json::from_str::<AppConfig>(&s) {
            Ok(parsed) => {
                let normalized = parsed.clone().normalize();
                if normalized != parsed {
                    persist_config(path, &normalized).await;
                }
                normalized
            }
            Err(_) => {
                let config = AppConfig::default().normalize();
                persist_config(path, &config).await;
                config
            }
        },
        Err(_) => {
            let config = AppConfig::default().normalize();
            persist_config(path, &config).await;
            config
        }
    };

    // Migrate secrets from plaintext config to OS credential store
    if let Some(pat) = crate::credentials::migrate_secret("github-pat", config.github_pat.as_deref()) {
        if config.github_pat.as_deref() != Some(&pat) {
            config.github_pat = Some(pat);
        }
    }

    config
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> std::result::Result<AppConfig, AppError> {
    Ok(state.config.read().await.clone().normalize())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let normalized = config.normalize();

    // Store GitHub PAT in OS credential store, keep only a marker in config
    if let Some(pat) = &normalized.github_pat {
        if !pat.is_empty() {
            let _ = crate::credentials::store_secret("github-pat", pat);
        }
    }

    let json = serde_json::to_string_pretty(&normalized)?;
    if let Some(parent) = state.config_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&state.config_path, json).await?;
    *state.config.write().await = normalized;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_machine_neutral() {
        let cfg = AppConfig::default();
        assert!(cfg.vault_path.is_empty());
        assert!(cfg.close_to_tray);
        assert!(cfg.repo_root_path.is_empty());
        assert!(cfg.workspace_presets.is_empty());
        assert!(cfg.workspace_grant_roots.is_empty());
        assert!(cfg.default_workspace_id.is_none());
        assert_eq!(cfg.persona_presets, default_persona_presets());
        assert_eq!(cfg.provider_commands, default_provider_commands());
        assert!(cfg.pm_tool_url.is_empty());
        assert!(cfg.pm_tool_dashboard_url.is_empty());
        assert!(cfg.updater_endpoint.is_empty());
        assert!(cfg.updater_public_key.is_empty());
        assert!(!cfg.auto_check_for_updates);
        assert!(cfg.launch_targets.is_empty());
        assert!(cfg.github_targets.is_empty());
        assert!(!cfg.uap_token.is_empty());
        assert_eq!(cfg.agent_auth_tokens.len(), 2);
        assert!(cfg.agent_auth_tokens.contains_key("forge"));
        assert!(cfg.agent_auth_tokens.contains_key("prism"));
    }

    #[test]
    fn normalize_config_clamps_and_generates_safe_values() {
        let cfg = AppConfig {
            theme: "broken".into(),
            close_to_tray: false,
            vault_path: "  D:/vault  ".into(),
            notes_subdir: "   ".into(),
            repo_root_path: "  C:/Repos  ".into(),
            workspace_presets: vec![
                WorkspacePreset {
                    id: String::new(),
                    name: " Core ".into(),
                    root_path: " C:/Repos/UMBRA ".into(),
                    writable: true,
                    allowed_providers: vec![" codex ".into(), " ".into()],
                    allowed_agents: vec![" forge ".into(), String::new()],
                },
                WorkspacePreset {
                    id: "bad".into(),
                    name: " ".into(),
                    root_path: " ".into(),
                    writable: false,
                    allowed_providers: vec![],
                    allowed_agents: vec![],
                },
            ],
            workspace_grant_roots: vec![" C:/Trusted ".into(), " c:/trusted ".into()],
            default_workspace_id: Some(" Core ".into()),
            persona_presets: vec![
                PersonaPreset {
                    id: " ".into(),
                    name: " Builder ".into(),
                    description: " focused ".into(),
                    prompt: " do the work ".into(),
                },
                PersonaPreset {
                    id: "bad".into(),
                    name: " ".into(),
                    description: String::new(),
                    prompt: " ".into(),
                },
            ],
            provider_commands: vec![
                ProviderCommandConfig {
                    provider_id: " Codex ".into(),
                    command: " C:/tools/codex.exe ".into(),
                },
                ProviderCommandConfig {
                    provider_id: " ".into(),
                    command: " ".into(),
                },
            ],
            launch_targets: vec![
                LaunchTarget {
                    id: String::new(),
                    name: "  VS Code ".into(),
                    path: " code ".into(),
                    icon: Some("  ".into()),
                },
                LaunchTarget {
                    id: "bad".into(),
                    name: " ".into(),
                    path: " ".into(),
                    icon: None,
                },
            ],
            github_targets: vec![
                GithubTarget {
                    id: String::new(),
                    name: " UMBRA ".into(),
                    owner: " AstroGolem224 ".into(),
                    repo: " UMBRA ".into(),
                },
                GithubTarget {
                    id: "bad".into(),
                    name: String::new(),
                    owner: String::new(),
                    repo: String::new(),
                },
            ],
            pm_tool_url: " https://pm.local/api/ ".into(),
            pm_tool_dashboard_url: " https://pm.local/ ".into(),
            pm_tool_poll_seconds: 1,
            updater_endpoint:
                " https://updates.example.com/latest.json ; https://fallback.example.com/latest.json/ ".into(),
            updater_public_key: "  PUBLIC-KEY  ".into(),
            auto_check_for_updates: true,
            uap_advertise_host: "   ".into(),
            uap_port: 0,
            uap_token: "   ".into(),
            cron_jobs: vec![],
            github_pat: Some("  secret  ".into()),
            task_lane_prefs: HashMap::from([
                ("done".into(), true),
                ("weird".into(), false),
            ]),
            agent_notes: HashMap::from([
                (
                    " forge ".into(),
                    AgentNote {
                        notes: " hello ".into(),
                        link: " https://example.com ".into(),
                    },
                ),
                (
                    " ".into(),
                    AgentNote {
                        notes: "ignored".into(),
                        link: String::new(),
                    },
                ),
            ]),
            agent_auth_tokens: HashMap::from([
                ("forge".into(), "  forge-token  ".into()),
                ("ghost".into(), "gone".into()),
            ]),
            custom_agents: vec![
                CustomAgentConfig {
                    id: String::new(),
                    name: " Forge ".into(),
                    role: " Builder ".into(),
                    skills: vec![" rust ".into(), " ".into()],
                    allowed_tools: vec![" git ".into(), String::new()],
                },
                CustomAgentConfig {
                    id: "bad".into(),
                    name: " ".into(),
                    role: " ".into(),
                    skills: vec![],
                    allowed_tools: vec![],
                },
            ],
        }
        .normalize();

        assert_eq!(cfg.theme, "ember");
        assert!(!cfg.close_to_tray);
        assert_eq!(cfg.vault_path, "D:/vault");
        assert_eq!(cfg.notes_subdir, "UMBRA_Notes");
        assert_eq!(cfg.repo_root_path, "C:/Repos");
        assert_eq!(cfg.workspace_presets.len(), 1);
        assert_eq!(cfg.workspace_grant_roots.len(), 2);
        assert_eq!(cfg.workspace_grant_roots[0], "C:/Repos/UMBRA");
        assert_eq!(cfg.workspace_grant_roots[1], "C:/Trusted");
        assert_eq!(cfg.workspace_presets[0].name, "Core");
        assert_eq!(cfg.workspace_presets[0].root_path, "C:/Repos/UMBRA");
        assert_eq!(cfg.workspace_presets[0].allowed_providers, vec!["codex"]);
        assert_eq!(cfg.workspace_presets[0].allowed_agents, vec!["forge"]);
        assert!(cfg.default_workspace_id.is_none());
        assert_eq!(cfg.persona_presets.len(), 1);
        assert_eq!(cfg.persona_presets[0].name, "Builder");
        assert_eq!(cfg.persona_presets[0].description, "focused");
        assert_eq!(cfg.persona_presets[0].prompt, "do the work");
        assert_eq!(cfg.provider_commands.len(), 4);
        assert_eq!(cfg.provider_commands[0].provider_id, "codex");
        assert_eq!(cfg.provider_commands[0].command, "C:/tools/codex.exe");
        assert_eq!(cfg.provider_commands[1].command, "claude");
        assert_eq!(cfg.provider_commands[2].command, "gemini");
        assert_eq!(cfg.provider_commands[3].command, "kimi");
        assert_eq!(cfg.pm_tool_url, "https://pm.local/api");
        assert_eq!(cfg.pm_tool_dashboard_url, "https://pm.local");
        assert_eq!(cfg.pm_tool_poll_seconds, 5);
        assert_eq!(
            cfg.updater_endpoint,
            "https://updates.example.com/latest.json\nhttps://fallback.example.com/latest.json"
        );
        assert_eq!(cfg.updater_public_key, "PUBLIC-KEY");
        assert!(cfg.auto_check_for_updates);
        assert_eq!(cfg.uap_advertise_host, "127.0.0.1");
        assert_eq!(cfg.uap_port, 8765);
        assert!(!cfg.uap_token.is_empty());
        assert_eq!(cfg.github_pat.as_deref(), Some("secret"));
        assert_eq!(cfg.launch_targets.len(), 1);
        assert_eq!(cfg.launch_targets[0].name, "VS Code");
        assert_eq!(cfg.launch_targets[0].path, "code");
        assert_eq!(cfg.launch_targets[0].icon, None);
        assert!(!cfg.launch_targets[0].id.is_empty());
        assert_eq!(cfg.github_targets.len(), 1);
        assert_eq!(cfg.github_targets[0].owner, "AstroGolem224");
        assert_eq!(cfg.github_targets[0].repo, "UMBRA");
        assert!(cfg.task_lane_prefs.contains_key("done"));
        assert!(!cfg.task_lane_prefs.contains_key("weird"));
        assert_eq!(cfg.agent_notes.len(), 1);
        assert_eq!(cfg.agent_notes["forge"].notes, "hello");
        assert_eq!(cfg.agent_notes["forge"].link, "https://example.com");
        assert_eq!(cfg.custom_agents.len(), 1);
        assert_eq!(cfg.custom_agents[0].name, "Forge");
        assert_eq!(cfg.custom_agents[0].role, "Builder");
        assert_eq!(cfg.custom_agents[0].skills, vec!["rust"]);
        assert_eq!(cfg.custom_agents[0].allowed_tools, vec!["git"]);
        assert_eq!(cfg.agent_auth_tokens.len(), 3);
        assert_eq!(cfg.agent_auth_tokens["forge"], "forge-token");
        assert!(cfg.agent_auth_tokens.contains_key("prism"));
        assert!(cfg.agent_auth_tokens.contains_key(&cfg.custom_agents[0].id));
        assert!(!cfg.agent_auth_tokens.contains_key("ghost"));
    }
}
