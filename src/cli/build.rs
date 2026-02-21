use crate::config::{PROFILE, flags};
use crate::core::builder::{BuildContext, build as build_project};
use crate::core::config::Config;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use std::fs;
use std::time::Instant;

pub fn build(args: &[String]) -> i32 {
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
    let active_profile = match parse_profile(args) {
        Ok(profile) => profile,
        Err(code) => return code,
    };
    let project_name = get_config_str(&config, "package.name");
    let project_compiler = get_config_str(&config, "build.compiler");
    let build_language = get_config_str(&config, "build.language");
    let build_standard = get_config_str(&config, "build.standard");

    println!(
        "    Building project `{}`\n    Profile: {}\n    Compiler: {}\n",
        colored(&project_name, BOLD_GREEN),
        colored(&active_profile, BOLD_GREEN),
        colored(&project_compiler, BOLD_GREEN)
    );

    ensure_target_dirs(&items, &active_profile);

    let ctx = BuildContext {
        profile: &active_profile,
        project_name: &project_name,
        compiler: &project_compiler,
        language: &build_language,
        standard: &build_standard,
    };
    match run_build(&ctx) {
        Ok(times) => {
            println!(
                "    {} Build completed successfully in {} seconds",
                colored("✔", BOLD_GREEN),
                colored(&times.to_string(), BOLD_GREEN)
            );
            0
        }
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

fn parse_profile(args: &[String]) -> Result<String, i32> {
    if let Some(first_arg) = args.first() {
        if first_arg.starts_with("--") {
            let candidate = first_arg.trim_start_matches("--");
            if flags(candidate).is_some() {
                return Ok(candidate.to_string());
            }
            warn("Unknown build flag");
            return Err(1);
        }
        warn("Unknown argument");
        return Err(1);
    }
    Ok(PROFILE.to_string())
}

fn get_config_str(config: &Config, key: &str) -> String {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn ensure_target_dirs(items: &[String], profile: &str) {
    if !items.contains(&"target".to_string()) {
        let _ = fs::create_dir("./target");
    }
    let target_items = check_dir(Some("target")).unwrap_or_default();
    if !target_items.contains(&profile.to_string()) {
        let _ = fs::create_dir(format!("./target/{profile}"));
    }
}

fn run_build(ctx: &BuildContext) -> Result<f64, String> {
    let start_time = Instant::now();
    match build_project(ctx) {
        Ok(times) => {
            let times = if times == 0.0 {
                ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0
            } else {
                times
            };
            Ok(times)
        }
        Err(_) => Err("Build failed".to_string()),
    }
}
