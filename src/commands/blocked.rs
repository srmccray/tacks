use std::path::Path;

use crate::db::Database;

/// List tasks that are blocked by open dependencies.
pub fn run(db_path: &Path, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;
    let tasks = db.get_blocked_tasks()?;
    super::print_tasks(&tasks, json)
}
