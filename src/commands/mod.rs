pub mod blocked;
pub mod children;
pub mod close;
pub mod comment;
pub mod create;
pub mod dep;
pub mod epic;
pub mod init;
pub mod list;
pub mod prime;
pub mod ready;
pub mod show;
pub mod stats;
pub mod update;

use crate::models::Task;
use colored::Colorize;

/// Format a priority number as a colored string.
pub fn format_priority(p: u8) -> String {
    match p {
        0 => "P0".red().bold().to_string(),
        1 => "P1".yellow().bold().to_string(),
        2 => "P2".white().to_string(),
        3 => "P3".bright_black().to_string(),
        _ => format!("P{p}"),
    }
}

/// Format a status as a colored string.
pub fn format_status(s: &crate::models::Status) -> String {
    match s {
        crate::models::Status::Open => "open".green().to_string(),
        crate::models::Status::InProgress => "in_progress".cyan().to_string(),
        crate::models::Status::Done => "done".bright_black().to_string(),
        crate::models::Status::Blocked => "blocked".red().to_string(),
    }
}

/// Print a list of tasks as a table or JSON.
pub fn print_tasks(tasks: &[Task], json: bool) -> Result<(), String> {
    if json {
        let j = serde_json::to_string_pretty(tasks).map_err(|e| format!("json error: {e}"))?;
        println!("{j}");
        return Ok(());
    }

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // Simple aligned table output
    println!(
        "{:<12} {:<4} {:<12} {:<50} TAGS",
        "ID", "PRI", "STATUS", "TITLE"
    );
    println!("{}", "-".repeat(90));
    for t in tasks {
        let tags = if t.tags.is_empty() {
            String::new()
        } else {
            t.tags.join(", ")
        };
        let title = if t.title.chars().count() > 48 {
            let truncated: String = t.title.chars().take(45).collect();
            format!("{truncated}...")
        } else {
            t.title.clone()
        };
        println!(
            "{:<12} {:<4} {:<12} {:<50} {}",
            t.id,
            format_priority(t.priority),
            format_status(&t.status),
            title,
            tags,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Status, Task};
    use chrono::Utc;

    fn make_task(title: &str) -> Task {
        let now = Utc::now();
        Task {
            id: "tk-0001".to_string(),
            title: title.to_string(),
            description: None,
            status: Status::Open,
            priority: 2,
            assignee: None,
            parent_id: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            close_reason: None,
            notes: None,
        }
    }

    /// Verify that print_tasks does not panic when a title contains multi-byte
    /// UTF-8 characters (em dash is 3 bytes) and the character count exceeds 48.
    #[test]
    fn test_print_tasks_with_multibyte_chars() {
        // Build a title that is >48 chars long with em dashes (3 bytes each).
        // "Task with em dash \u{2014} " prefix (24 chars) + 30 em dashes = 54 chars total.
        let long_title = format!("Task with em dash \u{2014} {}", "\u{2014}".repeat(30));
        assert!(long_title.chars().count() > 48);

        let tasks = vec![make_task(&long_title)];
        // Should not panic — that is the regression being guarded.
        print_tasks(&tasks, false).expect("print_tasks must not panic on multibyte chars");
    }
}
