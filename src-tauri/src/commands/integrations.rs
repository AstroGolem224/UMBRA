use std::time::Duration;
use tauri::State;

use crate::errors::AppError;
use crate::state::AppState;

fn pm_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

fn kind_to_status(kind: &str) -> &'static str {
    match kind {
        "in_progress" => "in-progress",
        "review" => "in-progress",
        "done" => "done",
        _ => "todo",
    }
}

fn enrich_task(
    mut task: serde_json::Value,
    col_kind: &std::collections::HashMap<String, String>,
    project_names: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    let col_id = task["column_id"].as_str().unwrap_or("").to_string();
    let proj_id = task["project_id"].as_str().unwrap_or("").to_string();
    let kind = col_kind.get(&col_id).map(|s| s.as_str()).unwrap_or("backlog");
    let status = kind_to_status(kind);
    let proj_name = project_names.get(&proj_id).cloned().unwrap_or_default();

    let comments = task["comments"]
        .as_array()
        .map(|items| {
            items.iter()
                .map(|comment| {
                    serde_json::json!({
                        "id": comment["id"],
                        "taskId": comment["task_id"],
                        "content": comment["content"],
                        "createdAt": comment["created_at"],
                        "updatedAt": comment["updated_at"],
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    task["status"] = serde_json::json!(status);
    task["project"] = serde_json::json!(proj_name);
    task["projectId"] = serde_json::json!(proj_id);
    task["columnId"] = serde_json::json!(col_id);
    task["columnKind"] = serde_json::json!(kind);
    task["position"] = task["position"].clone();
    task["deadline"] = task["deadline"].clone();
    task["nextDueDate"] = task["next_due_date"].clone();
    task["createdAt"] = task["created_at"].clone();
    task["updatedAt"] = task["updated_at"].clone();
    task["comments"] = serde_json::Value::Array(comments);
    task
}

/// Manual one-shot fetch of PM Tool tasks, enriched with status + project name.
/// Projects response includes embedded columns — no separate column fetch needed.
/// Tasks are fetched per project via /api/projects/{id}/tasks.
#[tauri::command]
pub async fn get_pm_tasks(
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let base = {
        let cfg = state.config.read().await;
        cfg.pm_tool_url.clone()
    };
    let client = pm_client();

    // Fetch all projects (response includes embedded columns array)
    let projects: Vec<serde_json::Value> = client
        .get(format!("{}/api/projects", base))
        .send()
        .await?
        .json()
        .await
        .unwrap_or_default();

    // Build lookup maps from embedded columns
    let mut project_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut col_kind: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut project_ids: Vec<String> = Vec::new();

    for p in &projects {
        if let Some(id) = p["id"].as_str() {
            let name = p["name"].as_str().unwrap_or("").to_string();
            project_names.insert(id.to_string(), name);
            project_ids.push(id.to_string());

            if let Some(cols) = p["columns"].as_array() {
                for col in cols {
                    if let (Some(cid), Some(kind)) = (col["id"].as_str(), col["kind"].as_str()) {
                        col_kind.insert(cid.to_string(), kind.to_string());
                    }
                }
            }
        }
    }

    // Fetch tasks per project in parallel
    let task_futs: Vec<_> = project_ids
        .iter()
        .map(|id| client.get(format!("{}/api/projects/{}/tasks", base, id)).send())
        .collect();

    let mut all_tasks: Vec<serde_json::Value> = Vec::new();
    for fut in task_futs {
        if let Ok(resp) = fut.await {
            if let Ok(tasks) = resp.json::<Vec<serde_json::Value>>().await {
                all_tasks.extend(tasks);
            }
        }
    }

    // Map column kind → frontend status
    // Enrich tasks with status + project name
    let enriched: Vec<serde_json::Value> = all_tasks
        .into_iter()
        .map(|task| enrich_task(task, &col_kind, &project_names))
        .collect();

    Ok(serde_json::Value::Array(enriched))
}

/// Fetch all projects from PM Tool.
#[tauri::command]
pub async fn get_pm_projects(
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/projects", cfg.pm_tool_url)
    };
    let resp = pm_client().get(&url).send().await?;
    Ok(resp.json::<serde_json::Value>().await?)
}

/// Fetch columns for a project.
#[tauri::command]
pub async fn get_pm_columns(
    project_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/projects/{}/columns", cfg.pm_tool_url, project_id)
    };
    let resp = pm_client().get(&url).send().await?;
    Ok(resp.json::<serde_json::Value>().await?)
}

/// Create a new task in the PM Tool.
#[tauri::command]
pub async fn create_pm_task(
    title: String,
    project_id: String,
    column_id: String,
    priority: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/tasks", cfg.pm_tool_url)
    };

    let body = serde_json::json!({
        "title": title,
        "project_id": project_id,
        "column_id": column_id,
        "priority": priority.unwrap_or_else(|| "medium".into()),
        "description": description.unwrap_or_default(),
    });

    let resp = pm_client()
        .post(&url)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!("PM Tool {status}: {text}")));
    }

    Ok(resp.json::<serde_json::Value>().await?)
}

