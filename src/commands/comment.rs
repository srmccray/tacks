use std::path::Path;

use crate::db::Database;

pub fn run(db_path: &Path, id: &str, body: &str, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;
    let comment = db.add_comment(id, body)?;

    if json {
        let j = serde_json::to_string_pretty(&comment).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
    } else {
        println!("Added comment to {id}");
    }

    Ok(())
}
