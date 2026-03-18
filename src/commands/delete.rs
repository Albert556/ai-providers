use anyhow::Result;
use colored::*;
use std::io::{self, Write};

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager, profile: &str, force: bool) -> Result<()> {
    if !manager.profile_exists(profile) {
        println!("{}", format!("Profile '{}' not found", profile).red());
        return Ok(());
    }

    let current = manager.get_current_profile()?;
    if current.as_deref() == Some(profile) {
        println!(
            "{}",
            format!("Warning: '{}' is the currently active profile", profile).yellow()
        );
    }

    if !force {
        print!(
            "Are you sure you want to delete profile '{}'? (y/n): ",
            profile
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled");
            return Ok(());
        }
    }

    manager.delete_profile(profile)?;
    println!(
        "{}",
        format!("Profile '{}' deleted successfully", profile).green()
    );

    Ok(())
}
