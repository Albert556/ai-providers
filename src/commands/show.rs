use anyhow::Result;

use crate::profile::manager::ProfileManager;
use crate::profile::storage;

pub fn execute(manager: &ProfileManager, profile: &str, merged: bool) -> Result<()> {
    let content = manager.get_profile(profile)?;

    if merged {
        if let Some(common) = manager.get_common_config()? {
            let merged_content = storage::deep_merge(&common, &content);
            println!("Profile: {} (merged with common)", profile);
            println!("{}", serde_json::to_string_pretty(&merged_content)?);
        } else {
            println!("Profile: {} (no common config found)", profile);
            println!("{}", serde_json::to_string_pretty(&content)?);
        }
    } else {
        println!("Profile: {}", profile);
        println!("{}", serde_json::to_string_pretty(&content)?);
    }

    Ok(())
}
