use std::collections::HashMap;
use std::path::Path;

use crate::db::Database;

pub fn run(db_path: &Path, oneline: bool, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;

    let by_status = db.task_count_by_status()?;
    let by_priority = db.task_count_by_priority()?;
    let by_tag = db.task_count_by_tag()?;

    if json {
        let status_map: HashMap<&str, i64> =
            by_status.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let priority_map: HashMap<String, i64> = by_priority
            .iter()
            .map(|(p, c)| (format!("P{p}"), *c))
            .collect();
        let tag_map: HashMap<&str, i64> = by_tag.iter().map(|(t, c)| (t.as_str(), *c)).collect();

        let out = serde_json::json!({
            "by_status": status_map,
            "by_priority": priority_map,
            "by_tag": tag_map,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| format!("json error: {e}"))?
        );
        return Ok(());
    }

    if oneline {
        let parts: Vec<String> = by_status.iter().map(|(s, c)| format!("{c} {s}")).collect();
        if parts.is_empty() {
            println!("no tasks");
        } else {
            println!("{}", parts.join(", "));
        }
        return Ok(());
    }

    // Default: full table output
    if by_status.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // By status
    println!("By Status");
    println!("{}", "-".repeat(24));
    for (status, count) in &by_status {
        println!("  {:<14} {}", status, count);
    }

    // By priority
    if !by_priority.is_empty() {
        println!();
        println!("By Priority");
        println!("{}", "-".repeat(24));
        for (priority, count) in &by_priority {
            println!("  {:<14} {}", format!("P{priority}"), count);
        }
    }

    // By tag
    if !by_tag.is_empty() {
        println!();
        println!("By Tag");
        println!("{}", "-".repeat(24));
        for (tag, count) in &by_tag {
            println!("  {:<14} {}", tag, count);
        }
    }

    Ok(())
}
