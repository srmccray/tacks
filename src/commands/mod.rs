pub mod close;
pub mod comment;
pub mod create;
pub mod dep;
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
        let title = if t.title.len() > 48 {
            format!("{}...", &t.title[..45])
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
