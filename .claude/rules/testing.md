# Testing Rules

- Every new command needs at least one integration test
- Use `tempfile::TempDir` for test databases -- never use production paths
- Test both human-readable and JSON output modes
- Use `assert_cmd` for CLI integration tests
- Unit tests go in the same file as the code they test (Rust convention)
- Name test functions descriptively: `test_create_task_with_tags`
- Test error cases: invalid IDs, missing tasks, duplicate dependencies
