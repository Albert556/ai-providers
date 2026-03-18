use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute() -> Result<()> {
    let manager = ProfileManager::new()?;

    match manager.get_claude_config() {
        Ok(content) => {
            println!("Current Claude Code configuration:");
            println!("{}", serde_json::to_string_pretty(&content)?);
        }
        Err(_) => {
            println!("{}", "⚠ Claude Code configuration not found".yellow());
            println!("Expected location: ~/.claude/settings.json");
        }
    }

    Ok(())
}
