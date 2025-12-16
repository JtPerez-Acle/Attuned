//! Attuned CLI tool for development and testing.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "attuned")]
#[command(about = "CLI tool for Attuned development and testing")]
#[command(version)]
struct Cli {
    /// Server URL for remote operations.
    #[arg(long, env = "ATTUNED_SERVER")]
    server: Option<String>,

    /// Output format.
    #[arg(long, default_value = "pretty")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    /// Raw JSON output.
    Json,
    /// Human-readable formatted output.
    #[default]
    Pretty,
    /// Minimal output for scripts.
    Quiet,
}

#[derive(Subcommand)]
enum Commands {
    /// State management commands.
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    /// Translate state to context.
    Translate {
        /// User ID to translate.
        user_id: String,
    },
    /// List available axes.
    Axes,
    /// Start a local server.
    Serve {
        /// Port to listen on.
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Check server health.
    Health,
}

#[derive(Subcommand)]
enum StateCommands {
    /// Get state for a user.
    Get {
        /// User ID.
        user_id: String,
    },
    /// Set axis values for a user.
    Set {
        /// User ID.
        user_id: String,
        /// Axis values in the form axis=value.
        #[arg(short, long)]
        axis: Vec<String>,
    },
    /// Delete state for a user.
    Delete {
        /// User ID.
        user_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::State { command } => match command {
            StateCommands::Get { user_id } => {
                println!("Getting state for user: {}", user_id);
                // TODO: Implement (TASK-007)
            }
            StateCommands::Set { user_id, axis } => {
                println!("Setting state for user: {} with axes: {:?}", user_id, axis);
                // TODO: Implement (TASK-007)
            }
            StateCommands::Delete { user_id } => {
                println!("Deleting state for user: {}", user_id);
                // TODO: Implement (TASK-007)
            }
        },
        Commands::Translate { user_id } => {
            println!("Translating state for user: {}", user_id);
            // TODO: Implement (TASK-007)
        }
        Commands::Axes => {
            println!("Available axes:");
            for axis in attuned_core::CANONICAL_AXES {
                println!("  {} ({}): {}", axis.name, axis.category, axis.description);
            }
        }
        Commands::Serve { port } => {
            println!("Starting server on port {}...", port);
            // TODO: Implement (TASK-007)
        }
        Commands::Health => {
            if let Some(server) = cli.server {
                println!("Checking health of server: {}", server);
            } else {
                println!("No server specified, checking local health...");
            }
            // TODO: Implement (TASK-007)
        }
    }

    Ok(())
}
