use std::path::Path;

use crate::db::Database;

pub fn run(db_path: &Path, prefix: &str) -> Result<(), String> {
    // Create the .tacks directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create directory: {e}"))?;
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    db.set_config("prefix", prefix)?;
    db.set_config("version", env!("CARGO_PKG_VERSION"))?;

    println!("Initialized tacks database at {}", db_path.display());
    println!("Task prefix: {prefix}");
    Ok(())
}
