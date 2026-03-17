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

#[cfg(test)]
mod tests {
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
}
