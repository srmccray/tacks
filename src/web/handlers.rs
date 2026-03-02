use askama::Template;
use axum::Form;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::atomic::Ordering;

use crate::models::{Comment, Task, validate_close_reason};
use crate::web::AppState;
use crate::web::errors::AppError;

/// Template for the index/home page.
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

/// Render an askama template into an axum HTML response.
fn render_template<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("template error: {e}"),
        )
            .into_response(),
    }
}

/// Index page handler — renders the home template.
pub async fn index() -> Response {
    render_template(IndexTemplate)
}

// ---------------------------------------------------------------------------
// Request body types
// ---------------------------------------------------------------------------

/// Request body for POST /api/tasks.
#[derive(Debug, Deserialize)]
pub struct CreateTaskBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub parent_id: Option<String>,
}

/// Request body for PATCH /api/tasks/:id.
#[derive(Debug, Deserialize)]
pub struct UpdateTaskBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<u8>,
    pub assignee: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

/// Request body for POST /api/tasks/:id/close.
#[derive(Debug, Deserialize)]
pub struct CloseTaskBody {
    pub reason: Option<String>,
    pub comment: Option<String>,
}

/// Request body for POST /api/tasks/:id/deps.
#[derive(Debug, Deserialize)]
pub struct AddDepBody {
    pub parent_id: String,
}

/// Request body for POST /api/tasks/:id/comments.
#[derive(Debug, Deserialize)]
pub struct AddCommentBody {
    pub body: String,
}

// ---------------------------------------------------------------------------
// Query parameter types
// ---------------------------------------------------------------------------

/// Deserialize an optional string field, treating empty strings as `None`.
fn deserialize_empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(_) => Ok(s),
    }
}

/// Query parameters for GET /api/tasks.
///
/// `status` and `priority` accept comma-separated values for multi-select OR filtering
/// (e.g. `status=open,in_progress` or `priority=1,2`).
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    /// Comma-separated status values for multi-select OR filtering.
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub status: Option<String>,
    /// Comma-separated priority values for multi-select OR filtering.
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub tag: Option<String>,
    pub all: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub parent: Option<String>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub search: Option<String>,
}

/// Query parameters for GET /api/tasks/ready.
#[derive(Debug, Deserialize)]
pub struct ReadyTasksQuery {
    pub limit: Option<u32>,
}

// ---------------------------------------------------------------------------
// Stats response type
// ---------------------------------------------------------------------------

/// Response body for GET /api/stats.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub by_status: Map<String, Value>,
    pub by_priority: Map<String, Value>,
    pub by_tag: Map<String, Value>,
}

// ---------------------------------------------------------------------------
// API handlers
// ---------------------------------------------------------------------------

/// POST /api/tasks — Create a new task (201).
pub async fn api_create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    // title is required
    let title = body
        .title
        .ok_or_else(|| AppError::Validation("title is required".to_string()))?;

    let priority = body.priority.unwrap_or(2);
    let description = body.description.clone();
    let tags = body.tags.clone().unwrap_or_default();
    let parent_id = body.parent_id.clone();

    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Task, String> {
        let db = db.lock().unwrap();

        // Generate ID
        let id = if let Some(ref pid) = parent_id {
            // Verify parent exists
            db.get_task(pid)?
                .ok_or_else(|| format!("parent task not found: {pid}"))?;
            db.generate_child_id(pid)?
        } else {
            db.generate_id()?
        };

        let now = chrono::Utc::now();
        let task = Task {
            id: id.clone(),
            title: title.clone(),
            description: description.clone(),
            status: crate::models::Status::Open,
            priority,
            assignee: None,
            parent_id: parent_id.clone(),
            tags: tags.clone(),
            created_at: now,
            updated_at: now,
            close_reason: None,
            notes: None,
        };

        db.insert_task(&task)?;

        // Auto-tag parent as epic when a child is created
        if let Some(ref pid) = parent_id {
            let mut parent_tags = db.get_task_tags(pid)?;
            if !parent_tags.contains(&"epic".to_string()) {
                parent_tags.push("epic".to_string());
                db.update_tags(pid, &parent_tags)?;
            }
        }

        Ok(task)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok((StatusCode::CREATED, Json(result)))
}

/// Parse a comma-separated tag query param into a list of trimmed, non-empty tags.
fn parse_tags(tag_param: Option<&str>) -> Vec<String> {
    match tag_param {
        None => vec![],
        Some(s) => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
    }
}

