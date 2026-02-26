# Rust Conventions

- Use `Result<(), String>` for fallible functions that surface errors to users
- Prefer `&str` parameters over `String` where possible
- Use `rusqlite::params!` for parameterized queries -- never interpolate SQL
- All public functions need doc comments
- Error messages start lowercase, no trailing period
- Use `unwrap_or_default()` or `unwrap_or_else()` over `unwrap()` except in tests
- Avoid `clone()` when a reference suffices
- Use `thiserror` or typed errors if error handling grows beyond String
