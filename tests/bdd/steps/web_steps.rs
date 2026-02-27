#![allow(deprecated)]
use cucumber::{given, then, when};

use crate::TacksWorld;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Start an in-process axum test server using the world's temp database.
/// Binds to a random free port (port 0), stores the port and task handle
/// in the world for later use and cleanup.
pub async fn start_test_server(world: &mut TacksWorld) -> u16 {
    let db_path = world
        .db_path
        .as_ref()
        .expect("db_path not set — did you forget 'Given a tacks database is initialized'?")
        .clone();

    let db = tacks::db::Database::open(&db_path).expect("failed to open database for web server");
    let state = tacks::web::AppState {
        db: std::sync::Arc::new(std::sync::Mutex::new(db)),
        last_data_version: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(0)),
    };
    let app = tacks::web::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind to ephemeral port");
    let port = listener
        .local_addr()
        .expect("failed to get local addr")
        .port();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("web server error in test");
    });

    world.server_port = Some(port);
    world.server_handle = Some(handle);

    // Brief poll to ensure the server is accepting connections before the
    // scenario's When/Then steps run.  We try up to 20 times (100 ms total).
    for _ in 0..20 {
        if world
            .http_client
            .get(format!("http://127.0.0.1:{port}/"))
            .send()
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    port
}

/// Perform a GET request against the running test server and store the status
/// code, content-type header, and body on the world.  Panics if the server
/// port is not set.
pub async fn http_get(world: &mut TacksWorld, path: &str) -> (u16, String) {
    let port = world
        .server_port
        .expect("server not started — add 'Given the web server is running'");
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = world
        .http_client
        .get(&url)
        .send()
        .await
        .unwrap_or_else(|e| panic!("GET {url} failed: {e}"));
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = resp
        .text()
        .await
        .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
    world.last_response_status = Some(status);
    world.last_response_content_type = content_type;
    world.last_response_body = Some(body.clone());
    (status, body)
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

/// Start the in-process web server backed by the world's temp database.
#[given("the web server is running")]
async fn the_web_server_is_running(world: &mut TacksWorld) {
    start_test_server(world).await;
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

/// Perform a GET request to `path` on the test server and store the response.
#[when(expr = "I GET {string}")]
async fn i_get_path(world: &mut TacksWorld, path: String) {
    http_get(world, &path).await;
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

/// Assert that the most recent HTTP response had the given status code.
#[then(expr = "the response status is {int}")]
async fn the_response_status_is(world: &mut TacksWorld, expected: u16) {
    let actual = world
        .last_response_status
        .expect("no HTTP response recorded — did you make a request?");
    assert_eq!(
        actual, expected,
        "expected HTTP status {expected} but got {actual}"
    );
}

/// Assert that the most recent HTTP response body contains the given substring.
#[then(expr = "the response body contains {string}")]
async fn the_response_body_contains(world: &mut TacksWorld, expected: String) {
    let body = world
        .last_response_body
        .as_deref()
        .expect("no HTTP response body recorded — did you make a request?");
    assert!(
        body.contains(&expected),
        "expected response body to contain {expected:?}, but body was:\n{body}"
    );
}

/// Assert that the most recent HTTP response has a Content-Type header
/// containing the given value (partial match, e.g. "text/html").
#[then(expr = "the response content type is {string}")]
async fn the_response_content_type_is(world: &mut TacksWorld, expected: String) {
    let actual = world
        .last_response_content_type
        .as_deref()
        .unwrap_or("<no content-type header>");
    assert!(
        actual.contains(&expected),
        "expected Content-Type to contain {expected:?} but got {actual:?}"
    );
}

/// Perform a GET request to the HTML detail page for a task identified by
/// alias (e.g. GET /tasks/:id).
#[when(expr = "I GET the HTML task {string}")]
async fn i_get_the_html_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(world, &format!("/tasks/{id}")).await;
}
