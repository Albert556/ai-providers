use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod config;
mod profile;

#[derive(Parser)]
#[command(name = "aip")]
#[command(about = "AI Providers - Manage Claude Code configuration profiles")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all profiles
    #[command(alias = "ls")]
    List,
    /// Show current active profile
    Current,
    /// Show profile details
    Show {
        /// Profile name
        profile: String,
    },
    /// Show current Claude Code configuration
    Config,
    /// Add a new profile
    Add {
        /// Profile name
        profile: String,
        /// Copy from existing profile
        #[arg(short, long)]
        from: Option<String>,
        /// Create empty profile (default: copy from current config)
        #[arg(short, long)]
        empty: bool,
    },
    /// Delete a profile
    Delete {
        /// Profile name
        profile: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Edit a profile
    Edit {
        /// Profile name
        profile: String,
    },
    /// Switch to a profile
    Use {
        /// Profile name
        profile: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list::execute()?,
        Commands::Current => commands::current::execute()?,
        Commands::Show { profile } => commands::show::execute(&profile)?,
        Commands::Config => commands::config::execute()?,
        Commands::Add { profile, from, empty } => commands::add::execute(&profile, from, empty)?,
        Commands::Delete { profile, force } => commands::delete::execute(&profile, force)?,
        Commands::Edit { profile } => commands::edit::execute(&profile)?,
        Commands::Use { profile } => commands::use_cmd::execute(&profile)?,
    }

    Ok(())
}

