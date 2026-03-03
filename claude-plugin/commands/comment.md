---
description: Add an append-only comment to a task's activity log
---

Run `tk comment <id> <body>` to append a comment to a task. Comments are permanent and append-only — they form the task's activity history.

## Usage

```bash
tk comment <id> "<body>"
```

## Instructions

1. Identify the task ID and compose the comment text.
2. Run `tk comment <id> "<body>"` using the Bash tool.
3. Confirm: "Comment added to `<id>`."

## Examples

```bash
# Add a progress update
tk comment tk-a1b2 "Investigated the issue — root cause is in the auth middleware"

# Add a decision record
tk comment tk-a1b2 "Decided to use JWT tokens instead of session cookies per team discussion"

# Note a blocker discovered during work
tk comment tk-a1b2 "Blocked on upstream API change in service B — waiting for their release"
```

## Notes

- Comments are **append-only** — they cannot be edited or deleted. Use them for permanent activity log entries.
- For mutable working context (notes you want to update as you go), use `tk update --notes "<text>"` instead.
- Comments appear in `tk show <id>` output under the comments section.
- Use comments to record: decisions made, findings from investigation, handoff context, or status updates.
