use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

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
