use crate::config::{PROFILE, flags};
use crate::core::builder::{BuildContext, build as build_project};
use crate::core::config::Config;
use crate::core::deps::resolve_deps;
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
    let build_target = get_config_str(&config, "build.target");
    let build_kind = get_config_str(&config, "build.kind");
    let build_platform = get_config_str(&config, "build.platform");
    let tc_cc = get_config_opt(&config, "toolchain.cc");
    let tc_cxx = get_config_opt(&config, "toolchain.cxx");
    let tc_as = get_config_opt(&config, "toolchain.as");
    let tc_ar = get_config_opt(&config, "toolchain.ar");
    let tc_ld = get_config_opt(&config, "toolchain.ld");
    let build_cflags = match get_config_list(&config, "build.cflags") {
        Ok(v) => v,
        Err(msg) => {
            error(&msg);
            return 1;
        }
    };
    let build_ldflags = match get_config_list(&config, "build.ldflags") {
        Ok(v) => v,
        Err(msg) => {
            error(&msg);
            return 1;
        }
    };

    let resolved_compiler = resolve_compiler(
        &build_language,
        &project_compiler,
        tc_cc.as_deref(),
        tc_cxx.as_deref(),
        tc_as.as_deref(),
    );
    let resolved_linker = resolve_tool("DCR_LD", tc_ld.as_deref());
    let resolved_archiver = resolve_tool("DCR_AR", tc_ar.as_deref());

    println!(
        "    Building project `{}`\n    Profile: {}\n    Compiler: {}\n",
        colored(&project_name, BOLD_GREEN),
        colored(&active_profile, BOLD_GREEN),
        colored(&resolved_compiler, BOLD_GREEN)
    );

    ensure_target_dirs(&items, &active_profile, normalize_target(&build_target));

    let project_root = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => {
            error("Failed to determine project root");
            return 1;
        }
    };
    let resolved = match resolve_deps(&config, &active_profile, &project_root) {
        Ok(r) => r,
        Err(msg) => {
            error(&msg);
            return 1;
        }
    };
    let ctx = BuildContext {
        profile: &active_profile,
        project_name: &project_name,
        compiler: &resolved_compiler,
        language: &build_language,
        standard: &build_standard,
        target_dir: normalize_target(&build_target),
        kind: normalize_kind(&build_kind),
        platform: normalize_platform(&build_platform),
        linker: resolved_linker.as_deref(),
        archiver: resolved_archiver.as_deref(),
        include_dirs: &resolved.include_dirs,
        lib_dirs: &resolved.lib_dirs,
        libs: &resolved.libs,
        cflags: &build_cflags,
        ldflags: &build_ldflags,
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

fn get_config_opt(config: &Config, key: &str) -> Option<String> {
    let value = config.get(key)?.as_str()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn get_config_list(config: &Config, key: &str) -> Result<Vec<String>, String> {
    let value = match config.get(key) {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array of strings"))?;
    let mut out = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| format!("{key} must be an array of strings"))?;
        out.push(s.to_string());
    }
    Ok(out)
}

fn ensure_target_dirs(items: &[String], profile: &str, target_dir: Option<&str>) {
    if let Some(dir) = target_dir {
        let _ = fs::create_dir_all(dir);
        return;
    }
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

fn normalize_target(target: &str) -> Option<&str> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn normalize_kind(kind: &str) -> &str {
    let trimmed = kind.trim();
    if trimmed.is_empty() { "bin" } else { trimmed }
}

fn normalize_platform(platform: &str) -> Option<&str> {
    let trimmed = platform.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_compiler(
    language: &str,
    compiler: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> String {
    let lang = language.to_lowercase();
    env_override_compiler(&lang)
        .or_else(|| toolchain_override_compiler(&lang, tc_cc, tc_cxx, tc_as))
        .unwrap_or_else(|| compiler.to_string())
}

fn env_override_compiler(lang: &str) -> Option<String> {
    if let Ok(value) = std::env::var("DCR_COMPILER") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if lang == "asm" {
        if let Ok(value) = std::env::var("DCR_AS") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        return None;
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Ok(value) = std::env::var("DCR_CXX")
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(value) = std::env::var("DCR_CC") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn toolchain_override_compiler(
    lang: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> Option<String> {
    if lang == "asm" {
        return tc_as.map(|v| v.to_string());
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Some(v) = tc_cxx
    {
        return Some(v.to_string());
    }
    tc_cc.map(|v| v.to_string())
}

fn resolve_tool(env_key: &str, fallback: Option<&str>) -> Option<String> {
    if let Ok(value) = std::env::var(env_key) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    fallback.map(|v| v.to_string())
}
