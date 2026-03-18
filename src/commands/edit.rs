use anyhow::{anyhow, Context, Result};
use colored::*;
use std::io::{self, Write};
use std::process::Command;

use crate::profile::manager::ProfileManager;

pub fn execute(profile: &str) -> Result<()> {
    let manager = ProfileManager::new()?;

    if !manager.profile_exists(profile) {
        println!("{}", format!("✗ Profile '{}' not found", profile).red());
        return Ok(());
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if Command::new("vim").arg("--version").output().is_ok() {
                "vim".to_string()
            } else if Command::new("vi").arg("--version").output().is_ok() {
                "vi".to_string()
            } else {
                "nano".to_string()
            }
        });

    let paths = crate::config::paths::Paths::new()?;
    let profile_path = paths.profile_path(profile);

    loop {
        let status = Command::new(&editor)
            .arg(&profile_path)
            .status()
            .with_context(|| format!("Failed to launch editor '{}'", editor))?;

        if !status.success() {
            return Err(anyhow!("Editor exited with non-zero status"));
        }

        match manager.get_profile(profile) {
            Ok(_) => {
                println!("{}", "✓ Profile saved successfully".green());
                break;
            }
            Err(e) => {
                println!("{}", format!("✗ Invalid JSON: {}", e).red());
                print!("Do you want to edit again? (y/n): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    return Err(anyhow!("Profile contains invalid JSON"));
                }
            }
        }
    }

    Ok(())
}

