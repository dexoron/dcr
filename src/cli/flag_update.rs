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

use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, printc};
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/dexoron/dcr/releases/latest";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn flag_update(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc("    dcr --update", BOLD_CYAN);
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Updates DCR to the latest version from GitHub Releases.");
        return 0;
    }

    if !args.is_empty() {
        warn("Command does not support additional arguments");
        return 1;
    }

    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            error("Failed to resolve current binary path");
            return 1;
        }
    };

    if let Some(package_name) = pacman_owned_package(&current_exe) {
        warn("This dcr binary is managed by pacman/AUR");
        println!(
            "Update via package manager: yay/paru -Syu {package_name} or sudo pacman -Syu {package_name}"
        );
        return 0;
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let target = env!("DCR_TARGET");

    let release = match fetch_latest_release() {
        Ok(release) => release,
        Err(err) => {
            error(&format!("Failed to check for updates: {err}"));
            return 1;
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v');
    if latest_version == current_version {
        println!("Latest version is already installed: {current_version}");
        return 0;
    }

    let candidate_names = asset_candidates(target);
    let Some(asset) = release
        .assets
        .iter()
        .find(|asset| candidate_names.iter().any(|name| name == &asset.name))
    else {
        error(&format!("Binary for target {target} not found"));
        return 1;
    };

    let bytes = match download_asset(&asset.browser_download_url) {
        Ok(bytes) => bytes,
        Err(err) => {
            error(&format!("Failed to download update: {err}"));
            return 1;
        }
    };

    let temp_path = temp_binary_path(&current_exe);

    if fs::write(&temp_path, &bytes).is_err() {
        error("Failed to write temporary binary");
        return 1;
    }
    set_executable_permissions(&temp_path);

    if self_replace::self_replace(&temp_path).is_err() {
        let _ = fs::remove_file(&temp_path);
        error("Failed to replace current binary");
        return 1;
    }

    let _ = fs::remove_file(&temp_path);
    println!("Update completed: {current_version} -> {latest_version}");
    0
}

fn fetch_latest_release() -> Result<Release, String> {
    let response = ureq::get(LATEST_RELEASE_URL)
        .set("User-Agent", "dcr-updater")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("GitHub API returned status {code}"),
            ureq::Error::Transport(e) => format!("GitHub API request failed: {e}"),
        })?;

    response
        .into_json::<Release>()
        .map_err(|_| "GitHub API response has an unexpected format".to_string())
}

fn download_asset(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url)
        .set("User-Agent", "dcr-updater")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("Download returned status {code}"),
            ureq::Error::Transport(e) => format!("Download request failed: {e}"),
        })?;

    let mut reader = response.into_reader();
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|_| "Failed to read downloaded data".to_string())?;
    Ok(data)
}

fn asset_candidates(target: &str) -> Vec<String> {
    let mut names = vec![format!("dcr-{target}")];

    if target.contains("-windows-") || target.ends_with("-windows") {
        names.push(format!("dcr-{target}.exe"));
    }

    names
}

fn temp_binary_path(current_exe: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    let mut extension = format!("new-{stamp}");
    if cfg!(windows) {
        extension.push_str(".exe");
    }
    current_exe.with_extension(extension)
}

fn set_executable_permissions(_path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(_path, perms);
        }
    }
}

#[cfg(target_os = "linux")]
fn pacman_owned_package(path: &Path) -> Option<String> {
    let output = Command::new("pacman").arg("-Qoq").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let package_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if package_name.is_empty() {
        return None;
    }

    Some(package_name)
}

#[cfg(not(target_os = "linux"))]
fn pacman_owned_package(_path: &Path) -> Option<String> {
    None
}
