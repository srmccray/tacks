#![allow(deprecated)]
use cucumber::{given, then, when};
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
// When steps — update / claim
// ---------------------------------------------------------------------------

#[when(expr = "I claim the task {string}")]
async fn i_claim_the_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["--json", "update", &id, "--claim"]);
}

// ---------------------------------------------------------------------------
// When steps — comment
// ---------------------------------------------------------------------------

#[when(expr = "I add a comment {string} to the task {string}")]
async fn i_add_a_comment_to_task(world: &mut TacksWorld, body: String, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["comment", &id, &body]);
}

#[when(expr = "I show the task {string}")]
async fn i_show_the_named_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["--json", "show", &id]);
}

// ---------------------------------------------------------------------------
// Steps — close (named alias).
// Registered as both `given` and `when` so it works after either keyword.
// ---------------------------------------------------------------------------

#[given(expr = "I close the task {string}")]
async fn given_i_close_the_named_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["close", &id]);
}

#[when(expr = "I close the task {string}")]
async fn when_i_close_the_named_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["close", &id]);
}

// ---------------------------------------------------------------------------
// When steps — stats
// ---------------------------------------------------------------------------

#[when("I run tk stats with json output")]
async fn i_run_tk_stats_json(world: &mut TacksWorld) {
    run_tk(world, &["--json", "stats"]);
}

#[when("I run tk stats with oneline output")]
async fn i_run_tk_stats_oneline(world: &mut TacksWorld) {
    run_tk(world, &["stats", "--oneline"]);
}

// ---------------------------------------------------------------------------
// When steps — ready
// ---------------------------------------------------------------------------

#[when("I run tk ready with json output")]
async fn i_run_tk_ready_json(world: &mut TacksWorld) {
    run_tk(world, &["--json", "ready"]);
}

#[when(expr = "I run tk ready with limit {int}")]
async fn i_run_tk_ready_with_limit(world: &mut TacksWorld, limit: i64) {
    let limit_str = limit.to_string();
    run_tk(world, &["--json", "ready", "--limit", &limit_str]);
}

// ---------------------------------------------------------------------------
// Then steps — update / claim assertions
// ---------------------------------------------------------------------------

#[then(expr = "the task {string} has status {string}")]
async fn the_task_has_status(world: &mut TacksWorld, alias: String, expected_status: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["--json", "show", &id]);

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("show output is not valid JSON");
    let actual = json["status"].as_str().unwrap_or("");
    assert_eq!(
        actual, expected_status,
        "expected status '{}' but got '{}' for task '{}'",
        expected_status, actual, alias
    );
}

#[then(expr = "the task {string} has assignee {string}")]
async fn the_task_has_assignee(world: &mut TacksWorld, alias: String, expected_assignee: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["--json", "show", &id]);

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("show output is not valid JSON");
    let actual = json["assignee"].as_str().unwrap_or("");
    assert_eq!(
        actual, expected_assignee,
        "expected assignee '{}' but got '{}' for task '{}'",
        expected_assignee, actual, alias
    );
}

// ---------------------------------------------------------------------------
// Then steps — comment assertions
// ---------------------------------------------------------------------------

#[then(expr = "the task details show a comment with body {string}")]
async fn the_task_details_show_comment(world: &mut TacksWorld, expected_body: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");

    let comments = json["comments"]
        .as_array()
        .expect("show JSON has no 'comments' array");

    let found = comments
        .iter()
        .any(|c| c["body"].as_str().unwrap_or("") == expected_body);

    assert!(
        found,
        "expected comment '{}' in task details, but got comments: {}",
        expected_body,
        serde_json::to_string_pretty(comments).unwrap_or_default()
    );
}

// ---------------------------------------------------------------------------
// Then steps — stats assertions
// ---------------------------------------------------------------------------

#[then(expr = "the stats JSON shows {string} count of {int}")]
async fn the_stats_json_shows_count(world: &mut TacksWorld, status: String, expected: i64) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("stats output is not valid JSON");

    let count = json["by_status"][&status].as_i64().unwrap_or(0);
    assert_eq!(
        count, expected,
        "expected by_status[\"{}\"] == {} but got {}",
        status, expected, count
    );
}

#[then(expr = "the stats JSON has a {string} field")]
async fn the_stats_json_has_field(world: &mut TacksWorld, field: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("stats output is not valid JSON");

    assert!(
        json.get(&field).is_some(),
        "expected stats JSON to have field '{}' but got: {}",
        field,
        world.last_stdout
    );
}

#[then(expr = "the oneline output contains {string}")]
async fn the_oneline_output_contains(world: &mut TacksWorld, expected: String) {
    assert!(
        world.last_stdout.contains(&expected),
        "expected oneline output to contain '{}' but got: {}",
        expected,
        world.last_stdout
    );
}

// ---------------------------------------------------------------------------
// Then steps — ready list size
// ---------------------------------------------------------------------------

#[then(expr = "the ready list contains exactly {int} task")]
async fn the_ready_list_contains_exactly(world: &mut TacksWorld, expected_count: i64) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("ready output is not valid JSON");
    let tasks = json.as_array().expect("ready JSON is not an array");

    assert_eq!(
        tasks.len() as i64,
        expected_count,
        "expected {} task(s) in ready list but got {}",
        expected_count,
        tasks.len()
    );
}

#[then("the ready list is empty")]
async fn the_ready_list_is_empty(world: &mut TacksWorld) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("ready output is not valid JSON");
    let tasks = json.as_array().expect("ready JSON is not an array");

    assert!(
        tasks.is_empty(),
        "expected empty ready list but got {} task(s): {}",
        tasks.len(),
        serde_json::to_string_pretty(tasks).unwrap_or_default()
    );
}
