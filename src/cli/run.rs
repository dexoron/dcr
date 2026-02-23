use crate::cli::build::build;
use crate::config::{PROFILE, flags};
use crate::core::config::Config;
use crate::platform;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use std::path::Path;
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut active_profile = PROFILE.to_string();

    let items = check_dir(None).unwrap_or_default();
    if !items.contains(&"dcr.toml".to_string()) {
        error("dcr.toml file not found");
        return 1;
    }

    let config = match Config::open("./dcr.toml") {
        Ok(cfg) => cfg,
        Err(_) => {
            error("dcr.toml file not found");
            return 1;
        }
    };
    let project_name: &str = config
        .get("package.name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let target_dir = config
        .get("build.target")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if let Some(first_arg) = args.first() {
        if first_arg.starts_with("--") {
            let candidate = first_arg.trim_start_matches("--");
            if flags(candidate).is_some() {
                active_profile = candidate.to_string();
            } else {
                warn("Unknown build flag");
                return 1;
            }
        } else {
            warn("Unknown argument");
            return 1;
        }
    }

    let build_status = build(args);
    let bin_path = platform::bin_path(&active_profile, project_name, target_dir);
    if build_status == 0 {
        println!("\n    {} {}", colored("Running", BOLD_GREEN), bin_path);
        println!("--------------------------------");
        return Command::new(bin_path)
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    if target_dir.is_none()
        && check_dir(Some(&format!("target/{active_profile}")))
            .unwrap_or_default()
            .contains(&active_profile)
    {
        warn("Launch of the latest release");
        return Command::new(platform::bin_path(
            &active_profile,
            project_name,
            target_dir,
        ))
        .status()
        .map(|status| status.code().unwrap_or(1))
        .unwrap_or(1);
    }
    if target_dir.is_some() && Path::new(&bin_path).exists() {
        warn("Launch of the latest release");
        return Command::new(bin_path)
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    error("Fix errors in the code to run the project");
    0
}
