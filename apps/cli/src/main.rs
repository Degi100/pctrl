use clap::{Parser, Subcommand};
use pctrl_core::Mode;
use pctrl_database::Database;
use std::path::PathBuf;
use std::sync::Arc;

mod handlers;
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
    /// Project management
    #[command(alias = "p")]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Server management
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },

    /// Domain management
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },

    /// Database credentials management
    #[command(alias = "db")]
    Database {
        #[command(subcommand)]
        command: DatabaseCommands,
    },

    /// Script management
    Script {
        #[command(subcommand)]
        command: ScriptCommands,
    },

    /// Credential management (SSH keys, API tokens, etc.)
    #[command(alias = "cred")]
    Credential {
        #[command(subcommand)]
        command: CredentialCommands,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROJECT COMMANDS
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
        /// Resource type: server, container, database, domain, script
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
// SERVER COMMANDS
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
        /// Credential name/ID for SSH access
        #[arg(short, long)]
        credential: Option<String>,
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
    /// Execute a command on the server via SSH
    Exec {
        /// Server name or ID
        name: String,
        /// Command to execute
        command: String,
    },
    /// Check server status (connectivity, uptime)
    Status {
        /// Server name or ID
        name: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// DOMAIN COMMANDS
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
        /// SSL certificate expiry date (e.g., "2025-12-31")
        #[arg(long)]
        ssl_expiry: Option<String>,
        /// Cloudflare Zone ID
        #[arg(long)]
        cloudflare_zone: Option<String>,
        /// Cloudflare DNS Record ID
        #[arg(long)]
        cloudflare_record: Option<String>,
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
// DATABASE COMMANDS
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
        /// Server ID where this database runs
        #[arg(short, long)]
        server: Option<String>,
        /// Container ID (for dockerized databases)
        #[arg(long)]
        container: Option<String>,
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
// SCRIPT COMMANDS
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
        #[arg(short = 't', long, default_value = "local")]
        script_type: String,
        /// Server ID to run on (for ssh scripts)
        #[arg(short, long)]
        server: Option<String>,
        /// Project ID (optional)
        #[arg(short, long)]
        project: Option<String>,
        /// Docker host ID (for docker scripts)
        #[arg(long)]
        docker_host: Option<String>,
        /// Container ID/name (for docker scripts)
        #[arg(long)]
        container: Option<String>,
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

// ═══════════════════════════════════════════════════════════════════════════════
// CREDENTIAL COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
pub enum CredentialCommands {
    /// List all credentials
    List,
    /// Add a new credential
    Add {
        /// Credential name (e.g., "My SSH Key", "Coolify API")
        name: String,
        /// Credential type: ssh, api, basic, oauth
        #[arg(short = 't', long = "type")]
        cred_type: String,
        /// Username (for SSH and basic auth)
        #[arg(short, long)]
        user: Option<String>,
        /// Port (for SSH, default 22)
        #[arg(short, long)]
        port: Option<u16>,
        /// SSH key path (for SSH credentials)
        #[arg(short, long)]
        key: Option<String>,
        /// API token (for API/OAuth credentials)
        #[arg(long)]
        token: Option<String>,
        /// Password (for basic auth or SSH passphrase)
        #[arg(short = 'P', long)]
        password: Option<String>,
        /// URL (for API/OAuth/basic auth)
        #[arg(long)]
        url: Option<String>,
    },
    /// Show credential details
    Show {
        /// Credential name
        name: String,
    },
    /// Remove a credential
    Remove {
        /// Credential name
        name: String,
    },
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
    // 2. Mode auswählen
    // ─────────────────────────────────────────────────────────────────────────

    // If a subcommand is provided, always use CLI mode to handle it
    if let Some(command) = cli.command {
        handlers::handle_command(command, db.clone()).await?;
    } else {
        // No subcommand - use the specified mode (default: TUI)
        match mode {
            Mode::Cli => {
                // Styled status display
                style::print_banner(env!("CARGO_PKG_VERSION"));

                // Load v6 entity counts
                let projects = db.list_projects().await.unwrap_or_default();
                let servers = db.list_servers().await.unwrap_or_default();
                let domains = db.list_domains().await.unwrap_or_default();
                let databases = db.list_database_credentials().await.unwrap_or_default();
                let scripts = db.list_scripts().await.unwrap_or_default();
                let credentials = db.list_credentials().await.unwrap_or_default();

                println!("  {}Status{}", style::BOLD, style::RESET);
                style::divider();
                style::kv_count("Projects", projects.len());
                style::kv_count("Servers", servers.len());
                style::kv_count("Domains", domains.len());
                style::kv_count("Databases", databases.len());
                style::kv_count("Scripts", scripts.len());
                style::kv_count("Credentials", credentials.len());
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
                tui::run(db.clone()).await?;
            }
            Mode::Gui => {
                println!("GUI mode requires the desktop application (Tauri)");
                println!("Run: cd apps/desktop && npm run tauri dev");
            }
        }
    }

    Ok(())
}
