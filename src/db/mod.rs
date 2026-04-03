use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::Path;
use std::str::FromStr;

use crate::models::{Comment, Dependency, Status, Task, validate_close_reason};

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
                "INSERT INTO tasks (id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at, close_reason, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                    task.close_reason,
                    task.notes,
                ],
            )
            .map_err(|e| format!("failed to insert task: {e}"))?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at, close_reason, notes
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

    /// List tasks with optional filters.
    ///
    /// - `include_done`: when `true`, done tasks are included even without a status_filter
    /// - `status_filter`: exact status match (overrides include_done)
    /// - `priority_filter`: exact priority match
    /// - `tag_filter`: task must contain this tag
    /// - `parent_filter`: task must have this parent_id
    /// - `search`: case-insensitive substring match on title
    /// - `completed_after`: RFC3339 timestamp; when `Some`, only tasks with `updated_at > ts` are returned
    #[allow(clippy::too_many_arguments)]
    pub fn list_tasks(
        &self,
        include_done: bool,
        status_filter: Option<&str>,
        priority_filter: Option<u8>,
        tag_filter: Option<&str>,
        parent_filter: Option<&str>,
        search: Option<&str>,
        completed_after: Option<&str>,
    ) -> Result<Vec<Task>, String> {
        let mut sql = String::from(
            "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at, close_reason, notes FROM tasks WHERE 1=1",
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
            param_idx += 1;
        }

        if let Some(parent) = parent_filter {
            sql.push_str(&format!(" AND parent_id = ?{param_idx}"));
            param_values.push(Box::new(parent.to_string()));
            param_idx += 1;
        }

        if let Some(s) = search {
            sql.push_str(&format!(
                " AND title LIKE '%' || ?{param_idx} || '%' COLLATE NOCASE"
            ));
            param_values.push(Box::new(s.to_string()));
            param_idx += 1;
        }

        if let Some(ts) = completed_after {
            sql.push_str(&format!(" AND updated_at > ?{param_idx}"));
            param_values.push(Box::new(ts.to_string()));
            let _ = param_idx; // suppress unused warning after last param
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

    #[allow(clippy::too_many_arguments)]
    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        priority: Option<u8>,
        status: Option<&str>,
        description: Option<&str>,
        assignee: Option<&str>,
        close_reason: Option<&str>,
        notes: Option<&str>,
    ) -> Result<(), String> {
        self.update_task_with_parent(
            id,
            title,
            priority,
            status,
            description,
            assignee,
            close_reason,
            notes,
            None,
        )
    }

    /// Update task fields, optionally reparenting to a new parent.
    ///
    /// `new_parent` uses a two-level Option:
    /// - `None` — do not change parent_id
    /// - `Some(Some("tk-xxxx"))` — set parent_id to the given task
    /// - `Some(None)` — clear parent_id (promote to top-level)
    #[allow(clippy::too_many_arguments)]
    pub fn update_task_with_parent(
        &self,
        id: &str,
        title: Option<&str>,
        priority: Option<u8>,
        status: Option<&str>,
        description: Option<&str>,
        assignee: Option<&str>,
        close_reason: Option<&str>,
        notes: Option<&str>,
        new_parent: Option<Option<&str>>,
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
        if let Some(r) = close_reason {
            validate_close_reason(r)?;
            sets.push(format!("close_reason = ?{idx}"));
            param_values.push(Box::new(r.to_string()));
            idx += 1;
        }
        if let Some(n) = notes {
            sets.push(format!("notes = ?{idx}"));
            param_values.push(Box::new(n.to_string()));
            idx += 1;
        }

        // Handle reparenting
        if let Some(maybe_parent) = new_parent {
            match maybe_parent {
                Some(new_pid) => {
                    // Validate: no self-parenting
                    if new_pid == id {
                        return Err("cannot reparent a task under itself".to_string());
                    }
                    // Validate: new parent must exist
                    self.get_task(new_pid)?
                        .ok_or_else(|| format!("parent task not found: {new_pid}"))?;
                    // Validate: new parent must not itself be a subtask (max depth 1)
                    let parent_task = self.get_task(new_pid)?.unwrap();
                    if parent_task.parent_id.is_some() {
                        return Err(format!(
                            "cannot reparent under {new_pid}: it is already a subtask (max depth is 1)"
                        ));
                    }
                    // Validate: no circular parenting (child is not an ancestor of new parent)
                    // Check if new_pid is a child of id (which would create a cycle)
                    let my_children = self.get_children(id)?;
                    for child in &my_children {
                        if child.id == new_pid {
                            return Err(
                                "circular parenting: the new parent is a child of this task"
                                    .to_string(),
                            );
                        }
                    }

                    sets.push(format!("parent_id = ?{idx}"));
                    param_values.push(Box::new(new_pid.to_string()));
                    idx += 1;

                    // Auto-tag new parent as epic
                    let mut parent_tags = self.get_task_tags(new_pid)?;
                    if !parent_tags.contains(&"epic".to_string()) {
                        parent_tags.push("epic".to_string());
                        self.update_tags(new_pid, &parent_tags)?;
                    }
                }
                None => {
                    // Clear parent_id (promote to top-level)
                    sets.push("parent_id = NULL".to_string());
                    // No param needed for NULL
                }
            }
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

    /// Close a task: set status to done and record the close_reason.
    ///
    /// Automatically syncs the parent epic's status if this task is a subtask.
    pub fn close_task(&self, id: &str, reason: Option<&str>) -> Result<(), String> {
        self.update_task(id, None, None, Some("done"), None, None, reason, None)?;
        self.sync_parent_epic(id)?;
        Ok(())
    }

    /// If the given task has a parent, recalculate and update the parent epic's status.
    fn sync_parent_epic(&self, task_id: &str) -> Result<(), String> {
        if let Some(task) = self.get_task(task_id)?
            && let Some(pid) = &task.parent_id
        {
            self.sync_epic_status(pid)?;
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
        // Verify both tasks exist
        self.get_task(child_id)?
            .ok_or_else(|| format!("task not found: {child_id}"))?;
        self.get_task(parent_id)?
            .ok_or_else(|| format!("task not found: {parent_id}"))?;

        let rows = self
            .conn
            .execute(
                "DELETE FROM dependencies WHERE child_id = ?1 AND parent_id = ?2",
                params![child_id, parent_id],
            )
            .map_err(|e| format!("failed to remove dependency: {e}"))?;

        if rows == 0 {
            return Err(format!(
                "no dependency found: {child_id} is not blocked by {parent_id}"
            ));
        }
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
    pub fn get_dependents(&self, task_id: &str) -> Result<Vec<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.title, t.description, t.status, t.priority, t.assignee,
                        t.parent_id, t.tags, t.created_at, t.updated_at, t.close_reason, t.notes
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
            SELECT t.id, t.title, t.description, t.status, t.priority, t.assignee, t.parent_id, t.tags, t.created_at, t.updated_at, t.close_reason, t.notes
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

    /// Get tasks that have at least one open/in_progress blocker.
    pub fn get_blocked_tasks(&self) -> Result<Vec<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT t.id, t.title, t.description, t.status, t.priority, t.assignee,
                    t.parent_id, t.tags, t.created_at, t.updated_at, t.close_reason, t.notes
             FROM tasks t
             JOIN dependencies d ON t.id = d.child_id
             JOIN tasks blocker ON d.parent_id = blocker.id
             WHERE t.status != 'done'
               AND blocker.status IN ('open', 'in_progress', 'blocked')
             ORDER BY t.priority ASC, t.created_at ASC",
            )
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

    /// Return the current SQLite `PRAGMA data_version` value.
    ///
    /// This integer increments whenever the database is modified by any connection,
    /// making it suitable as a lightweight change-detection signal for polling clients.
    pub fn data_version(&self) -> Result<i64, String> {
        self.conn
            .query_row("PRAGMA data_version", [], |row| row.get::<_, i64>(0))
            .map_err(|e| format!("failed to read data_version: {e}"))
    }

    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Task>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, status, priority, assignee, parent_id, tags, created_at, updated_at, close_reason, notes
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

    /// Traverse the dependency graph from a starting task and return all reachable tasks with depth.
    ///
    /// `direction` controls which edges are followed:
    /// - `"up"` — follows blocker edges (what does this task depend on, transitively)
    /// - `"down"` — follows dependent edges (what tasks depend on this task, transitively)
    /// - `"both"` — follows both directions
    ///
    /// The root task itself (depth 0) is **not** included in the result — the caller already holds it.
    /// All other reachable tasks are returned at their BFS depth (minimum hops from root), ordered
    /// breadth-first. Diamond dependencies are handled correctly: a task reachable via multiple paths
    /// appears only once, at the shallowest depth at which it was first discovered.
    ///
    /// A visited set guards against infinite loops even though cycles are rejected at write time.
    pub fn get_dependency_chain(
        &self,
        task_id: &str,
        direction: &str,
    ) -> Result<Vec<(Task, usize)>, String> {
        use std::collections::{HashSet, VecDeque};

        if direction != "up" && direction != "down" && direction != "both" {
            return Err(format!(
                "invalid direction '{direction}': must be 'up', 'down', or 'both'"
            ));
        }

        // Helper: fetch direct blocker IDs for a given task (up direction)
        let fetch_up = |current: &str| -> Result<Vec<String>, String> {
            let mut stmt = self
                .conn
                .prepare("SELECT parent_id FROM dependencies WHERE child_id = ?1")
                .map_err(|e| format!("query error: {e}"))?;
            let ids: Vec<String> = stmt
                .query_map(params![current], |row| row.get::<_, String>(0))
                .map_err(|e| format!("query error: {e}"))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(ids)
        };

        // Helper: fetch direct dependent IDs for a given task (down direction)
        let fetch_down = |current: &str| -> Result<Vec<String>, String> {
            let mut stmt = self
                .conn
                .prepare("SELECT child_id FROM dependencies WHERE parent_id = ?1")
                .map_err(|e| format!("query error: {e}"))?;
            let ids: Vec<String> = stmt
                .query_map(params![current], |row| row.get::<_, String>(0))
                .map_err(|e| format!("query error: {e}"))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(ids)
        };

        // BFS in one direction; returns (id, depth) pairs ordered by discovery
        let bfs = |start: &str, go_up: bool| -> Result<Vec<(String, usize)>, String> {
            let mut visited: HashSet<String> = HashSet::new();
            let mut queue: VecDeque<(String, usize)> = VecDeque::new();
            let mut result: Vec<(String, usize)> = Vec::new();

            visited.insert(start.to_string());
            queue.push_back((start.to_string(), 0));

            while let Some((current, depth)) = queue.pop_front() {
                let neighbors = if go_up {
                    fetch_up(&current)?
                } else {
                    fetch_down(&current)?
                };

                for neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor.clone());
                        result.push((neighbor.clone(), depth + 1));
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }

            Ok(result)
        };

        // Collect (id, depth) pairs according to direction, deduplicating for "both"
        let id_depth_pairs: Vec<(String, usize)> = match direction {
            "up" => bfs(task_id, true)?,
            "down" => bfs(task_id, false)?,
            "both" => {
                let mut seen: HashSet<String> = HashSet::new();
                let mut combined: Vec<(String, usize)> = Vec::new();

                for (id, depth) in bfs(task_id, true)?.into_iter().chain(bfs(task_id, false)?) {
                    if !seen.contains(&id) {
                        seen.insert(id.clone());
                        combined.push((id, depth));
                    }
                }
                combined
            }
            _ => unreachable!(),
        };

        // Resolve IDs to Task structs
        let mut result = Vec::with_capacity(id_depth_pairs.len());
        for (id, depth) in id_depth_pairs {
            if let Some(task) = self.get_task(&id)? {
                result.push((task, depth));
            }
        }

        Ok(result)
    }

    /// Recalculate and update an epic's status based on its children's statuses.
    ///
    /// - All children open/blocked → epic status = open
    /// - Any children done or in_progress (but not all done) → epic status = in_progress
    /// - All children done → epic status = done
    /// - No children → no change
    ///
    /// Call this after any operation that changes a child task's status.
    pub fn sync_epic_status(&self, epic_id: &str) -> Result<(), String> {
        let children = self.get_children(epic_id)?;
        if children.is_empty() {
            return Ok(());
        }

        let all_done = children
            .iter()
            .all(|c| c.status == crate::models::Status::Done);
        let any_done = children
            .iter()
            .any(|c| c.status == crate::models::Status::Done);
        let any_in_progress = children
            .iter()
            .any(|c| c.status == crate::models::Status::InProgress);

        let new_status = if all_done {
            "done"
        } else if any_done || any_in_progress {
            "in_progress"
        } else {
            "open"
        };

        // Only update if status actually changed
        let epic = self.get_task(epic_id)?;
        if let Some(epic) = epic {
            let current = epic.status.as_str();
            if current != new_status {
                self.update_task(
                    epic_id,
                    None,
                    None,
                    Some(new_status),
                    None,
                    None,
                    None,
                    None,
                )?;
            }
        }

        Ok(())
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

    if version < 1 {
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE tasks ADD COLUMN close_reason TEXT;
             COMMIT;",
        )
        .map_err(|e| format!("migration v1 failed: {e}"))?;
        set_schema_version(conn, 1)?;
    }

    if version < 2 {
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE tasks ADD COLUMN notes TEXT;
             COMMIT;",
        )
        .map_err(|e| format!("migration v2 failed: {e}"))?;
        set_schema_version(conn, 2)?;
    }

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
    let close_reason: Option<String> = row.get(10).unwrap_or(None);
    let notes: Option<String> = row.get(11).unwrap_or(None);

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
        close_reason,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create an in-memory-like temp database and return it along with the TempDir
    /// (must keep TempDir alive for the test's duration).
    fn open_test_db() -> (Database, TempDir) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open");
        db.migrate().expect("migrate");
        db.set_config("prefix", "tk").expect("set prefix");
        (db, dir)
    }

    fn make_task(db: &Database, title: &str) -> Task {
        let id = db.generate_id().expect("generate_id");
        let now = Utc::now();
        let task = Task {
            id,
            title: title.to_string(),
            description: None,
            status: crate::models::Status::Open,
            priority: 2,
            assignee: None,
            parent_id: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            close_reason: None,
            notes: None,
        };
        db.insert_task(&task).expect("insert_task");
        task
    }

    #[test]
    fn test_dependency_chain_empty_no_deps() {
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "A");

        let up = db.get_dependency_chain(&a.id, "up").expect("chain up");
        let down = db.get_dependency_chain(&a.id, "down").expect("chain down");
        let both = db.get_dependency_chain(&a.id, "both").expect("chain both");

        assert!(up.is_empty(), "no blockers expected");
        assert!(down.is_empty(), "no dependents expected");
        assert!(both.is_empty(), "no connections expected");
    }

    #[test]
    fn test_dependency_chain_single_dep() {
        // A is blocked by B (A depends on B)
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "A");
        let b = make_task(&db, "B");
        db.add_dependency(&a.id, &b.id).expect("add dep A->B");

        let up = db.get_dependency_chain(&a.id, "up").expect("chain up");
        assert_eq!(up.len(), 1, "A has one blocker");
        assert_eq!(up[0].0.id, b.id);
        assert_eq!(up[0].1, 1, "depth should be 1");

        let down = db.get_dependency_chain(&b.id, "down").expect("chain down");
        assert_eq!(down.len(), 1, "B has one dependent");
        assert_eq!(down[0].0.id, a.id);
        assert_eq!(down[0].1, 1, "depth should be 1");
    }

    #[test]
    fn test_dependency_chain_linear_chain() {
        // C is blocked by B, B is blocked by A
        // Up from C: [B@1, A@2]
        // Down from A: [B@1, C@2]
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "A");
        let b = make_task(&db, "B");
        let c = make_task(&db, "C");
        db.add_dependency(&b.id, &a.id).expect("B dep A");
        db.add_dependency(&c.id, &b.id).expect("C dep B");

        let up_from_c = db.get_dependency_chain(&c.id, "up").expect("up from C");
        assert_eq!(up_from_c.len(), 2);
        // BFS order: B at depth 1, A at depth 2
        let b_entry = up_from_c.iter().find(|(t, _)| t.id == b.id).expect("B");
        let a_entry = up_from_c.iter().find(|(t, _)| t.id == a.id).expect("A");
        assert_eq!(b_entry.1, 1);
        assert_eq!(a_entry.1, 2);

        let down_from_a = db.get_dependency_chain(&a.id, "down").expect("down from A");
        assert_eq!(down_from_a.len(), 2);
        let b_entry = down_from_a.iter().find(|(t, _)| t.id == b.id).expect("B");
        let c_entry = down_from_a.iter().find(|(t, _)| t.id == c.id).expect("C");
        assert_eq!(b_entry.1, 1);
        assert_eq!(c_entry.1, 2);
    }

    #[test]
    fn test_dependency_chain_diamond() {
        // Diamond: A and B both depend on X.
        // C depends on A and B.
        // Graph (up): C -> A -> X, C -> B -> X
        //             X should appear only once in "up from C"
        let (db, _dir) = open_test_db();
        let x = make_task(&db, "X");
        let a = make_task(&db, "A");
        let b = make_task(&db, "B");
        let c = make_task(&db, "C");
        db.add_dependency(&a.id, &x.id).expect("A dep X");
        db.add_dependency(&b.id, &x.id).expect("B dep X");
        db.add_dependency(&c.id, &a.id).expect("C dep A");
        db.add_dependency(&c.id, &b.id).expect("C dep B");

        let up_from_c = db.get_dependency_chain(&c.id, "up").expect("up from C");

        // Should contain A, B (depth 1) and X (depth 2) — X only once
        assert_eq!(up_from_c.len(), 3, "A, B, X — X appears only once");

        let ids: Vec<&str> = up_from_c.iter().map(|(t, _)| t.id.as_str()).collect();
        assert!(ids.contains(&a.id.as_str()));
        assert!(ids.contains(&b.id.as_str()));
        assert!(ids.contains(&x.id.as_str()));

        // X appears exactly once
        let x_count = up_from_c.iter().filter(|(t, _)| t.id == x.id).count();
        assert_eq!(x_count, 1, "X must appear exactly once (diamond dedup)");

        // X depth should be 2 (reached via A or B)
        let x_depth = up_from_c.iter().find(|(t, _)| t.id == x.id).unwrap().1;
        assert_eq!(x_depth, 2);
    }

    #[test]
    fn test_dependency_chain_invalid_direction() {
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "A");
        let result = db.get_dependency_chain(&a.id, "sideways");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid direction"));
    }

    #[test]
    fn test_dependency_chain_both_direction() {
        // B depends on A, C depends on B
        // From B with "both": should see A (up@1) and C (down@1)
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "A");
        let b = make_task(&db, "B");
        let c = make_task(&db, "C");
        db.add_dependency(&b.id, &a.id).expect("B dep A");
        db.add_dependency(&c.id, &b.id).expect("C dep B");

        let both = db.get_dependency_chain(&b.id, "both").expect("both");
        assert_eq!(both.len(), 2);
        let ids: Vec<&str> = both.iter().map(|(t, _)| t.id.as_str()).collect();
        assert!(ids.contains(&a.id.as_str()));
        assert!(ids.contains(&c.id.as_str()));
    }

    #[test]
    fn test_list_tasks_completed_after_none_returns_all() {
        // completed_after = None should not change results vs. the baseline query
        let (db, _dir) = open_test_db();
        let a = make_task(&db, "Task A");
        let b = make_task(&db, "Task B");

        let all = db
            .list_tasks(true, None, None, None, None, None, None)
            .expect("list_tasks with no filter");

        let ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&a.id.as_str()), "Task A should be returned");
        assert!(ids.contains(&b.id.as_str()), "Task B should be returned");
    }

    #[test]
    fn test_list_tasks_completed_after_filters_by_updated_at() {
        // Only tasks with updated_at > completed_after should be returned
        let (db, _dir) = open_test_db();

        // Insert a task and immediately close it (sets updated_at to now)
        let task = make_task(&db, "Recent Done Task");
        db.update_task(&task.id, None, None, Some("done"), None, None, None, None)
            .expect("close task");

        // A timestamp well in the past — the task updated_at should be after this
        let past_ts = "2000-01-01T00:00:00+00:00";
        // A timestamp well in the future — the task updated_at should be before this
        let future_ts = "2099-01-01T00:00:00+00:00";

        let after_past = db
            .list_tasks(true, Some("done"), None, None, None, None, Some(past_ts))
            .expect("list_tasks completed_after past");
        let ids_past: Vec<&str> = after_past.iter().map(|t| t.id.as_str()).collect();
        assert!(
            ids_past.contains(&task.id.as_str()),
            "done task should be returned when completed_after is in the past"
        );

        let after_future = db
            .list_tasks(true, Some("done"), None, None, None, None, Some(future_ts))
            .expect("list_tasks completed_after future");
        let ids_future: Vec<&str> = after_future.iter().map(|t| t.id.as_str()).collect();
        assert!(
            !ids_future.contains(&task.id.as_str()),
            "done task should NOT be returned when completed_after is in the future"
        );
    }
}
