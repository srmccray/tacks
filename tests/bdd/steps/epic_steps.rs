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
// When steps — subtask creation
// ---------------------------------------------------------------------------

#[when(expr = "I create a subtask of {string} with title {string}")]
async fn i_create_subtask(world: &mut TacksWorld, parent_alias: String, title: String) {
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    let db_path = world.db_path.as_ref().expect("db_path not set").clone();

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", &db_path)
        .args(["--json", "create", &title, "--parent", &parent_id])
        .output()
        .expect("failed to run tk create");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.last_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    world.last_exit_code = output.status.code().unwrap_or(-1);
    world.last_stdout = stdout.clone();

    if output.status.success() {
        let json: Value = serde_json::from_str(&stdout).expect("create output is not valid JSON");
        if let Some(id) = json["id"].as_str() {
            world
                .task_ids
                .insert("last_subtask".to_string(), id.to_string());
        }
    }
}

// ---------------------------------------------------------------------------
// Then steps — tag assertions
// ---------------------------------------------------------------------------

#[then(expr = "the task details include tag {string}")]
async fn the_task_details_include_tag(world: &mut TacksWorld, expected_tag: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let tags = json["tags"]
        .as_array()
        .expect("no 'tags' array in show output");
    let found = tags
        .iter()
        .any(|t| t.as_str().unwrap_or("") == expected_tag);
    assert!(
        found,
        "expected tag '{}' not found in: {:?}",
        expected_tag, tags
    );
}

#[then(expr = "the task details show exactly one {string} tag")]
async fn the_task_details_show_exactly_one_tag(world: &mut TacksWorld, tag: String) {
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let tags = json["tags"]
        .as_array()
        .expect("no 'tags' array in show output");
    let count = tags
        .iter()
        .filter(|t| t.as_str().unwrap_or("") == tag)
        .count();
    assert_eq!(
        count, 1,
        "expected exactly 1 '{}' tag but found {}",
        tag, count
    );
}

// ---------------------------------------------------------------------------
// Then steps — subtask parent ID
// ---------------------------------------------------------------------------

#[then(expr = "the subtask has parent ID matching {string}")]
async fn the_subtask_has_parent_id(world: &mut TacksWorld, parent_alias: String) {
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    let subtask_id = world
        .task_ids
        .get("last_subtask")
        .expect("no subtask created — use 'When I create a subtask of ...' first")
        .clone();
    run_tk(world, &["--json", "show", &subtask_id]);
    let json: Value =
        serde_json::from_str(&world.last_stdout).expect("last output is not valid JSON");
    let actual_parent = json["parent_id"].as_str().unwrap_or("");
    assert_eq!(
        actual_parent, parent_id,
        "expected parent_id '{}' but got '{}'",
        parent_id, actual_parent
    );
}
