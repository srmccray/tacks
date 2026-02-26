#![allow(deprecated)]
use cucumber::{then, when};
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

// ---------------------------------------------------------------------------
// When steps — notes
// ---------------------------------------------------------------------------

#[when(expr = "I update task {string} with notes {string}")]
async fn i_update_task_with_notes(world: &mut TacksWorld, alias: String, notes: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    run_tk(world, &["update", &id, "--notes", &notes]);
    assert_eq!(
        world.last_exit_code, 0,
        "tk update --notes failed: {}",
        world.last_stderr
    );
}

// ---------------------------------------------------------------------------
// Then steps — notes assertions
// ---------------------------------------------------------------------------

#[then(expr = "the task details show notes {string}")]
async fn the_task_details_show_notes(world: &mut TacksWorld, expected: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let actual = json["notes"].as_str().unwrap_or("null");
    assert_eq!(
        actual, expected,
        "expected notes '{}' but got '{}'",
        expected, actual
    );
}

#[then("the task details have no notes")]
async fn the_task_details_have_no_notes(world: &mut TacksWorld) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    assert!(
        json["notes"].is_null(),
        "expected notes to be null but got: {}",
        json["notes"]
    );
}
