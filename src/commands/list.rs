use std::path::Path;

use super::print_tasks;
use crate::db::Database;

pub fn run(
    db_path: &Path,
    all: bool,
    status: Option<&str>,
    priority: Option<u8>,
    tag: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let db = Database::open(db_path)?;
    let tasks = db.list_tasks(all, status, priority, tag)?;
    print_tasks(&tasks, json)
}
