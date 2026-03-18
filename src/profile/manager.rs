use anyhow::{anyhow, Context, Result};

use crate::config::paths::Paths;
use crate::profile::storage::{read_json_file, read_state, write_json_file, write_state, State};

pub enum ProfileSource {
    Empty,
    FromCurrent,
    FromProfile(String),
}

pub struct ProfileManager {
    paths: Paths,
}

impl ProfileManager {
    pub fn new() -> Result<Self> {
        let paths = Paths::new()?;
        paths.ensure_dirs()?;
        Ok(Self { paths })
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();

        if !self.paths.profiles_dir.exists() {
            return Ok(profiles);
        }

        let entries = std::fs::read_dir(&self.paths.profiles_dir)
            .context("Failed to read profiles directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "state" {
                        profiles.push(stem.to_string());
                    }
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    pub fn get_current_profile(&self) -> Result<Option<String>> {
        let state = read_state(&self.paths.state_file)?;
        Ok(state.current_profile)
    }

    pub fn get_profile(&self, name: &str) -> Result<serde_json::Value> {
        let path = self.paths.profile_path(name);
        if !path.exists() {
            return Err(anyhow!("Profile '{}' not found", name));
        }
        read_json_file(&path)
    }

    pub fn get_claude_config(&self) -> Result<serde_json::Value> {
        if !self.paths.claude_config.exists() {
            return Err(anyhow!("Claude Code configuration not found at {}", self.paths.claude_config.display()));
        }
        read_json_file(&self.paths.claude_config)
    }

    pub fn add_profile(&self, name: &str, source: ProfileSource) -> Result<()> {
        self.validate_profile_name(name)?;

        if self.profile_exists(name) {
            return Err(anyhow!("Profile '{}' already exists", name));
        }

        let content = match source {
            ProfileSource::Empty => serde_json::json!({}),
            ProfileSource::FromCurrent => {
                self.get_claude_config().unwrap_or_else(|_| serde_json::json!({}))
            }
            ProfileSource::FromProfile(ref profile_name) => {
                self.get_profile(profile_name)?
            }
        };

        let path = self.paths.profile_path(name);
        write_json_file(&path, &content)?;

        Ok(())
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        if !self.profile_exists(name) {
            return Err(anyhow!("Profile '{}' not found", name));
        }

        let path = self.paths.profile_path(name);
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to delete profile '{}'", name))?;

        let current = self.get_current_profile()?;
        if current.as_deref() == Some(name) {
            let mut state = State::new();
            state.current_profile = None;
            write_state(&self.paths.state_file, &state)?;
        }

        Ok(())
    }

    pub fn use_profile(&self, name: &str) -> Result<()> {
        let profile_content = self.get_profile(name)?;

        write_json_file(&self.paths.claude_config, &profile_content)?;

        let mut state = State::new();
        state.current_profile = Some(name.to_string());
        state.last_updated = Some(chrono::Utc::now().to_rfc3339());
        write_state(&self.paths.state_file, &state)?;

        Ok(())
    }

    pub fn profile_exists(&self, name: &str) -> bool {
        self.paths.profile_path(name).exists()
    }

    pub fn validate_profile_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow!("Profile name cannot be empty"));
        }

        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(anyhow!("Profile name cannot contain path separators"));
        }

        if name.starts_with('.') {
            return Err(anyhow!("Profile name cannot start with a dot"));
        }

        Ok(())
    }
}
