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
// When steps — blocked command
// ---------------------------------------------------------------------------

#[when("I run tk blocked with JSON")]
async fn i_run_tk_blocked_json(world: &mut TacksWorld) {
    run_tk(world, &["--json", "blocked"]);
}

// ---------------------------------------------------------------------------
// Then steps — blocked output assertions
// ---------------------------------------------------------------------------

#[then(expr = "the blocked output contains {string}")]
async fn the_blocked_output_contains(world: &mut TacksWorld, expected_title: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("blocked output is not valid JSON");
    let tasks = json.as_array().expect("blocked JSON is not an array");
    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);
    assert!(
        found,
        "expected '{}' in blocked output but got: {}",
        expected_title, world.last_stdout
    );
}

#[then(expr = "the blocked output does not contain {string}")]
async fn the_blocked_output_does_not_contain(world: &mut TacksWorld, expected_title: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("blocked output is not valid JSON");
    let tasks = json.as_array().expect("blocked JSON is not an array");
    let found = tasks
        .iter()
        .any(|t| t["title"].as_str().unwrap_or("") == expected_title);
    assert!(
        !found,
        "expected '{}' to be absent from blocked output but it was present",
        expected_title
    );
}
