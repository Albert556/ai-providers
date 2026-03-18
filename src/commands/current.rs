use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager) -> Result<()> {
    let current = manager.get_current_profile()?;

    match current {
        Some(profile) => {
            println!("Current profile: {}", profile.green().bold());
        }
        None => {
            println!("No profile is currently active");
        }
    }

    Ok(())
}
