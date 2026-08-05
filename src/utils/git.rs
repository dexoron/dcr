use std::path::Path;
use std::process::Command;

/// Checks if the `git` executable is available in the system's PATH.
///
/// This function executes `git --version` and returns `true` if the command succeeds, `false` otherwise.
pub fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Initializes a new Git repository at the specified path.
pub fn git_init(path: &Path) -> Result<(), String> {
    if !is_git_available() {
        return Err("git executable not found in PATH".to_string());
    }

    let status = Command::new("git")
        .arg("init")
        .current_dir(path)
        .status()
        .map_err(|e| format!("failed to execute git init: {}", e))?;

    if !status.success() {
        return Err("git init failed".to_string());
    }

    Ok(())
}
