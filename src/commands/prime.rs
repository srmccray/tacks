use std::path::Path;

use crate::db::Database;
use crate::models::Task;

const READY_LIMIT: u32 = 5;

const COMMAND_REFERENCE: &[&str] = &[
    "tk create <title> [-p priority] [-d desc] [-t tags] [--parent id]",
    "tk list [-s status] [-p pri] [-t tag] [--json]",
    "tk ready [--limit N] [--json]",
    "tk show <id> [--json]",
    "tk update <id> [fields...] [--claim]",
    "tk close <id> [-c comment]",
    "tk dep add|remove <child> <parent>",
    "tk comment <id> <body>",
    "tk stats [--oneline] [--json]",
];

/// Run the `tk prime` command.
///
/// Outputs an AI-optimized context summary composed of stats, in-progress tasks,
/// and the ready queue. If no `.tacks/` database exists, exits silently.
pub fn run(db_path: &Path, json: bool) -> Result<(), String> {
    // Silent exit when no tacks database is present — hooks call this on every
    // session, so it must be a no-op in projects that don't use tacks.
    if !db_path.exists() {
        return Ok(());
    }

    let db = Database::open(db_path)?;

    let by_status = db.task_count_by_status()?;
    let in_progress = db.list_tasks(false, Some("in_progress"), None, None, None, None)?;
    let ready = db.get_ready_tasks(Some(READY_LIMIT))?;

    if json {
        print_json(&by_status, &in_progress, &ready)
    } else {
        print_markdown(&by_status, &in_progress, &ready)
    }
}

fn print_markdown(
    by_status: &[(String, i64)],
    in_progress: &[Task],
    ready: &[Task],
) -> Result<(), String> {
    println!("# Tacks: Project Status");

    // Stats section
    println!();
    println!("## Stats");
    if by_status.is_empty() {
        println!("no tasks");
    } else {
        let parts: Vec<String> = by_status.iter().map(|(s, c)| format!("{c} {s}")).collect();
        println!("{}", parts.join(", "));
    }

    // In Progress section
    println!();
    println!("## In Progress");
    if in_progress.is_empty() {
        println!("none");
    } else {
        for task in in_progress {
            let mut line = format!("- {}: {} [P{}]", task.id, task.title, task.priority);
            if let Some(assignee) = &task.assignee {
                line.push_str(&format!(" (assigned: {assignee})"));
            }
            println!("{line}");
        }
    }

    // Ready section
    println!();
    println!("## Ready (next {READY_LIMIT})");
    if ready.is_empty() {
        println!("none");
    } else {
        for task in ready {
            println!("- {}: {} [P{}]", task.id, task.title, task.priority);
        }
    }

    // Command Reference section
    println!();
    println!("## Command Reference");
    for cmd in COMMAND_REFERENCE {
        println!("{cmd}");
    }

    Ok(())
}

