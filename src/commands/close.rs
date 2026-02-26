use std::path::Path;

use crate::db::Database;
use crate::models::validate_close_reason;

/// Close a task, optionally recording a comment and close reason.
pub fn run(
    db_path: &Path,
    id: &str,
    comment: Option<&str>,
    reason: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let db = Database::open(db_path)?;

    // Validate reason before touching the DB.
    if let Some(r) = reason {
        validate_close_reason(r)?;
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
