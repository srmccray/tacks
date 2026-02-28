use std::path::Path;

use crate::db::Database;
use crate::models::Status;

/// Show epic progress: tasks tagged 'epic' with child completion stats.
pub fn run(db_path: &Path, json: bool) -> Result<(), String> {
    let db = Database::open(db_path)?;

    // Get all tasks tagged as epic
    let epics = db.list_tasks(false, None, None, Some("epic"), None, None)?;

    if json {
        let mut results = Vec::new();
        for epic in &epics {
            let children = db.get_children(&epic.id)?;
            let total = children.len();
            let done = children.iter().filter(|c| c.status == Status::Done).count();
            let pct = if total > 0 {
                (done as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };
            results.push(serde_json::json!({
                "id": epic.id,
                "title": epic.title,
                "status": epic.status,
                "priority": epic.priority,
                "children_total": total,
                "children_done": done,
                "progress_pct": pct,
            }));
        }
        let j = serde_json::to_string_pretty(&results).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
        return Ok(());
    }

    // Human-readable output
    if epics.is_empty() {
        println!("No epics found.");
        return Ok(());
    }

    println!(
        "{:<12} {:<4} {:<12} {:<40} PROGRESS",
        "ID", "PRI", "STATUS", "TITLE"
    );
    println!("{}", "-".repeat(80));
    for epic in &epics {
        let children = db.get_children(&epic.id)?;
        let total = children.len();
        let done = children.iter().filter(|c| c.status == Status::Done).count();
        let pct = if total > 0 {
            (done as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };
        let title = if epic.title.len() > 38 {
            format!("{}...", &epic.title[..35])
        } else {
            epic.title.clone()
        };
        println!(
            "{:<12} {:<4} {:<12} {:<40} {}/{} ({}%)",
            epic.id,
            super::format_priority(epic.priority),
            super::format_status(&epic.status),
            title,
            done,
            total,
            pct,
        );
    }
    Ok(())
}
