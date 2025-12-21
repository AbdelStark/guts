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

    /// Manage pull requests
    #[command(name = "pr")]
    PullRequest {
        #[command(subcommand)]
        command: PullRequestCommands,
    },

    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },

    /// Manage workflows (CI/CD)
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },

    /// Manage workflow runs (CI/CD)
    Run {
        #[command(subcommand)]
        command: RunCommands,
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

#[derive(Subcommand, Debug)]
enum PullRequestCommands {
    /// List pull requests
    List {
        /// Filter by state (open, closed, merged, all)
        #[arg(short, long, default_value = "open")]
        state: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Create a new pull request
    Create {
        /// Pull request title
        #[arg(short, long)]
        title: String,
        /// Pull request description
        #[arg(short, long, default_value = "")]
        body: String,
        /// Source branch
        #[arg(long)]
        source: String,
        /// Target branch
        #[arg(long, default_value = "main")]
        target: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Show pull request details
    Show {
        /// Pull request number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Merge a pull request
    Merge {
        /// Pull request number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Close a pull request
    Close {
        /// Pull request number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },
}

#[derive(Subcommand, Debug)]
enum IssueCommands {
    /// List issues
    List {
        /// Filter by state (open, closed, all)
        #[arg(short, long, default_value = "open")]
        state: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Create a new issue
    Create {
        /// Issue title
        #[arg(short, long)]
        title: String,
        /// Issue description
        #[arg(short, long, default_value = "")]
        body: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Show issue details
    Show {
        /// Issue number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Close an issue
    Close {
        /// Issue number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Reopen an issue
    Reopen {
        /// Issue number
        number: u32,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },
}

#[derive(Subcommand, Debug)]
enum WorkflowCommands {
    /// List workflows
    List {
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Show workflow details
    Show {
        /// Workflow ID
        id: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Register a workflow from YAML file
    Register {
        /// Path to workflow YAML file
        #[arg(short, long)]
        file: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },
}

#[derive(Subcommand, Debug)]
enum RunCommands {
    /// List workflow runs
    List {
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
        /// Filter by workflow ID
        #[arg(short, long)]
        workflow: Option<String>,
    },

    /// Show run details
    Show {
        /// Run ID
        id: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Trigger a workflow run
    Trigger {
        /// Workflow ID
        workflow_id: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
        /// Git reference (branch or tag)
        #[arg(long, default_value = "main")]
        ref_name: String,
        /// Commit SHA
        #[arg(long)]
        sha: Option<String>,
    },

    /// Cancel a workflow run
    Cancel {
        /// Run ID
        id: String,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },

    /// Show run logs
    Logs {
        /// Run ID
        id: String,
        /// Job name (optional)
        #[arg(short, long)]
        job: Option<String>,
        /// Node API URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        node: String,
        /// Repository (owner/name)
        #[arg(short, long)]
        repo: String,
    },
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
        Commands::PullRequest { command } => match command {
            PullRequestCommands::List { state, node, repo } => {
                commands::pr_list(&node, &repo, &state)
            }
            PullRequestCommands::Create {
                title,
                body,
                source,
                target,
                node,
                repo,
            } => commands::pr_create(&node, &repo, &title, &body, &source, &target),
            PullRequestCommands::Show { number, node, repo } => {
                commands::pr_show(&node, &repo, number)
            }
            PullRequestCommands::Merge { number, node, repo } => {
                commands::pr_merge(&node, &repo, number)
            }
            PullRequestCommands::Close { number, node, repo } => {
                commands::pr_close(&node, &repo, number)
            }
        },
        Commands::Issue { command } => match command {
            IssueCommands::List { state, node, repo } => commands::issue_list(&node, &repo, &state),
            IssueCommands::Create {
                title,
                body,
                node,
                repo,
            } => commands::issue_create(&node, &repo, &title, &body),
            IssueCommands::Show { number, node, repo } => {
                commands::issue_show(&node, &repo, number)
            }
            IssueCommands::Close { number, node, repo } => {
                commands::issue_close(&node, &repo, number)
            }
            IssueCommands::Reopen { number, node, repo } => {
                commands::issue_reopen(&node, &repo, number)
            }
        },
        Commands::Workflow { command } => match command {
            WorkflowCommands::List { node, repo } => commands::workflow_list(&node, &repo),
            WorkflowCommands::Show { id, node, repo } => commands::workflow_show(&node, &repo, &id),
            WorkflowCommands::Register { file, node, repo } => {
                commands::workflow_register(&node, &repo, &file)
            }
        },
        Commands::Run { command } => match command {
            RunCommands::List {
                node,
                repo,
                workflow,
            } => commands::run_list(&node, &repo, workflow.as_deref()),
            RunCommands::Show { id, node, repo } => commands::run_show(&node, &repo, &id),
            RunCommands::Trigger {
                workflow_id,
                node,
                repo,
                ref_name,
                sha,
            } => commands::run_trigger(&node, &repo, &workflow_id, &ref_name, sha.as_deref()),
            RunCommands::Cancel { id, node, repo } => commands::run_cancel(&node, &repo, &id),
            RunCommands::Logs {
                id,
                job,
                node,
                repo,
            } => commands::run_logs(&node, &repo, &id, job.as_deref()),
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
