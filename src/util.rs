use std::process::Command;

/// Resolve the user's preferred editor from environment variables or common defaults.
pub fn resolve_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if Command::new("vim").arg("--version").output().is_ok() {
                "vim".to_string()
            } else if Command::new("vi").arg("--version").output().is_ok() {
                "vi".to_string()
            } else {
                "nano".to_string()
            }
        })
}
