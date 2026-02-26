use std::path::Path;

use crate::db::Database;

pub fn run(db_path: &Path, id: &str, comment: Option<&str>, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;

    db.update_task(id, None, None, Some("done"), None, None)?;

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
