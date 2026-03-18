use anyhow::Result;

use crate::profile::manager::ProfileManager;

pub fn execute(manager: &ProfileManager, profile: &str) -> Result<()> {
    let content = manager.get_profile(profile)?;

    println!("Profile: {}", profile);
    println!("{}", serde_json::to_string_pretty(&content)?);

    Ok(())
}
