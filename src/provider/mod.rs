use anyhow::Result;
use std::path::PathBuf;

pub mod claude;

pub trait Provider {
    /// Provider identifier, used for CLI subcommand and storage directory name
    fn name(&self) -> &str;

    /// Path to the provider's configuration file (e.g., ~/.claude/settings.json)
    fn config_path(&self) -> PathBuf;

    /// Validate JSON content as a valid configuration (default: accept any valid JSON)
    fn validate_config(&self, _content: &serde_json::Value) -> Result<()> {
        Ok(())
    }
}
