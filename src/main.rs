use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod profile;
mod provider;
mod tui;
mod util;

use profile::manager::ProfileManager;
use provider::claude::ClaudeProvider;

#[derive(Parser)]
#[command(name = "aip")]
#[command(about = "AI Providers - Manage AI tool configuration profiles")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<ProviderCommand>,
}

#[derive(Subcommand)]
enum ProviderCommand {
    /// Manage Claude Code profiles
    Claude {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Launch interactive TUI
    Tui,
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all profiles
    #[command(alias = "ls")]
    List,
    /// Show current active profile
    Current,
    /// Show profile details
    Show {
        /// Profile name
        profile: String,
        /// Show merged result with common config
        #[arg(short, long)]
        merged: bool,
    },
    /// Show current configuration file content
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
    /// Edit a profile with $EDITOR
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
        None | Some(ProviderCommand::Tui) => {
            let provider = ClaudeProvider;
            let manager = ProfileManager::new(&provider)?;
            tui::run_tui(&manager)?;
        }
        Some(ProviderCommand::Claude { command }) => {
            let provider = ClaudeProvider;
            let manager = ProfileManager::new(&provider)?;
            handle_profile_command(&manager, command)?;
        }
    }

    Ok(())
}

fn handle_profile_command(manager: &ProfileManager, command: ProfileCommands) -> Result<()> {
    match command {
        ProfileCommands::List => commands::list::execute(manager),
        ProfileCommands::Current => commands::current::execute(manager),
        ProfileCommands::Show { profile, merged } => {
            commands::show::execute(manager, &profile, merged)
        }
        ProfileCommands::Config => commands::config::execute(manager),
        ProfileCommands::Add {
            profile,
            from,
            empty,
        } => commands::add::execute(manager, &profile, from, empty),
        ProfileCommands::Delete { profile, force } => {
            commands::delete::execute(manager, &profile, force)
        }
        ProfileCommands::Edit { profile } => commands::edit::execute(manager, &profile),
        ProfileCommands::Use { profile } => commands::use_cmd::execute(manager, &profile),
    }
}
