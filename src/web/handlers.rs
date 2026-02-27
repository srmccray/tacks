use axum::response::Html;

/// Placeholder index handler.
pub async fn index() -> Html<&'static str> {
    Html("<h1>Tacks</h1><p>Web UI coming soon.</p>")
}
