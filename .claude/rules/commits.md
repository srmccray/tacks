# Commit Rules

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix all warnings before committing
- Run `cargo test` and ensure all tests pass before committing
- Commit messages should be imperative mood, lowercase, no period
- Format: `<type>: <description>` where type is one of: feat, fix, refactor, test, docs, chore
- Keep commits focused on a single change
