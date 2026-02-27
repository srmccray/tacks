# Definition of Done

## New CLI Command

- [ ] Command implemented in `src/commands/<name>.rs`
- [ ] Wired into `src/main.rs` CLI dispatch
- [ ] Supports `--json` output via `cli.json` global flag
- [ ] Human-readable table output as default
- [ ] JSON output uses existing Task/Comment/Dependency structs (no ad-hoc fields)
- [ ] At least one BDD feature file in `tests/features/`
- [ ] Step definitions in `tests/bdd/steps/`
- [ ] `cargo test --test bdd` passes all scenarios
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Doc comments on public functions

## Modifying Existing Command

- [ ] No existing flags removed or renamed (see `rules/stability.md`)
- [ ] No JSON output fields removed or type-changed
- [ ] New flags are optional with sensible defaults
- [ ] Existing BDD scenarios still pass without modification
- [ ] `cargo test --test bdd` passes all scenarios
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean

## Schema Change

- [ ] Version-gated migration in `src/db/mod.rs` (increment `schema_version`)
- [ ] `row_to_task` updated if columns added
- [ ] All callers of changed DB functions updated
- [ ] BDD scenarios cover the new behavior
- [ ] Backward-compatible (new columns nullable or defaulted)

## Bug Fix

- [ ] Root cause identified and documented in commit message
- [ ] Fix implemented with minimal scope
- [ ] Regression test added (BDD scenario or unit test)
- [ ] `cargo test --test bdd` passes all scenarios
- [ ] `cargo clippy -- -D warnings` clean

## Refactor

- [ ] No behavior change (BDD scenarios still pass without modification)
- [ ] `cargo test --test bdd` passes all scenarios
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
