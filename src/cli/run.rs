use crate::cli::build::build;
use crate::config::{PROFILE, flags};
use crate::core::config::Config;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
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
    if build_status == 0 {
        println!(
            "\n    {} target/{active_profile}/{project_name}",
            colored("Running", BOLD_GREEN)
        );
        println!("--------------------------------");
        return Command::new(format!("./target/{active_profile}/{project_name}"))
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    if check_dir(Some(&format!("target/{active_profile}")))
        .unwrap_or_default()
        .contains(&active_profile)
    {
        warn("Launch of the latest release");
        return Command::new(format!("./target/{active_profile}/{project_name}"))
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    error("Fix errors in the code to run the project");
    0
}
