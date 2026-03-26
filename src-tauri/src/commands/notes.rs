use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::State;
use tokio::fs;
use uuid::Uuid;

use crate::errors::{AppError, Result};
use crate::state::AppState;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedNoteAttachment {
    pub file_name: String,
    pub absolute_path: String,
    pub relative_path: String,
    pub markdown: String,
    pub is_image: bool,
}

fn notes_base_dir(vault: &str, subdir: &str) -> PathBuf {
    PathBuf::from(vault).join(subdir)
}

fn note_attachments_dir(vault: &str, subdir: &str, note_id: &str) -> PathBuf {
    notes_base_dir(vault, subdir)
        .join("_attachments")
        .join(note_id)
}

fn sanitize_category(category: &str) -> Result<String> {
    let trimmed = category.trim();
    if trimmed.is_empty() {
        return Err(AppError::PathTraversal("Category cannot be empty".into()));
    }

    if matches!(trimmed, "." | "..") {
        return Err(AppError::PathTraversal(format!(
            "Invalid category: {category}"
        )));
    }

    if trimmed.ends_with(' ') || trimmed.ends_with('.') {
        return Err(AppError::PathTraversal(format!(
            "Invalid category: {category}"
        )));
    }

    if trimmed.chars().any(|ch| {
        ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    }) {
        return Err(AppError::PathTraversal(format!(
            "Invalid category: {category}"
        )));
    }

    let uppercase = trimmed.to_ascii_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&uppercase.as_str()) {
        return Err(AppError::PathTraversal(format!(
            "Invalid category: {category}"
        )));
    }

    Ok(trimmed.to_string())
}

fn sanitize_note_id(note_id: &str) -> Result<String> {
    let trimmed = note_id.trim();
    if trimmed.is_empty() {
        return Err(AppError::PathTraversal("Note id cannot be empty".into()));
    }

    if trimmed.contains("..")
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        return Err(AppError::PathTraversal(format!(
            "Invalid note id: {note_id}"
        )));
    }

    Ok(trimmed.to_string())
}

fn extension_from_mime(mime_type: Option<&str>) -> Option<&'static str> {
    match mime_type {
        Some("image/png") => Some("png"),
        Some("image/jpeg") => Some("jpg"),
        Some("image/webp") => Some("webp"),
        Some("image/gif") => Some("gif"),
        Some("image/svg+xml") => Some("svg"),
        Some("application/pdf") => Some("pdf"),
        Some("text/plain") => Some("txt"),
        _ => None,
    }
}

fn sanitize_attachment_name(file_name: &str, mime_type: Option<&str>) -> String {
    let raw_name = Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .trim();

    let (raw_stem, raw_ext) = raw_name
        .rsplit_once('.')
        .map(|(stem, ext)| (stem, Some(ext)))
        .unwrap_or((raw_name, None));

    let stem = raw_stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let ext = raw_ext
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|value| !value.is_empty())
        .or_else(|| extension_from_mime(mime_type).map(ToString::to_string))
        .unwrap_or_else(|| "bin".to_string());

    let safe_stem = if stem.is_empty() {
        format!("attachment-{}", Utc::now().format("%Y%m%d-%H%M%S"))
    } else {
        stem
    };

    format!("{safe_stem}.{ext}")
}

async fn ensure_unique_attachment_name(dir: &Path, file_name: &str) -> Result<String> {
    if !fs::try_exists(dir.join(file_name)).await? {
        return Ok(file_name.to_string());
    }

    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment");
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin");

    for suffix in 2..10_000 {
        let candidate = format!("{stem}-{suffix}.{ext}");
        if !fs::try_exists(dir.join(&candidate)).await? {
            return Ok(candidate);
        }
    }

    Err(AppError::Other(
        "Could not allocate a unique attachment name".into(),
    ))
}

fn is_image_attachment(file_name: &str, mime_type: Option<&str>) -> bool {
    mime_type.is_some_and(|value| value.starts_with("image/"))
        || matches!(
            Path::new(file_name)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
                .as_deref(),
            Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg")
        )
}

