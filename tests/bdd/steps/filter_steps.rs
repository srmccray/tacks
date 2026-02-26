#![allow(deprecated)]
use cucumber::{then, when};
use serde_json::Value;

use crate::TacksWorld;

// ---------------------------------------------------------------------------
// Helpers (local to this module)
// ---------------------------------------------------------------------------

/// Run `tk` with the given args against the world's database.
fn run_tk(world: &mut TacksWorld, args: &[&str]) {
    let db_path = world
        .db_path
        .as_ref()
        .expect("db_path not set — did you forget 'Given a tacks database is initialized'?");

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", db_path)
        .args(args)
        .output()
        .expect("failed to run tk");

    world.last_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.last_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    world.last_exit_code = output.status.code().unwrap_or(-1);
}

// ---------------------------------------------------------------------------
// When steps — filtered list variants
// ---------------------------------------------------------------------------

#[when(expr = "I list tasks filtered by status {string}")]
async fn i_list_tasks_filtered_by_status(world: &mut TacksWorld, status: String) {
    run_tk(world, &["--json", "list", "--status", &status]);
}

#[when(expr = "I list tasks filtered by priority {int}")]
async fn i_list_tasks_filtered_by_priority(world: &mut TacksWorld, priority: i64) {
    let priority_str = priority.to_string();
    run_tk(world, &["--json", "list", "--priority", &priority_str]);
}

#[when(expr = "I list tasks filtered by tag {string}")]
async fn i_list_tasks_filtered_by_tag(world: &mut TacksWorld, tag: String) {
    run_tk(world, &["--json", "list", "--tag", &tag]);
}

#[when("I list all tasks including closed")]
async fn i_list_all_tasks_including_closed(world: &mut TacksWorld) {
    run_tk(world, &["--json", "list", "--all"]);
}

#[when("I list tasks with default settings")]
async fn i_list_tasks_with_default_settings(world: &mut TacksWorld) {
    run_tk(world, &["--json", "list"]);
}

// ---------------------------------------------------------------------------
// Then steps — filtered list assertions
// ---------------------------------------------------------------------------

#[then(expr = "the filtered list contains {string}")]
async fn the_filtered_list_contains(world: &mut TacksWorld, expected_title: String) {
    assert_eq!(
        world.last_exit_code, 0,
        "tk list failed: {}",
        world.last_stderr
    );

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("list output is not valid JSON");
    let tasks = json.as_array().expect("list JSON is not an array");

    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);

    assert!(
        found,
        "expected to find '{}' in filtered list, but got: {}",
        expected_title,
        serde_json::to_string_pretty(tasks).unwrap_or_default()
    );
}

#[then(expr = "the filtered list does not contain {string}")]
async fn the_filtered_list_does_not_contain(world: &mut TacksWorld, expected_title: String) {
    assert_eq!(
        world.last_exit_code, 0,
        "tk list failed: {}",
        world.last_stderr
    );

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("list output is not valid JSON");
    let tasks = json.as_array().expect("list JSON is not an array");

    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);

    assert!(
        !found,
        "expected '{}' to be absent from filtered list, but it was present",
        expected_title
    );
}
