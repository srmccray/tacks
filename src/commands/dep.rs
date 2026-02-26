use std::path::Path;

use crate::db::Database;

pub fn add(db_path: &Path, child: &str, parent: &str) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.add_dependency(child, parent)?;
    println!("Added dependency: {child} is blocked by {parent}");
    Ok(())
}

pub fn remove(db_path: &Path, child: &str, parent: &str) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.remove_dependency(child, parent)?;
    println!("Removed dependency: {child} no longer blocked by {parent}");
    Ok(())
}
