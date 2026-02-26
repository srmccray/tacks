use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::Path;

use crate::models::{Comment, Dependency, Status, Task};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at the given path.
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("failed to open database: {e}"))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("failed to set pragmas: {e}"))?;

        Ok(Database { conn })
    }

    /// Create the schema tables if they don't exist, then run any pending version-gated migrations.
    pub fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                description TEXT,
                status      TEXT NOT NULL DEFAULT 'open',
                priority    INTEGER NOT NULL DEFAULT 2,
                assignee    TEXT,
                parent_id   TEXT REFERENCES tasks(id),
                tags        TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS dependencies (
                child_id  TEXT NOT NULL REFERENCES tasks(id),
                parent_id TEXT NOT NULL REFERENCES tasks(id),
                PRIMARY KEY (child_id, parent_id),
                CHECK (child_id != parent_id)
            );

            CREATE TABLE IF NOT EXISTS comments (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id    TEXT NOT NULL REFERENCES tasks(id),
                body       TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
            CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_id);
            CREATE INDEX IF NOT EXISTS idx_deps_child ON dependencies(child_id);
            CREATE INDEX IF NOT EXISTS idx_deps_parent ON dependencies(parent_id);
            CREATE INDEX IF NOT EXISTS idx_comments_task ON comments(task_id);
            ",
            )
            .map_err(|e| format!("migration failed: {e}"))?;

        // Ensure schema_version exists in config (fresh databases get version 0).
        self.conn
            .execute(
                "INSERT OR IGNORE INTO config (key, value) VALUES ('schema_version', '0')",
                [],
            )
            .map_err(|e| format!("failed to seed schema_version: {e}"))?;

        run_migrations(&self.conn)
    }

    // -- Config --

    pub fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| format!("failed to set config: {e}"))?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM config WHERE key = ?1")
            .map_err(|e| format!("query error: {e}"))?;
        let mut rows = stmt
            .query_map(params![key], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query error: {e}"))?;
        match rows.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(format!("query error: {e}")),
            None => Ok(None),
        }
    }

    // -- Tasks --

    pub fn insert_task(&self, task: &Task) -> Result<(), String> {
        let tags_str = task.tags.join(",");
        self.conn
            .execute(
                "INSERT INTO tasks (id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    task.id,
                    task.title,
                    task.description,
                    task.status.as_str(),
                    task.priority,
                    task.assignee,
                    task.parent_id,
                    tags_str,
                    task.created_at.to_rfc3339(),
                    task.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| format!("failed to insert task: {e}"))?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at
                 FROM tasks WHERE id = ?1",
            )
            .map_err(|e| format!("query error: {e}"))?;

        let mut rows = stmt
            .query_map(params![id], |row| Ok(row_to_task(row)))
            .map_err(|e| format!("query error: {e}"))?;

        match rows.next() {
            Some(Ok(task)) => Ok(Some(task)),
            Some(Err(e)) => Err(format!("query error: {e}")),
            None => Ok(None),
        }
    }

    pub fn list_tasks(
        &self,
        include_done: bool,
        status_filter: Option<&str>,
        priority_filter: Option<u8>,
        tag_filter: Option<&str>,
    ) -> Result<Vec<Task>, String> {
        let mut sql = String::from(
            "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at FROM tasks WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(status) = status_filter {
            sql.push_str(&format!(" AND status = ?{param_idx}"));
            param_values.push(Box::new(status.to_string()));
            param_idx += 1;
        } else if !include_done {
            sql.push_str(&format!(" AND status != ?{param_idx}"));
            param_values.push(Box::new("done".to_string()));
            param_idx += 1;
        }

        if let Some(p) = priority_filter {
            sql.push_str(&format!(" AND priority = ?{param_idx}"));
            param_values.push(Box::new(p));
            param_idx += 1;
        }

        if let Some(tag) = tag_filter {
            sql.push_str(&format!(
                " AND (',' || tags || ',') LIKE '%,' || ?{param_idx} || ',%'"
            ));
            param_values.push(Box::new(tag.to_string()));
            let _ = param_idx; // suppress unused warning
        }

        sql.push_str(" ORDER BY priority ASC, created_at ASC");

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| format!("query error: {e}"))?;

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(params_ref.as_slice(), |row| Ok(row_to_task(row)))
            .map_err(|e| format!("query error: {e}"))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(tasks)
    }

    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        priority: Option<u8>,
        status: Option<&str>,
        description: Option<&str>,
        assignee: Option<&str>,
    ) -> Result<(), String> {
        let mut sets = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(t) = title {
            sets.push(format!("title = ?{idx}"));
            param_values.push(Box::new(t.to_string()));
            idx += 1;
        }
        if let Some(p) = priority {
            sets.push(format!("priority = ?{idx}"));
            param_values.push(Box::new(p));
            idx += 1;
        }
        if let Some(s) = status {
            // Validate status
            Status::from_str(s)?;
            sets.push(format!("status = ?{idx}"));
            param_values.push(Box::new(s.to_string()));
            idx += 1;
        }
        if let Some(d) = description {
            sets.push(format!("description = ?{idx}"));
            param_values.push(Box::new(d.to_string()));
            idx += 1;
        }
        if let Some(a) = assignee {
            sets.push(format!("assignee = ?{idx}"));
            param_values.push(Box::new(a.to_string()));
            idx += 1;
        }

        if sets.is_empty() {
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();
        sets.push(format!("updated_at = ?{idx}"));
        param_values.push(Box::new(now));
        idx += 1;

        let sql = format!("UPDATE tasks SET {} WHERE id = ?{idx}", sets.join(", "));
        param_values.push(Box::new(id.to_string()));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows_changed = self
            .conn
            .execute(&sql, params_ref.as_slice())
            .map_err(|e| format!("update failed: {e}"))?;

        if rows_changed == 0 {
            return Err(format!("task not found: {id}"));
        }
        Ok(())
    }

    pub fn update_tags(&self, id: &str, tags: &[String]) -> Result<(), String> {
        let tags_str = tags.join(",");
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE tasks SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                params![tags_str, now, id],
            )
            .map_err(|e| format!("tag update failed: {e}"))?;
        Ok(())
    }

    pub fn get_task_tags(&self, id: &str) -> Result<Vec<String>, String> {
        let task = self
            .get_task(id)?
            .ok_or_else(|| format!("task not found: {id}"))?;
        Ok(task.tags)
    }

    // -- Dependencies --

    pub fn add_dependency(&self, child_id: &str, parent_id: &str) -> Result<(), String> {
        // Verify both tasks exist
        self.get_task(child_id)?
            .ok_or_else(|| format!("task not found: {child_id}"))?;
        self.get_task(parent_id)?
            .ok_or_else(|| format!("task not found: {parent_id}"))?;

        // Detect duplicate before inserting
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies WHERE child_id = ?1 AND parent_id = ?2",
                params![child_id, parent_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .map_err(|e| format!("query error: {e}"))?;

        if exists {
            return Err(format!(
                "dependency already exists: {child_id} is already blocked by {parent_id}"
            ));
        }

        // Guard against cycles: check whether parent_id transitively depends on child_id
        if would_create_cycle(&self.conn, child_id, parent_id)? {
            return Err(
                "circular dependency detected: adding this dependency would create a cycle"
                    .to_string(),
            );
        }

        self.conn
            .execute(
                "INSERT INTO dependencies (child_id, parent_id) VALUES (?1, ?2)",
                params![child_id, parent_id],
            )
            .map_err(|e| format!("failed to add dependency: {e}"))?;
        Ok(())
    }

    pub fn remove_dependency(&self, child_id: &str, parent_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM dependencies WHERE child_id = ?1 AND parent_id = ?2",
                params![child_id, parent_id],
            )
            .map_err(|e| format!("failed to remove dependency: {e}"))?;
        Ok(())
    }

    pub fn get_blockers(&self, task_id: &str) -> Result<Vec<Dependency>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT child_id, parent_id FROM dependencies WHERE child_id = ?1")
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(Dependency {
                    child_id: row.get(0)?,
                    parent_id: row.get(1)?,
                })
            })
            .map_err(|e| format!("query error: {e}"))?;

        let mut deps = Vec::new();
        for row in rows {
            deps.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(deps)
    }

    /// Get all tasks that are blocked by the given task (reverse of `get_blockers`).
    ///
    /// Returns every task whose work cannot proceed until `task_id` is resolved.
    /// This is the "dependents" direction: `task_id` is the blocker, and the
    /// returned tasks are the ones waiting on it.
    #[allow(dead_code)]
    pub fn get_dependents(&self, task_id: &str) -> Result<Vec<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.title, t.description, t.status, t.priority, t.assignee,
                        t.parent_id, t.tags, t.created_at, t.updated_at
                 FROM tasks t
                 JOIN dependencies d ON t.id = d.child_id
                 WHERE d.parent_id = ?1
                 ORDER BY t.priority ASC, t.created_at ASC",
            )
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map(params![task_id], |row| Ok(row_to_task(row)))
            .map_err(|e| format!("query error: {e}"))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(tasks)
    }

    /// Get tasks that are ready: open and have no open/in_progress blockers.
    /// If `limit` is `Some(n)`, return at most `n` tasks.
    pub fn get_ready_tasks(&self, limit: Option<u32>) -> Result<Vec<Task>, String> {
        let mut sql = String::from(
            "
            SELECT t.id, t.title, t.description, t.status, t.priority, t.assignee, t.parent_id, t.tags, t.created_at, t.updated_at
            FROM tasks t
            WHERE t.status = 'open'
              AND NOT EXISTS (
                SELECT 1 FROM dependencies d
                JOIN tasks blocker ON d.parent_id = blocker.id
                WHERE d.child_id = t.id
                  AND blocker.status IN ('open', 'in_progress', 'blocked')
              )
            ORDER BY t.priority ASC, t.created_at ASC
        ",
        );

        if let Some(n) = limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_task(row)))
            .map_err(|e| format!("query error: {e}"))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(tasks)
    }

    // -- Comments --

    pub fn add_comment(&self, task_id: &str, body: &str) -> Result<Comment, String> {
        // Verify task exists
        self.get_task(task_id)?
            .ok_or_else(|| format!("task not found: {task_id}"))?;

        let now = Utc::now();
        self.conn
            .execute(
                "INSERT INTO comments (task_id, body, created_at) VALUES (?1, ?2, ?3)",
                params![task_id, body, now.to_rfc3339()],
            )
            .map_err(|e| format!("failed to add comment: {e}"))?;

        let id = self.conn.last_insert_rowid();
        Ok(Comment {
            id,
            task_id: task_id.to_string(),
            body: body.to_string(),
            created_at: now,
        })
    }

    pub fn get_comments(&self, task_id: &str) -> Result<Vec<Comment>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, task_id, body, created_at FROM comments WHERE task_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map(params![task_id], |row| {
                let created_str: String = row.get(3)?;
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(Comment {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    body: row.get(2)?,
                    created_at,
                })
            })
            .map_err(|e| format!("query error: {e}"))?;

        let mut comments = Vec::new();
        for row in rows {
            comments.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(comments)
    }

    // -- Stats --

    /// Count tasks grouped by status.
    pub fn task_count_by_status(&self) -> Result<Vec<(String, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM tasks GROUP BY status ORDER BY status")
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| format!("query error: {e}"))?;

        let mut counts = Vec::new();
        for row in rows {
            counts.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(counts)
    }

    /// Count tasks grouped by priority.
    pub fn task_count_by_priority(&self) -> Result<Vec<(u8, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT priority, COUNT(*) FROM tasks GROUP BY priority ORDER BY priority")
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, u8>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| format!("query error: {e}"))?;

        let mut counts = Vec::new();
        for row in rows {
            counts.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(counts)
    }

    /// Count tasks grouped by tag (tasks with multiple tags are counted once per tag).
    pub fn task_count_by_tag(&self) -> Result<Vec<(String, i64)>, String> {
        // Pull all non-empty tags columns and split them in Rust
        let mut stmt = self
            .conn
            .prepare("SELECT tags FROM tasks WHERE tags != ''")
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query error: {e}"))?;

        let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in rows {
            let tags_str = row.map_err(|e| format!("row error: {e}"))?;
            for tag in tags_str.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    *map.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut counts: Vec<(String, i64)> = map.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        Ok(counts)
    }

    /// Generate a short hash-based ID with the configured prefix.
    pub fn generate_id(&self) -> Result<String, String> {
        let prefix = self
            .get_config("prefix")?
            .unwrap_or_else(|| "tk".to_string());
        let uuid = uuid::Uuid::new_v4();
        let hash = &format!("{:x}", uuid.as_u128())[..4];
        Ok(format!("{prefix}-{hash}"))
    }

    /// Generate a child ID under a parent.
    pub fn generate_child_id(&self, parent_id: &str) -> Result<String, String> {
        // Count existing children to determine next index
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM tasks WHERE parent_id = ?1")
            .map_err(|e| format!("query error: {e}"))?;
        let count: i64 = stmt
            .query_row(params![parent_id], |row| row.get(0))
            .map_err(|e| format!("query error: {e}"))?;
        Ok(format!("{parent_id}.{}", count + 1))
    }

    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at
                 FROM tasks WHERE parent_id = ?1 ORDER BY id ASC",
            )
            .map_err(|e| format!("query error: {e}"))?;

        let rows = stmt
            .query_map(params![parent_id], |row| Ok(row_to_task(row)))
            .map_err(|e| format!("query error: {e}"))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| format!("row error: {e}"))?);
        }
        Ok(tasks)
    }
}

