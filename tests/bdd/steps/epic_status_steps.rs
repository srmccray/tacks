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
// When steps — epic command
// ---------------------------------------------------------------------------

#[when("I run tk epic with JSON")]
async fn i_run_tk_epic_json(world: &mut TacksWorld) {
    run_tk(world, &["--json", "epic"]);
}

// ---------------------------------------------------------------------------
// Then steps — epic progress assertions
// ---------------------------------------------------------------------------

#[then(expr = "the epic output shows {string} with {int} of {int} done")]
async fn the_epic_output_shows_progress(
    world: &mut TacksWorld,
    title: String,
    done: i64,
    total: i64,
) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let epics = json.as_array().expect("epic JSON output is not an array");
    let epic = epics
        .iter()
        .find(|e| e["title"].as_str().unwrap_or("") == title)
        .unwrap_or_else(|| {
            panic!(
                "epic '{}' not found in output: {}",
                title, world.last_stdout
            )
        });
    let actual_done = epic["children_done"].as_i64().unwrap_or(-1);
    let actual_total = epic["children_total"].as_i64().unwrap_or(-1);
    assert_eq!(
        actual_done, done,
        "expected {} children done but got {}",
        done, actual_done
    );
    assert_eq!(
        actual_total, total,
        "expected {} children total but got {}",
        total, actual_total
    );
}
