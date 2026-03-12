#![allow(deprecated)]
use cucumber::{given, then, when};
use serde_json::Value;

use crate::TacksWorld;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn create_subtask_with_alias(world: &mut TacksWorld, parent_alias: &str, alias: &str, title: &str) {
    let parent_id = world
        .task_ids
        .get(parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    let db_path = world.db_path.as_ref().expect("db_path not set").clone();

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", &db_path)
        .args(["--json", "create", title, "--parent", &parent_id])
        .output()
        .expect("failed to run tk create");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "tk create subtask failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_str(&stdout).expect("create output is not valid JSON");
    let id = json["id"]
        .as_str()
        .expect("create JSON has no 'id' field")
        .to_string();

    world.task_ids.insert(alias.to_string(), id);
    world.last_stdout = stdout;
    world.last_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    world.last_exit_code = output.status.code().unwrap_or(-1);
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given(expr = "I have a subtask of {string} called {string} with title {string}")]
async fn i_have_a_subtask_of(
    world: &mut TacksWorld,
    parent_alias: String,
    alias: String,
    title: String,
) {
    create_subtask_with_alias(world, &parent_alias, &alias, &title);
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(expr = "I reparent {string} under {string}")]
async fn i_reparent_under(world: &mut TacksWorld, task_alias: String, parent_alias: String) {
    let task_id = world
        .task_ids
        .get(&task_alias)
        .unwrap_or_else(|| panic!("no task with alias '{task_alias}'"))
        .clone();

    let parent_value = if parent_alias == "none" {
        "none".to_string()
    } else {
        world
            .task_ids
            .get(&parent_alias)
            .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
            .clone()
    };

    run_tk(world, &["update", &task_id, "--parent", &parent_value]);
    assert_eq!(
        world.last_exit_code, 0,
        "reparent failed: {}",
        world.last_stderr
    );
}

#[when(expr = "I try to reparent {string} under {string}")]
async fn i_try_to_reparent_under(world: &mut TacksWorld, task_alias: String, parent_alias: String) {
    let task_id = world
        .task_ids
        .get(&task_alias)
        .unwrap_or_else(|| panic!("no task with alias '{task_alias}'"))
        .clone();

    let parent_value = if parent_alias == "none" {
        "none".to_string()
    } else {
        world
            .task_ids
            .get(&parent_alias)
            .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
            .clone()
    };

    run_tk(world, &["update", &task_id, "--parent", &parent_value]);
}

#[when(expr = "I run tk update for {string} with --parent {string}")]
async fn i_run_tk_update_with_raw_parent(
    world: &mut TacksWorld,
    task_alias: String,
    raw_parent: String,
) {
    let task_id = world
        .task_ids
        .get(&task_alias)
        .unwrap_or_else(|| panic!("no task with alias '{task_alias}'"))
        .clone();

    run_tk(world, &["update", &task_id, "--parent", &raw_parent]);
}

#[when(expr = "I store the ID of {string}")]
async fn i_store_the_id_of(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    // Store in a dedicated field on the world
    world.stored_created_at = Some(id); // Reuse this field to store the ID
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(expr = "the JSON field {string} equals the ID of {string}")]
async fn the_json_field_equals_id_of(world: &mut TacksWorld, field: String, alias: String) {
    let expected_id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let actual = json[&field].as_str().unwrap_or("");
    assert_eq!(
        actual, expected_id,
        "expected {} '{}' but got '{}'",
        field, expected_id, actual
    );
}

#[then(expr = "the JSON field {string} is null")]
async fn the_json_field_is_null(world: &mut TacksWorld, field: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    assert!(
        json[&field].is_null(),
        "expected {} to be null but got: {}",
        field,
        json[&field]
    );
}

#[then("the task ID matches the stored ID")]
async fn the_task_id_matches_stored(world: &mut TacksWorld) {
    let stored_id = world
        .stored_created_at
        .as_ref()
        .expect("no stored ID — use 'When I store the ID of ...' first")
        .clone();

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let actual = json["id"].as_str().unwrap_or("");
    assert_eq!(
        actual, stored_id,
        "expected task ID '{}' but got '{}' — task ID changed during reparent",
        stored_id, actual
    );
}
