# Team Decisions

## Architecture
- Local-only storage — no git integration, no sync, no distributed concerns
- Tags over type enum for epic/bug classification (epic = reserved tag, auto-tagged on child creation)
- Schema migration via version-gated ALTER TABLEs in config table, no framework, no rollback
- Close reason as structured nullable column with constrained enum, not a comment
- Notes field as mutable working context, distinct from append-only comments

## Conventions
- BDD-driven development with cucumber-rs and Gherkin feature files
- Feature files written BEFORE implementation for new features (red-green BDD cycle)
- Steps shell out to `tk` binary via assert_cmd, assert against `--json` output
- Feature files serve dual purpose: executable tests + agent-readable behavioral documentation
- Three-phase sprint: Phase 1 (compatibility) → Phase 2 (foundation) → Phase 3 (mutation features)

## Skipped Features (explicit decisions NOT to build)
- auto-block/auto-unblock on dependency changes
- tk swarm validate (orchestration logic belongs in blossom, not task manager)
- transitive dependency traversal
- dep tree visualization
- 7-type IssueType system (only epic/task/bug via tags)
