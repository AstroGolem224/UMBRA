use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

/// Resolve a safe path within the vault. Rejects any path traversal attempts.
fn resolve_note_path(vault: &str, subdir: &str, category: &str, filename: &str) -> Result<PathBuf> {
    if !VALID_CATEGORIES.contains(&category) {
        return Err(AppError::PathTraversal(format!("Invalid category: {category}")));
    }
    // Reject path traversal in filename
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::PathTraversal(format!("Invalid filename: {filename}")));
    }
    let base = PathBuf::from(vault).join(subdir).join(category);
    Ok(base.join(filename))
}

#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> std::result::Result<Vec<Note>, AppError> {
    let cfg = state.config.read().await;
    let base = PathBuf::from(&cfg.vault_path).join(&cfg.notes_subdir);
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
            let Ok(content) = fs::read_to_string(&path).await else { continue };
            let filename = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let meta = fs::metadata(&path).await.ok();
            let updated_at = meta
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
                        .unwrap_or_default()
                        .to_rfc3339()
                })
                .unwrap_or_else(|| Utc::now().to_rfc3339());

            // Extract title from first H1, fall back to filename
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l[2..].trim().to_string())
                .unwrap_or_else(|| filename.clone());

            notes.push(Note {
                id: filename.clone(),
                title,
                content,
                category: category.to_string(),
                tags: vec![],
                created_at: updated_at.clone(),
                updated_at,
                file_path: path.to_string_lossy().to_string(),
            });
        }
    }

    Ok(notes)
}

#[tauri::command]
pub async fn save_note(note: Note, state: State<'_, AppState>) -> std::result::Result<Note, AppError> {
    let cfg = state.config.read().await;
    let vault = cfg.vault_path.clone();
    let subdir = cfg.notes_subdir.clone();
    drop(cfg);

    let filename = if note.id.is_empty() {
        format!("{}.md", Uuid::new_v4())
    } else {
        format!("{}.md", note.id)
    };

    let path = resolve_note_path(&vault, &subdir, &note.category, &filename)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(&path, &note.content).await?;

    let now = Utc::now().to_rfc3339();
    let saved = Note {
        id: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
        updated_at: now.clone(),
        file_path: path.to_string_lossy().to_string(),
        created_at: if note.created_at.is_empty() { now } else { note.created_at },
        ..note
    };

    Ok(saved)
}

#[tauri::command]
pub async fn delete_note(id: String, category: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let cfg = state.config.read().await;
    let vault = cfg.vault_path.clone();
    let subdir = cfg.notes_subdir.clone();
    drop(cfg);

    let filename = format!("{}.md", id);
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
}
