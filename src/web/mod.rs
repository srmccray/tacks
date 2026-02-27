use crate::db::Database;
use axum::{Router, routing::get};
use std::sync::{Arc, Mutex};

/// Shared application state for the web server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
}

mod errors;
mod handlers;

/// Build the axum router with all routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::index))
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
