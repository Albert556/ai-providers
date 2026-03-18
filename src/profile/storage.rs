use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in file: {}", path.display()))?;

    Ok(json)
}

pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    let content = serde_json::to_string_pretty(value)
        .context("Failed to serialize JSON")?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &content)
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

pub fn remove_file(path: &Path) -> Result<()> {
    fs::remove_file(path)
        .with_context(|| format!("Failed to remove file: {}", path.display()))
}

/// Read the current profile for a specific provider from state.json
pub fn read_current_profile(state_path: &Path, provider: &str) -> Result<Option<String>> {
    if !state_path.exists() {
        return Ok(None);
    }

    let state = read_json(state_path)?;
    let current = state
        .get(provider)
        .and_then(|v| v.get("current_profile"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(current)
}

/// Update the current profile for a specific provider in state.json
pub fn update_current_profile(
    state_path: &Path,
    provider: &str,
    profile: Option<&str>,
) -> Result<()> {
    let mut state = if state_path.exists() {
        read_json(state_path)?
    } else {
        Value::Object(Map::new())
    };

    let obj = state
        .as_object_mut()
        .context("state.json is not a JSON object")?;

    match profile {
        Some(name) => {
            let mut provider_state = Map::new();
            provider_state.insert(
                "current_profile".to_string(),
                Value::String(name.to_string()),
            );
            obj.insert(provider.to_string(), Value::Object(provider_state));
        }
        None => {
            obj.remove(provider);
        }
    }

    write_json(state_path, &state)
}
