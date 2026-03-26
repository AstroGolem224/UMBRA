use std::collections::HashMap;
use std::time::Duration;

use reqwest::{header::CONTENT_TYPE, StatusCode};
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
    col_kind: &HashMap<String, String>,
    project_names: &HashMap<String, String>,
) -> serde_json::Value {
    let col_id = task["column_id"].as_str().unwrap_or("").to_string();
    let proj_id = task["project_id"].as_str().unwrap_or("").to_string();
    let kind = col_kind
        .get(&col_id)
        .map(|s| s.as_str())
        .unwrap_or("backlog");
    let status = kind_to_status(kind);
    let proj_name = project_names.get(&proj_id).cloned().unwrap_or_default();

    let comments = task["comments"]
        .as_array()
        .map(|items| {
            items
                .iter()
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

fn pm_body_excerpt(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "empty response body".into();
    }

    const LIMIT: usize = 240;
    if trimmed.chars().count() <= LIMIT {
        trimmed.to_string()
    } else {
        let prefix = trimmed.chars().take(LIMIT).collect::<String>();
        format!("{prefix}...")
    }
}

fn looks_like_json(content_type: Option<&str>, body: &str) -> bool {
    let from_header = content_type
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            lower.contains("application/json") || lower.contains("+json")
        })
        .unwrap_or(false);

    from_header || matches!(body.trim().chars().next(), Some('{') | Some('['))
}

fn parse_pm_response_body(
    action: &str,
    status: StatusCode,
    content_type: Option<&str>,
    body: &str,
) -> std::result::Result<serde_json::Value, AppError> {
    let trimmed = body.trim();

    if !status.is_success() {
        return Err(AppError::Other(format!(
            "PM Tool {action} {status}: {}",
            pm_body_excerpt(trimmed)
        )));
    }

    if status == StatusCode::NO_CONTENT || trimmed.is_empty() {
        return Ok(serde_json::Value::Null);
    }

    if looks_like_json(content_type, trimmed) {
        return serde_json::from_str(trimmed).map_err(|err| {
            AppError::Other(format!("PM Tool {action} returned invalid JSON: {err}"))
        });
    }

    Ok(serde_json::Value::String(trimmed.to_string()))
}

async fn parse_pm_response(
    action: &str,
    resp: reqwest::Response,
) -> std::result::Result<serde_json::Value, AppError> {
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let body = resp.text().await.unwrap_or_default();
    parse_pm_response_body(action, status, content_type.as_deref(), &body)
}

async fn fetch_pm_projects_raw(
    base: &str,
    client: &reqwest::Client,
) -> std::result::Result<Vec<serde_json::Value>, AppError> {
    let resp = client.get(format!("{}/api/projects", base)).send().await?;
    let value = parse_pm_response("projects", resp).await?;
    value
        .as_array()
        .cloned()
        .ok_or_else(|| AppError::Other("PM Tool projects response was not a JSON array".into()))
}

fn collect_project_lookups(
    projects: &[serde_json::Value],
) -> (
    HashMap<String, String>,
    HashMap<String, String>,
    Vec<String>,
) {
    let mut project_names = HashMap::new();
    let mut col_kind = HashMap::new();
    let mut project_ids = Vec::new();

    for project in projects {
        if let Some(id) = project["id"].as_str() {
            project_names.insert(
                id.to_string(),
                project["name"].as_str().unwrap_or("").to_string(),
            );
            project_ids.push(id.to_string());

            if let Some(columns) = project["columns"].as_array() {
                for column in columns {
                    if let (Some(column_id), Some(kind)) =
                        (column["id"].as_str(), column["kind"].as_str())
                    {
                        col_kind.insert(column_id.to_string(), kind.to_string());
                    }
                }
            }
        }
    }

    (project_names, col_kind, project_ids)
}

async fn fetch_project_tasks(
    base: &str,
    client: &reqwest::Client,
    project_id: &str,
) -> std::result::Result<Vec<serde_json::Value>, AppError> {
    let resp = client
        .get(format!("{}/api/projects/{}/tasks", base, project_id))
        .send()
        .await?;
    let value = parse_pm_response("tasks", resp).await?;
    value.as_array().cloned().ok_or_else(|| {
        AppError::Other(format!(
            "PM Tool tasks response for project {project_id} was not a JSON array"
        ))
    })
}

async fn fetch_pm_task_direct(
    base: &str,
    client: &reqwest::Client,
    task_id: &str,
) -> std::result::Result<Option<serde_json::Value>, AppError> {
    let resp = client
        .get(format!("{}/api/tasks/{}", base, task_id))
        .send()
        .await?;

    if resp.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let value = parse_pm_response("task", resp).await?;
    Ok(Some(value))
}

