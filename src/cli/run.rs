use crate::cli::build::build;
use crate::cli::flags::parse_build_run_flags;
use crate::core::config::Config;
use crate::core::runner::run_binary;
use crate::utils::build::{normalize_target_os, parse_version_info};
use crate::utils::fs::find_project_root;
use crate::utils::fs::with_dir;
use crate::utils::log::error;
use crate::utils::text::{BOLD_GREEN, colored};
use std::process::Command;

fn get_run_cmd(
    config: &Config,
    profile: &str,
    target: Option<&str>,
    version: &str,
) -> Option<String> {
    let base = config.get("run.cmd").and_then(|v| v.as_str());
    let target_cmd = if let Some(t) = target {
        let normalized_t = normalize_target_os(t);
        config
            .get(&format!("run.{}.cmd", normalized_t))
            .or_else(|| config.get(&format!("run.{}.cmd", t)))
            .and_then(|v| v.as_str())
    } else {
        None
    };
    let profile_cmd = config
        .get(&format!("run.{}.cmd", profile))
        .and_then(|v| v.as_str());
    let cmd = target_cmd.or(profile_cmd).or(base)?;
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(substitute_run_vars(trimmed, profile, version))
    }
}

pub fn run(args: &[String]) -> i32 {
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
    let config = match with_dir(&root, || {
        Config::open("./dcr.toml").map_err(|err| err.to_string())
    }) {
        Ok(cfg) => cfg,
        Err(err) => {
            error(&err);
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

    let mut flags = match parse_build_run_flags(args) {
        Ok(v) => v,
        Err(_) => return 1,
    };

    // If no target specified, use default host target for target-specific config
    if flags.target.is_none() {
        let default_target = if cfg!(target_os = "linux") {
            "x86_64-unknown-linux-gnu"
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else {
            "unknown"
        };
        flags.target = Some(default_target.to_string());
    }

    let normalized_target_dir = flags
        .target
        .as_ref()
        .and_then(|t| crate::cli::build::normalize_target(t, &flags.profile));

    let version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let run_cmd = get_run_cmd(&config, &flags.profile, flags.target.as_deref(), version);

    let kind = build_kind.trim();
    if run_cmd.is_none() && (kind == "staticlib" || kind == "sharedlib") {
        error("Cannot run library build");
        return 1;
    }
    let build_status = build(args);
    let bin_path = crate::platform::bin_path(
        &flags.profile,
        project_name,
        normalized_target_dir.as_deref(),
    );
    if build_status == 0 {
        if let Some(cmd) = run_cmd {
            println!("\n    {} {}", colored("Running", BOLD_GREEN), cmd);
            println!("--------------------------------");
            return run_shell(&cmd);
        }
        println!("\n    {} {}", colored("Running", BOLD_GREEN), bin_path);
        println!("--------------------------------");
        return run_binary(project_name, &flags.profile, target_dir);
    }

    let fallback_code = if let Some(cmd) = run_cmd {
        run_shell(&cmd)
    } else {
        run_binary(project_name, &flags.profile, target_dir)
    };
    if fallback_code != 1 {
        return fallback_code;
    }

    error("Fix errors in the code to run the project");
    1
}

fn run_shell(cmd: &str) -> i32 {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(cmd).status()
    } else {
        Command::new("sh").arg("-c").arg(cmd).status()
    };
    match status {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(_) => 1,
    }
}

fn substitute_run_vars(cmd: &str, profile: &str, version: &str) -> String {
    let info = parse_version_info(version);
    cmd.replace("{profile}", profile)
        .replace("{version}", &info.full)
        .replace("{version_major}", &info.major)
        .replace("{version_minor}", &info.minor)
        .replace("{version_patch}", &info.patch)
        .replace("{version_suffix}", &info.suffix)
        .replace("{version_suffix_dash}", &info.suffix_dash)
}
