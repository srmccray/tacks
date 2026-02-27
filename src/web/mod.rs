use crate::db::Database;
use axum::{
    Router,
    extract::Path as AxumPath,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use rust_embed::Embed;
use std::sync::{Arc, Mutex};

/// Shared application state for the web server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}

pub mod errors;
mod handlers;

/// Embedded static assets (htmx, pico CSS, etc.) compiled into the binary.
#[derive(Embed)]
#[folder = "static/"]
struct StaticAssets;

/// Serve embedded static files at /static/{path}.
async fn static_handler(AxumPath(path): AxumPath<String>) -> Response {
    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".css") {
                "text/css"
            } else {
                "application/octet-stream"
            };
            ([(header::CONTENT_TYPE, mime)], content.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Build the axum router with all routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // HTML routes
        .route("/", get(handlers::index))
        .route("/static/{*path}", get(static_handler))
        // API routes — specific routes before parameterized ones
        .route(
            "/api/tasks",
            get(handlers::api_list_tasks).post(handlers::api_create_task),
        )
        .route("/api/tasks/ready", get(handlers::api_ready_tasks))
        .route("/api/tasks/blocked", get(handlers::api_blocked_tasks))
        .route("/api/epics", get(handlers::api_epics))
        .route("/api/prime", get(handlers::api_prime))
        .route(
            "/api/tasks/{id}",
            get(handlers::api_show_task).patch(handlers::api_update_task),
        )
        .route("/api/tasks/{id}/close", post(handlers::api_close_task))
        .route("/api/tasks/{id}/deps", post(handlers::api_add_dep))
        .route(
            "/api/tasks/{child_id}/deps/{parent_id}",
            delete(handlers::api_remove_dep),
        )
        .route(
            "/api/tasks/{id}/comments",
            get(handlers::api_list_comments).post(handlers::api_add_comment),
        )
        .route("/api/tasks/{id}/children", get(handlers::api_children))
        .route("/api/tasks/{id}/blockers", get(handlers::api_blockers))
        .route("/api/tasks/{id}/dependents", get(handlers::api_dependents))
        .route("/api/stats", get(handlers::api_stats))
        .with_state(state)
}

/// Start the web server on the given port, shutting down gracefully on Ctrl+C.
pub async fn serve(db_path: &std::path::Path, port: u16) -> Result<(), String> {
    let db = Database::open(db_path)?;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
    };
    let app = create_router(state);
    let addr = format!("127.0.0.1:{port}");
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("failed to bind to {addr}: {e}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c");
        })
        .await
        .map_err(|e| format!("server error: {e}"))
}
