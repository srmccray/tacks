#![allow(deprecated)]
use cucumber::{then, when};
use serde_json::Value;

use crate::TacksWorld;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `tk` with the given args against the world's database.
/// Stores stdout, stderr, and exit code on the world.
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

/// Run `tk create` with `--json` and store the returned task ID in the world
/// under the given alias.
fn create_task_with_alias(world: &mut TacksWorld, alias: &str, extra_args: &[&str]) {
    let db_path = world.db_path.as_ref().expect("db_path not set").clone();

    let mut cmd_args: Vec<&str> = vec!["--json", "create"];
    cmd_args.extend_from_slice(extra_args);

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", &db_path)
        .args(&cmd_args)
        .output()
        .expect("failed to run tk create");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code().unwrap_or(-1);

    assert!(
        output.status.success(),
        "tk create failed (exit {exit_code}): {stderr}"
    );

    let json: Value = serde_json::from_str(&stdout).expect("create output is not valid JSON");
    let id = json["id"]
        .as_str()
        .expect("create JSON has no 'id' field")
        .to_string();

    world.task_ids.insert(alias.to_string(), id);
    world.last_stdout = stdout;
    world.last_stderr = stderr;
    world.last_exit_code = exit_code;
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(expr = "I create a task with title {string}")]
async fn i_create_a_task_with_title(world: &mut TacksWorld, title: String) {
    create_task_with_alias(world, "last", &[&title]);
}

#[when(expr = "I create a task with title {string} and priority {int} and tags {string}")]
async fn i_create_a_task_with_title_priority_tags(
    world: &mut TacksWorld,
    title: String,
    priority: i64,
    tags: String,
) {
    let priority_str = priority.to_string();
    create_task_with_alias(world, "last", &[&title, "-p", &priority_str, "-t", &tags]);
}

#[when("I show the task")]
async fn i_show_the_task(world: &mut TacksWorld) {
    let id = world
        .task_ids
        .get("last")
        .expect("no 'last' task id — create a task first")
        .clone();
    run_tk(world, &["--json", "show", &id]);
}

#[when("I close the task")]
async fn i_close_the_task(world: &mut TacksWorld) {
    let id = world
        .task_ids
        .get("last")
        .expect("no 'last' task id — create a task first")
        .clone();
    run_tk(world, &["--json", "close", &id]);
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(expr = "the task list contains {string}")]
async fn the_task_list_contains(world: &mut TacksWorld, expected_title: String) {
    run_tk(world, &["--json", "list"]);

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
        "expected to find task '{}' in list, but got: {}",
        expected_title,
        serde_json::to_string_pretty(tasks).unwrap_or_default()
    );
}

#[then(expr = "the task list does not contain {string}")]
async fn the_task_list_does_not_contain(world: &mut TacksWorld, expected_title: String) {
    run_tk(world, &["--json", "list"]);

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
        "expected task '{}' to be absent from list, but it was present",
        expected_title
    );
}

#[then(expr = "the task details show title {string}")]
async fn the_task_details_show_title(world: &mut TacksWorld, expected_title: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");

    let actual = json["title"].as_str().unwrap_or("");
    assert_eq!(
        actual, expected_title,
        "expected title '{}' but got '{}'",
        expected_title, actual
    );
}

#[then(expr = "the task details show status {string}")]
async fn the_task_details_show_status(world: &mut TacksWorld, expected_status: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");

    let actual = json["status"].as_str().unwrap_or("");
    assert_eq!(
        actual, expected_status,
        "expected status '{}' but got '{}'",
        expected_status, actual
    );
}

#[then(expr = "the task details show priority {int}")]
async fn the_task_details_show_priority(world: &mut TacksWorld, expected_priority: i64) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");

    let actual = json["priority"].as_i64().unwrap_or(-1);
    assert_eq!(
        actual, expected_priority,
        "expected priority {} but got {}",
        expected_priority, actual
    );
}
