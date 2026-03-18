use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager) -> Result<()> {
    match manager.get_active_config() {
        Ok(content) => {
            println!("Current configuration:");
            println!("{}", serde_json::to_string_pretty(&content)?);
        }
        Err(_) => {
            println!("{}", "Configuration file not found".yellow());
        }
    }

    Ok(())
}
