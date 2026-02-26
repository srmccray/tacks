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
// When steps — children command
// ---------------------------------------------------------------------------

#[when(expr = "I run tk children for {string}")]
async fn i_run_tk_children(world: &mut TacksWorld, alias: String) {
    // If the alias is in task_ids, resolve it; otherwise treat it as a literal ID.
    let id_str = world
        .task_ids
        .get(&alias)
        .cloned()
        .unwrap_or_else(|| alias.clone());
    run_tk(world, &["children", &id_str]);
}

#[when(expr = "I run tk children for {string} with JSON")]
async fn i_run_tk_children_json(world: &mut TacksWorld, alias: String) {
    let id_str = world
        .task_ids
        .get(&alias)
        .cloned()
        .unwrap_or_else(|| alias.clone());
    run_tk(world, &["--json", "children", &id_str]);
}

// ---------------------------------------------------------------------------
// Then steps — output assertions
// ---------------------------------------------------------------------------

#[then(expr = "the output contains {string}")]
async fn the_output_contains(world: &mut TacksWorld, expected: String) {
    assert!(
        world.last_stdout.contains(&expected),
        "expected stdout to contain '{}' but got: {}",
        expected,
        world.last_stdout
    );
}

#[then("the JSON output is an empty array")]
async fn the_json_output_is_empty_array(world: &mut TacksWorld) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let arr = json.as_array().expect("JSON output is not an array");
    assert!(
        arr.is_empty(),
        "expected empty JSON array but got {} items: {}",
        arr.len(),
        world.last_stdout
    );
}