/// Move a task to a different column (Kanban column change).
#[tauri::command]
pub async fn move_pm_task(
    task_id: String,
    column_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/tasks/{}/move", cfg.pm_tool_url, task_id)
    };

    let body = serde_json::json!({ "column_id": column_id, "position": 0 });
    let resp = pm_client().patch(&url).json(&body).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!("PM Tool move {status}: {text}")));
    }

    Ok(resp.json::<serde_json::Value>().await?)
}

/// Reorder tasks inside a column after drag-and-drop.
#[tauri::command]
pub async fn reorder_pm_tasks(
    column_id: String,
    task_ids: Vec<String>,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/tasks/reorder", cfg.pm_tool_url)
    };

    let body = serde_json::json!({
        "column_id": column_id,
        "task_ids": task_ids,
    });

    let resp = pm_client().patch(&url).json(&body).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!("PM Tool reorder {status}: {text}")));
    }

    Ok(resp.json::<serde_json::Value>().await?)
}

/// Update task title, description, and/or priority.
#[tauri::command]
pub async fn update_pm_task(
    task_id: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/tasks/{}", cfg.pm_tool_url, task_id)
    };

    let body = serde_json::json!({
        "title": title,
        "description": description,
        "priority": priority,
    });

    let resp = pm_client().put(&url).json(&body).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!("PM Tool update {status}: {text}")));
    }

    Ok(resp.json::<serde_json::Value>().await?)
}

/// Add a comment/audit trail to a task.
#[tauri::command]
pub async fn add_pm_comment(
    task_id: String,
    content: String,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let url = {
        let cfg = state.config.read().await;
        format!("{}/api/comments", cfg.pm_tool_url)
    };

    let body = serde_json::json!({ "task_id": task_id, "content": content });
    let resp = pm_client().post(&url).json(&body).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!("PM Tool comment {status}: {text}")));
    }

    Ok(resp.json::<serde_json::Value>().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_lane_maps_to_in_progress_status() {
        assert_eq!(kind_to_status("review"), "in-progress");
        assert_eq!(kind_to_status("done"), "done");
        assert_eq!(kind_to_status("backlog"), "todo");
    }

    #[test]
    fn enrich_task_sets_frontend_fields() {
        let mut project_names = std::collections::HashMap::new();
        project_names.insert("p1".to_string(), "UMBRA".to_string());
        let mut col_kind = std::collections::HashMap::new();
        col_kind.insert("c1".to_string(), "review".to_string());

        let raw = serde_json::json!({
            "id": "t1",
            "project_id": "p1",
            "column_id": "c1",
            "title": "Task"
        });

        let enriched = enrich_task(raw, &col_kind, &project_names);

        assert_eq!(enriched["status"], "in-progress");
        assert_eq!(enriched["project"], "UMBRA");
        assert_eq!(enriched["projectId"], "p1");
        assert_eq!(enriched["columnId"], "c1");
        assert_eq!(enriched["columnKind"], "review");
    }

    #[test]
    fn enrich_task_camel_cases_comment_and_dates() {
        let enriched = enrich_task(
            serde_json::json!({
                "id": "t1",
                "project_id": "p1",
                "column_id": "c1",
                "title": "Task",
                "created_at": "2026-03-20T10:00:00Z",
                "updated_at": "2026-03-20T11:00:00Z",
                "deadline": "2026-03-21T10:00:00Z",
                "next_due_date": "2026-03-22T10:00:00Z",
                "position": 4,
                "comments": [
                    {
                        "id": "comment-1",
                        "task_id": "t1",
                        "content": "hello",
                        "created_at": "2026-03-20T10:30:00Z",
                        "updated_at": "2026-03-20T10:30:00Z"
                    }
                ]
            }),
            &std::collections::HashMap::from([(String::from("c1"), String::from("backlog"))]),
            &std::collections::HashMap::from([(String::from("p1"), String::from("UMBRA"))]),
        );

        assert_eq!(enriched["createdAt"], "2026-03-20T10:00:00Z");
        assert_eq!(enriched["updatedAt"], "2026-03-20T11:00:00Z");
        assert_eq!(enriched["deadline"], "2026-03-21T10:00:00Z");
        assert_eq!(enriched["nextDueDate"], "2026-03-22T10:00:00Z");
        assert_eq!(enriched["position"], 4);
        assert_eq!(enriched["comments"][0]["taskId"], "t1");
        assert_eq!(enriched["comments"][0]["createdAt"], "2026-03-20T10:30:00Z");
    }
}
