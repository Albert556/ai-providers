use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script_path(name: &str) -> PathBuf {
    repo_root().join("scripts").join("ci").join(name)
}

fn make_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("aip-{name}-{unique}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn run(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "command failed: status={:?}, stdout={}, stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn init_git_repo(path: &Path) {
    run(Command::new("git").arg("init").arg(path));
    run(Command::new("git")
        .current_dir(path)
        .args(["config", "user.name", "Codex Test"]));
    run(Command::new("git")
        .current_dir(path)
        .args(["config", "user.email", "codex@example.com"]));
}

fn commit_all(path: &Path, message: &str) -> String {
    run(Command::new("git").current_dir(path).args(["add", "."]));
    run(Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", message]));
    let output = run(Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"]));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn read_cargo_version_returns_package_version() {
    let temp_dir = make_temp_dir("read-version");
    let manifest = temp_dir.join("Cargo.toml");
    write_file(
        &manifest,
        r#"[package]
name = "demo"
version = "1.2.3"
edition = "2021"
"#,
    );

    let output = run(Command::new("bash")
        .arg(script_path("read_cargo_version.sh"))
        .arg(&manifest));

    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "1.2.3");
}

#[test]
fn check_release_needed_detects_version_change_between_push_bounds() {
    let temp_dir = make_temp_dir("check-release");
    init_git_repo(&temp_dir);
    write_file(
        &temp_dir.join("Cargo.toml"),
        r#"[package]
name = "demo"
version = "0.1.0"
edition = "2021"
"#,
    );
    let before = commit_all(&temp_dir, "initial version");

    write_file(
        &temp_dir.join("Cargo.toml"),
        r#"[package]
name = "demo"
version = "0.1.1"
edition = "2021"
"#,
    );
    let after = commit_all(&temp_dir, "bump version");

    let github_output = temp_dir.join("github-output.txt");
    let output = run(Command::new("bash")
        .current_dir(&temp_dir)
        .arg(script_path("check_release_needed.sh"))
        .arg(&before)
        .arg(&after)
        .env("GITHUB_OUTPUT", &github_output)
        .env("GITEA_SERVER_URL", "https://gitea.example.com")
        .env("GITHUB_REPOSITORY", "owner/repo"));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let outputs = fs::read_to_string(github_output).unwrap();

    assert!(stdout.contains("version_changed=true"));
    assert!(outputs.contains("old_version=0.1.0"));
    assert!(outputs.contains("new_version=0.1.1"));
    assert!(outputs.contains("version_changed=true"));
    assert!(outputs.contains("tag=v0.1.1"));
}

#[test]
fn package_binary_renames_release_artifact_with_target_suffix() {
    let temp_dir = make_temp_dir("package-binary");
    let source = temp_dir.join("aip");
    write_file(&source, "binary");

    let output = run(Command::new("bash")
        .arg(script_path("package_binary.sh"))
        .arg(&source)
        .arg("0.1.1")
        .arg("x86_64-unknown-linux-gnu"));

    let packaged = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());

    assert!(packaged.exists());
    assert_eq!(
        packaged.file_name().unwrap().to_string_lossy(),
        "aip-v0.1.1-x86_64-unknown-linux-gnu"
    );
    assert_eq!(fs::read_to_string(packaged).unwrap(), "binary");
}

#[test]
fn check_release_needed_skips_when_version_is_unchanged() {
    let temp_dir = make_temp_dir("skip-release");
    init_git_repo(&temp_dir);
    write_file(
        &temp_dir.join("Cargo.toml"),
        r#"[package]
name = "demo"
version = "0.1.1"
edition = "2021"
"#,
    );
    let before = commit_all(&temp_dir, "initial version");

    write_file(&temp_dir.join("README.md"), "# demo\n");
    let after = commit_all(&temp_dir, "docs only");

    let github_output = temp_dir.join("github-output.txt");
    let output = run(Command::new("bash")
        .current_dir(&temp_dir)
        .arg(script_path("check_release_needed.sh"))
        .arg(&before)
        .arg(&after)
        .env("GITHUB_OUTPUT", &github_output));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let outputs = fs::read_to_string(github_output).unwrap();

    assert!(stdout.contains("version_changed=false"));
    assert!(outputs.contains("version_changed=false"));
    assert!(outputs.contains("should_release=false"));
    assert!(outputs.contains("tag=v0.1.1"));
}
