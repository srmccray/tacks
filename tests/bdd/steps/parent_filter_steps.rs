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
// When steps — list with parent filter
// ---------------------------------------------------------------------------

#[when(expr = "I list tasks with parent {string}")]
async fn i_list_tasks_with_parent(world: &mut TacksWorld, parent_alias: String) {
    let parent_id = world
        .task_ids
        .get(&parent_alias)
        .unwrap_or_else(|| panic!("no task with alias '{parent_alias}'"))
        .clone();
    run_tk(world, &["--json", "list", "--parent", &parent_id]);
}
