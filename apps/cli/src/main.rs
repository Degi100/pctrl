use clap::{Parser, Subcommand};
use pctrl_core::{Config, Mode};
use pctrl_database::Database;
use std::path::PathBuf;
use std::sync::Arc;

mod cli;
mod tui;

/// Default database path
fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pctrl")
        .join("pctrl.db")
}

#[derive(Parser)]
#[command(name = "pctrl")]
#[command(about = "Mission Control for Self-Hosters & Indie Devs", long_about = None)]
#[command(version)]
struct Cli {
    /// Operation mode
    #[arg(short, long, value_enum, default_value = "cli")]
    mode: CliMode,

    /// Database path (default: ~/.local/share/pctrl/pctrl.db)
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, clap::ValueEnum)]
enum CliMode {
    Cli,
    Tui,
    Gui,
}

impl From<CliMode> for Mode {
    fn from(mode: CliMode) -> Self {
        match mode {
            CliMode::Cli => Mode::Cli,
            CliMode::Tui => Mode::Tui,
            CliMode::Gui => Mode::Gui,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// SSH connection management
    Ssh {
        #[command(subcommand)]
        command: SshCommands,
    },
    /// Docker container management
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    /// Coolify deployment management
    Coolify {
        #[command(subcommand)]
        command: CoolifyCommands,
    },
    /// Git release management
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
}

#[derive(Subcommand)]
enum SshCommands {
    /// List SSH connections
    List,
    /// Add a new SSH connection
    Add {
        /// Connection name (used as ID)
        name: String,
        /// Host address
        host: String,
        /// Username
        #[arg(short, long)]
        user: String,
        /// Port (default: 22)
        #[arg(short, long, default_value = "22")]
        port: u16,
        /// Path to private key (default: ~/.ssh/id_rsa)
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Remove an SSH connection
    Remove {
        /// Connection ID to remove
        id: String,
    },
    /// Connect to an SSH host
    Connect { id: String },
    /// Execute command on remote host
    Exec { id: String, command: String },
}

#[derive(Subcommand)]
enum DockerCommands {
    /// List Docker containers
    List { host_id: String },
    /// Start a container
    Start {
        host_id: String,
        container_id: String,
    },
    /// Stop a container
    Stop {
        host_id: String,
        container_id: String,
    },
}

#[derive(Subcommand)]
enum CoolifyCommands {
    /// List deployments
    List { instance_id: String },
    /// Deploy a project
    Deploy {
        instance_id: String,
        project_id: String,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// List releases/tags
    List { repo_id: String },
    /// Create a new release
    Create {
        repo_id: String,
        tag: String,
        message: String,
    },
    /// Push tags to remote
    Push { repo_id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let mode: Mode = cli.mode.into();

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Database initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let db_path = cli.db.unwrap_or_else(default_db_path);

    // Stelle sicher dass das Verzeichnis existiert
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::new(db_path.to_str().unwrap_or("pctrl.db"), None)
        .await
        .map_err(|e| anyhow::anyhow!("Database init failed: {}", e))?;

    let db = Arc::new(db);

    // ─────────────────────────────────────────────────────────────────────────
    // 2. Config laden
    // ─────────────────────────────────────────────────────────────────────────
    let config = db
        .load_config()
        .await
        .unwrap_or_else(|_| Config::default());

    let config = Arc::new(config);

    // ─────────────────────────────────────────────────────────────────────────
    // 3. Mode auswählen
    // ─────────────────────────────────────────────────────────────────────────
    match mode {
        Mode::Cli => {
            if let Some(command) = cli.command {
                cli::handle_command(command, config.clone(), db.clone()).await?;
            } else {
                println!("pctrl - Mission Control for Self-Hosters & Indie Devs");
                println!("Use --help for more information");
                println!();
                println!("Database: {}", db_path.display());
                println!("SSH Connections: {}", config.ssh_connections.len());
                println!("Docker Hosts: {}", config.docker_hosts.len());
                println!("Coolify Instances: {}", config.coolify_instances.len());
                println!("Git Repos: {}", config.git_repos.len());
            }
        }
        Mode::Tui => {
            tui::run(config.clone(), db.clone()).await?;
        }
        Mode::Gui => {
            println!("GUI mode requires the desktop application (Tauri)");
            println!("Run: cd apps/desktop && npm run tauri dev");
        }
    }

    Ok(())
}
