use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub current_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            current_profile: None,
            last_updated: None,
        }
    }
}

pub fn read_json_file(path: &Path) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in file: {}", path.display()))?;

    Ok(json)
}

pub fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let content = serde_json::to_string_pretty(value)
        .context("Failed to serialize JSON")?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temp file: {}", temp_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&temp_path, perms)?;
    }

    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;

    Ok(())
}

pub fn read_state(path: &Path) -> Result<State> {
    if !path.exists() {
        return Ok(State::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;

    let state: State = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in state file: {}", path.display()))?;

    Ok(state)
}

pub fn write_state(path: &Path, state: &State) -> Result<()> {
    let content = serde_json::to_string_pretty(state)
        .context("Failed to serialize state")?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temp file: {}", temp_path.display()))?;

    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;

    Ok(())
}
