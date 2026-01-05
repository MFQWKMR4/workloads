use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::orchestrator;

#[derive(Parser)]
#[command(name = "wl", about = "Workload generator CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available runtimes and workload features on this host
    List,
    /// Generate workload processes based on the config
    Gen {
        /// Path to config.json
        #[arg(short = 'c', long = "config", default_value = "config.yaml")]
        config: PathBuf,
    },
    /// Generate local sample runtimes
    Samples {
        /// Output directory for samples
        #[arg(short = 'o', long = "output", default_value = "samples")]
        output: PathBuf,
    },
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => orchestrator::list_available(),
        Commands::Gen { config } => orchestrator::generate(&config),
        Commands::Samples { output } => orchestrator::samples(&output),
    }
}
