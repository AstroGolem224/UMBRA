use std::path::{Path, PathBuf};

use tauri::State;

use crate::errors::{AppError, Result};
use crate::state::AppState;

/// Launch an IDE/app by target ID. Only targets defined in config are allowed.
#[tauri::command]
pub async fn launch_target(target_id: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let cfg = state.config.read().await;
    let target = cfg
        .launch_targets
        .iter()
        .find(|t| t.id == target_id)
        .ok_or_else(|| AppError::TargetNotFound(target_id.clone()))?
        .clone();
    drop(cfg);

    // Verify the path is in the whitelist (it came from config, so it is by definition)
    // Additional check: no shell metacharacters in the resolved path
    let path = &target.path;
    if path.contains(';') || path.contains('&') || path.contains('|') || path.contains('`') {
        return Err(AppError::TargetNotAllowed(format!(
            "Path contains shell metacharacters: {path}"
        )));
    }

    tokio::process::Command::new(path)
        .spawn()
        .map_err(AppError::Io)?;

    Ok(())
}

/// Open any GitHub URL in the browser (no whitelist — URL must start with https://github.com/).
#[tauri::command]
pub async fn open_github_url(url: String) -> std::result::Result<(), AppError> {
    if !url.starts_with("https://github.com/") {
        return Err(AppError::TargetNotAllowed(format!("Not a GitHub URL: {url}")));
    }
    open_url(&url)?;
    Ok(())
}

/// Open a GitHub repository in the system browser.
#[tauri::command]
pub async fn open_github(owner: String, repo: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let cfg = state.config.read().await;

    // Verify the repo is in the configured github_targets whitelist
    let allowed = cfg.github_targets.iter().any(|t| t.owner == owner && t.repo == repo);
    drop(cfg);

    if !allowed {
        return Err(AppError::TargetNotAllowed(format!("{owner}/{repo}")));
    }

    // owner/repo are validated against whitelist — safe to construct URL
    let url = format!("https://github.com/{owner}/{repo}");
    open_url(&url)?;
    Ok(())
}

#[tauri::command]
pub async fn open_local_repo_folder(repo_name: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let repo_path = resolve_local_repo_path(&repo_name, &state).await?;
    open_folder(&repo_path)?;
    Ok(())
}

#[tauri::command]
pub async fn open_local_repo_terminal(repo_name: String, state: State<'_, AppState>) -> std::result::Result<(), AppError> {
    let repo_path = resolve_local_repo_path(&repo_name, &state).await?;
    open_terminal(&repo_path)?;
    Ok(())
}

async fn resolve_local_repo_path(repo_name: &str, state: &State<'_, AppState>) -> Result<PathBuf> {
    validate_repo_name(repo_name)?;

    let cfg = state.config.read().await;
    let root = PathBuf::from(cfg.repo_root_path.clone());
    drop(cfg);

    if !root.exists() {
        return Err(AppError::TargetNotFound(root.display().to_string()));
    }

    let root_canonical = std::fs::canonicalize(&root)?;
    let repo_path = root.join(repo_name);
    if !repo_path.exists() {
        return Err(AppError::TargetNotFound(repo_name.into()));
    }

    let repo_canonical = std::fs::canonicalize(&repo_path)?;
    if !repo_canonical.starts_with(&root_canonical) {
        return Err(AppError::PathTraversal(repo_name.into()));
    }

    Ok(repo_canonical)
}

fn validate_repo_name(repo_name: &str) -> Result<()> {
    if repo_name.trim().is_empty() {
        return Err(AppError::TargetNotAllowed("empty repo name".into()));
    }
    if repo_name.contains("..")
        || repo_name.contains('/')
        || repo_name.contains('\\')
        || repo_name.contains(':')
    {
        return Err(AppError::PathTraversal(repo_name.into()));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_url(url: &str) -> Result<()> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn open_url(url: &str) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_folder(path: &Path) -> Result<()> {
    std::process::Command::new("explorer")
        .arg(path)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn open_folder(path: &Path) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_terminal(path: &Path) -> Result<()> {
    let escaped = path.display().to_string().replace('\'', "''");
    std::process::Command::new("powershell")
        .args(["-NoExit", "-Command", &format!("Set-Location -LiteralPath '{escaped}'")])
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn open_terminal(path: &Path) -> Result<()> {
    std::process::Command::new("x-terminal-emulator")
        .arg("--working-directory")
        .arg(path)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_repo_name;

    // Whitelist enforcement is tested via integration — the whitelist check lives in
    // the command body and depends on AppState/config. Unit-testable piece:

    #[test]
    fn metachar_detection() {
        let paths = vec![
            ("code", false),
            (r"C:\Program Files\VSCode\code.exe", false),
            ("code; rm -rf /", true),
            ("code && evil", true),
            ("code | evil", true),
        ];
        for (path, should_reject) in paths {
            let has_meta = path.contains(';') || path.contains('&') || path.contains('|') || path.contains('`');
            assert_eq!(has_meta, should_reject, "path: {path}");
        }
    }

    #[test]
    fn rejects_repo_name_traversal() {
        assert!(validate_repo_name("../UMBRA").is_err());
        assert!(validate_repo_name(r"..\UMBRA").is_err());
        assert!(validate_repo_name("foo/bar").is_err());
        assert!(validate_repo_name("C:/tmp").is_err());
    }

    #[test]
    fn accepts_plain_repo_name() {
        assert!(validate_repo_name("UMBRA").is_ok());
        assert!(validate_repo_name("my-repo_2").is_ok());
    }
}
