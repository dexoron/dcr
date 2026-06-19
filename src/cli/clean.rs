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

use crate::core::build_config::Config;
use crate::core::workspace::parse_workspace;
use crate::utils::build::{default_profile_flags, default_target_triple, parse_version_info};
use crate::utils::fs::{check_dir, find_project_root, with_dir};
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use glob::glob;
use std::fs;
use std::path::Path;

pub fn clean(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr clean [--debug | --release] [--target <triple>] [--all]",
            BOLD_CYAN,
        );
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Removes build artifacts from the target directory.");
        println!();
        printc("OPTIONS:", BOLD_GREEN);
        println!("    --debug              Clean debug artifacts (default)");
        println!("    --release            Clean release artifacts");
        println!("    --target <triple>    Clean artifacts for a specific target");
        println!("    --all                Clean all workspace members");
        return 0;
    }

    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };
    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => dir,
        Ok(None) => {
            error("dcr.toml file not found");
            return 1;
        }
        Err(_) => {
            error("Failed to find project root");
            return 1;
        }
    };
    let flags = match parse_clean_flags(args) {
        Ok(v) => v,
        Err(msg) => {
            error(&msg);
            return 1;
        }
    };

    match with_dir(&root, || clean_from_root(&root, &flags)) {
        Ok(()) => 0,
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

struct CleanFlags {
    profile: Option<String>,
    target: Option<String>,
    all: bool,
}

fn parse_clean_flags(args: &[String]) -> Result<CleanFlags, String> {
    let mut profile: Option<String> = None;
    let mut target: Option<String> = None;
    let mut all = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--all" {
            all = true;
            continue;
        }
        if arg == "--target" {
            if let Some(t) = iter.next() {
                target = Some(t.clone());
            } else {
                return Err("--target requires a value".to_string());
            }
            continue;
        }
        if arg.starts_with("--") {
            let candidate = arg.trim_start_matches("--").to_string();
            if !default_profile_flags(&candidate).is_empty() {
                if profile.is_some() {
                    return Err("Duplicate profile flag".to_string());
                }
                profile = Some(candidate);
                continue;
            }
        }
        return Err("Unknown argument".to_string());
    }
    Ok(CleanFlags {
        profile,
        target,
        all,
    })
}

fn clean_from_root(root: &Path, flags: &CleanFlags) -> Result<(), String> {
    let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;

    let target = flags
        .target
        .clone()
        .or_else(|| {
            config
                .get("build.target")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| Some(default_target_triple()));

    if flags.all
        && let Some(workspace) = parse_workspace(
            &config,
            flags.profile.as_deref().unwrap_or("debug"),
            target.as_deref(),
            root,
        )?
    {
        for member in &workspace.members {
            clean_project_at(&member.path, flags.profile.as_deref(), target.as_deref())?;
        }
    }
    clean_project_at(root, flags.profile.as_deref(), target.as_deref())
}

fn clean_project_at(
    project_root: &Path,
    profile: Option<&str>,
    target: Option<&str>,
) -> Result<(), String> {
    with_dir(project_root, || {
        let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;
        let project_name = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
            .unwrap_or_else(|| "project".to_string());
        let items = check_dir(None).map_err(|_| "Failed to read project directory".to_string())?;
        if !items.contains(&"dcr.toml".to_string()) {
            return Err("dcr.toml file not found".to_string());
        }
        println!(
            "    Cleaning project `{}`",
            colored(&project_name, BOLD_GREEN)
        );
        if let Some(profile) = profile {
            let target_dir = if let Some(t) = target {
                format!("target/{t}/{profile}")
            } else {
                format!("target/{profile}")
            };
            let target_items = check_dir(Some("target")).unwrap_or_default();
            let parent_dir = target.unwrap_or("");
            let dir_exists = if target.is_some() {
                target_items.contains(&parent_dir.to_string())
            } else {
                target_items.contains(&profile.to_string())
            };
            if !dir_exists {
                warn(&format!("Directory target/{} not found", target_dir));
            } else {
                println!("    Profile: {}", colored(profile, BOLD_GREEN));
                if let Some(t) = target {
                    println!("    Target: {}", colored(t, BOLD_GREEN));
                }
                let _ = fs::remove_dir_all(&target_dir);
                println!(
                    "{} Removed directory {}",
                    colored("\n    ✔", BOLD_GREEN),
                    target_dir
                );
            }
            clean_custom_paths(&config, profile)?;
            return Ok(());
        }

        if items.contains(&"target".to_string()) {
            let _ = fs::remove_dir_all("target");
            println!(
                "{} Removed directory target",
                colored("\n    ✔", BOLD_GREEN)
            );
        } else {
            warn("Directory target not found");
        }
        clean_custom_paths(&config, "debug")?;
        clean_custom_paths(&config, "release")?;
        Ok(())
    })
}

fn clean_custom_paths(config: &Config, profile: &str) -> Result<(), String> {
    let patterns = match config.get("build.clean") {
        Some(v) => v
            .as_array()
            .ok_or_else(|| "build.clean must be an array of strings".to_string())?
            .iter()
            .filter_map(|item| item.as_str())
            .map(|s| {
                let value = s.replace("{profile}", profile);
                substitute_clean_vars(&value, config)
            })
            .collect::<Vec<String>>(),
        None => Vec::new(),
    };
    if patterns.is_empty() {
        return Ok(());
    }
    for pattern in patterns {
        for entry in glob(&pattern).map_err(|err| format!("glob error: {err}"))? {
            let path = entry.map_err(|err| format!("glob error: {err}"))?;
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else if path.is_file() {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

fn substitute_clean_vars(value: &str, config: &Config) -> String {
    let version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let info = parse_version_info(version);
    crate::utils::build::substitute_version_vars(value, &info)
}
