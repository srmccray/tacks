---
description: Get AI context output — stats, in-progress tasks, and the ready queue
---

Run `tk prime` to get a compact, AI-optimized snapshot of the current backlog state. This is automatically run at session start and before context compaction via hooks.

## Usage

```bash
tk prime [--json]
```

## Instructions

1. Run `tk prime --json` using the Bash tool.
2. Parse the output to orient yourself:
   - **stats**: How many tasks are open, in-progress, done?
   - **in_progress**: What is currently being worked on?
   - **ready**: What unblocked tasks are available to pick up next?
3. Use this information to make decisions about what to work on and report status to the user.

## Examples

```bash
# Get AI context (human-readable)
tk prime

# Get AI context (JSON for parsing)
tk prime --json
```

## Notes

- `tk prime` is the recommended first command when starting a session — it gives you full situational awareness in one call.
- The output combines: backlog stats, currently in-progress tasks, and the top ready (unblocked) tasks.
- This command is automatically invoked by the plugin hooks at `SessionStart` and `PreCompact` so context is never lost during compaction.
- For more detail on any specific task, follow up with `tk show <id>`.
