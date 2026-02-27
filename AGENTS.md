# Agent Instructions

This project uses **tk** (tacks) for task tracking. Run `tk prime` to get oriented.

## Quick Reference

```bash
tk prime              # AI context: stats + in-progress + ready queue
tk ready              # Find available work
tk ready --limit 1    # Next task for agent to pick
tk show <id>          # View task details
tk update <id> --claim  # Claim task (in_progress + assignee)
tk close <id> -c "Done" # Complete work
tk stats              # Backlog overview
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File tasks for remaining work** - Create tasks for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update task status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
