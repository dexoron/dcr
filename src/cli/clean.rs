use crate::config::flags;
use crate::core::config::Config;
use crate::core::workspace::parse_workspace;
use crate::utils::fs::{check_dir, find_project_root};
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use std::fs;
use std::path::Path;

pub fn clean(args: &[String]) -> i32 {
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
    all: bool,
}

fn parse_clean_flags(args: &[String]) -> Result<CleanFlags, String> {
    let mut profile: Option<String> = None;
    let mut all = false;
    for arg in args {
        if arg == "--all" {
            all = true;
            continue;
        }
        if arg.starts_with("--") {
            let candidate = arg.trim_start_matches("--").to_string();
            if flags(&candidate).is_some() {
                if profile.is_some() {
                    return Err("Duplicate profile flag".to_string());
                }
                profile = Some(candidate);
                continue;
            }
        }
        return Err("Unknown argument".to_string());
    }
    Ok(CleanFlags { profile, all })
}

fn clean_from_root(root: &Path, flags: &CleanFlags) -> Result<(), String> {
    let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;
    if flags.all
        && let Some(workspace) = parse_workspace(&config, root)?
    {
        for member in &workspace.members {
            clean_project_at(&member.path, flags.profile.as_deref())?;
        }
    }
    clean_project_at(root, flags.profile.as_deref())
}

fn clean_project_at(project_root: &Path, profile: Option<&str>) -> Result<(), String> {
    with_dir(project_root, || {
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
        if !items.contains(&"target".to_string()) {
            warn("Directory target not found");
            return Ok(());
        }

        if let Some(profile) = profile {
            let target_items = check_dir(Some("target")).unwrap_or_default();
            if !target_items.contains(&profile.to_string()) {
                warn(&format!("Directory target/{profile} not found"));
                return Ok(());
            }
            println!("    Profile: {}", colored(profile, BOLD_GREEN));
            let _ = fs::remove_dir_all(format!("target/{profile}"));
            println!(
                "{} Removed directory target/{profile}",
                colored("\n    ✔", BOLD_GREEN)
            );
            return Ok(());
        }

        let _ = fs::remove_dir_all("target");
        println!(
            "{} Removed directory target",
            colored("\n    ✔", BOLD_GREEN)
        );
        Ok(())
    })
}

fn with_dir<F, T>(dir: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let prev = std::env::current_dir().map_err(|_| "Failed to get current dir".to_string())?;
    std::env::set_current_dir(dir).map_err(|_| "Failed to change directory".to_string())?;
    let result = f();
    let _ = std::env::set_current_dir(prev);
    result
}
