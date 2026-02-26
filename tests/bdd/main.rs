mod steps;

use std::collections::HashMap;
use std::path::PathBuf;

use cucumber::World;

/// Shared state carried through each scenario.
#[derive(Debug, Default, World)]
pub struct TacksWorld {
    /// Temporary directory that owns the database file.
    pub db_dir: Option<tempfile::TempDir>,
    /// Path to the SQLite database file inside `db_dir`.
    pub db_path: Option<PathBuf>,
    /// The raw stdout of the most recent `tk` invocation.
    pub last_stdout: String,
    /// The raw stderr of the most recent `tk` invocation.
    pub last_stderr: String,
    /// Exit code of the most recent `tk` invocation.
    pub last_exit_code: i32,
    /// Alias to actual task ID map, populated by create steps.
    pub task_ids: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    TacksWorld::run("tests/features").await;
}
