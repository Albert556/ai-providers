use anyhow::Result;
use colored::*;

use crate::profile::manager::{ProfileManager, ProfileSource};

pub fn execute(profile: &str, from: Option<String>, empty: bool) -> Result<()> {
    let manager = ProfileManager::new()?;

    let source = if empty {
        ProfileSource::Empty
    } else if let Some(from_profile) = from {
        ProfileSource::FromProfile(from_profile)
    } else {
        ProfileSource::FromCurrent
    };

    manager.add_profile(profile, source)?;

    println!("{}", format!("✓ Profile '{}' created successfully", profile).green());

    Ok(())
}
