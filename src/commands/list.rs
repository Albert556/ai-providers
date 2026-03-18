use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager) -> Result<()> {
    let profiles = manager.list_profiles()?;
    let current = manager.get_current_profile()?;

    if profiles.is_empty() {
        println!("No profiles found.");
        println!(
            "{}",
            format!(
                "Tip: Use 'aip {} add <name>' to create a new profile",
                manager.provider_name()
            )
            .blue()
        );
        return Ok(());
    }

    println!("Available profiles:");
    for profile in profiles {
        if current.as_deref() == Some(&profile) {
            println!(
                "  {} {}  {}",
                "*".green().bold(),
                profile.green().bold(),
                "(current)".green()
            );
        } else {
            println!("    {}", profile);
        }
    }

    Ok(())
}
