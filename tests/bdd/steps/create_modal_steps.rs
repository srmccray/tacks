#![allow(deprecated)]
use cucumber::{then, when};
use serde_json::{Value, json};

use crate::TacksWorld;
use crate::steps::web_api_steps::http_post;
use crate::steps::web_steps::http_get;

// ---------------------------------------------------------------------------
// When steps — modal form submission
// ---------------------------------------------------------------------------

/// POST the modal create endpoint with a title and priority.
///
/// The modal form POSTs to the JSON API at POST /api/tasks, which returns 201
/// on success.  This helper covers the happy-path scenario where all required
/// fields are provided.
#[when(expr = "I POST the modal create form with title {string} and priority {int}")]
async fn i_post_modal_create_form_with_title_and_priority(
    world: &mut TacksWorld,
    title: String,
    priority: i64,
) {
    http_post(
        world,
        "/api/tasks",
        json!({"title": title, "priority": priority}),
    )
    .await;
}

/// POST the modal create endpoint with an empty title to trigger validation
/// failure.  Sends an empty string as title so the server returns 422.
#[when("I POST the modal create form with an empty title")]
async fn i_post_modal_create_form_with_empty_title(world: &mut TacksWorld) {
    http_post(world, "/api/tasks", json!({"title": ""})).await;
}

/// POST the modal create endpoint with a title and parent_id resolved from
/// `alias`.  The child task is stored in the world under the alias "modal-child".
#[when(expr = "I POST the modal create form with title {string} under {string}")]
async fn i_post_modal_create_form_with_parent(
    world: &mut TacksWorld,
    title: String,
    parent_alias: String,
) {
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    let (status, body_text) = http_post(
        world,
        "/api/tasks",
        json!({"title": title, "parent_id": parent_id}),
    )
    .await;
    // Store the created task id for later assertions
    if status == 201 {
        if let Ok(json) = serde_json::from_str::<Value>(&body_text) {
            if let Some(id) = json["id"].as_str() {
                world
                    .task_ids
                    .insert("modal-child".to_string(), id.to_string());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Then steps — parent tag assertion
// ---------------------------------------------------------------------------

/// Assert that the task identified by `alias` has the given tag by fetching it
/// via GET /api/tasks/:id and inspecting the tags array.
#[then(expr = "the parent task {string} has tag {string}")]
async fn the_parent_task_has_tag(world: &mut TacksWorld, alias: String, expected_tag: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let path = format!("/api/tasks/{id}");
    let (status, body) = http_get(world, &path).await;
    assert_eq!(
        status, 200,
        "expected 200 from GET {path} but got {status}: {body}"
    );
    let json: Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("GET {path} response is not valid JSON: {e}\n{body}"));
    let tags = json["tags"]
        .as_array()
        .unwrap_or_else(|| panic!("task JSON 'tags' is not an array: {json}"));
    let found = tags
        .iter()
        .any(|t| t.as_str().unwrap_or("") == expected_tag);
    assert!(
        found,
        "expected task '{alias}' tags to include '{expected_tag}' but got: {:?}",
        tags.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>()
    );
}