async fn find_pm_task_snapshot(
    base: &str,
    client: &reqwest::Client,
    task_id: &str,
) -> std::result::Result<Option<serde_json::Value>, AppError> {
    if let Some(task) = fetch_pm_task_direct(base, client, task_id).await? {
        return Ok(Some(task));
    }

    let projects = fetch_pm_projects_raw(base, client).await?;
    let (_, _, project_ids) = collect_project_lookups(&projects);

    for project_id in project_ids {
        let tasks = fetch_project_tasks(base, client, &project_id).await?;
        if let Some(task) = tasks
            .into_iter()
            .find(|task| task["id"].as_str().unwrap_or_default() == task_id)
        {
            return Ok(Some(task));
        }
    }

    Ok(None)
}

fn ensure_task_field_matches(
    task: &serde_json::Value,
    field: &str,
    expected: Option<&str>,
    action: &str,
) -> std::result::Result<(), AppError> {
    let Some(expected) = expected else {
        return Ok(());
    };

    let actual = task[field].as_str().unwrap_or_default();
    if actual == expected {
        return Ok(());
    }

    Err(AppError::Other(format!(
        "PM Tool {action} reported success but task field '{field}' is '{actual}' instead of '{expected}'"
    )))
}

fn ensure_task_comment_present(
    task: &serde_json::Value,
    content: &str,
) -> std::result::Result<(), AppError> {
    let found = task["comments"]
        .as_array()
        .map(|comments| {
            comments
                .iter()
                .any(|comment| comment["content"].as_str().unwrap_or_default() == content)
        })
        .unwrap_or(false);

    if found {
        Ok(())
    } else {
        Err(AppError::Other(
            "PM Tool comment reported success but the comment was not present on refetch".into(),
        ))
    }
}

async fn verify_task_after_move(
    base: &str,
    client: &reqwest::Client,
    task_id: &str,
    column_id: &str,
) -> std::result::Result<serde_json::Value, AppError> {
    let task = find_pm_task_snapshot(base, client, task_id)
        .await?
        .ok_or_else(|| {
            AppError::Other(format!(
                "PM Tool move reported success but task {task_id} could not be re-fetched"
            ))
        })?;
    ensure_task_field_matches(&task, "column_id", Some(column_id), "move")?;
    Ok(task)
}

async fn verify_task_after_update(
    base: &str,
    client: &reqwest::Client,
    task_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    priority: Option<&str>,
) -> std::result::Result<serde_json::Value, AppError> {
    let task = find_pm_task_snapshot(base, client, task_id)
        .await?
        .ok_or_else(|| {
            AppError::Other(format!(
                "PM Tool update reported success but task {task_id} could not be re-fetched"
            ))
        })?;
    ensure_task_field_matches(&task, "title", title, "update")?;
    ensure_task_field_matches(&task, "description", description, "update")?;
    ensure_task_field_matches(&task, "priority", priority, "update")?;
    Ok(task)
}

async fn verify_task_after_comment(
    base: &str,
    client: &reqwest::Client,
    task_id: &str,
    content: &str,
) -> std::result::Result<serde_json::Value, AppError> {
    let task = find_pm_task_snapshot(base, client, task_id)
        .await?
        .ok_or_else(|| {
            AppError::Other(format!(
                "PM Tool comment reported success but task {task_id} could not be re-fetched"
            ))
        })?;
    ensure_task_comment_present(&task, content)?;
    Ok(task)
}

/// Manual one-shot fetch of PM Tool tasks, enriched with status + project name.
/// Projects response includes embedded columns, so there is no separate column fetch.
#[tauri::command]
pub async fn get_pm_tasks(
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let base = {
        let cfg = state.config.read().await;
        cfg.pm_tool_url.clone()
    };
    let client = pm_client();

    let projects = fetch_pm_projects_raw(&base, &client).await?;
    let (project_names, col_kind, project_ids) = collect_project_lookups(&projects);

    let mut all_tasks = Vec::new();
    for project_id in project_ids {
        let tasks = fetch_project_tasks(&base, &client, &project_id).await?;
        all_tasks.extend(tasks);
    }

    let enriched = all_tasks
        .into_iter()
        .map(|task| enrich_task(task, &col_kind, &project_names))
        .collect::<Vec<_>>();

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
    parse_pm_response("projects", resp).await
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
    parse_pm_response("columns", resp).await
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

    let resp = pm_client().post(&url).json(&body).send().await?;
    parse_pm_response("create", resp).await
}