/// Read the current schema version from the config table.
fn get_schema_version(conn: &Connection) -> Result<i32, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM config WHERE key = 'schema_version'")
        .map_err(|e| format!("failed to read schema_version: {e}"))?;
    let mut rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("failed to query schema_version: {e}"))?;
    match rows.next() {
        Some(Ok(v)) => v
            .parse::<i32>()
            .map_err(|e| format!("invalid schema_version value: {e}")),
        Some(Err(e)) => Err(format!("failed to read schema_version row: {e}")),
        None => Ok(0),
    }
}

/// Persist the schema version to the config table.
#[allow(dead_code)]
fn set_schema_version(conn: &Connection, version: i32) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value) VALUES ('schema_version', ?1)",
        params![version.to_string()],
    )
    .map_err(|e| format!("failed to set schema_version: {e}"))?;
    Ok(())
}

/// Run all pending schema migrations in order.
///
/// Each migration should be wrapped in a transaction so that a partial failure
/// does not leave the schema in an inconsistent state. Version 0 is the
/// baseline created by the `CREATE TABLE IF NOT EXISTS` block in `migrate()`;
/// future migrations (v1, v2, ...) will be added as additional `if version < N`
/// blocks here.
fn run_migrations(conn: &Connection) -> Result<(), String> {
    let version = get_schema_version(conn)?;

    // v0 is the baseline -- no ALTER TABLE statements needed.
    // Future migrations follow this pattern:
    //
    // if version < 1 {
    //     conn.execute_batch(
    //         "BEGIN;
    //          ALTER TABLE tasks ADD COLUMN close_reason TEXT;
    //          COMMIT;",
    //     )
    //     .map_err(|e| format!("migration v1 failed: {e}"))?;
    //     set_schema_version(conn, 1)?;
    // }
    //
    // if version < 2 {
    //     conn.execute_batch(
    //         "BEGIN;
    //          ALTER TABLE tasks ADD COLUMN notes TEXT;
    //          COMMIT;",
    //     )
    //     .map_err(|e| format!("migration v2 failed: {e}"))?;
    //     set_schema_version(conn, 2)?;
    // }

    // Suppress unused-variable lint while no active migrations exist.
    let _ = version;

    Ok(())
}

