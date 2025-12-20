//! Guts CLI - Command-line interface for Guts.

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

/// Guts - Decentralized code collaboration
#[derive(Parser, Debug)]
#[command(name = "guts")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
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

    /// Show status
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
}

fn main() {
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

    let result = match cli.command {
        Commands::Init { name, path } => commands::init(&name, path.as_deref()),
        Commands::Clone { url, path } => commands::clone(&url, path.as_deref()),
        Commands::Identity { command } => match command {
            IdentityCommands::Generate { output } => commands::identity_generate(output.as_deref()),
            IdentityCommands::Show => commands::identity_show(),
        },
        Commands::Status => commands::status(),
        Commands::Version => {
            println!("guts {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
