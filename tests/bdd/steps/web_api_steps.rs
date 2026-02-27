#![allow(deprecated)]
use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::TacksWorld;
use crate::steps::web_steps::http_get;

// ---------------------------------------------------------------------------
// HTTP helper functions
// ---------------------------------------------------------------------------

/// Perform a POST request with a JSON body against the running test server.
/// Stores the status code and body on the world.
pub async fn http_post(world: &mut TacksWorld, path: &str, body: Value) -> (u16, String) {
    let port = world
        .server_port
        .expect("server not started — add 'Given the web server is running'");
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = world
        .http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {url} failed: {e}"));
    let status = resp.status().as_u16();
    let body_text = resp
        .text()
        .await
        .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
    world.last_response_status = Some(status);
    world.last_response_body = Some(body_text.clone());
    (status, body_text)
}

/// Perform a PATCH request with a JSON body against the running test server.
/// Stores the status code and body on the world.
pub async fn http_patch(world: &mut TacksWorld, path: &str, body: Value) -> (u16, String) {
    let port = world
        .server_port
        .expect("server not started — add 'Given the web server is running'");
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = world
        .http_client
        .patch(&url)
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("PATCH {url} failed: {e}"));
    let status = resp.status().as_u16();
    let body_text = resp
        .text()
        .await
        .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
    world.last_response_status = Some(status);
    world.last_response_body = Some(body_text.clone());
    (status, body_text)
}

/// Perform a DELETE request against the running test server.
/// Stores the status code and body on the world.
pub async fn http_delete(world: &mut TacksWorld, path: &str) -> (u16, String) {
    let port = world
        .server_port
        .expect("server not started — add 'Given the web server is running'");
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = world
        .http_client
        .delete(&url)
        .send()
        .await
        .unwrap_or_else(|e| panic!("DELETE {url} failed: {e}"));
    let status = resp.status().as_u16();
    let body_text = resp
        .text()
        .await
        .unwrap_or_else(|e| panic!("failed to read response body: {e}"));
    world.last_response_status = Some(status);
    world.last_response_body = Some(body_text.clone());
    (status, body_text)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse the last response body as JSON, panicking with a descriptive message
/// if it is not valid JSON.
fn parse_last_response(world: &TacksWorld) -> Value {
    let body = world
        .last_response_body
        .as_deref()
        .expect("no HTTP response body recorded");
    serde_json::from_str(body)
        .unwrap_or_else(|e| panic!("response body is not valid JSON: {e}\nbody: {body}"))
}

/// Create a task via the REST API, store its id under `alias` in the world,
/// and update last_response_status/body.
async fn api_create_task(world: &mut TacksWorld, alias: &str, body: Value) {
    let (status, body_text) = http_post(world, "/api/tasks", body).await;
    assert_eq!(
        status, 201,
        "expected 201 from POST /api/tasks but got {status}: {body_text}"
    );
    let json: Value = serde_json::from_str(&body_text)
        .unwrap_or_else(|e| panic!("POST /api/tasks response is not valid JSON: {e}\n{body_text}"));
    let id = json["id"]
        .as_str()
        .unwrap_or_else(|| panic!("POST /api/tasks response has no 'id' field: {json}"))
        .to_string();
    world.task_ids.insert(alias.to_string(), id);
}

// ---------------------------------------------------------------------------
// Given steps — API task setup
// ---------------------------------------------------------------------------

#[given(expr = "I created a task via API with title {string} as {string}")]
async fn i_created_a_task_via_api(world: &mut TacksWorld, title: String, alias: String) {
    api_create_task(world, &alias, json!({"title": title})).await;
}

#[given(expr = "I created a task via API with title {string} and priority {int} as {string}")]
async fn i_created_a_task_via_api_with_priority(
    world: &mut TacksWorld,
    title: String,
    priority: i64,
    alias: String,
) {
    api_create_task(world, &alias, json!({"title": title, "priority": priority})).await;
}

#[given(expr = "I created a task via API with title {string} and description {string} as {string}")]
async fn i_created_a_task_via_api_with_description(
    world: &mut TacksWorld,
    title: String,
    description: String,
    alias: String,
) {
    api_create_task(
        world,
        &alias,
        json!({"title": title, "description": description}),
    )
    .await;
}

#[given(expr = "I created a task via API with title {string} and tag {string} as {string}")]
async fn i_created_a_task_via_api_with_tag(
    world: &mut TacksWorld,
    title: String,
    tag: String,
    alias: String,
) {
    api_create_task(world, &alias, json!({"title": title, "tags": [tag]})).await;
}

#[given(expr = "I created a subtask via API with title {string} under {string} as {string}")]
async fn i_created_a_subtask_via_api(
    world: &mut TacksWorld,
    title: String,
    parent_alias: String,
    alias: String,
) {
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    api_create_task(
        world,
        &alias,
        json!({"title": title, "parent_id": parent_id}),
    )
    .await;
}

#[given(expr = "I closed the API task {string}")]
async fn i_closed_the_api_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let path = format!("/api/tasks/{id}/close");
    let (status, body) = http_post(world, &path, json!({"reason": "done"})).await;
    assert_eq!(
        status, 200,
        "expected 200 from POST {path} but got {status}: {body}"
    );
}

