# Tacks Agents

Agent catalog for the tacks project.

## Agents

| Agent | Purpose | Model | Location |
|-------|---------|-------|----------|
| core-dev | Core feature development: commands, database, models | sonnet | `.claude/agents/core-dev.md` |
| test-agent | Test authoring and coverage for CLI and database | sonnet | `.claude/agents/test-agent.md` |
| db-agent | SQLite schema, queries, migrations, optimization | sonnet | `.claude/agents/db-agent.md` |
| cli-ux | Output formatting, help text, error messages, UX | sonnet | `.claude/agents/cli-ux.md` |
| integration | Git hooks, Claude Code hooks, MCP, import/export | sonnet | `.claude/agents/integration.md` |

## When to Use

- **Adding a new command**: core-dev (implementation) then test-agent (tests)
- **Changing the schema**: db-agent first, then core-dev for command changes
- **Improving output**: cli-ux
- **Adding git/tool integration**: integration
- **Writing tests for existing code**: test-agent
