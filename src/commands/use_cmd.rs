use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute(profile: &str) -> Result<()> {
    let manager = ProfileManager::new()?;

    manager.use_profile(profile)?;

    println!("{}", format!("✓ Switched to profile '{}'", profile).green());

    Ok(())
}
