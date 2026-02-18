use crate::config::{PROFILE, PROJECT_COMPILER, flags};
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use std::fs;
use std::process::Command;
use std::time::Instant;

pub fn build(args: &[String]) -> i32 {
    let mut active_profile = PROFILE.to_string();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    let items = check_dir(None).unwrap_or_default();
    if !items.contains(&"dcr.toml".to_string()) {
        error("dcr.toml file not found");
        return 1;
    }

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

    println!(
        "    Building project `{}`\n    Profile: {}\n    Compiler: {}\n",
        colored(&project_name, BOLD_GREEN),
        colored(&active_profile, BOLD_GREEN),
        colored(PROJECT_COMPILER, BOLD_GREEN)
    );

    if !items.contains(&"target".to_string()) {
        let _ = fs::create_dir("./target");
    }
    let target_items = check_dir(Some("target")).unwrap_or_default();
    if !target_items.contains(&active_profile) {
        let _ = fs::create_dir(format!("./target/{active_profile}"));
    }

    let src_items = check_dir(Some("src")).unwrap_or_default();
    if src_items.contains(&"main.c".to_string()) {
        let compile_flags = flags(&active_profile).unwrap_or(&[]);
        let start_time = Instant::now();
        let mut cmd = Command::new(PROJECT_COMPILER);
        cmd.arg("./src/main.c");
        for flag in compile_flags {
            cmd.arg(flag);
        }
        cmd.arg("-o").arg(format!("./target/{active_profile}/main"));

        match cmd.status() {
            Ok(status) if status.success() => {
                let times = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
                println!(
                    "    {} Build completed successfully in {} seconds",
                    colored("✔", BOLD_GREEN),
                    colored(&times.to_string(), BOLD_GREEN)
                );
                return 0;
            }
            _ => {
                error("Build failed");
                return 1;
            }
        }
    }

    0
}
