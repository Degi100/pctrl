use clap::{Parser, Subcommand};
use pctrl_core::{Config, Mode};
use pctrl_database::Database;
use std::path::PathBuf;
use std::sync::Arc;

mod cli;
mod style;
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
    /// Operation mode (default: tui for interactive mode, use -m cli for single commands)
    #[arg(short, long, value_enum, default_value = "tui")]
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
pub enum Commands {
    // ═══════════════════════════════════════════════════════════════════════════
    // v6: PROJECT-CENTRIC COMMANDS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Project management (v6 core)
    #[command(alias = "p")]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Server management (v6)
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },

    /// Domain management (v6)
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },

    /// Database credentials management (v6)
    #[command(alias = "db")]
    Database {
        #[command(subcommand)]
        command: DatabaseCommands,
    },

    /// Script management (v6)
    Script {
        #[command(subcommand)]
        command: ScriptCommands,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // LEGACY COMMANDS (still supported)
    // ═══════════════════════════════════════════════════════════════════════════
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

// ═══════════════════════════════════════════════════════════════════════════════
// v6: PROJECT COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List all projects
    List,
    /// Add a new project
    Add {
        /// Project name
        name: String,
        /// Project description
        #[arg(short, long)]
        description: Option<String>,
        /// Tech stack (comma-separated, e.g., "rust,tauri,react")
        #[arg(short, long)]
        stack: Option<String>,
        /// Status: dev, staging, live, archived
        #[arg(long, default_value = "dev")]
        status: String,
    },
    /// Show project details
    Show {
        /// Project name or ID
        name: String,
    },
    /// Remove a project
    Remove {
        /// Project name or ID
        name: String,
    },
    /// Link a resource to a project
    Link {
        /// Project name or ID
        project: String,
        /// Resource type: server, container, database, domain, git, coolify, script
        resource_type: String,
        /// Resource ID
        resource_id: String,
        /// Role description (e.g., "production_db", "staging_server")
        #[arg(short, long)]
        role: Option<String>,
    },
    /// Unlink a resource from a project
    Unlink {
        /// Project name or ID
        project: String,
        /// Resource link ID
        link_id: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: SERVER COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum ServerCommands {
    /// List all servers
    List,
    /// Add a new server
    Add {
        /// Server name
        name: String,
        /// Server host (IP or hostname)
        host: String,
        /// Server type: vps, dedicated, local, cloud
        #[arg(short = 't', long, default_value = "vps")]
        server_type: String,
        /// Provider (e.g., hetzner, aws, digitalocean)
        #[arg(short, long)]
        provider: Option<String>,
        /// SSH connection ID to use
        #[arg(short, long)]
        ssh: Option<String>,
        /// Location (e.g., "Falkenstein, DE")
        #[arg(short, long)]
        location: Option<String>,
    },
    /// Show server details
    Show {
        /// Server name or ID
        name: String,
    },
    /// Remove a server
    Remove {
        /// Server name or ID
        name: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: DOMAIN COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum DomainCommands {
    /// List all domains
    List,
    /// Add a new domain
    Add {
        /// Domain name (e.g., app.example.com)
        domain: String,
        /// Domain type: production, staging, dev
        #[arg(short = 't', long, default_value = "production")]
        domain_type: String,
        /// Server ID this domain points to
        #[arg(short, long)]
        server: Option<String>,
        /// SSL enabled
        #[arg(long, default_value = "true")]
        ssl: bool,
    },
    /// Show domain details
    Show {
        /// Domain name
        domain: String,
    },
    /// Remove a domain
    Remove {
        /// Domain name
        domain: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: DATABASE COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum DatabaseCommands {
    /// List all database credentials
    List,
    /// Add database credentials
    Add {
        /// Database name (for display)
        name: String,
        /// Database type: mongodb, postgres, mysql, redis, sqlite
        #[arg(short = 't', long)]
        db_type: String,
        /// Database host
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// Database port
        #[arg(short, long)]
        port: Option<u16>,
        /// Database name
        #[arg(short = 'd', long)]
        database: Option<String>,
        /// Username
        #[arg(short, long)]
        user: Option<String>,
        /// Password
        #[arg(short = 'P', long)]
        password: Option<String>,
        /// Connection string (alternative to individual fields)
        #[arg(short, long)]
        connection_string: Option<String>,
    },
    /// Show database credentials
    Show {
        /// Database name or ID
        name: String,
    },
    /// Get specific field (user, pass, url)
    Get {
        /// Database name or ID
        name: String,
        /// Field to get: user, pass, url, host, port, database
        field: String,
    },
    /// Remove database credentials
    Remove {
        /// Database name or ID
        name: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: SCRIPT COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum ScriptCommands {
    /// List all scripts
    List,
    /// Add a new script
    Add {
        /// Script name
        name: String,
        /// Command to execute
        #[arg(short, long)]
        command: String,
        /// Script description
        #[arg(short, long)]
        description: Option<String>,
        /// Script type: ssh, local, docker
        #[arg(short = 't', long, default_value = "ssh")]
        script_type: String,
        /// Server ID to run on
        #[arg(short, long)]
        server: Option<String>,
        /// Project ID (optional)
        #[arg(short, long)]
        project: Option<String>,
        /// Mark as dangerous (requires confirmation)
        #[arg(long)]
        dangerous: bool,
    },
    /// Show script details
    Show {
        /// Script name or ID
        name: String,
    },
    /// Run a script
    Run {
        /// Script name or ID
        name: String,
        /// Force run without confirmation (for dangerous scripts)
        #[arg(short, long)]
        force: bool,
    },
    /// Remove a script
    Remove {
        /// Script name or ID
        name: String,
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
    /// List configured Docker hosts
    Hosts,
    /// Add a new Docker host
    Add {
        /// Host name (used as ID)
        name: String,
        /// Docker socket URL (e.g., unix:///var/run/docker.sock or tcp://localhost:2375)
        #[arg(short, long, default_value = "unix:///var/run/docker.sock")]
        url: String,
    },
    /// Remove a Docker host
    Remove { id: String },
    /// List containers on a host
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
    /// List configured Coolify instances
    Instances,
    /// Add a new Coolify instance
    Add {
        /// Instance name (used as ID)
        name: String,
        /// Coolify URL (e.g., https://coolify.example.com)
        #[arg(short, long)]
        url: String,
        /// API token
        #[arg(short, long)]
        token: String,
    },
    /// Remove a Coolify instance
    Remove { id: String },
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
    /// List configured Git repositories
    Repos,
    /// Add a Git repository
    Add {
        /// Repository name (used as ID)
        name: String,
        /// Path to local repository
        #[arg(short, long)]
        path: String,
    },
    /// Remove a Git repository
    Remove { id: String },
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
    let config = db.load_config().await.unwrap_or_else(|_| Config::default());

    let config = Arc::new(config);

    // ─────────────────────────────────────────────────────────────────────────
    // 3. Mode auswählen
    // ─────────────────────────────────────────────────────────────────────────

    // If a subcommand is provided, always use CLI mode to handle it
    if let Some(command) = cli.command {
        cli::handle_command(command, config.clone(), db.clone()).await?;
    } else {
        // No subcommand - use the specified mode (default: TUI)
        match mode {
            Mode::Cli => {
                // Styled status display
                style::print_banner(env!("CARGO_PKG_VERSION"));

                println!("  {}Status{}", style::BOLD, style::RESET);
                style::divider();
                style::kv_count("SSH Connections", config.ssh_connections.len());
                style::kv_count("Docker Hosts", config.docker_hosts.len());
                style::kv_count("Coolify Instances", config.coolify_instances.len());
                style::kv_count("Git Repos", config.git_repos.len());
                style::divider();
                println!(
                    "  {}Database:{} {}",
                    style::GRAY,
                    style::RESET,
                    style::format_path(&db_path.display().to_string(), 40)
                );
                println!();
                println!(
                    "  {}Use {}pctrl --help{} for available commands{}",
                    style::DIM,
                    style::CYAN,
                    style::DIM,
                    style::RESET
                );
                println!();
            }
            Mode::Tui => {
                tui::run(config.clone(), db.clone()).await?;
            }
            Mode::Gui => {
                println!("GUI mode requires the desktop application (Tauri)");
                println!("Run: cd apps/desktop && npm run tauri dev");
            }
        }
    }

    Ok(())
}
