use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::State;
use tokio::fs;
use uuid::Uuid;

use crate::errors::{AppError, Result};
use crate::state::AppState;

const VALID_CATEGORIES: &[&str] = &["prompts", "cli", "agents", "skills", "misc"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: String,
}

fn notes_base_dir(vault: &str, subdir: &str) -> PathBuf {
    PathBuf::from(vault).join(subdir)
}

/// Resolve a safe path within the vault. Rejects any path traversal attempts.
fn resolve_note_path(vault: &str, subdir: &str, category: &str, filename: &str) -> Result<PathBuf> {
    if !VALID_CATEGORIES.contains(&category) {
        return Err(AppError::PathTraversal(format!("Invalid category: {category}")));
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::PathTraversal(format!("Invalid filename: {filename}")));
    }
    Ok(notes_base_dir(vault, subdir).join(category).join(filename))
}

fn parse_frontmatter(content: &str) -> (HashMap<String, String>, Vec<String>, String) {
    let mut meta = HashMap::new();
    let mut tags = Vec::new();
    let mut lines = content.lines();

    if lines.next().map(str::trim) != Some("---") {
        return (meta, tags, content.to_string());
    }

    let mut current_list: Option<String> = None;
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter {
            if line.trim() == "---" {
                in_frontmatter = false;
                continue;
            }

            let trimmed = line.trim();
            if let Some(list_key) = &current_list {
                if let Some(item) = trimmed.strip_prefix("- ") {
                    if list_key == "tags" {
                        tags.push(item.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                    continue;
                }
                current_list = None;
            }

            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim();
                if key == "tags" {
                    if value.is_empty() {
                        current_list = Some("tags".into());
                    } else if value == "[]" {
                        // no tags
                    } else {
                        tags.extend(
                            value
                                .trim_matches('[')
                                .trim_matches(']')
                                .split(',')
                                .map(|item| item.trim().trim_matches('"').trim_matches('\''))
                                .filter(|item| !item.is_empty())
                                .map(ToString::to_string),
                        );
                    }
                } else {
                    meta.insert(key, value.trim_matches('"').trim_matches('\'').to_string());
                }
            }
        } else {
            body_lines.push(line);
        }
    }

    let body = body_lines.join("\n").trim_start_matches('\n').to_string();
    (meta, tags, body)
}

fn split_title_from_body(body: &str, fallback_title: &str) -> (String, String) {
    let mut lines = body.lines();
    let first_non_empty = lines.find(|line| !line.trim().is_empty());

    if let Some(title_line) = first_non_empty {
        if let Some(title) = title_line.trim().strip_prefix("# ") {
            let remaining = body
                .lines()
                .skip_while(|line| line.trim().is_empty())
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim_start_matches('\n')
                .to_string();
            return (title.trim().to_string(), remaining);
        }
    }

    (fallback_title.to_string(), body.to_string())
}

fn yaml_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn build_disk_content(note: &Note, created_at: &str, updated_at: &str) -> String {
    let (_, cleaned_body) = split_title_from_body(&note.content, &note.title);
    let body = cleaned_body.trim_start_matches('\n');

    let mut frontmatter = vec![
        "---".to_string(),
        format!("title: '{}'", yaml_escape(note.title.trim())),
        format!("category: '{}'", yaml_escape(&note.category)),
        format!("created_at: '{}'", yaml_escape(created_at)),
        format!("updated_at: '{}'", yaml_escape(updated_at)),
    ];

    if note.tags.is_empty() {
        frontmatter.push("tags: []".into());
    } else {
        frontmatter.push("tags:".into());
        for tag in &note.tags {
            frontmatter.push(format!("  - '{}'", yaml_escape(tag)));
        }
    }

    frontmatter.push("---".into());

    if body.trim().is_empty() {
        frontmatter.join("\n")
    } else {
        format!("{}\n\n{}", frontmatter.join("\n"), body)
    }
}

fn safe_existing_note_path(base: &Path, note: &Note) -> Option<PathBuf> {
    if note.file_path.trim().is_empty() {
        return None;
    }

    let path = PathBuf::from(&note.file_path);
    let stem = path.file_stem()?.to_string_lossy();
    if stem != note.id {
        return None;
    }
    if !path.starts_with(base) {
        return None;
    }

    Some(path)
}

