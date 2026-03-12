#![allow(deprecated)]
use cucumber::{then, when};
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

#[when("I run tk epic")]
async fn i_run_tk_epic(world: &mut TacksWorld) {
    run_tk(world, &["epic"]);
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(expr = "the epic JSON shows {string} with {int} done, {int} in_progress, {int} open")]
async fn the_epic_json_shows_three_part(
    world: &mut TacksWorld,
    title: String,
    done: i64,
    in_progress: i64,
    open: i64,
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
    let actual_in_progress = epic["children_in_progress"].as_i64().unwrap_or(-1);
    let actual_open = epic["children_open"].as_i64().unwrap_or(-1);

    assert_eq!(
        actual_done, done,
        "expected {} done but got {}",
        done, actual_done
    );
    assert_eq!(
        actual_in_progress, in_progress,
        "expected {} in_progress but got {}",
        in_progress, actual_in_progress
    );
    assert_eq!(
        actual_open, open,
        "expected {} open but got {}",
        open, actual_open
    );
}

#[then(expr = "the epic output contains {string}")]
async fn the_epic_output_contains(world: &mut TacksWorld, expected: String) {
    assert!(
        world.last_stdout.contains(&expected),
        "expected stdout to contain '{}' but got:\n{}",
        expected,
        world.last_stdout
    );
}
