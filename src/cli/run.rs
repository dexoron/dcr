use crate::cli::build::build;
use crate::cli::flags::parse_build_run_flags;
use crate::core::config::Config;
use crate::core::runner::run_binary;
use crate::utils::fs::find_project_root;
use crate::utils::log::error;
use crate::utils::text::{BOLD_GREEN, colored};
use std::path::Path;
use std::process::Command;

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

    let flags = match parse_build_run_flags(args) {
        Ok(v) => v,
        Err(_) => return 1,
    };

    let version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let run_cmd = config
        .get("run.cmd")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|cmd| substitute_run_vars(&cmd, &flags.profile, version));

    let kind = build_kind.trim();
    if run_cmd.is_none() && (kind == "staticlib" || kind == "sharedlib") {
        error("Cannot run library build");
        return 1;
    }
    let build_status = build(args);
    let bin_path = crate::platform::bin_path(&flags.profile, project_name, target_dir);
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

struct VersionInfo {
    full: String,
    major: String,
    minor: String,
    patch: String,
    suffix: String,
    suffix_dash: String,
}

fn parse_version_info(version: &str) -> VersionInfo {
    let mut full = version.trim().to_string();
    if full.is_empty() {
        full = "0.0.0".to_string();
    }
    let full_clone = full.clone();
    let (base, suffix) = match full_clone.split_once('-') {
        Some((head, tail)) => (head, tail),
        None => (full_clone.as_str(), ""),
    };
    let mut parts = base.split('.');
    let major = parts.next().unwrap_or("0").to_string();
    let minor = parts.next().unwrap_or("0").to_string();
    let patch = parts.next().unwrap_or("0").to_string();
    let suffix_dash = if suffix.is_empty() {
        "".to_string()
    } else {
        format!("-{suffix}")
    };
    VersionInfo {
        full,
        major,
        minor,
        patch,
        suffix: suffix.to_string(),
        suffix_dash,
    }
}
