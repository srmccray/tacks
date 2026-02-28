use askama::Template;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
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

/// Deserialize an optional numeric field that may arrive as an empty string.
fn deserialize_optional_u8<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => v.parse::<u8>().map(Some).map_err(serde::de::Error::custom),
    }
}

/// Query parameters for GET /api/tasks.
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_u8")]
    pub priority: Option<u8>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub tag: Option<String>,
    pub all: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub parent: Option<String>,
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

/// GET /api/tasks — List tasks with optional filters (200).
pub async fn api_list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, AppError> {
    let show_all = query.all.unwrap_or(false);
    let status_filter = query.status.clone();
    let priority_filter = query.priority;
    let tag_filter = query.tag.clone();
    let parent_filter = query.parent.clone();

    let db = state.db.clone();
    let tasks = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.list_tasks(
            show_all,
            status_filter.as_deref(),
            priority_filter,
            tag_filter.as_deref(),
            parent_filter.as_deref(),
        )
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
            let epics = db.list_tasks(true, None, None, Some("epic"), None)?;
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

        let in_progress = db.list_tasks(false, Some("in_progress"), None, None, None)?;
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

/// Template for the task list page at GET /tasks.
#[derive(Template)]
#[template(path = "tasks_list.html")]
struct TaskListTemplate {
    tasks: Vec<Task>,
    status_filter: Option<String>,
    priority_filter: Option<String>,
    tag_filter: Option<String>,
    /// Pre-built query string for HTMX polling (preserves current filters).
    poll_query: String,
}

/// Template for the task detail page at GET /tasks/:id.
#[derive(Template)]
#[template(path = "task_detail.html")]
struct TaskDetailTemplate {
    task: Task,
    blockers: Vec<Task>,
    dependents: Vec<Task>,
    comments: Vec<Comment>,
}

/// Template for the task detail modal fragment loaded via HTMX.
#[derive(Template)]
#[template(path = "task_detail_fragment.html")]
struct TaskDetailFragmentTemplate {
    task: Task,
    blockers: Vec<Task>,
    dependents: Vec<Task>,
    comments: Vec<Comment>,
}

/// Template for the kanban board page at GET /board.
#[derive(Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    open_tasks: Vec<Task>,
    in_progress_tasks: Vec<Task>,
    blocked_tasks: Vec<Task>,
    done_tasks: Vec<Task>,
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

/// Template for the create task form at GET /tasks/new.
#[derive(Template)]
#[template(path = "task_new.html")]
struct TaskNewTemplate;

/// Build a query string from current filter params for HTMX polling.
fn build_poll_query(status: &Option<String>, priority: Option<u8>, tag: &Option<String>) -> String {
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
    let has_filter = params.status.is_some() || params.priority.is_some() || params.tag.is_some();
    let show_all = has_filter || params.all.unwrap_or(false);
    let status_filter = params.status.clone();
    let priority_filter = params.priority;
    let tag_filter = params.tag.clone();

    let db = state.db.clone();
    let tasks = tokio::task::spawn_blocking(move || {
        let db = db.lock().unwrap();
        db.list_tasks(
            show_all,
            status_filter.as_deref(),
            priority_filter,
            tag_filter.as_deref(),
            None,
        )
    })
    .await
    .unwrap()
    .unwrap_or_default();

    let poll_query = build_poll_query(&params.status, params.priority, &params.tag);

    render_template(TaskListTemplate {
        tasks,
        status_filter: params.status,
        priority_filter: params.priority.map(|p| p.to_string()),
        tag_filter: params.tag,
        poll_query,
    })
}

/// GET /tasks/new — Create task form.
pub async fn task_new() -> Response {
    render_template(TaskNewTemplate)
}

/// All data needed to render a task detail view (full page or modal fragment).
struct TaskDetailData {
    task: Task,
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
                    blockers: data.blockers,
                    dependents: data.dependents,
                    comments: data.comments,
                })
            } else {
                render_template(TaskDetailTemplate {
                    task: data.task,
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

/// GET /board — Kanban board view grouped by status.
pub async fn board(State(state): State<AppState>) -> Response {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<BoardTemplate, String> {
        let db = db.lock().unwrap();
        let open_tasks = db.list_tasks(false, Some("open"), None, None, None)?;
        let in_progress_tasks = db.list_tasks(false, Some("in_progress"), None, None, None)?;
        let blocked_tasks = db.list_tasks(false, Some("blocked"), None, None, None)?;
        let done_tasks = db.list_tasks(true, Some("done"), None, None, None)?;
        Ok(BoardTemplate {
            open_tasks,
            in_progress_tasks,
            blocked_tasks,
            done_tasks,
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
        let epic_tasks = db.list_tasks(true, None, None, Some("epic"), None)?;
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
