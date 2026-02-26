use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Valid close reasons for a task.
pub const VALID_CLOSE_REASONS: &[&str] = &["done", "duplicate", "absorbed", "stale", "superseded"];

/// Validate a close reason string.
pub fn validate_close_reason(reason: &str) -> Result<(), String> {
    if VALID_CLOSE_REASONS.contains(&reason) {
        Ok(())
    } else {
        Err(format!(
            "invalid close reason: {reason}. valid reasons: {}",
            VALID_CLOSE_REASONS.join(", ")
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    InProgress,
    Done,
    Blocked,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in_progress",
            Status::Done => "done",
            Status::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Status::Open),
            "in_progress" | "in-progress" | "inprogress" => Ok(Status::InProgress),
            "done" | "closed" => Ok(Status::Done),
            "blocked" => Ok(Status::Blocked),
            _ => Err(format!("unknown status: {s}")),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: u8,
    pub assignee: Option<String>,
    pub parent_id: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub close_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub task_id: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub child_id: String,
    pub parent_id: String,
}
