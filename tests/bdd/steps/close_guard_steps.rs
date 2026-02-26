#![allow(deprecated)]
use cucumber::when;

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

// ---------------------------------------------------------------------------
// When steps — close guard
// ---------------------------------------------------------------------------

#[when(expr = "I try to close task {string}")]
async fn i_try_to_close_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["close", &id]);
}

#[when(expr = "I force close task {string}")]
async fn i_force_close_task(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["close", &id, "--force"]);
}

#[when(expr = "I force close subtask {string}")]
async fn i_force_close_subtask(world: &mut TacksWorld, title: String) {
    // Look up the subtask by title — scan the full list since subtasks are stored
    // under "last_subtask" alias and we need to find by title.
    let db_path = world.db_path.as_ref().expect("db_path not set").clone();

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", &db_path)
        .args(["--json", "list", "-a"])
        .output()
        .expect("failed to run tk list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tasks: serde_json::Value =
        serde_json::from_str(&stdout).expect("tk list --json output is not valid JSON");

    let task_id = tasks
        .as_array()
        .expect("tk list --json is not an array")
        .iter()
        .find(|t| t["title"].as_str().unwrap_or("") == title)
        .unwrap_or_else(|| panic!("task with title '{}' not found in list", title))["id"]
        .as_str()
        .expect("task has no 'id' field")
        .to_string();

    run_tk(world, &["close", &task_id, "--force"]);
}
