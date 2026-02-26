#![allow(deprecated)]
use cucumber::{given, then, when};
use serde_json::Value;

use crate::TacksWorld;

// ---------------------------------------------------------------------------
// Helpers (local to this module)
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
// Given steps
// ---------------------------------------------------------------------------

#[given(expr = "I have a task called {string} with title {string}")]
async fn i_have_a_task_called_with_title(world: &mut TacksWorld, alias: String, title: String) {
    create_task_with_alias(world, &alias, &[&title]);
}

#[given(expr = "I have a task called {string} with title {string} and priority {int}")]
async fn i_have_a_task_called_with_title_and_priority(
    world: &mut TacksWorld,
    alias: String,
    title: String,
    priority: i64,
) {
    let priority_str = priority.to_string();
    create_task_with_alias(world, &alias, &[&title, "-p", &priority_str]);
}

#[given(expr = "I have a task called {string} with title {string} and tag {string}")]
async fn i_have_a_task_called_with_title_and_tag(
    world: &mut TacksWorld,
    alias: String,
    title: String,
    tag: String,
) {
    create_task_with_alias(world, &alias, &[&title, "-t", &tag]);
}

// ---------------------------------------------------------------------------
// When steps — dependencies
// ---------------------------------------------------------------------------

#[when(expr = "I add a dependency so {string} is blocked by {string}")]
async fn i_add_a_dependency_so_blocked_by(
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
    run_tk(world, &["dep", "add", &child_id, &parent_id]);
}

#[when(expr = "I remove the dependency so {string} is no longer blocked by {string}")]
async fn i_remove_the_dependency(
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
    run_tk(world, &["dep", "remove", &child_id, &parent_id]);
}

#[when(expr = "I try to add a dependency so {string} is blocked by {string}")]
async fn i_try_to_add_dependency_invalid(
    world: &mut TacksWorld,
    child_alias: String,
    parent_id_literal: String,
) {
    // child_alias is a real task alias; parent_id_literal may be a raw invalid ID
    let child_id = world
        .task_ids
        .get(&child_alias)
        .unwrap_or_else(|| panic!("no task with alias '{child_alias}'"))
        .clone();
    // parent_id_literal may be "tk-0000" — used literally, not looked up as an alias
    run_tk(world, &["dep", "add", &child_id, &parent_id_literal]);
}

#[when(expr = "I try to remove a dependency so {string} is no longer blocked by {string}")]
async fn i_try_to_remove_dependency_invalid(
    world: &mut TacksWorld,
    child_alias: String,
    parent_id_literal: String,
) {
    // child_alias is a real task alias; parent_id_literal may be a raw invalid ID
    let child_id = world
        .task_ids
        .get(&child_alias)
        .unwrap_or_else(|| panic!("no task with alias '{child_alias}'"))
        .clone();
    // parent_id_literal may be "tk-0000" — used literally, not looked up as an alias
    run_tk(world, &["dep", "remove", &child_id, &parent_id_literal]);
}

#[when(expr = "I try to add a self-dependency for {string}")]
async fn i_try_to_add_self_dependency(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["dep", "add", &id, &id]);
}

// ---------------------------------------------------------------------------
// When steps — show
// ---------------------------------------------------------------------------

#[when(expr = "I show task {string} in JSON")]
async fn i_show_task_in_json(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["--json", "show", &id]);
}

// ---------------------------------------------------------------------------
// Then steps — show dependents/blockers
// ---------------------------------------------------------------------------

#[then(expr = "the task details include dependent {string}")]
async fn the_task_details_include_dependent(world: &mut TacksWorld, expected_title: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let dependents = json["dependents"]
        .as_array()
        .expect("no 'dependents' array in show output");
    let found = dependents
        .iter()
        .any(|d| d["title"].as_str().unwrap_or("") == expected_title);
    assert!(
        found,
        "expected dependent '{}' not found in: {:?}",
        expected_title, dependents
    );
}

#[then(expr = "the task details include blocker {string}")]
async fn the_task_details_include_blocker(world: &mut TacksWorld, expected_title: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let blockers = json["blockers"]
        .as_array()
        .expect("no 'blockers' array in show output");
    let found = blockers
        .iter()
        .any(|b| b["title"].as_str().unwrap_or("") == expected_title);
    assert!(
        found,
        "expected blocker '{}' not found in: {:?}",
        expected_title, blockers
    );
}

// ---------------------------------------------------------------------------
// Then steps — ready list
// ---------------------------------------------------------------------------

#[then(expr = "the ready list contains {string}")]
async fn the_ready_list_contains(world: &mut TacksWorld, expected_title: String) {
    run_tk(world, &["--json", "ready"]);

    assert_eq!(
        world.last_exit_code, 0,
        "tk ready failed: {}",
        world.last_stderr
    );

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("ready output is not valid JSON");
    let tasks = json.as_array().expect("ready JSON is not an array");

    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);

    assert!(
        found,
        "expected to find '{}' in ready list, but got: {}",
        expected_title,
        serde_json::to_string_pretty(tasks).unwrap_or_default()
    );
}

#[then(expr = "the ready list does not contain {string}")]
async fn the_ready_list_does_not_contain(world: &mut TacksWorld, expected_title: String) {
    run_tk(world, &["--json", "ready"]);

    assert_eq!(
        world.last_exit_code, 0,
        "tk ready failed: {}",
        world.last_stderr
    );

    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("ready output is not valid JSON");
    let tasks = json.as_array().expect("ready JSON is not an array");

    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);

    assert!(
        !found,
        "expected '{}' to be absent from ready list, but it was present",
        expected_title
    );
}

// ---------------------------------------------------------------------------
// Then steps — command success/failure
// ---------------------------------------------------------------------------

#[then("the command should fail")]
async fn the_command_should_fail(world: &mut TacksWorld) {
    assert_ne!(
        world.last_exit_code, 0,
        "expected command to fail but it succeeded"
    );
}

#[then(expr = "the error output contains {string}")]
async fn the_error_output_contains(world: &mut TacksWorld, expected: String) {
    assert!(
        world.last_stderr.contains(&expected),
        "expected stderr to contain '{}' but got: {}",
        expected,
        world.last_stderr
    );
}
