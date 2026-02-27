# Agent Catalog

Quick reference for which agent to dispatch for each task type.

## Agents

| Agent | Purpose | Model | File |
|-------|---------|-------|------|
| core-dev | Implement CLI commands, models, main.rs wiring | sonnet | `agents/core-dev.md` |
| bdd | BDD feature files, step definitions, test harness | sonnet | `agents/bdd.md` |
| db | SQLite schema, migrations, queries, data integrity | sonnet | `agents/db-agent.md` |
| code-reviewer | Review changes for quality, correctness, security | sonnet | `agents/code-reviewer.md` |
| debugger | Diagnose test failures, build errors, runtime bugs | sonnet | `agents/debugger.md` |

## When to Invoke

| Situation | Agent(s) |
|-----------|----------|
| Adding a new CLI command | bdd (feature file first), then core-dev (implementation) |
| Changing the SQLite schema | db (migration + queries), then core-dev (caller updates) |
| Writing tests for existing code | bdd |
| Reviewing before merge | code-reviewer |
| BDD scenario failing | debugger, then the agent that owns the fix |
| Build or clippy failure | debugger |
| Runtime error from `tk` command | debugger |
| Refactoring command internals | core-dev |
| Optimizing queries or adding indexes | db |

## Agent Capabilities Matrix

| Agent | Reads Code | Writes Code | Runs Tests | Read-Only |
|-------|-----------|-------------|-----------|-----------|
| core-dev | Y | Y | Y | N |
| bdd | Y | Y | Y | N |
| db | Y | Y | Y | N |
| code-reviewer | Y | N | Y | Y |
| debugger | Y | N | Y | Y |

## File Ownership

| Path | Owner |
|------|-------|
| `src/commands/**` | core-dev |
| `src/models/**` | core-dev |
| `src/main.rs` | core-dev |
| `src/db/**` | db |
| `tests/**` | bdd |

## Common Workflows

### New Command
1. **bdd** -- Write feature file FIRST (red-green BDD cycle)
2. **db** -- Add schema changes if needed (migration, new queries)
3. **core-dev** -- Implement command, wire into main.rs
4. **bdd** -- Verify scenarios pass, add edge case scenarios
5. **code-reviewer** -- Final review before merge

### Schema Change
1. **db** -- Design migration, update queries, update row_to_task()
2. **core-dev** -- Update callers in src/commands/ for new function signatures
3. **bdd** -- Add scenarios covering new behavior
4. **code-reviewer** -- Review

### Bug Fix
1. **debugger** -- Diagnose root cause, identify owning agent
2. **core-dev** or **db** -- Apply fix (depends on where bug lives)
3. **bdd** -- Add regression test
4. **code-reviewer** -- Review fix

### Refactor
1. **core-dev** or **db** -- Apply refactor
2. **bdd** -- Verify all existing scenarios still pass (no behavior change)
3. **code-reviewer** -- Review
