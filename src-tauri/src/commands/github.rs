use std::time::{Duration, Instant};
use tauri::State;

use crate::errors::AppError;
use crate::state::AppState;

const CACHE_TTL: Duration = Duration::from_secs(300);
const GITHUB_API: &str = "https://api.github.com";

fn user_repos_url(pat: Option<&str>) -> Result<String, AppError> {
    match pat.map(str::trim).filter(|token| !token.is_empty()) {
        Some(_) => Ok(format!(
            "{}/user/repos?per_page=100&sort=updated&affiliation=owner",
            GITHUB_API
        )),
        None => {
            Err(AppError::Other("GitHub PAT required to list account repositories. Add one in Settings.".into()))
        }
    }
}

/// Fetch all repos owned by the authenticated user.
#[tauri::command]
pub async fn list_user_repos(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, AppError> {
    let pat = {
        let cfg = state.config.read().await;
        cfg.github_pat.clone()
    };

    let client = reqwest::Client::builder()
        .user_agent("UMBRA/1.0")
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = user_repos_url(pat.as_deref())?;

    let mut req = client.get(&url);
    if let Some(token) = &pat {
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }
    }

    let resp = req.send().await?;
    let repos: Vec<serde_json::Value> = resp.json().await?;

    // Return minimal shape: name, full_name, html_url, private, description, pushed_at
    let slim: Vec<serde_json::Value> = repos
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r["name"],
                "fullName": r["full_name"],
                "htmlUrl": r["html_url"],
                "private": r["private"],
                "description": r["description"],
                "pushedAt": r["pushed_at"],
            })
        })
        .collect();

    Ok(slim)
}

#[tauri::command]
pub async fn get_github_repos(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    // Check cache
    {
        let cache = state.github_cache.lock().await;
        if let Some((data, ts)) = &*cache {
            if ts.elapsed() < CACHE_TTL {
                return Ok(data.clone());
            }
        }
    }

    let (pat, targets) = {
        let cfg = state.config.read().await;
        (cfg.github_pat.clone(), cfg.github_targets.clone())
    };

    if targets.is_empty() {
        return Ok(serde_json::Value::Array(vec![]));
    }

    let client = reqwest::Client::builder()
        .user_agent("UMBRA/1.0")
        .build()?;

    let mut repos: Vec<serde_json::Value> = Vec::new();

    for target in &targets {
        let url = format!("{}/repos/{}/{}", GITHUB_API, target.owner, target.repo);
        let mut req = client.get(&url);
        if let Some(token) = &pat {
            if !token.is_empty() {
                req = req.bearer_auth(token);
            }
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    // Normalise to RepoInfo shape expected by frontend
                    let info = serde_json::json!({
                        "id": target.id,
                        "name": target.name,
                        "owner": target.owner,
                        "repo": target.repo,
                        "fullName": json["full_name"],
                        "openIssues": json["open_issues_count"],
                        "pushedAt": json["pushed_at"],
                        "htmlUrl": json["html_url"],
                    });
                    repos.push(info);
                }
            }
            Ok(resp) => {
                let status = resp.status();
                repos.push(serde_json::json!({
                    "id": target.id,
                    "name": target.name,
                    "owner": target.owner,
                    "repo": target.repo,
                    "error": format!("HTTP {}", status),
                }));
            }
            Err(e) => {
                repos.push(serde_json::json!({
                    "id": target.id,
                    "name": target.name,
                    "owner": target.owner,
                    "repo": target.repo,
                    "error": e.to_string(),
                }));
            }
        }
    }

    let value = serde_json::Value::Array(repos);

    // Store in cache
    {
        let mut cache = state.github_cache.lock().await;
        *cache = Some((value.clone(), Instant::now()));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_repo_listing_requires_pat() {
        let err = user_repos_url(None).unwrap_err();
        assert!(err.to_string().contains("GitHub PAT required"));
    }

    #[test]
    fn user_repo_listing_uses_authenticated_endpoint() {
        let url = user_repos_url(Some("ghp_test")).unwrap();
        assert_eq!(
            url,
            "https://api.github.com/user/repos?per_page=100&sort=updated&affiliation=owner"
        );
        assert!(!url.contains("/users/"));
    }
}
