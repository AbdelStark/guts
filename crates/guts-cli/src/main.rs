//! # Guts CLI
//!
//! Command-line interface for interacting with Guts.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

/// Guts - Decentralized code collaboration
#[derive(Parser, Debug)]
#[command(name = "guts")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new repository
    Init {
        /// Repository name
        name: String,
        /// Path to initialize (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Clone a repository
    Clone {
        /// Repository URL or ID
        url: String,
        /// Destination path
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Manage identity
    Identity {
        #[command(subcommand)]
        command: IdentityCommands,
    },

    /// Show node status
    Status,

    /// Show version information
    Version,
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    /// Generate a new identity
    Generate {
        /// Output path for the keypair
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show current identity
    Show,

    /// Export identity
    Export {
        /// Output path
        #[arg(short, long)]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("guts={log_level}").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Init { name, path } => {
            commands::init(&name, path.as_deref())?;
        }
        Commands::Clone { url, path } => {
            commands::clone(&url, path.as_deref()).await?;
        }
        Commands::Identity { command } => match command {
            IdentityCommands::Generate { output } => {
                commands::identity_generate(output.as_deref())?;
            }
            IdentityCommands::Show => {
                commands::identity_show()?;
            }
            IdentityCommands::Export { output } => {
                commands::identity_export(&output)?;
            }
        },
        Commands::Status => {
            commands::status().await?;
        }
        Commands::Version => {
            println!("guts {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
