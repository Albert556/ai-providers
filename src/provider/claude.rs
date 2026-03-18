use std::path::PathBuf;

use super::Provider;

pub struct ClaudeProvider;

impl Provider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn config_path(&self) -> PathBuf {
        let home = std::env::var("HOME").expect("Cannot determine home directory");
        PathBuf::from(home).join(".claude").join("settings.json")
    }
}
