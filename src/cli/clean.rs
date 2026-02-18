use crate::config::flags;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use std::fs;

pub fn clean(args: &[String]) -> i32 {
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());
    let items = check_dir(None).unwrap_or_default();

    if args.len() > 1 {
        error("Unknown argument");
        return 1;
    }
    if !items.contains(&"dcr.toml".to_string()) {
        error("dcr.toml file not found");
        return 1;
    }

    println!(
        "    Cleaning project `{}`",
        colored(&project_name, BOLD_GREEN)
    );
    if !items.contains(&"target".to_string()) {
        warn("Directory target not found");
        return 1;
    }

    if let Some(arg) = args.first() {
        let mut profile = arg.clone();
        if profile.starts_with("--") {
            profile = profile.trim_start_matches("--").to_string();
        }
        if flags(&profile).is_none() {
            error("Unknown profile");
            return 1;
        }

        let target_items = check_dir(Some("target")).unwrap_or_default();
        if !target_items.contains(&profile) {
            warn(&format!("Directory target/{profile} not found"));
            return 1;
        }

        println!("    Profile: {}", colored(&profile, BOLD_GREEN));
        let _ = fs::remove_dir_all(format!("target/{profile}"));
        println!(
            "{} Removed directory target/{profile}",
            colored("\n    ✔", BOLD_GREEN)
        );
        return 0;
    }

    let _ = fs::remove_dir_all("target");
    println!(
        "{} Removed directory target",
        colored("\n    ✔", BOLD_GREEN)
    );
    0
}