#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> std::result::Result<Vec<Note>, AppError> {
    let cfg = state.config.read().await;
    let base = notes_base_dir(&cfg.vault_path, &cfg.notes_subdir);
    drop(cfg);

    let mut notes = Vec::new();

    for category in VALID_CATEGORIES {
        let cat_dir = base.join(category);
        let Ok(mut entries) = fs::read_dir(&cat_dir).await else { continue };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let Ok(raw_content) = fs::read_to_string(&path).await else { continue };
            let filename = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let meta = fs::metadata(&path).await.ok();
            let fallback_updated_at = meta
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
                        .unwrap_or_default()
                        .to_rfc3339()
                })
                .unwrap_or_else(|| Utc::now().to_rfc3339());

            let (frontmatter, tags, body) = parse_frontmatter(&raw_content);
            let fallback_title = frontmatter
                .get("title")
                .cloned()
                .unwrap_or_else(|| filename.clone());
            let (body_title, cleaned_body) = split_title_from_body(&body, &fallback_title);

            let category = frontmatter
                .get("category")
                .cloned()
                .filter(|value| VALID_CATEGORIES.contains(&value.as_str()))
                .unwrap_or_else(|| category.to_string());
            let title = frontmatter
                .get("title")
                .cloned()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(body_title);
            let created_at = frontmatter
                .get("created_at")
                .cloned()
                .unwrap_or_else(|| fallback_updated_at.clone());
            let updated_at = frontmatter
                .get("updated_at")
                .cloned()
                .unwrap_or_else(|| fallback_updated_at.clone());

            notes.push(Note {
                id: filename,
                title,
                content: cleaned_body,
                category,
                tags,
                created_at,
                updated_at,
                file_path: path.to_string_lossy().to_string(),
            });
        }
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

#[tauri::command]
pub async fn save_note(note: Note, state: State<'_, AppState>) -> std::result::Result<Note, AppError> {
    let cfg = state.config.read().await;
    let vault = cfg.vault_path.clone();
    let subdir = cfg.notes_subdir.clone();
    drop(cfg);

    let note_id = if note.id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        note.id.clone()
    };
    let filename = format!("{note_id}.md");
    let path = resolve_note_path(&vault, &subdir, &note.category, &filename)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let base = notes_base_dir(&vault, &subdir);
    if let Some(existing_path) = safe_existing_note_path(&base, &note) {
        if existing_path != path && existing_path.exists() {
            fs::remove_file(existing_path).await?;
        }
    }

    let created_at = if note.created_at.is_empty() {
        Utc::now().to_rfc3339()
    } else {
        note.created_at.clone()
    };
    let updated_at = Utc::now().to_rfc3339();
    let disk_content = build_disk_content(&note, &created_at, &updated_at);
    fs::write(&path, &disk_content).await?;

    let (_, tags, body) = parse_frontmatter(&disk_content);

    Ok(Note {
        id: note_id,
        title: note.title.trim().to_string(),
        content: body,
        category: note.category,
        tags,
        created_at,
        updated_at,
        file_path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn delete_note(id: String, category: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let cfg = state.config.read().await;
    let vault = cfg.vault_path.clone();
    let subdir = cfg.notes_subdir.clone();
    drop(cfg);

    let filename = format!("{id}.md");
    let path = resolve_note_path(&vault, &subdir, &category, &filename)?;

    if path.exists() {
        fs::remove_file(&path).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_category_resolves_path() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "prompts", "my-note.md");
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_category_rejected() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "../../etc", "passwd.md");
        assert!(result.is_err());
    }

    #[test]
    fn path_traversal_in_filename_rejected() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "prompts", "../evil.md");
        assert!(result.is_err());
    }

    #[test]
    fn absolute_path_in_filename_rejected() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "prompts", r"C:\evil.md");
        assert!(result.is_err());
    }

    #[test]
    fn parses_frontmatter_and_tags() {
        let raw = "---\ntitle: 'hello'\ncategory: 'skills'\ntags:\n  - 'rust'\n  - 'tauri'\n---\n\nbody";
        let (meta, tags, body) = parse_frontmatter(raw);
        assert_eq!(meta.get("title"), Some(&"hello".to_string()));
        assert_eq!(meta.get("category"), Some(&"skills".to_string()));
        assert_eq!(tags, vec!["rust".to_string(), "tauri".to_string()]);
        assert_eq!(body, "body");
    }

    #[test]
    fn strips_leading_h1_from_body() {
        let (title, body) = split_title_from_body("# Heading\n\ntext", "fallback");
        assert_eq!(title, "Heading");
        assert_eq!(body, "text");
    }
}