fn print_json(
    by_status: &[(String, i64)],
    in_progress: &[Task],
    ready: &[Task],
) -> Result<(), String> {
    // Build a stats object with the four canonical statuses always present.
    let mut stats = serde_json::Map::new();
    let canonical = ["open", "in_progress", "blocked", "done"];
    for key in canonical {
        stats.insert(key.to_string(), serde_json::Value::Number(0.into()));
    }
    for (status, count) in by_status {
        stats.insert(status.clone(), serde_json::Value::Number((*count).into()));
    }

    let cmd_ref: Vec<serde_json::Value> = COMMAND_REFERENCE
        .iter()
        .map(|s| serde_json::Value::String(s.to_string()))
        .collect();

    let out = serde_json::json!({
        "stats": stats,
        "in_progress": in_progress,
        "ready": ready,
        "command_reference": cmd_ref,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| format!("json error: {e}"))?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use serde_json::Value;
    use tempfile::TempDir;

    fn tk(tmp: &TempDir) -> Command {
        let db_path = tmp.path().join("tacks.db");
        let mut cmd = Command::cargo_bin("tk").expect("tk binary not found");
        cmd.env("TACKS_DB", &db_path);
        cmd
    }

    fn init_db(tmp: &TempDir) {
        tk(tmp).args(["init"]).assert().success();
    }

    #[test]
    fn test_prime_silent_exit_when_no_db() {
        // prime should exit 0 silently when the database does not exist
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("no_such.db");
        Command::cargo_bin("tk")
            .unwrap()
            .env("TACKS_DB", &db_path)
            .args(["prime"])
            .assert()
            .success()
            .stdout("");
    }

    #[test]
    fn test_prime_markdown_empty_db() {
        let tmp = TempDir::new().unwrap();
        init_db(&tmp);

        let output = tk(&tmp).args(["prime"]).output().unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("# Tacks: Project Status"));
        assert!(stdout.contains("## Stats"));
        assert!(stdout.contains("## In Progress"));
        assert!(stdout.contains("## Ready"));
        assert!(stdout.contains("## Command Reference"));
        assert!(stdout.contains("tk create"));
    }

    #[test]
    fn test_prime_json_empty_db() {
        let tmp = TempDir::new().unwrap();
        init_db(&tmp);

        let output = tk(&tmp).args(["--json", "prime"]).output().unwrap();

        assert!(output.status.success());
        let json: Value =
            serde_json::from_slice(&output.stdout).expect("prime --json output is not valid JSON");

        // All four canonical status keys are present
        assert!(json["stats"]["open"].is_number());
        assert!(json["stats"]["in_progress"].is_number());
        assert!(json["stats"]["blocked"].is_number());
        assert!(json["stats"]["done"].is_number());

        assert!(json["in_progress"].is_array());
        assert!(json["ready"].is_array());
        assert!(json["command_reference"].is_array());
        assert!(!json["command_reference"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_prime_shows_in_progress_tasks() {
        let tmp = TempDir::new().unwrap();
        init_db(&tmp);

        // Create a task and claim it (sets to in_progress)
        let create_out = tk(&tmp)
            .args(["--json", "create", "My in-progress task", "-p", "1"])
            .output()
            .unwrap();
        let create_json: Value = serde_json::from_slice(&create_out.stdout).unwrap();
        let task_id = create_json["id"].as_str().unwrap().to_string();

        tk(&tmp)
            .args(["update", &task_id, "--claim"])
            .assert()
            .success();

        let output = tk(&tmp).args(["prime"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("My in-progress task"),
            "expected in-progress task in prime output; got:\n{stdout}"
        );
    }

    #[test]
    fn test_prime_shows_ready_tasks() {
        let tmp = TempDir::new().unwrap();
        init_db(&tmp);

        tk(&tmp)
            .args(["create", "Ready task", "-p", "2"])
            .assert()
            .success();

        let output = tk(&tmp).args(["prime"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Ready task"),
            "expected ready task in prime output; got:\n{stdout}"
        );
    }

    #[test]
    fn test_prime_json_in_progress_and_ready() {
        let tmp = TempDir::new().unwrap();
        init_db(&tmp);

        // Create a ready task (open, no blockers)
        tk(&tmp).args(["create", "Ready item"]).assert().success();

        // Create and claim a task so it becomes in_progress
        let create_out = tk(&tmp)
            .args(["--json", "create", "Working item"])
            .output()
            .unwrap();
        let id: Value = serde_json::from_slice(&create_out.stdout).unwrap();
        let task_id = id["id"].as_str().unwrap().to_string();
        tk(&tmp)
            .args(["update", &task_id, "--claim"])
            .assert()
            .success();

        let out = tk(&tmp).args(["--json", "prime"]).output().unwrap();
        let json: Value = serde_json::from_slice(&out.stdout).unwrap();

        // in_progress array should contain "Working item"
        let ip = json["in_progress"].as_array().unwrap();
        assert!(
            ip.iter()
                .any(|t| t["title"].as_str() == Some("Working item")),
            "expected 'Working item' in in_progress; got {ip:?}"
        );

        // stats should show 1 in_progress
        assert_eq!(json["stats"]["in_progress"], 1);
    }
}