#[given(expr = "I added API dependency so {string} is blocked by {string}")]
async fn i_added_api_dependency(world: &mut TacksWorld, child_alias: String, parent_alias: String) {
    let child_id = world
        .task_ids
        .get(&child_alias)
        .unwrap_or_else(|| panic!("no task with alias '{child_alias}'"))
        .clone();
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    let path = format!("/api/tasks/{child_id}/deps");
    let (status, body) = http_post(world, &path, json!({"parent_id": parent_id})).await;
    assert_eq!(
        status, 201,
        "expected 201 from POST {path} but got {status}: {body}"
    );
}

#[given(expr = "I posted a comment {string} on API task {string}")]
async fn i_posted_a_comment_on_api_task(
    world: &mut TacksWorld,
    comment_body: String,
    alias: String,
) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let path = format!("/api/tasks/{id}/comments");
    let (status, body) = http_post(world, &path, json!({"body": comment_body})).await;
    assert_eq!(
        status, 201,
        "expected 201 from POST {path} but got {status}: {body}"
    );
}

// ---------------------------------------------------------------------------
// When steps — raw HTTP verbs
// ---------------------------------------------------------------------------

#[when(expr = "I POST {string} with body {string}")]
async fn i_post_path_with_body(world: &mut TacksWorld, path: String, raw_body: String) {
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    http_post(world, &path, body).await;
}

#[when(expr = "I PATCH {string} with body {string}")]
async fn i_patch_path_with_body(world: &mut TacksWorld, path: String, raw_body: String) {
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    http_patch(world, &path, body).await;
}

// ---------------------------------------------------------------------------
// When steps — alias-based HTTP verbs (resolve task alias → id)
// ---------------------------------------------------------------------------

#[when(expr = "I GET the API task {string}")]
async fn i_get_the_api_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(world, &format!("/api/tasks/{id}")).await;
}

#[when(expr = "I PATCH the API task {string} with body {string}")]
async fn i_patch_the_api_task(world: &mut TacksWorld, alias: String, raw_body: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    http_patch(world, &format!("/api/tasks/{id}"), body).await;
}

#[when(expr = "I POST the close endpoint for API task {string} with body {string}")]
async fn i_post_close_endpoint(world: &mut TacksWorld, alias: String, raw_body: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    http_post(world, &format!("/api/tasks/{id}/close"), body).await;
}

#[when(expr = "I POST the deps endpoint for API task {string} with body {string}")]
async fn i_post_deps_endpoint(world: &mut TacksWorld, alias: String, raw_body: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    // raw_body may contain an alias like "dep-parent" for parent_id — resolve it
    let mut body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    // If parent_id is an alias, resolve it to the actual task id
    if let Some(parent_ref) = body.get("parent_id").and_then(|v| v.as_str()) {
        if let Some(resolved_id) = world.task_ids.get(parent_ref).cloned() {
            body["parent_id"] = Value::String(resolved_id);
        }
    }
    http_post(world, &format!("/api/tasks/{id}/deps"), body).await;
}

#[when(expr = "I DELETE the API dependency from {string} to {string}")]
async fn i_delete_the_api_dependency(
    world: &mut TacksWorld,
    child_alias: String,
    parent_alias: String,
) {
    let child_id = world
        .task_ids
        .get(&child_alias)
        .unwrap_or_else(|| panic!("no task with alias '{child_alias}'"))
        .clone();
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    http_delete(world, &format!("/api/tasks/{child_id}/deps/{parent_id}")).await;
}

#[when(expr = "I POST the comments endpoint for API task {string} with body {string}")]
async fn i_post_comments_endpoint(world: &mut TacksWorld, alias: String, raw_body: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    http_post(world, &format!("/api/tasks/{id}/comments"), body).await;
}

#[when(expr = "I GET the comments endpoint for API task {string}")]
async fn i_get_comments_endpoint(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(world, &format!("/api/tasks/{id}/comments")).await;
}

#[when(expr = "I GET the children endpoint for API task {string}")]
async fn i_get_children_endpoint(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(world, &format!("/api/tasks/{id}/children")).await;
}

// ---------------------------------------------------------------------------
// Then steps — JSON response assertions
// ---------------------------------------------------------------------------

