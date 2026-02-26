use std::path::Path;

use crate::db::Database;

/// List child tasks of a parent task.
pub fn run(db_path: &Path, id: &str, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;

    // Verify parent exists
    db.get_task(id)?
        .ok_or_else(|| format!("task not found: {id}"))?;

    let children = db.get_children(id)?;
    super::print_tasks(&children, json)
}
