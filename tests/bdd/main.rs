mod steps;

use std::collections::HashMap;
use std::path::PathBuf;

use cucumber::World;

/// Shared state carried through each scenario.
#[derive(Debug, World)]
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
    /// Port the in-process test web server is listening on.
    pub server_port: Option<u16>,
    /// Handle to the spawned test server task (used for cleanup).
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shared HTTP client for web test steps.
    pub http_client: reqwest::Client,
    /// HTTP status code of the most recent response.
    pub last_response_status: Option<u16>,
    /// Content-Type header of the most recent response.
    pub last_response_content_type: Option<String>,
    /// Body text of the most recent response.
    pub last_response_body: Option<String>,
    /// The ID of the most recently created task via inline-edit steps.
    pub last_task_id: Option<String>,
    /// Stored created_at timestamp for datetime-immutability assertions.
    pub stored_created_at: Option<String>,
}

impl Default for TacksWorld {
    fn default() -> Self {
        Self {
            db_dir: None,
            db_path: None,
            last_stdout: String::new(),
            last_stderr: String::new(),
            last_exit_code: 0,
            task_ids: HashMap::new(),
            server_port: None,
            server_handle: None,
            http_client: reqwest::Client::new(),
            last_response_status: None,
            last_response_content_type: None,
            last_response_body: None,
            last_task_id: None,
            stored_created_at: None,
        }
    }
}

#[tokio::main]
async fn main() {
    TacksWorld::run("tests/features").await;
}
