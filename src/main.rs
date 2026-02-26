mod commands;
mod db;
mod models;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tk",
    version,
    about = "Lightweight task manager for AI coding agents"
)]
struct Cli {
    /// Path to the database file (default: .tacks/tacks.db in current dir)
    #[arg(long, env = "TACKS_DB")]
    db: Option<PathBuf>,

    /// Output as JSON instead of table
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize tacks in the current directory
    Init {
        /// Task ID prefix (default: "tk")
        #[arg(long, default_value = "tk")]
        prefix: String,
    },
    /// Create a new task
    Create {
        /// Task title
        title: String,
        /// Priority (0=critical, 1=high, 2=medium, 3=low)
        #[arg(short, long, default_value_t = 2)]
        priority: u8,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
        /// Parent task ID (creates subtask)
        #[arg(long)]
        parent: Option<String>,
    },
    /// List tasks (default: open tasks)
    List {
        /// Show all tasks including closed
        #[arg(short, long)]
        all: bool,
        /// Filter by status (open, in_progress, done, blocked)
        #[arg(short, long)]
        status: Option<String>,
        /// Filter by priority
        #[arg(short, long)]
        priority: Option<u8>,
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Filter by parent task ID
        #[arg(long)]
        parent: Option<String>,
    },
    /// Show tasks that are ready to work on (no open blockers)
    Ready {
        /// Limit output to N tasks
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Show task counts by status, priority, and tag
    Stats {
        /// Output a compact single-line summary
        #[arg(long)]
        oneline: bool,
    },
    /// Output an AI-optimized context summary for session bootstrapping
    Prime,
    /// Show detailed info for a task
    Show {
        /// Task ID
        id: String,
    },
    /// Update a task
    Update {
        /// Task ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New priority
        #[arg(short, long)]
        priority: Option<u8>,
        /// New status (open, in_progress, done, blocked)
        #[arg(short, long)]
        status: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// Claim task (set assignee + in_progress)
        #[arg(long)]
        claim: bool,
        /// Assignee name
        #[arg(long)]
        assignee: Option<String>,
        /// Tags to add (comma-separated)
        #[arg(long)]
        add_tags: Option<String>,
        /// Tags to remove (comma-separated)
        #[arg(long)]
        remove_tags: Option<String>,
        /// Working notes (overwrites previous value)
        #[arg(long)]
        notes: Option<String>,
    },
    /// Close a task
    Close {
        /// Task ID
        id: String,
        /// Closing comment
        #[arg(short, long)]
        comment: Option<String>,
        /// Close reason (done, duplicate, absorbed, stale, superseded)
        #[arg(short, long, default_value = "done")]
        reason: String,
        /// Force close even if open dependents exist
        #[arg(long)]
        force: bool,
    },
    /// List child tasks of a parent
    Children {
        /// Parent task ID
        id: String,
    },
    /// Show epic progress (tasks tagged as epic with child completion stats)
    Epic,
    /// Add a dependency between tasks
    Dep {
        #[command(subcommand)]
        action: DepAction,
    },
    /// Add a comment to a task
    Comment {
        /// Task ID
        id: String,
        /// Comment text
        body: String,
    },
    /// Show blocked tasks (tasks with open blockers)
    Blocked,
}

#[derive(Subcommand)]
enum DepAction {
    /// Add a dependency (child is blocked by parent)
    Add {
        /// Task that is blocked
        child: String,
        /// Task that blocks
        parent: String,
    },
    /// Remove a dependency
    Remove {
        /// Task that was blocked
        child: String,
        /// Task that was blocking
        parent: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let db_path = cli.db.unwrap_or_else(|| {
        let mut p = std::env::current_dir().expect("cannot determine current directory");
        p.push(".tacks");
        p.push("tacks.db");
        p
    });

    let result = match cli.command {
        Commands::Init { prefix } => commands::init::run(&db_path, &prefix),
        Commands::Create {
            title,
            priority,
            description,
            tags,
            parent,
        } => commands::create::run(
            &db_path,
            &title,
            priority,
            description.as_deref(),
            tags.as_deref(),
            parent.as_deref(),
            cli.json,
        ),
        Commands::List {
            all,
            status,
            priority,
            tag,
            parent,
        } => commands::list::run(
            &db_path,
            all,
            status.as_deref(),
            priority,
            tag.as_deref(),
            parent.as_deref(),
            cli.json,
        ),
        Commands::Ready { limit } => commands::ready::run(&db_path, limit, cli.json),
        Commands::Stats { oneline } => commands::stats::run(&db_path, oneline, cli.json),
        Commands::Prime => commands::prime::run(&db_path, cli.json),
        Commands::Show { id } => commands::show::run(&db_path, &id, cli.json),
        Commands::Update {
            id,
            title,
            priority,
            status,
            description,
            claim,
            assignee,
            add_tags,
            remove_tags,
            notes,
        } => commands::update::run(
            &db_path,
            &id,
            title.as_deref(),
            priority,
            status.as_deref(),
            description.as_deref(),
            claim,
            assignee.as_deref(),
            add_tags.as_deref(),
            remove_tags.as_deref(),
            notes.as_deref(),
            cli.json,
        ),
        Commands::Close {
            id,
            comment,
            reason,
            force,
        } => commands::close::run(
            &db_path,
            &id,
            comment.as_deref(),
            Some(&reason),
            force,
            cli.json,
        ),
        Commands::Children { id } => commands::children::run(&db_path, &id, cli.json),
        Commands::Epic => commands::epic::run(&db_path, cli.json),
        Commands::Dep { action } => match action {
            DepAction::Add { child, parent } => commands::dep::add(&db_path, &child, &parent),
            DepAction::Remove { child, parent } => commands::dep::remove(&db_path, &child, &parent),
        },
        Commands::Comment { id, body } => commands::comment::run(&db_path, &id, &body, cli.json),
        Commands::Blocked => commands::blocked::run(&db_path, cli.json),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
