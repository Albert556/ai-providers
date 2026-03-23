use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::PathBuf;

use crate::profile::storage;
use crate::provider::Provider;

const COMMON_PROFILE_NAME: &str = "common";

pub enum ProfileSource {
    Empty,
    FromCurrent,
    FromProfile(String),
}

pub struct ProfileManager<'a> {
    provider: &'a dyn Provider,
    profiles_dir: PathBuf,
    state_file: PathBuf,
}

impl<'a> ProfileManager<'a> {
    pub fn new(provider: &'a dyn Provider) -> Result<Self> {
        let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
        let base_dir = PathBuf::from(home).join(".ai-providers");
        let profiles_dir = base_dir.join(provider.name());
        let state_file = base_dir.join("state.json");

        if !profiles_dir.exists() {
            std::fs::create_dir_all(&profiles_dir).with_context(|| {
                format!("Failed to create directory: {}", profiles_dir.display())
            })?;
        }

        Ok(Self {
            provider,
            profiles_dir,
            state_file,
        })
    }

    pub fn is_common_profile(name: &str) -> bool {
        name == COMMON_PROFILE_NAME
    }

    pub fn get_common_config(&self) -> Result<Option<Value>> {
        let path = self.profile_path(COMMON_PROFILE_NAME);
        if !path.exists() {
            return Ok(None);
        }
        storage::read_json(&path).map(Some)
    }

    pub fn has_common_config(&self) -> bool {
        self.profile_path(COMMON_PROFILE_NAME).exists()
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();

        if !self.profiles_dir.exists() {
            return Ok(profiles);
        }

        let entries =
            std::fs::read_dir(&self.profiles_dir).context("Failed to read profiles directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    profiles.push(stem.to_string());
                }
            }
        }

        profiles.retain(|p| p != COMMON_PROFILE_NAME);
        profiles.sort();
        Ok(profiles)
    }

    pub fn get_current_profile(&self) -> Result<Option<String>> {
        storage::read_current_profile(&self.state_file, self.provider.name())
    }

    pub fn get_profile(&self, name: &str) -> Result<serde_json::Value> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(anyhow!("Profile '{}' not found", name));
        }
        storage::read_json(&path)
    }

    pub fn get_active_config(&self) -> Result<serde_json::Value> {
        let config_path = self.provider.config_path();
        if !config_path.exists() {
            return Err(anyhow!(
                "Configuration not found at {}",
                config_path.display()
            ));
        }
        storage::read_json(&config_path)
    }

    pub fn add_profile(&self, name: &str, source: ProfileSource) -> Result<()> {
        self.validate_profile_name(name)?;

        if self.profile_exists(name) {
            return Err(anyhow!("Profile '{}' already exists", name));
        }

        let content = match source {
            ProfileSource::Empty => serde_json::json!({}),
            ProfileSource::FromCurrent => self
                .get_active_config()
                .unwrap_or_else(|_| serde_json::json!({})),
            ProfileSource::FromProfile(ref source_name) => self.get_profile(source_name)?,
        };

        let path = self.profile_path(name);
        storage::write_json(&path, &content)?;

        Ok(())
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        if !self.profile_exists(name) {
            return Err(anyhow!("Profile '{}' not found", name));
        }

        let path = self.profile_path(name);
        storage::remove_file(&path)?;

        let current = self.get_current_profile()?;
        if current.as_deref() == Some(name) {
            storage::update_current_profile(&self.state_file, self.provider.name(), None)?;
        }

        Ok(())
    }

    pub fn use_profile(&self, name: &str) -> Result<()> {
        if Self::is_common_profile(name) {
            return Err(anyhow!(
                "'common' is a shared base config, not a switchable profile. \
                 Use 'aip {} edit common' to modify it.",
                self.provider.name()
            ));
        }

        let profile_content = self.get_profile(name)?;

        // Merge with common config if it exists
        let final_content = if let Some(common) = self.get_common_config()? {
            storage::deep_merge(&common, &profile_content)
        } else {
            profile_content
        };

        let config_path = self.provider.config_path();
        storage::write_json(&config_path, &final_content)?;

        storage::update_current_profile(&self.state_file, self.provider.name(), Some(name))?;

        Ok(())
    }

    pub fn profile_exists(&self, name: &str) -> bool {
        self.profile_path(name).exists()
    }

    pub fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", name))
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

        if name == "state" {
            return Err(anyhow!("'state' is a reserved name"));
        }

        Ok(())
    }

    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
}
