#![allow(deprecated)]
use cucumber::given;

use crate::TacksWorld;

/// Initialize a fresh tacks database into the world's temp dir.
#[given("a tacks database is initialized")]
async fn a_tacks_database_is_initialized(world: &mut TacksWorld) {
    let dir = tempfile::TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("tacks.db");

    let output = assert_cmd::Command::cargo_bin("tk")
        .expect("tk binary not found")
        .env("TACKS_DB", &db_path)
        .arg("init")
        .output()
        .expect("failed to run tk init");

    assert!(
        output.status.success(),
        "tk init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    world.db_path = Some(db_path);
    // Keep the TempDir alive for the lifetime of the scenario.
    world.db_dir = Some(dir);
}
