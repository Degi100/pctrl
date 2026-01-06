use clap::{Parser, Subcommand};
use pctrl_core::Mode;

mod cli;
mod tui;

#[derive(Parser)]
#[command(name = "pctrl")]
#[command(about = "Mission Control for Self-Hosters & Indie Devs", long_about = None)]
#[command(version)]
struct Cli {
    /// Operation mode
    #[arg(short, long, value_enum, default_value = "cli")]
    mode: CliMode,

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

    match mode {
        Mode::Cli => {
            if let Some(command) = cli.command {
                cli::handle_command(command).await?;
            } else {
                println!("pctrl - Mission Control for Self-Hosters & Indie Devs");
                println!("Use --help for more information");
            }
        }
        Mode::Tui => {
            tui::run().await?;
        }
        Mode::Gui => {
            println!("GUI mode requires the desktop application (Tauri)");
            println!("Run: cd apps/desktop && npm run tauri dev");
        }
    }

    Ok(())
}
