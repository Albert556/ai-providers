use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct Paths {
    pub profiles_dir: PathBuf,
    pub state_file: PathBuf,
    pub claude_config: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .context("Failed to get HOME environment variable")?;
        let home_path = PathBuf::from(home);

        let profiles_dir = home_path.join(".ai-providers");
        let state_file = profiles_dir.join("state.json");
        let claude_config = home_path.join(".claude").join("settings.json");

        Ok(Self {
            profiles_dir,
            state_file,
            claude_config,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        if !self.profiles_dir.exists() {
            std::fs::create_dir_all(&self.profiles_dir)
                .context("Failed to create profiles directory")?;
        }
        Ok(())
    }

    pub fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", name))
    }
}
