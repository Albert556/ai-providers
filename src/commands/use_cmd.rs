use anyhow::Result;
use colored::*;

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager, profile: &str) -> Result<()> {
    manager.use_profile(profile)?;

    println!("{}", format!("Switched to profile '{}'", profile).green());

    Ok(())
}
