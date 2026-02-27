use crate::db::Database;
use axum::{
    Router,
    extract::Path as AxumPath,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::Embed;
use std::sync::{Arc, Mutex};

/// Shared application state for the web server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}

mod errors;
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
        .route("/", get(handlers::index))
        .route("/static/{*path}", get(static_handler))
        .with_state(state)
}

/// Start the web server on the given port.
pub async fn serve(db_path: &std::path::Path, port: u16) -> Result<(), String> {
    let db = Database::open(db_path)?;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
    };
    let app = create_router(state);
    let addr = format!("127.0.0.1:{port}");
    println!("Tacks web UI: http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("failed to bind to {addr}: {e}"))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("server error: {e}"))
}
