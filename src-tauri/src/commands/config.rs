use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::fs;

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTarget {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubTarget {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn bool_true() -> bool { true }
fn default_last_status() -> String { "pending".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_vault_path")]
    pub vault_path: String,
    #[serde(default = "default_notes_subdir")]
    pub notes_subdir: String,
    #[serde(default)]
    pub launch_targets: Vec<LaunchTarget>,
    #[serde(default)]
    pub github_targets: Vec<GithubTarget>,
    #[serde(default = "default_pm_url")]
    pub pm_tool_url: String,
    #[serde(default = "default_poll_seconds")]
    pub pm_tool_poll_seconds: u64,
    #[serde(default)]
    pub cron_jobs: Vec<CronJobConfig>,
}

fn default_theme() -> String { "ember".into() }
fn default_vault_path() -> String {
    r"D:\Obsidian\2nd-brain\2nd-brain".into()
}
fn default_notes_subdir() -> String { "UMBRA_Notes".into() }
fn default_pm_url() -> String { "http://localhost:4173".into() }
fn default_poll_seconds() -> u64 { 30 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            vault_path: default_vault_path(),
            notes_subdir: default_notes_subdir(),
            launch_targets: vec![
                LaunchTarget {
                    id: "vscode".into(),
                    name: "VS Code".into(),
                    path: "code".into(),
                    icon: Some("⬡".into()),
                },
                LaunchTarget {
                    id: "godot".into(),
                    name: "Godot 4".into(),
                    path: r"C:\Godot\Godot_v4.3-stable_win64.exe".into(),
                    icon: Some("◈".into()),
                },
            ],
            github_targets: vec![
                GithubTarget {
                    id: "umbra".into(),
                    name: "UMBRA".into(),
                    owner: "AstroGolem224".into(),
                    repo: "UMBRA".into(),
                },
                GithubTarget {
                    id: "mmc".into(),
                    name: "Meat Machine Cycle".into(),
                    owner: "AstroGolem224".into(),
                    repo: "MMC".into(),
                },
            ],
            pm_tool_url: default_pm_url(),
            pm_tool_poll_seconds: default_poll_seconds(),
            cron_jobs: vec![],
        }
    }
}

pub async fn load_config(path: &std::path::Path) -> AppConfig {
    match fs::read_to_string(path).await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> std::result::Result<AppConfig, AppError> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
    let json = serde_json::to_string_pretty(&config)?;
    if let Some(parent) = state.config_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&state.config_path, json).await?;
    *state.config.write().await = config;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_two_launch_targets() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.launch_targets.len(), 2);
    }

    #[test]
    fn config_roundtrip() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.theme, decoded.theme);
    }
}
