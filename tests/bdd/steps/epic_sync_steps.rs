#![allow(deprecated)]
use cucumber::when;
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

/// Find a task ID by title using `tk list -a --json`.
fn find_task_id_by_title(world: &TacksWorld, title: &str) -> String {
    let db_path = world.db_path.as_ref().expect("db_path not set");

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", db_path)
        .args(["--json", "list", "-a"])
        .output()
        .expect("failed to run tk list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tasks: Value =
        serde_json::from_str(&stdout).expect("tk list --json output is not valid JSON");

    tasks
        .as_array()
        .expect("tk list --json is not an array")
        .iter()
        .find(|t| t["title"].as_str().unwrap_or("") == title)
        .unwrap_or_else(|| panic!("task with title '{}' not found in list", title))["id"]
        .as_str()
        .expect("task has no 'id' field")
        .to_string()
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(expr = "I claim subtask {string}")]
async fn i_claim_subtask(world: &mut TacksWorld, title: String) {
    let id = find_task_id_by_title(world, &title);
    run_tk(world, &["update", &id, "--claim"]);
    assert_eq!(
        world.last_exit_code, 0,
        "claim failed: {}",
        world.last_stderr
    );
}

#[when(expr = "I reopen subtask {string}")]
async fn i_reopen_subtask(world: &mut TacksWorld, title: String) {
    let id = find_task_id_by_title(world, &title);
    run_tk(world, &["update", &id, "--status", "open"]);
    assert_eq!(
        world.last_exit_code, 0,
        "reopen failed: {}",
        world.last_stderr
    );
}