/// Resolve a safe path within the vault. Rejects any path traversal attempts.
fn resolve_note_path(vault: &str, subdir: &str, category: &str, filename: &str) -> Result<PathBuf> {
    let safe_category = sanitize_category(category)?;
    if filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
        || filename
            .chars()
            .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        return Err(AppError::PathTraversal(format!(
            "Invalid filename: {filename}"
        )));
    }
    Ok(notes_base_dir(vault, subdir)
        .join(safe_category)
        .join(filename))
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
    let Ok(mut categories) = fs::read_dir(&base).await else {
        return Ok(notes);
    };

    while let Ok(Some(category_entry)) = categories.next_entry().await {
        let Ok(file_type) = category_entry.file_type().await else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let dir_name = category_entry.file_name().to_string_lossy().to_string();
        let Ok(category) = sanitize_category(&dir_name) else {
            continue;
        };

        let Ok(mut entries) = fs::read_dir(category_entry.path()).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let Ok(raw_content) = fs::read_to_string(&path).await else {
                continue;
            };
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
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
                category: category.clone(),
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
pub async fn save_note(
    note: Note,
    state: State<'_, AppState>,
) -> std::result::Result<Note, AppError> {
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
pub async fn delete_note(
    id: String,
    category: String,
    state: State<'_, AppState>,
) -> std::result::Result<(), AppError> {
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

#[tauri::command]
pub async fn save_note_attachment(
    note_id: String,
    category: String,
    file_name: String,
    bytes: Vec<u8>,
    mime_type: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<SavedNoteAttachment, AppError> {
    if bytes.is_empty() {
        return Err(AppError::Other("Attachment payload was empty".into()));
    }

    let note_id = sanitize_note_id(&note_id)?;
    let _category = sanitize_category(&category)?;

    let cfg = state.config.read().await;
    let vault = cfg.vault_path.clone();
    let subdir = cfg.notes_subdir.clone();
    drop(cfg);

    let dir = note_attachments_dir(&vault, &subdir, &note_id);
    fs::create_dir_all(&dir).await?;

    let sanitized_name = sanitize_attachment_name(&file_name, mime_type.as_deref());
    let unique_name = ensure_unique_attachment_name(&dir, &sanitized_name).await?;
    let absolute_path = dir.join(&unique_name);
    fs::write(&absolute_path, bytes).await?;

    let relative_path = format!("../_attachments/{note_id}/{unique_name}");
    let is_image = is_image_attachment(&unique_name, mime_type.as_deref());
    let label = Path::new(&unique_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment");
    let markdown = if is_image {
        format!("![{label}]({relative_path})")
    } else {
        format!("[{unique_name}]({relative_path})")
    };

    Ok(SavedNoteAttachment {
        file_name: unique_name,
        absolute_path: absolute_path.to_string_lossy().to_string(),
        relative_path,
        markdown,
        is_image,
    })
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
    fn custom_category_resolves_path() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "boss fights", "my-note.md");
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_category_rejected() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "boss/fights", "passwd.md");
        assert!(result.is_err());
    }

    #[test]
    fn reserved_category_rejected() {
        let result = resolve_note_path(r"D:\vault", "UMBRA_Notes", "CON", "passwd.md");
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
        let raw =
            "---\ntitle: 'hello'\ncategory: 'skills'\ntags:\n  - 'rust'\n  - 'tauri'\n---\n\nbody";
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

    #[test]
    fn sanitizes_attachment_names() {
        let result = sanitize_attachment_name(
            r"C:\Users\matth\Desktop\Quarterly Report.png",
            Some("image/png"),
        );
        assert_eq!(result, "quarterly-report.png");
    }

    #[test]
    fn infers_attachment_extension_from_mime() {
        let result = sanitize_attachment_name("", Some("image/png"));
        assert!(result.ends_with(".png"));
    }
}
