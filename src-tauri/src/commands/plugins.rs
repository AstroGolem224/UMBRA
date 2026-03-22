use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::SystemTime;
use serde::Serialize;
use tauri::State;
use chrono::{DateTime, Utc};

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsidianStats {
    pub connected: bool,
    pub total_notes: usize,
    pub recent_notes: Vec<RecentNote>,
    pub vault_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentNote {
    pub name: String,
    pub modified: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TmTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub project: String,
    pub priority: String,
}

fn scan_vault_md(vault: &Path, max: usize) -> (usize, Vec<(String, SystemTime)>) {
    let mut count = 0usize;
    let mut recent: Vec<(String, SystemTime)> = Vec::new();
    let mut queue: VecDeque<std::path::PathBuf> = VecDeque::new();
    queue.push_back(vault.to_path_buf());

    'outer: while let Some(dir) = queue.pop_front() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }
                queue.push_back(path);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                count += 1;
                if let Ok(meta) = entry.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        let name = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        recent.push((name, mtime));
                    }
                }
                if count >= max {
                    break 'outer;
                }
            }
        }
    }

    recent.sort_by(|a, b| b.1.cmp(&a.1));
    recent.truncate(5);
    (count, recent)
}

fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut lines = content.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return map;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            map.insert(key, val);
        }
    }
    map
}

fn system_time_to_rfc3339(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.to_rfc3339()
}

#[tauri::command]
pub async fn get_obsidian_stats(
    state: State<'_, AppState>,
) -> std::result::Result<ObsidianStats, AppError> {
    let vault_path = state.config.read().await.vault_path.clone();
    let path = Path::new(&vault_path);

    if !path.exists() {
        return Ok(ObsidianStats {
            connected: false,
            total_notes: 0,
            recent_notes: vec![],
            vault_path,
        });
    }

    let (total, recent_raw) = scan_vault_md(path, 2000);

    let recent_notes = recent_raw
        .into_iter()
        .map(|(name, mtime)| RecentNote {
            name,
            modified: system_time_to_rfc3339(mtime),
        })
        .collect();

    Ok(ObsidianStats {
        connected: true,
        total_notes: total,
        recent_notes,
        vault_path,
    })
}

#[tauri::command]
pub async fn get_tmlite_tasks(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<TmTask>, AppError> {
    let vault_path = state.config.read().await.vault_path.clone();
    let tasks_dir = Path::new(&vault_path).join("Tasks");

    if !tasks_dir.exists() {
        return Ok(vec![]);
    }

    let mut tasks: Vec<TmTask> = Vec::new();

    let entries = std::fs::read_dir(&tasks_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let fm = parse_frontmatter(&content);

            let status = fm.get("status").cloned().unwrap_or_default();
            if status == "done" || status == "cancelled" {
                continue;
            }

            let id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let title = fm
                .get("title")
                .cloned()
                .unwrap_or_else(|| id.clone());

            tasks.push(TmTask {
                id,
                title,
                status,
                project: fm.get("project").cloned().unwrap_or_default(),
                priority: fm.get("priority").cloned().unwrap_or_else(|| "medium".into()),
            });
        }
    }

    tasks.sort_by(|a, b| {
        let priority_rank = |p: &str| match p {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        };
        priority_rank(&a.priority).cmp(&priority_rank(&b.priority))
    });

    Ok(tasks)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub agents: Vec<String>,
    pub content: String,
    pub folder: String,
}

fn parse_skill_md(content: &str) -> (String, String) {
    // Parse SKILL.md YAML frontmatter between --- delimiters
    // Returns (name, description)
    let mut name = String::new();
    let mut desc = String::new();
    let mut in_fm = false;
    let mut in_desc = false;

    for (i, line) in content.lines().enumerate() {
        if i == 0 && line.trim() == "---" {
            in_fm = true;
            continue;
        }
        if in_fm && line.trim() == "---" {
            break;
        }
        if !in_fm {
            continue;
        }
        if line.starts_with("name:") {
            name = line["name:".len()..].trim().trim_matches('"').to_string();
            in_desc = false;
        } else if line.starts_with("description:") {
            let inline = line["description:".len()..].trim();
            if inline == "|" || inline.is_empty() {
                in_desc = true;
            } else {
                desc = inline.trim_matches('"').to_string();
                in_desc = false;
            }
        } else if in_desc {
            if line.starts_with(' ') {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if !desc.is_empty() { desc.push(' '); }
                    desc.push_str(trimmed);
                }
            } else {
                in_desc = false;
            }
        }
    }
    (name, desc)
}

fn infer_skill_category(skills_dir: &std::path::Path, skill_dir: &std::path::Path) -> String {
    let Ok(relative) = skill_dir.strip_prefix(skills_dir) else {
        return "core".into();
    };
    let mut components = relative.components();
    let Some(first) = components.next() else {
        return "core".into();
    };
    let first = first.as_os_str().to_string_lossy().into_owned();
    if components.next().is_none() {
        "core".into()
    } else {
        first.trim_start_matches('.').to_string()
    }
}

fn infer_skill_agents(name: &str, description: &str, content: &str) -> Vec<String> {
    let haystack = format!("{name}\n{description}\n{content}").to_lowercase();
    let mut agents = Vec::new();

    let forge_terms = ["web", "frontend", "vue", "react", "openai", "full-stack", "api"];
    if forge_terms.iter().any(|term| haystack.contains(term)) {
        agents.push("forge".to_string());
    }

    let prism_terms = ["godot", "game", "gdscript", "scene", "animation", "physics"];
    if prism_terms.iter().any(|term| haystack.contains(term)) {
        agents.push("prism".to_string());
    }

    let jim_terms = ["shell", "powershell", "ops", "ci", "ship", "debug", "monitor", "cron"];
    if jim_terms.iter().any(|term| haystack.contains(term)) {
        agents.push("jim".to_string());
    }

    if agents.is_empty() {
        agents.push("forge".to_string());
    }

    agents
}

#[tauri::command]
pub async fn list_skills() -> std::result::Result<Vec<SkillInfo>, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    let skills_dir = std::path::Path::new(&home).join(".claude").join("skills");

    if !skills_dir.exists() {
        return Ok(vec![]);
    }

    let mut skills: Vec<SkillInfo> = Vec::new();

    let entries = std::fs::read_dir(&skills_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = path.file_name().unwrap_or_default().to_string_lossy().into_owned();

        // Read VERSION
        let version = std::fs::read_to_string(path.join("VERSION"))
            .unwrap_or_default()
            .trim()
            .to_string();

        let content = std::fs::read_to_string(path.join("SKILL.md")).unwrap_or_default();
        let (mut name, description) = parse_skill_md(&content);

        if name.is_empty() {
            name = id.clone();
        }

        let category = infer_skill_category(&skills_dir, &path);
        let agents = infer_skill_agents(&name, &description, &content);
        let folder = path
            .strip_prefix(&skills_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        skills.push(SkillInfo {
            id,
            name,
            version,
            description,
            category,
            agents,
            content,
            folder,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}
