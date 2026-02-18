use crate::utils::log::{error, warn};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
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
    if !args.is_empty() {
        warn("Command does not support additional arguments");
        return 1;
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let target = env!("DCR_TARGET");
    let client = match Client::builder().user_agent("dcr-updater").build() {
        Ok(client) => client,
        Err(_) => {
            error("Failed to initialize HTTP client");
            return 1;
        }
    };

    let release = match fetch_latest_release(&client) {
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

    let bytes = match download_asset(&client, &asset.browser_download_url) {
        Ok(bytes) => bytes,
        Err(err) => {
            error(&format!("Failed to download update: {err}"));
            return 1;
        }
    };

    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            error("Failed to resolve current binary path");
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

fn fetch_latest_release(client: &Client) -> Result<Release, String> {
    let response = client
        .get(LATEST_RELEASE_URL)
        .send()
        .map_err(|_| "GitHub API request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status {}", response.status()));
    }

    response
        .json::<Release>()
        .map_err(|_| "GitHub API response has an unexpected format".to_string())
}

fn download_asset(client: &Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .send()
        .map_err(|_| "Download request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Download returned status {}", response.status()));
    }

    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|_| "Failed to read downloaded data".to_string())
}

fn asset_candidates(target: &str) -> Vec<String> {
    let mut names = vec![format!("dcr-{target}")];

    if target == "x86_64-pc-windows-msvc" {
        names.push("dcr-x86_64-pc-windows-msvc.exe".to_string());
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

fn set_executable_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(path, perms);
        }
    }
}
