/// Rules content embedded at compile time.
const RULES_CONTENT: &str = r#"# Tacks Task Manager

Tacks (`tk`) is the task tracker for this project. Use it to manage tasks, track progress, and coordinate work.

## Session Start

Run `tk prime` at the start of each session to get context: backlog stats, in-progress tasks, and the ready queue.

## Command Reference

```
tk create <title> [-p priority] [-d desc] [-t tags] [--parent id]
tk list [-s status] [-p pri] [-t tag] [--parent id]
tk ready [--limit N]
tk show <id>
tk update <id> [fields...] [--claim] [--notes text] [--parent id|none]
tk close <id> [-c comment] [-r reason] [--force]
tk dep add|remove <child> <parent>
tk comment <id> <body>
tk children <id>
tk epic
tk blocked
tk stats [--oneline]
tk prime
```

All commands support `--json` for machine-readable output.

## Workflow

1. Run `tk prime` to orient
2. Pick a task: `tk ready --limit 1`
3. Claim it: `tk update <id> --claim`
4. Add working notes: `tk update <id> --notes "context"`
5. Close when done: `tk close <id> -c "summary"`

## Conventions

- Task IDs use `tk-XXXX` format, subtasks use `tk-XXXX.N`
- Priority: P0 (critical) through P3 (low), default P2
- Status: open → in_progress → done (or blocked)
- Close reasons: done, duplicate, absorbed, stale, superseded
- Tags are comma-separated: `-t "backend,api"`
- The `epic` tag is auto-added when a task gets children
- Use `--json` when parsing output programmatically
- Use `tk stats --oneline` for compact status checks
"#;

/// Install a Claude Code rules file that teaches Claude how to use `tk`.
///
/// Writes `tacks.md` to either `.claude/rules/` (project, default) or
/// `~/.claude/rules/` (global, when `--global` is passed).
pub fn run(global: bool) -> Result<(), String> {
    let rules_dir = if global {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        std::path::PathBuf::from(home).join(".claude").join("rules")
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cannot determine current directory: {e}"))?
            .join(".claude")
            .join("rules")
    };

    std::fs::create_dir_all(&rules_dir)
        .map_err(|e| format!("cannot create directory {}: {e}", rules_dir.display()))?;

    let output_path = rules_dir.join("tacks.md");

    std::fs::write(&output_path, RULES_CONTENT)
        .map_err(|e| format!("cannot write {}: {e}", output_path.display()))?;

    println!("Wrote Claude Code rules to {}", output_path.display());
    println!();
    println!("Contents:");
    println!("  - Session bootstrapping (tk prime)");
    println!("  - Command reference (15 commands)");
    println!("  - Workflow guide (claim → work → close)");
    println!("  - Conventions (IDs, priorities, statuses, tags)");

    Ok(())
}