/// Assert that the response body JSON has the given top-level field.
#[then(expr = "the response JSON has field {string}")]
async fn the_response_json_has_field(world: &mut TacksWorld, field: String) {
    let json = parse_last_response(world);
    assert!(
        json.get(&field).is_some(),
        "expected response JSON to have field '{field}' but got:\n{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}

/// Assert that the response body JSON has a string field equal to expected.
#[then(expr = "the response JSON field {string} equals {string}")]
async fn the_response_json_field_equals_string(
    world: &mut TacksWorld,
    field: String,
    expected: String,
) {
    let json = parse_last_response(world);
    let actual = json[&field].as_str().unwrap_or_else(|| {
        panic!(
            "expected field '{field}' to be a string in: {}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        )
    });
    assert_eq!(
        actual, expected,
        "expected response JSON field '{field}' to equal '{expected}' but got '{actual}'"
    );
}

/// Assert that the response body JSON has a numeric field equal to expected.
#[then(expr = "the response JSON field {string} equals {int}")]
async fn the_response_json_field_equals_int(world: &mut TacksWorld, field: String, expected: i64) {
    let json = parse_last_response(world);
    let actual = json[&field].as_i64().unwrap_or_else(|| {
        panic!(
            "expected field '{field}' to be a number in: {}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        )
    });
    assert_eq!(
        actual, expected,
        "expected response JSON field '{field}' to equal {expected} but got {actual}"
    );
}

/// Assert that the response body JSON is an array containing a task object
/// with the given title.
#[then(expr = "the response JSON array contains a task with title {string}")]
async fn the_response_json_array_contains_task(world: &mut TacksWorld, title: String) {
    let json = parse_last_response(world);
    let tasks = json
        .as_array()
        .unwrap_or_else(|| panic!("expected response body to be a JSON array but got: {json}"));
    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == title);
    assert!(
        found,
        "expected JSON array to contain a task with title '{title}' but got:\n{}",
        serde_json::to_string_pretty(tasks).unwrap_or_default()
    );
}

/// Assert that the response body JSON array does NOT contain a task with the
/// given title.
#[then(expr = "the response JSON array does not contain a task with title {string}")]
async fn the_response_json_array_not_contains_task(world: &mut TacksWorld, title: String) {
    let json = parse_last_response(world);
    let tasks = json
        .as_array()
        .unwrap_or_else(|| panic!("expected response body to be a JSON array but got: {json}"));
    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == title);
    assert!(
        !found,
        "expected JSON array NOT to contain a task with title '{title}', but it was present"
    );
}

/// Assert that the response body JSON array contains a comment with the given body.
#[then(expr = "the response JSON array contains a comment with body {string}")]
async fn the_response_json_array_contains_comment(world: &mut TacksWorld, body: String) {
    let json = parse_last_response(world);
    let comments = json
        .as_array()
        .unwrap_or_else(|| panic!("expected response body to be a JSON array but got: {json}"));
    let found = comments
        .iter()
        .any(|c| c["body"].as_str().unwrap_or("") == body);
    assert!(
        found,
        "expected JSON array to contain a comment with body '{body}' but got:\n{}",
        serde_json::to_string_pretty(comments).unwrap_or_default()
    );
}

/// Assert that the response body JSON array has exactly the given number of elements.
#[then(expr = "the response JSON array has length {int}")]
async fn the_response_json_array_has_length(world: &mut TacksWorld, expected: i64) {
    let json = parse_last_response(world);
    let arr = json
        .as_array()
        .unwrap_or_else(|| panic!("expected response body to be a JSON array but got: {json}"));
    assert_eq!(
        arr.len() as i64,
        expected,
        "expected JSON array length {expected} but got {}",
        arr.len()
    );
}

/// Assert that the response body JSON is an empty array.
#[then("the response JSON is an empty array")]
async fn the_response_json_is_empty_array(world: &mut TacksWorld) {
    let json = parse_last_response(world);
    let arr = json
        .as_array()
        .unwrap_or_else(|| panic!("expected response body to be a JSON array but got: {json}"));
    assert!(
        arr.is_empty(),
        "expected empty JSON array but got {} element(s):\n{}",
        arr.len(),
        serde_json::to_string_pretty(arr).unwrap_or_default()
    );
}

/// Assert that a nested JSON field (dot-separated path) equals an integer.
/// Example path: "by_status.done"
#[then(expr = "the response JSON nested field {string} equals {int}")]
async fn the_response_json_nested_field_equals_int(
    world: &mut TacksWorld,
    path: String,
    expected: i64,
) {
    let json = parse_last_response(world);
    let mut current = &json;
    for key in path.split('.') {
        current = current.get(key).unwrap_or_else(|| {
            panic!(
                "expected path '{path}' to exist in JSON but '{key}' not found at this level:\n{}",
                serde_json::to_string_pretty(current).unwrap_or_default()
            )
        });
    }
    let actual = current
        .as_i64()
        .unwrap_or_else(|| panic!("expected '{path}' to be a number but got: {current}"));
    assert_eq!(
        actual, expected,
        "expected JSON path '{path}' to equal {expected} but got {actual}"
    );
}
