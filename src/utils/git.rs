// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::Path;
use std::process::Command;

// checking installed git on system
pub fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// warning of git not in system
#[allow(dead_code)]
pub fn warn_if_git_missing() {
    if !is_git_available() {
        crate::utils::log::warn(
            "Git is not installed or not found in PATH. Some features (like dependency cloning and VCS initialization) may fail.",
        );
    }
}

// init repo in folder
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

// Clone the dependency and switch to the branch/tag/commit
#[allow(dead_code)]
pub fn git_clone_and_checkout(
    url: &str,
    target_dir: &Path,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
) -> Result<(), String> {
    if !is_git_available() {
        return Err("git executable not found in PATH".to_string());
    }

    let mut args = vec!["clone"];

    if let Some(b) = branch {
        args.push("--branch");
        args.push(b);
        args.push("--single-branch");
    } else if let Some(t) = tag {
        args.push("--branch");
        args.push(t);
        args.push("--single-branch");
    }

    args.push(url);

    let target_dir_str = target_dir.to_string_lossy().to_string();
    args.push(&target_dir_str);

    let status = Command::new("git")
        .args(&args)
        .status()
        .map_err(|e| format!("failed to execute git clone: {}", e))?;

    if !status.success() {
        return Err(format!("git clone of {} failed", url));
    }

    if let Some(r) = rev {
        let checkout_status = Command::new("git")
            .arg("checkout")
            .arg(r)
            .current_dir(target_dir)
            .status()
            .map_err(|e| format!("failed to execute git checkout: {}", e))?;

        if !checkout_status.success() {
            return Err(format!("git checkout to revision {} failed", r));
        }
    }

    Ok(())
}
