use std::path::Path;

use crate::db::Database;
use crate::models::validate_close_reason;

/// Close a task, optionally recording a comment and close reason.
pub fn run(
    db_path: &Path,
    id: &str,
    comment: Option<&str>,
    reason: Option<&str>,
    force: bool,
    json: bool,
) -> Result<(), String> {
    let db = Database::open(db_path)?;

    // Validate reason before touching the DB.
    if let Some(r) = reason {
        validate_close_reason(r)?;
    }

    // Close guard: refuse to close a parent task (epic) that still has open
    // subtask children. Use --force to override.
    //
    // Note: dependency-graph blocking (dep add) is a separate relationship;
    // closing a prerequisite (blocker) while dependents are still open is the
    // expected workflow and is not guarded here.
    let children = db.get_children(id)?;
    let open_children: Vec<_> = children
        .iter()
        .filter(|t| t.status != crate::models::Status::Done)
        .collect();

    if !open_children.is_empty() && !force {
        let names: Vec<String> = open_children
            .iter()
            .map(|t| format!("{} ({})", t.id, t.title))
            .collect();
        return Err(format!(
            "task {} has {} open dependent(s): {}. use --force to close anyway",
            id,
            open_children.len(),
            names.join(", ")
        ));
    }

    db.close_task(id, reason)?;

    if let Some(body) = comment {
        db.add_comment(id, body)?;
    }

    if json {
        let task = db
            .get_task(id)?
            .ok_or_else(|| format!("task not found: {id}"))?;
        let j = serde_json::to_string_pretty(&task).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
    } else {
        println!("Closed task {id}");
    }

    Ok(())
}
