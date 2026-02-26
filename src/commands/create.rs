use std::path::Path;

use chrono::Utc;

use crate::db::Database;
use crate::models::{Status, Task};

pub fn run(
    db_path: &Path,
    title: &str,
    priority: u8,
    description: Option<&str>,
    tags: Option<&str>,
    parent: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let db = Database::open(db_path)?;

    let id = if let Some(parent_id) = parent {
        // Verify parent exists
        db.get_task(parent_id)?
            .ok_or_else(|| format!("parent task not found: {parent_id}"))?;
        db.generate_child_id(parent_id)?
    } else {
        db.generate_id()?
    };

    let now = Utc::now();
    let tag_list: Vec<String> = tags
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let task = Task {
        id: id.clone(),
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        status: Status::Open,
        priority,
        assignee: None,
        parent_id: parent.map(|s| s.to_string()),
        tags: tag_list,
        created_at: now,
        updated_at: now,
        close_reason: None,
    };

    db.insert_task(&task)?;

    // Auto-tag parent as epic when a child is created
    if let Some(parent_id) = parent {
        let mut parent_tags = db.get_task_tags(parent_id)?;
        if !parent_tags.contains(&"epic".to_string()) {
            parent_tags.push("epic".to_string());
            db.update_tags(parent_id, &parent_tags)?;
        }
    }

    if json {
        let j = serde_json::to_string_pretty(&task).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
    } else {
        println!("Created task {id}: {title}");
    }

    Ok(())
}
