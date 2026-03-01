use crate::cli::build::build;
use crate::config::{PROFILE, flags};
use crate::core::config::Config;
use crate::core::runner::run_binary;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};

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
    let build_kind = config
        .get("build.kind")
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

    let kind = build_kind.trim();
    if kind == "staticlib" || kind == "sharedlib" {
        error("Cannot run library build");
        return 1;
    }
    let build_status = build(args);
    let bin_path = crate::platform::bin_path(&active_profile, project_name, target_dir);
    if build_status == 0 {
        println!("\n    {} {}", colored("Running", BOLD_GREEN), bin_path);
        println!("--------------------------------");
        return run_binary(project_name, &active_profile, target_dir);
    }

    let fallback_code = run_binary(project_name, &active_profile, target_dir);
    if fallback_code != 1 {
        return fallback_code;
    }

    error("Fix errors in the code to run the project");
    1
}
