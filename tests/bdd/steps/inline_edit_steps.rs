#![allow(deprecated)]
use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::TacksWorld;
use crate::steps::web_api_steps::{http_patch, http_post};
use crate::steps::web_steps::http_get;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Fetch the current state of `world.last_task_id` via GET /api/tasks/:id
/// and return the parsed JSON body.
async fn fetch_task(world: &mut TacksWorld) -> Value {
    let id = world
        .last_task_id
        .clone()
        .expect("no task id stored — did you use 'Given a task ... exists via API'?");
    let path = format!("/api/tasks/{id}");
    let (status, body) = http_get(world, &path).await;
    assert_eq!(
        status, 200,
        "expected 200 from GET {path} but got {status}: {body}"
    );
    serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("GET {path} response is not valid JSON: {e}\n{body}"))
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

/// Create a task with the given title, store its id in `world.last_task_id`.
#[given(expr = "a task {string} exists via API")]
async fn a_task_exists_via_api(world: &mut TacksWorld, title: String) {
    let (status, body_text) = http_post(world, "/api/tasks", json!({"title": title})).await;
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
    world.last_task_id = Some(id);
}

/// Capture the task's current created_at timestamp so a later step can assert
/// it did not change after a PATCH.
#[given("the task created_at is stored")]
async fn the_task_created_at_is_stored(world: &mut TacksWorld) {
    let task = fetch_task(world).await;
    let created_at = task["created_at"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'created_at' field: {task}"))
        .to_string();
    world.stored_created_at = Some(created_at);
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

/// PATCH the task stored in `world.last_task_id` with the given raw JSON body.
#[when(expr = "I PATCH the task with {string}")]
async fn i_patch_the_task_with(world: &mut TacksWorld, raw_body: String) {
    let id = world
        .last_task_id
        .clone()
        .expect("no task id stored — did you use 'Given a task ... exists via API'?");
    let body: Value = serde_json::from_str(&raw_body)
        .unwrap_or_else(|e| panic!("step body {raw_body:?} is not valid JSON: {e}"));
    let path = format!("/api/tasks/{id}");
    let (status, response_body) = http_patch(world, &path, body).await;
    assert_eq!(
        status, 200,
        "expected 200 from PATCH {path} but got {status}: {response_body}"
    );
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

/// Fetch the task and assert its title field equals the expected value.
#[then(expr = "the task title should be {string}")]
async fn the_task_title_should_be(world: &mut TacksWorld, expected: String) {
    let task = fetch_task(world).await;
    let actual = task["title"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'title' field: {task}"));
    assert_eq!(
        actual, expected,
        "expected task title to be '{expected}' but got '{actual}'"
    );
}

/// Fetch the task and assert its status field equals the expected value.
#[then(expr = "the task status should be {string}")]
async fn the_task_status_should_be(world: &mut TacksWorld, expected: String) {
    let task = fetch_task(world).await;
    let actual = task["status"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'status' field: {task}"));
    assert_eq!(
        actual, expected,
        "expected task status to be '{expected}' but got '{actual}'"
    );
}

/// Fetch the task and assert its priority field equals the expected integer.
#[then(expr = "the task priority should be {int}")]
async fn the_task_priority_should_be(world: &mut TacksWorld, expected: i64) {
    let task = fetch_task(world).await;
    let actual = task["priority"]
        .as_i64()
        .unwrap_or_else(|| panic!("task JSON has no numeric 'priority' field: {task}"));
    assert_eq!(
        actual, expected,
        "expected task priority to be {expected} but got {actual}"
    );
}

/// Fetch the task and assert its assignee field equals the expected value.
#[then(expr = "the task assignee should be {string}")]
async fn the_task_assignee_should_be(world: &mut TacksWorld, expected: String) {
    let task = fetch_task(world).await;
    let actual = task["assignee"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'assignee' field: {task}"));
    assert_eq!(
        actual, expected,
        "expected task assignee to be '{expected}' but got '{actual}'"
    );
}

/// Fetch the task and assert that its tags array includes the given tag.
#[then(expr = "the task tags should include {string}")]
async fn the_task_tags_should_include(world: &mut TacksWorld, expected: String) {
    let task = fetch_task(world).await;
    let tags = task["tags"]
        .as_array()
        .unwrap_or_else(|| panic!("task JSON 'tags' is not an array: {task}"));
    let found = tags.iter().any(|t| t.as_str().unwrap_or("") == expected);
    assert!(
        found,
        "expected task tags to include '{expected}' but got: {:?}",
        tags.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>()
    );
}

/// Fetch the task and assert its description field equals the expected value.
#[then(expr = "the task description should be {string}")]
async fn the_task_description_should_be(world: &mut TacksWorld, expected: String) {
    let task = fetch_task(world).await;
    let actual = task["description"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'description' field: {task}"));
    assert_eq!(
        actual, expected,
        "expected task description to be '{expected}' but got '{actual}'"
    );
}

/// Fetch the task and assert its created_at matches the value stored before
/// the PATCH was performed.
#[then("the task created_at should not change")]
async fn the_task_created_at_should_not_change(world: &mut TacksWorld) {
    let stored = world
        .stored_created_at
        .clone()
        .expect("created_at not stored — did you use 'Given the task created_at is stored'?");
    let task = fetch_task(world).await;
    let actual = task["created_at"]
        .as_str()
        .unwrap_or_else(|| panic!("task JSON has no 'created_at' field: {task}"));
    assert_eq!(
        actual, stored,
        "expected task created_at to remain '{stored}' but got '{actual}'"
    );
}
