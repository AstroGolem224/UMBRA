use std::time::{Duration, Instant};
use tauri::State;

use crate::state::AppState;

const CACHE_TTL: Duration = Duration::from_secs(300);
const GITHUB_API: &str = "https://api.github.com";

#[tauri::command]
pub async fn get_github_repos(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
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
        .build()
        .map_err(|e| e.to_string())?;

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
