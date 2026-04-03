#![allow(deprecated)]
use chrono::Utc;
use cucumber::given;
use rusqlite::{Connection, params};
use serde_json::json;

use crate::TacksWorld;
use crate::steps::web_api_steps::http_post;
use crate::steps::web_steps::http_get;

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

/// Close a task via the API and then backdate its `updated_at` timestamp so
/// that the done_since filter treats it as older than N days.
///
/// This is the only step that writes directly to the database rather than
/// going through the CLI/API, and it does so only to manipulate the
/// `updated_at` timestamp — which cannot be set through the public API.
#[given(expr = "I closed the API task {string} {int} days ago")]
async fn i_closed_the_api_task_n_days_ago(world: &mut TacksWorld, alias: String, days: i64) {
    // First close the task via the API so status is "done".
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    let path = format!("/api/tasks/{id}/close");
    let (status, body) = http_post(world, &path, json!({"reason": "done"})).await;
    assert_eq!(
        status, 200,
        "expected 200 from POST {path} but got {status}: {body}"
    );

    // Now backdate `updated_at` directly via SQLite so the done_since filter
    // treats this task as older than the requested window.
    let db_path = world
        .db_path
        .as_ref()
        .expect("db_path not set — did you forget 'Given a tacks database is initialized'?")
        .clone();

    let cutoff = Utc::now() - chrono::Duration::days(days);
    // Use an RFC 3339 timestamp string, which is how tacks stores datetimes.
    let backdated = cutoff.to_rfc3339();

    tokio::task::spawn_blocking(move || {
        let conn = Connection::open(&db_path)
            .unwrap_or_else(|e| panic!("failed to open DB for backdating: {e}"));
        conn.execute(
            "UPDATE tasks SET updated_at = ?1 WHERE id = ?2",
            params![backdated, id],
        )
        .unwrap_or_else(|e| panic!("failed to backdate task {id}: {e}"));
    })
    .await
    .expect("backdate task spawn_blocking failed");
}

// ---------------------------------------------------------------------------
// When steps — epic board navigation
// ---------------------------------------------------------------------------

/// Perform a GET request to the epic board view for a task identified by alias.
/// Equivalent to GET /epics/:id?view=board (uses default done_since).
#[cucumber::when(expr = "I GET the epic board for {string}")]
async fn i_get_the_epic_board_for(world: &mut TacksWorld, alias: String) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(world, &format!("/epics/{id}?view=board")).await;
}

/// Perform a GET request to the epic board view for a task identified by alias
/// with an explicit done_since parameter value.
#[cucumber::when(expr = "I GET the epic board for {string} with done_since {string}")]
async fn i_get_the_epic_board_for_with_done_since(
    world: &mut TacksWorld,
    alias: String,
    done_since: String,
) {
    let id = world
        .task_ids
        .get(&alias)
        .unwrap_or_else(|| panic!("no task with alias '{alias}'"))
        .clone();
    http_get(
        world,
        &format!("/epics/{id}?view=board&done_since={done_since}"),
    )
    .await;
}
