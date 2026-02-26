use std::path::Path;

use crate::db::Database;

#[allow(clippy::too_many_arguments)]
pub fn run(
    db_path: &Path,
    id: &str,
    title: Option<&str>,
    priority: Option<u8>,
    status: Option<&str>,
    description: Option<&str>,
    claim: bool,
    assignee: Option<&str>,
    add_tags: Option<&str>,
    remove_tags: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let db = Database::open(db_path)?;

    // Handle claim: set status to in_progress and assignee
    let effective_status = if claim { Some("in_progress") } else { status };

    let effective_assignee = if claim && assignee.is_none() {
        Some("agent")
    } else {
        assignee
    };

    db.update_task(
        id,
        title,
        priority,
        effective_status,
        description,
        effective_assignee,
        None,
    )?;

    // Handle tag changes
    if add_tags.is_some() || remove_tags.is_some() {
        let mut current_tags = db.get_task_tags(id)?;

        if let Some(add) = add_tags {
            for tag in add
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
            {
                if !current_tags.contains(&tag) {
                    current_tags.push(tag);
                }
            }
        }

        if let Some(remove) = remove_tags {
            let remove_set: Vec<String> = remove.split(',').map(|s| s.trim().to_string()).collect();
            current_tags.retain(|t| !remove_set.contains(t));
        }

        db.update_tags(id, &current_tags)?;
    }

    if json {
        let task = db
            .get_task(id)?
            .ok_or_else(|| format!("task not found: {id}"))?;
        let j = serde_json::to_string_pretty(&task).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
    } else {
        println!("Updated task {id}");
    }

    Ok(())
}