/// Return `true` if inserting the edge `child_id → parent_id` would create a cycle.
///
/// The dependency table records that `child_id` is blocked by `parent_id`.  A
/// cycle exists when `parent_id` already transitively depends on `child_id`
/// (i.e. `child_id` is reachable by following dependency edges starting from
/// `parent_id`).
///
/// The BFS walks from `parent_id` through its own blockers (rows where
/// `child_id = current`), looking for `child_id` in the visited set.  The
/// search is bounded by the total number of distinct nodes in the graph, so it
/// always terminates even on a large but acyclic graph.
fn would_create_cycle(conn: &Connection, child_id: &str, parent_id: &str) -> Result<bool, String> {
    use std::collections::{HashSet, VecDeque};

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    visited.insert(parent_id.to_string());
    queue.push_back(parent_id.to_string());

    while let Some(current) = queue.pop_front() {
        // Fetch all tasks that `current` depends on (its direct blockers)
        let mut stmt = conn
            .prepare("SELECT parent_id FROM dependencies WHERE child_id = ?1")
            .map_err(|e| format!("query error: {e}"))?;

        let blocker_ids: Vec<String> = stmt
            .query_map(params![current], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        for blocker in blocker_ids {
            if blocker == child_id {
                return Ok(true);
            }
            if !visited.contains(&blocker) {
                visited.insert(blocker.clone());
                queue.push_back(blocker);
            }
        }
    }

    Ok(false)
}

fn row_to_task(row: &rusqlite::Row) -> Task {
    let status_str: String = row.get(3).unwrap_or_default();
    let tags_str: String = row.get(7).unwrap_or_default();
    let created_str: String = row.get(8).unwrap_or_default();
    let updated_str: String = row.get(9).unwrap_or_default();

    Task {
        id: row.get(0).unwrap_or_default(),
        title: row.get(1).unwrap_or_default(),
        description: row.get(2).ok(),
        status: Status::from_str(&status_str).unwrap_or(Status::Open),
        priority: row.get::<_, u8>(4).unwrap_or(2),
        assignee: row
            .get(5)
            .ok()
            .and_then(|v: String| if v.is_empty() { None } else { Some(v) }),
        parent_id: row
            .get(6)
            .ok()
            .and_then(|v: String| if v.is_empty() { None } else { Some(v) }),
        tags: if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(|s| s.trim().to_string()).collect()
        },
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    }
}