/// Filter a task list to only tasks that have at least one of the given tags (OR semantics).
fn filter_by_tags(tasks: Vec<Task>, tags: &[String]) -> Vec<Task> {
    if tags.is_empty() {
        return tasks;
    }
    tasks
        .into_iter()
        .filter(|t| tags.iter().any(|tag| t.tags.contains(tag)))
        .collect()
}

/// Parse comma-separated priority values into a `Vec<u8>`.
fn parse_priority_values(s: &Option<String>) -> Vec<u8> {
    match s.as_deref() {
        None | Some("") => vec![],
        Some(v) => v
            .split(',')
            .filter_map(|p| p.trim().parse::<u8>().ok())
            .collect(),
    }
}

/// Parse comma-separated status values into a `Vec<String>`.
fn parse_status_values(s: &Option<String>) -> Vec<String> {
    match s.as_deref() {
        None | Some("") => vec![],
        Some(v) => v
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    }
}

/// GET /api/tasks — List tasks with optional filters (200).
///
/// `status` and `priority` accept comma-separated values for multi-select OR filtering.
pub async fn api_list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, AppError> {
    let show_all = query.all.unwrap_or(false);
    let status_values = parse_status_values(&query.status);
    let priority_values = parse_priority_values(&query.priority);
    let tag_param = query.tag.clone();
    let parent_filter = query.parent.clone();
    let search_filter = query.search.clone();

    // Parse comma-separated tags for multi-tag OR filtering
    let tags = parse_tags(tag_param.as_deref());
    // For DB query: use a single tag when exactly one is selected (uses indexed LIKE);
    // when multiple tags, skip DB tag filter and post-filter in Rust.
    let db_tag_filter = if tags.len() == 1 {
        tags.first().cloned()
    } else {
        None
    };
    let multi_tags = if tags.len() > 1 { tags } else { vec![] };

    let db = state.db.clone();
    let tasks = tokio::task::spawn_blocking(move || -> Result<Vec<Task>, String> {
        let db = db.lock().unwrap();
        // For single status/priority, pass directly to DB for efficiency.
        // For multi-value, load without that filter then post-filter in Rust.
        let (db_status, db_priority) = match (status_values.len(), priority_values.len()) {
            (0 | 1, 0 | 1) => (
                status_values.first().map(|s| s.as_str()),
                priority_values.first().copied(),
            ),
            _ => (None, None),
        };
        let mut tasks = db.list_tasks(
            show_all || !status_values.is_empty(),
            db_status,
            db_priority,
            db_tag_filter.as_deref(),
            parent_filter.as_deref(),
            search_filter.as_deref(),
        )?;
        // Post-filter for multi-value OR semantics
        if status_values.len() > 1 {
            let status_strs: Vec<&str> = status_values.iter().map(|s| s.as_str()).collect();
            tasks.retain(|t| {
                let s = match t.status {
                    crate::models::Status::Open => "open",
                    crate::models::Status::InProgress => "in_progress",
                    crate::models::Status::Done => "done",
                    crate::models::Status::Blocked => "blocked",
                };
                status_strs.contains(&s)
            });
        }
        if priority_values.len() > 1 {
            tasks.retain(|t| priority_values.contains(&t.priority));
        }
        let tasks = filter_by_tags(tasks, &multi_tags);
        Ok(tasks)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// GET /api/tasks/ready — Tasks with no open blockers (200).
pub async fn api_ready_tasks(
    State(state): State<AppState>,
    Query(query): Query<ReadyTasksQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit;
    let db = state.db.clone();
    let tasks = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_ready_tasks(limit)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// GET /api/tasks/blocked — Tasks with open blockers (200).
pub async fn api_blocked_tasks(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let tasks = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_blocked_tasks()
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// GET /api/tasks/:id — Show a task by ID (200 or 404).
pub async fn api_show_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let task = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_task(&id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound("task not found".to_string())),
    }
}

/// PATCH /api/tasks/:id — Update task fields (200 or 404).
pub async fn api_update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Task, String> {
        let db = db.lock().unwrap();

        // Verify task exists
        db.get_task(&id)?
            .ok_or_else(|| format!("task not found: {id}"))?;

        // Update tags separately if provided
        if let Some(ref tags) = body.tags {
            db.update_tags(&id, tags)?;
        }

        // Update remaining fields
        db.update_task(
            &id,
            body.title.as_deref(),
            body.priority,
            body.status.as_deref(),
            body.description.as_deref(),
            body.assignee.as_deref(),
            None,
            body.notes.as_deref(),
        )?;

        // Return the updated task
        db.get_task(&id)?
            .ok_or_else(|| format!("task not found after update: {id}"))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    match result {
        Ok(task) => Ok(Json(task)),
        Err(e) if e.contains("not found") => Err(AppError::NotFound(e)),
        Err(e) => Err(AppError::Internal(e)),
    }
}

/// POST /api/tasks/:id/close — Close a task (200, 404, or 422).
pub async fn api_close_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CloseTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    // Validate reason if provided
    let reason = body.reason.as_deref();
    if let Some(r) = reason {
        validate_close_reason(r).map_err(AppError::Validation)?;
    }

    let reason_owned = body.reason.clone();
    let comment_owned = body.comment.clone();

    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Task, String> {
        let db = db.lock().unwrap();

        // Verify task exists
        db.get_task(&id)?
            .ok_or_else(|| format!("task not found: {id}"))?;

        // Close the task
        db.close_task(&id, reason_owned.as_deref())?;

        // Add comment if provided
        if let Some(ref comment) = comment_owned {
            db.add_comment(&id, comment)?;
        }

        // Return the updated task
        db.get_task(&id)?
            .ok_or_else(|| format!("task not found after close: {id}"))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    match result {
        Ok(task) => Ok(Json(task)),
        Err(e) if e.contains("not found") => Err(AppError::NotFound(e)),
        Err(e) => Err(AppError::Internal(e)),
    }
}

/// POST /api/tasks/:id/deps — Add a dependency (201 or 409).
pub async fn api_add_dep(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddDepBody>,
) -> Result<impl IntoResponse, AppError> {
    let parent_id = body.parent_id.clone();
    let db = state.db.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.add_dependency(&id, &parent_id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    match result {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(e) if e.contains("circular") || e.contains("already exists") => {
            Err(AppError::Conflict(e))
        }
        Err(e) => Err(AppError::Internal(e)),
    }
}

/// DELETE /api/tasks/:child_id/deps/:parent_id — Remove a dependency (204).
pub async fn api_remove_dep(
    State(state): State<AppState>,
    Path((child_id, parent_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.remove_dependency(&child_id, &parent_id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    match result {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) if e.contains("not found") => Err(AppError::NotFound(e)),
        Err(e) => Err(AppError::Internal(e)),
    }
}

/// POST /api/tasks/:id/comments — Add a comment (201).
pub async fn api_add_comment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddCommentBody>,
) -> Result<impl IntoResponse, AppError> {
    let comment_body = body.body.clone();
    let db = state.db.clone();

    let comment = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.add_comment(&id, &comment_body)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok((StatusCode::CREATED, Json(comment)))
}

/// GET /api/tasks/:id/comments — List comments on a task (200).
pub async fn api_list_comments(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let comments: Vec<Comment> = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_comments(&id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(comments))
}

/// GET /api/tasks/:id/children — List subtasks (200).
pub async fn api_children(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let tasks: Vec<Task> = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_children(&id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// GET /api/tasks/:id/blockers — List blockers for a task as full Task objects (200).
pub async fn api_blockers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let tasks: Vec<Task> = tokio::task::spawn_blocking(move || -> Result<Vec<Task>, String> {
        let db = db.lock().unwrap();
        let deps = db.get_blockers(&id)?;
        let mut tasks = Vec::with_capacity(deps.len());
        for dep in deps {
            if let Some(t) = db.get_task(&dep.parent_id)? {
                tasks.push(t);
            }
        }
        Ok(tasks)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// GET /api/tasks/:id/dependents — List tasks that depend on this task (200).
pub async fn api_dependents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let tasks: Vec<Task> = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.get_dependents(&id)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(tasks))
}

/// Response body for GET /api/epics — epic task with child progress counts.
#[derive(Debug, Serialize)]
pub struct EpicProgress {
    pub task: Task,
    pub children_total: usize,
    pub children_done: usize,
}

/// GET /api/epics — List epics with child completion progress (200).
pub async fn api_epics(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let result: Vec<EpicProgress> =
        tokio::task::spawn_blocking(move || -> Result<Vec<EpicProgress>, String> {
            let db = db.lock().unwrap();
            let epics = db.list_tasks(true, None, None, Some("epic"), None, None)?;
            let mut out = Vec::with_capacity(epics.len());
            for epic in epics {
                let children = db.get_children(&epic.id)?;
                let children_total = children.len();
                let children_done = children
                    .iter()
                    .filter(|c| matches!(c.status, crate::models::Status::Done))
                    .count();
                out.push(EpicProgress {
                    task: epic,
                    children_total,
                    children_done,
                });
            }
            Ok(out)
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(AppError::Internal)?;

    Ok(Json(result))
}

/// Response body for GET /api/prime — AI context output.
#[derive(Debug, Serialize)]
pub struct PrimeResponse {
    pub stats: StatsResponse,
    pub in_progress: Vec<Task>,
    pub ready: Vec<Task>,
}

/// GET /api/prime — AI context: stats + in-progress tasks + ready queue (200).
pub async fn api_prime(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<PrimeResponse, String> {
        let db = db.lock().unwrap();

        let by_status_vec = db.task_count_by_status()?;
        let by_priority_vec = db.task_count_by_priority()?;
        let by_tag_vec = db.task_count_by_tag()?;

        let by_status: Map<String, Value> = by_status_vec
            .into_iter()
            .map(|(k, v)| (k, Value::Number(v.into())))
            .collect();

        let by_priority: Map<String, Value> = by_priority_vec
            .into_iter()
            .map(|(k, v)| (k.to_string(), Value::Number(v.into())))
            .collect();

        let by_tag: Map<String, Value> = by_tag_vec
            .into_iter()
            .map(|(k, v)| (k, Value::Number(v.into())))
            .collect();

        let stats = StatsResponse {
            by_status,
            by_priority,
            by_tag,
        };

        let in_progress = db.list_tasks(false, Some("in_progress"), None, None, None, None)?;
        let ready = db.get_ready_tasks(Some(5))?;

        Ok(PrimeResponse {
            stats,
            in_progress,
            ready,
        })
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(result))
}

/// GET /api/poll — Lightweight change-detection endpoint for HTMX polling.
///
/// Compares the current SQLite `PRAGMA data_version` against the last known value
/// stored in `AppState`. Returns:
/// - `304 Not Modified` when nothing has changed (HTMX treats this as no-swap)
/// - `200 OK` with `HX-Trigger: data-changed` header when data has changed
pub async fn api_poll(State(state): State<AppState>) -> Response {
    let db = state.db.clone();
    let version = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.data_version()
    })
    .await;

    let current = match version {
        Ok(Ok(v)) => v,
        _ => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let last = state.last_data_version.load(Ordering::Relaxed);

    if current == last {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        state.last_data_version.store(current, Ordering::Relaxed);
        (
            StatusCode::OK,
            [(axum_htmx::headers::HX_TRIGGER, "data-changed")],
            "",
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// HTML view handlers
// ---------------------------------------------------------------------------

/// A task row enriched with optional parent info for list/board views.
struct TaskRow {
    task: Task,
    /// Parent epic ID, if this task is a subtask.
    parent_id: Option<String>,
    /// Parent epic title, if this task is a subtask.
    parent_title: Option<String>,
}

impl TaskRow {
    /// Build a `TaskRow` from a task, looking up parent title from the provided map.
    fn from_task(task: Task, parents: &std::collections::HashMap<String, Task>) -> Self {
        let parent_id = task.parent_id.clone();
        let parent_title = parent_id
            .as_deref()
            .and_then(|pid| parents.get(pid))
            .map(|p| p.title.clone());
        TaskRow {
            task,
            parent_id,
            parent_title,
        }
    }
}

/// Fetch a map of task_id -> Task for a set of IDs (used to batch-load parent epics).
fn fetch_parent_map(
    db: &crate::db::Database,
    ids: impl Iterator<Item = String>,
) -> Result<std::collections::HashMap<String, Task>, String> {
    let mut map = std::collections::HashMap::new();
    for id in ids {
        if let Some(t) = db.get_task(&id)? {
            map.insert(id, t);
        }
    }
    Ok(map)
}

/// Template for the task list page at GET /tasks.
#[derive(Template)]
#[template(path = "tasks_list.html")]
struct TaskListTemplate {
    tasks: Vec<TaskRow>,
    status_filter: Option<String>,
    priority_filter: Option<String>,
    /// Comma-separated tag filter string (may contain multiple tags).
    tag_filter: Option<String>,
    /// Parsed list of selected tags (for rendering pills).
    selected_tags: Vec<String>,
    /// All available tags for the dropdown.
    all_tags: Vec<String>,
    search_filter: Option<String>,
    /// True when any filter (status, priority, tag, search) is active.
    has_filters: bool,
    /// Pre-built query string for HTMX polling (preserves current filters).
    poll_query: String,
}

/// Template for the task detail page at GET /tasks/:id.
#[derive(Template)]
#[template(path = "task_detail.html")]
struct TaskDetailTemplate {
    task: Task,
    /// Parent epic, if this task is a subtask.
    parent: Option<Task>,
    blockers: Vec<Task>,
    dependents: Vec<Task>,
    comments: Vec<Comment>,
}

/// Template for the task detail modal fragment loaded via HTMX.
#[derive(Template)]
#[template(path = "task_detail_fragment.html")]
struct TaskDetailFragmentTemplate {
    task: Task,
    /// Parent epic, if this task is a subtask.
    parent: Option<Task>,
    blockers: Vec<Task>,
    dependents: Vec<Task>,
    comments: Vec<Comment>,
}

/// Template for the kanban board page at GET /board.
#[derive(Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    open_tasks: Vec<TaskRow>,
    in_progress_tasks: Vec<TaskRow>,
    blocked_tasks: Vec<TaskRow>,
    done_tasks: Vec<TaskRow>,
    /// All epics available in the dropdown filter.
    epics: Vec<Task>,
    /// Currently selected epic ID filter (empty string = none).
    selected_epic: String,
    /// Currently selected priority filter (empty string = none).
    selected_priority: String,
    /// Pre-built query string for HTMX polling (preserves current filters).
    poll_query: String,
}

/// Query parameters for GET /board.
#[derive(Debug, Deserialize)]
pub struct BoardQuery {
    /// Comma-separated epic IDs for multi-select OR filtering (e.g. `epic=tk-abc1,tk-def2`).
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub epic: Option<String>,
    /// Comma-separated priority values for multi-select OR filtering (e.g. `priority=1,2`).
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub priority: Option<String>,
}

/// Template struct for one row in the epics view.
struct EpicRow {
    task: Task,
    children_total: usize,
    children_done: usize,
}

/// Template for the epics page at GET /epics.
#[derive(Template)]
#[template(path = "epics.html")]
struct EpicsTemplate {
    epics: Vec<EpicRow>,
}

/// Template for the epic detail page at GET /epics/:id.
#[derive(Template)]
#[template(path = "epic_detail.html")]
struct EpicDetailTemplate {
    task: Task,
    children: Vec<Task>,
    children_done: usize,
    children_total: usize,
    /// Current view mode: "list" (default) or "board".
    view: String,
}

/// Query parameters for GET /epics/:id.
#[derive(Debug, Deserialize)]
pub struct EpicDetailQuery {
    pub view: Option<String>,
}

/// Template for the create task form at GET /tasks/new.
#[derive(Template)]
#[template(path = "task_new.html")]
struct TaskNewTemplate;

/// Build a query string from current filter params for HTMX polling.
fn build_poll_query(
    status: &Option<String>,
    priority: &Option<String>,
    tag: &Option<String>,
    search: &Option<String>,
) -> String {
    let mut parts = Vec::new();
    if let Some(s) = status {
        parts.push(format!("status={s}"));
    }
    if let Some(p) = priority {
        parts.push(format!("priority={p}"));
    }
    if let Some(t) = tag {
        parts.push(format!("tag={t}"));
    }
    if let Some(q) = search {
        parts.push(format!("search={q}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

/// GET /tasks — Task list page with optional filter query params.
pub async fn task_list(
    State(state): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> Response {
    let has_filter = params.status.is_some()
        || params.priority.is_some()
        || params.tag.is_some()
        || params.search.as_deref().is_some_and(|s| !s.is_empty());
    let status_values = parse_status_values(&params.status);
    let priority_values = parse_priority_values(&params.priority);
    let show_all = has_filter || params.all.unwrap_or(false);
    let tag_param = params.tag.clone();
    let search_filter = params.search.clone();

    // Parse comma-separated tags for multi-tag OR filtering
    let selected_tags = parse_tags(tag_param.as_deref());
    let db_tag_filter = if selected_tags.len() == 1 {
        selected_tags.first().cloned()
    } else {
        None
    };
    let multi_tags = if selected_tags.len() > 1 {
        selected_tags.clone()
    } else {
        vec![]
    };

    let db = state.db.clone();
    let (task_rows, all_tags) =
        tokio::task::spawn_blocking(move || -> Result<(Vec<TaskRow>, Vec<String>), String> {
            let db = db.lock().unwrap();
            // For single status/priority, pass directly to DB for efficiency.
            // For multi-value, load without that filter then post-filter in Rust.
            let (db_status, db_priority) = match (status_values.len(), priority_values.len()) {
                (0 | 1, 0 | 1) => (
                    status_values.first().map(|s| s.as_str()),
                    priority_values.first().copied(),
                ),
                _ => (None, None),
            };
            let mut tasks = db.list_tasks(
                show_all,
                db_status,
                db_priority,
                db_tag_filter.as_deref(),
                None,
                search_filter.as_deref(),
            )?;
            // Post-filter for multi-value OR semantics
            if status_values.len() > 1 {
                let status_strs: Vec<&str> = status_values.iter().map(|s| s.as_str()).collect();
                tasks.retain(|t| {
                    let s = match t.status {
                        crate::models::Status::Open => "open",
                        crate::models::Status::InProgress => "in_progress",
                        crate::models::Status::Done => "done",
                        crate::models::Status::Blocked => "blocked",
                    };
                    status_strs.contains(&s)
                });
            }
            if priority_values.len() > 1 {
                tasks.retain(|t| priority_values.contains(&t.priority));
            }
            let tasks = filter_by_tags(tasks, &multi_tags);
            // Batch-load parent epics (avoids N+1: one lookup per unique parent_id)
            let parent_ids: std::collections::HashSet<String> =
                tasks.iter().filter_map(|t| t.parent_id.clone()).collect();
            let parents = fetch_parent_map(&db, parent_ids.into_iter())?;
            let rows: Vec<TaskRow> = tasks
                .into_iter()
                .map(|t| TaskRow::from_task(t, &parents))
                .collect();
            // Fetch all tags for the dropdown
            let all_tags: Vec<String> = db
                .task_count_by_tag()?
                .into_iter()
                .map(|(tag, _count)| tag)
                .collect();
            Ok((rows, all_tags))
        })
        .await
        .unwrap()
        .unwrap_or_else(|_| (vec![], vec![]));

    let poll_query = build_poll_query(
        &params.status,
        &params.priority,
        &params.tag,
        &params.search,
    );

    render_template(TaskListTemplate {
        tasks: task_rows,
        status_filter: params.status,
        priority_filter: params.priority,
        tag_filter: params.tag,
        selected_tags,
        all_tags,
        search_filter: params.search,
        has_filters: has_filter,
        poll_query,
    })
}

/// GET /tasks/new — Create task form.
pub async fn task_new() -> Response {
    render_template(TaskNewTemplate)
}

/// Form body for POST /tasks (HTML form submission from task_new.html).
#[derive(Debug, Deserialize)]
pub struct CreateTaskFormBody {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<u8>,
}

/// POST /tasks — Handle HTML form submission from the create-task form.
///
/// Accepts `application/x-www-form-urlencoded` body, creates the task, then
/// issues a 303 redirect to `/tasks` so the user lands on the task list.
/// The JSON API at `POST /api/tasks` is unchanged and continues to return 201.
pub async fn task_create_form(
    State(state): State<AppState>,
    Form(body): Form<CreateTaskFormBody>,
) -> Result<impl IntoResponse, AppError> {
    let title = body.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("title is required".to_string()));
    }
    let priority = body.priority.unwrap_or(2);
    let description = body
        .description
        .filter(|d| !d.trim().is_empty())
        .map(|d| d.trim().to_string());

    let db = state.db.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let db = db.lock().unwrap();
        let id = db.generate_id()?;
        let now = chrono::Utc::now();
        let task = Task {
            id,
            title,
            description,
            status: crate::models::Status::Open,
            priority,
            assignee: None,
            parent_id: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            close_reason: None,
            notes: None,
        };
        db.insert_task(&task)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Redirect::to("/tasks"))
}

/// All data needed to render a task detail view (full page or modal fragment).
struct TaskDetailData {
    task: Task,
    /// Parent epic, if this task is a subtask.
    parent: Option<Task>,
    blockers: Vec<Task>,
    dependents: Vec<Task>,
    comments: Vec<Comment>,
}

/// GET /tasks/:id — Task detail page (200 or 404).
///
/// When called from HTMX (`HX-Request: true`), renders a modal fragment.
/// When called via direct browser navigation, renders the full page template.
pub async fn task_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let is_htmx = headers.contains_key("HX-Request");
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Option<TaskDetailData>, String> {
        let db = db.lock().unwrap();
        let task = match db.get_task(&id)? {
            Some(t) => t,
            None => return Ok(None),
        };
        // Fetch parent epic if this task is a subtask
        let parent = if let Some(ref pid) = task.parent_id {
            db.get_task(pid)?
        } else {
            None
        };
        // Resolve blocker dependency records to full Task objects
        let blocker_deps = db.get_blockers(&id)?;
        let mut blockers = Vec::with_capacity(blocker_deps.len());
        for dep in blocker_deps {
            if let Some(t) = db.get_task(&dep.parent_id)? {
                blockers.push(t);
            }
        }
        let dependents = db.get_dependents(&id)?;
        let comments = db.get_comments(&id)?;
        Ok(Some(TaskDetailData {
            task,
            parent,
            blockers,
            dependents,
            comments,
        }))
    })
    .await
    .unwrap();

    match result {
        Ok(Some(data)) => {
            if is_htmx {
                render_template(TaskDetailFragmentTemplate {
                    task: data.task,
                    parent: data.parent,
                    blockers: data.blockers,
                    dependents: data.dependents,
                    comments: data.comments,
                })
            } else {
                render_template(TaskDetailTemplate {
                    task: data.task,
                    parent: data.parent,
                    blockers: data.blockers,
                    dependents: data.dependents,
                    comments: data.comments,
                })
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "task not found").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
    }
}

/// Build a poll query string for the board view (preserves epic + priority filters).
fn build_board_poll_query(epic: &Option<String>, priority: &Option<String>) -> String {
    let mut parts = Vec::new();
    if let Some(e) = epic {
        parts.push(format!("epic={e}"));
    }
    if let Some(p) = priority {
        parts.push(format!("priority={p}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

/// GET /board — Kanban board view grouped by status, with optional epic and priority filters.
pub async fn board(State(state): State<AppState>, Query(query): Query<BoardQuery>) -> Response {
    let epic_filter = query.epic.clone();
    let priority_filter = query.priority.clone();

    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<BoardTemplate, String> {
        let db = db.lock().unwrap();

        // Fetch all epics for the dropdown.
        let epics = db.list_tasks(true, None, None, Some("epic"), None, None)?;

        // Parse multi-select values.
        let epic_values = parse_status_values(&epic_filter); // epic IDs are strings
        let priority_values = parse_priority_values(&priority_filter);

        // Helper: fetch tasks for a given status, applying epic and priority filters.
        // For single values, pass directly to DB for efficiency; for multi-values, post-filter.
        let fetch = |status: &str, show_done: bool| -> Result<Vec<Task>, String> {
            let (db_parent, db_priority) = match (epic_values.len(), priority_values.len()) {
                (1, 1) => (
                    epic_values.first().map(|s| s.as_str()),
                    priority_values.first().copied(),
                ),
                (1, _) => (epic_values.first().map(|s| s.as_str()), None),
                (_, 1) => (None, priority_values.first().copied()),
                _ => (None, None),
            };
            db.list_tasks(show_done, Some(status), db_priority, None, db_parent, None)
        };

        // Fetch the set of task IDs that have at least one open blocker (via dep graph).
        // These tasks belong in the Blocked column regardless of their `status` field,
        // because `dep add` does not automatically change a task's status to "blocked".
        let dep_blocked_ids: std::collections::HashSet<String> =
            db.get_blocked_tasks()?.into_iter().map(|t| t.id).collect();

        // Fetch open tasks, then split: those with open blockers go to the blocked column.
        let open_raw = fetch("open", false)?;
        let (dep_blocked_open, open_raw_filtered): (Vec<Task>, Vec<Task>) = open_raw
            .into_iter()
            .partition(|t| dep_blocked_ids.contains(&t.id));

        let in_progress_raw = fetch("in_progress", false)?;
        // Combine status=blocked tasks with open tasks that have active dep blockers.
        let mut blocked_raw = fetch("blocked", false)?;
        blocked_raw.extend(dep_blocked_open);
        let done_raw = fetch("done", true)?;

        // Post-filter for multi-value epic or priority selections.
        let post_filter = |mut tasks: Vec<Task>| -> Vec<Task> {
            if epic_values.len() > 1 {
                tasks.retain(|t| {
                    t.parent_id
                        .as_deref()
                        .is_some_and(|pid| epic_values.contains(&pid.to_string()))
                });
            }
            if priority_values.len() > 1 {
                tasks.retain(|t| priority_values.contains(&t.priority));
            }
            tasks
        };

        let open_raw_filtered = post_filter(open_raw_filtered);
        let in_progress_raw = post_filter(in_progress_raw);
        let blocked_raw = post_filter(blocked_raw);
        let done_raw = post_filter(done_raw);

        // Batch-load all unique parent epics across all columns
        let all_tasks_iter = open_raw_filtered
            .iter()
            .chain(in_progress_raw.iter())
            .chain(blocked_raw.iter())
            .chain(done_raw.iter());
        let parent_ids: std::collections::HashSet<String> =
            all_tasks_iter.filter_map(|t| t.parent_id.clone()).collect();
        let parents = fetch_parent_map(&db, parent_ids.into_iter())?;

        let to_rows = |tasks: Vec<Task>| -> Vec<TaskRow> {
            tasks
                .into_iter()
                .map(|t| TaskRow::from_task(t, &parents))
                .collect()
        };

        let open_tasks = to_rows(open_raw_filtered);
        let in_progress_tasks = to_rows(in_progress_raw);
        let blocked_tasks = to_rows(blocked_raw);
        let done_tasks = to_rows(done_raw);

        let selected_epic = epic_filter.clone().unwrap_or_default();
        let selected_priority = priority_filter.clone().unwrap_or_default();
        let poll_query = build_board_poll_query(&epic_filter, &priority_filter);

        Ok(BoardTemplate {
            open_tasks,
            in_progress_tasks,
            blocked_tasks,
            done_tasks,
            epics,
            selected_epic,
            selected_priority,
            poll_query,
        })
    })
    .await
    .unwrap();

    match result {
        Ok(tmpl) => render_template(tmpl),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
    }
}

/// GET /epics — Epics overview with subtask progress.
pub async fn epics(State(state): State<AppState>) -> Response {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<EpicRow>, String> {
        let db = db.lock().unwrap();
        let epic_tasks = db.list_tasks(true, None, None, Some("epic"), None, None)?;
        let mut rows = Vec::with_capacity(epic_tasks.len());
        for task in epic_tasks {
            let children = db.get_children(&task.id)?;
            let children_total = children.len();
            let children_done = children
                .iter()
                .filter(|c| matches!(c.status, crate::models::Status::Done))
                .count();
            rows.push(EpicRow {
                task,
                children_total,
                children_done,
            });
        }
        Ok(rows)
    })
    .await
    .unwrap();

    match result {
        Ok(epics) => render_template(EpicsTemplate { epics }),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
    }
}

/// GET /epics/:id — Epic detail page with children task list (200 or 404).
pub async fn epic_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EpicDetailQuery>,
) -> Response {
    // Normalise view param: accept "board", default to "list" for anything else.
    let view = match query.view.as_deref() {
        Some("board") => "board".to_string(),
        _ => "list".to_string(),
    };
    let view_clone = view.clone();

    let db = state.db.clone();
    let result =
        tokio::task::spawn_blocking(move || -> Result<Option<EpicDetailTemplate>, String> {
            let db = db.lock().unwrap();
            let task = match db.get_task(&id)? {
                Some(t) => t,
                None => return Ok(None),
            };
            let mut children = db.get_children(&id)?;
            // Sort children by the numeric suffix of their hierarchical ID (e.g. `tk-xxxx.N`)
            // so that ordering is 1, 2, 3 … 10, 11 rather than lexicographic 1, 10, 11 … 2.
            children.sort_by_key(|c| {
                c.id.rfind('.')
                    .and_then(|pos| c.id[pos + 1..].parse::<u64>().ok())
                    .unwrap_or(0)
            });
            let children_total = children.len();
            let children_done = children
                .iter()
                .filter(|c| matches!(c.status, crate::models::Status::Done))
                .count();
            Ok(Some(EpicDetailTemplate {
                task,
                children,
                children_done,
                children_total,
                view: view_clone,
            }))
        })
        .await
        .unwrap();

    match result {
        Ok(Some(tmpl)) => render_template(tmpl),
        Ok(None) => (StatusCode::NOT_FOUND, "epic not found").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("database error: {e}"),
        )
            .into_response(),
    }
}

/// GET /api/tags — Unique tag names sorted by usage count descending (200).
pub async fn api_tags(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let tags: Vec<String> = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.task_count_by_tag()
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?
    .into_iter()
    .map(|(tag, _count)| tag)
    .collect();

    Ok(Json(tags))
}

/// GET /api/stats — Task statistics (200).
pub async fn api_stats(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<StatsResponse, String> {
        let db = db.lock().unwrap();

        let by_status_vec = db.task_count_by_status()?;
        let by_priority_vec = db.task_count_by_priority()?;
        let by_tag_vec = db.task_count_by_tag()?;

        let by_status: Map<String, Value> = by_status_vec
            .into_iter()
            .map(|(k, v)| (k, Value::Number(v.into())))
            .collect();

        let by_priority: Map<String, Value> = by_priority_vec
            .into_iter()
            .map(|(k, v)| (k.to_string(), Value::Number(v.into())))
            .collect();

        let by_tag: Map<String, Value> = by_tag_vec
            .into_iter()
            .map(|(k, v)| (k, Value::Number(v.into())))
            .collect();

        Ok(StatsResponse {
            by_status,
            by_priority,
            by_tag,
        })
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(AppError::Internal)?;

    Ok(Json(result))
}