/// Move a task to a different column.
#[tauri::command]
pub async fn move_pm_task(
    task_id: String,
    column_id: String,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let (base, url) = {
        let cfg = state.config.read().await;
        (
            cfg.pm_tool_url.clone(),
            format!("{}/api/tasks/{}/move", cfg.pm_tool_url, task_id),
        )
    };

    let body = serde_json::json!({ "column_id": column_id, "position": 0 });
    let client = pm_client();
    let resp = client.patch(&url).json(&body).send().await?;
    let mut result = parse_pm_response("move", resp).await?;
    let verified = verify_task_after_move(&base, &client, &task_id, &column_id).await?;
    if result.is_null() {
        result = verified;
    }
    Ok(result)
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
    parse_pm_response("reorder", resp).await
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
    let (base, url) = {
        let cfg = state.config.read().await;
        (
            cfg.pm_tool_url.clone(),
            format!("{}/api/tasks/{}", cfg.pm_tool_url, task_id),
        )
    };

    let body = serde_json::json!({
        "title": title,
        "description": description,
        "priority": priority,
    });

    let client = pm_client();
    let resp = client.put(&url).json(&body).send().await?;
    let mut result = parse_pm_response("update", resp).await?;
    let verified = verify_task_after_update(
        &base,
        &client,
        &task_id,
        title.as_deref(),
        description.as_deref(),
        priority.as_deref(),
    )
    .await?;
    if result.is_null() {
        result = verified;
    }
    Ok(result)
}

/// Add a comment or audit trail entry to a task.
#[tauri::command]
pub async fn add_pm_comment(
    task_id: String,
    content: String,
    state: State<'_, AppState>,
) -> std::result::Result<serde_json::Value, AppError> {
    let (base, url) = {
        let cfg = state.config.read().await;
        (
            cfg.pm_tool_url.clone(),
            format!("{}/api/comments", cfg.pm_tool_url),
        )
    };

    let body = serde_json::json!({ "task_id": task_id, "content": content });
    let client = pm_client();
    let resp = client.post(&url).json(&body).send().await?;
    let mut result = parse_pm_response("comment", resp).await?;
    let verified = verify_task_after_comment(&base, &client, &task_id, &content).await?;
    if result.is_null() {
        result = verified;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn review_lane_maps_to_in_progress_status() {
        assert_eq!(kind_to_status("review"), "in-progress");
        assert_eq!(kind_to_status("done"), "done");
        assert_eq!(kind_to_status("backlog"), "todo");
    }

    #[test]
    fn enrich_task_sets_frontend_fields() {
        let mut project_names = HashMap::new();
        project_names.insert("p1".to_string(), "UMBRA".to_string());
        let mut col_kind = HashMap::new();
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
            &HashMap::from([(String::from("c1"), String::from("backlog"))]),
            &HashMap::from([(String::from("p1"), String::from("UMBRA"))]),
        );

        assert_eq!(enriched["createdAt"], "2026-03-20T10:00:00Z");
        assert_eq!(enriched["updatedAt"], "2026-03-20T11:00:00Z");
        assert_eq!(enriched["deadline"], "2026-03-21T10:00:00Z");
        assert_eq!(enriched["nextDueDate"], "2026-03-22T10:00:00Z");
        assert_eq!(enriched["position"], 4);
        assert_eq!(enriched["comments"][0]["taskId"], "t1");
        assert_eq!(enriched["comments"][0]["createdAt"], "2026-03-20T10:30:00Z");
    }

    #[test]
    fn parse_pm_response_body_accepts_empty_success() {
        let parsed = parse_pm_response_body("move", StatusCode::NO_CONTENT, None, "").unwrap();
        assert_eq!(parsed, serde_json::Value::Null);
    }

    #[test]
    fn parse_pm_response_body_keeps_plain_text_success() {
        let parsed =
            parse_pm_response_body("comment", StatusCode::OK, Some("text/plain"), "ok").unwrap();
        assert_eq!(parsed, serde_json::Value::String("ok".into()));
    }

    #[test]
    fn parse_pm_response_body_rejects_invalid_json_payloads() {
        let err = parse_pm_response_body("move", StatusCode::OK, Some("application/json"), "{oops")
            .unwrap_err();
        assert!(err.to_string().contains("invalid JSON"));
    }

    #[test]
    fn ensure_task_field_matches_flags_silent_noops() {
        let err = ensure_task_field_matches(
            &serde_json::json!({ "column_id": "c-backlog" }),
            "column_id",
            Some("c-done"),
            "move",
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("task field 'column_id' is 'c-backlog' instead of 'c-done'"));
    }

    #[test]
    fn ensure_task_comment_present_requires_refetched_comment() {
        let err = ensure_task_comment_present(
            &serde_json::json!({
                "comments": [{ "content": "old comment" }]
            }),
            "new comment",
        )
        .unwrap_err();

        assert!(err.to_string().contains("comment was not present"));
    }
}
