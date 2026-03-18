use anyhow::Result;

use crate::profile::manager::ProfileManager;

pub fn execute(profile: &str) -> Result<()> {
    let manager = ProfileManager::new()?;
    let content = manager.get_profile(profile)?;

    println!("Profile: {}", profile);
    println!("{}", serde_json::to_string_pretty(&content)?);

    Ok(())
}
