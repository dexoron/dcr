// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::cli::clean::clean;
use crate::cli::flags::parse_build_run_flags;
use crate::core::builder::common;
use crate::core::builder::{BuildContext, build as build_project, collect_sources};
use crate::core::config::Config;
use crate::core::deps::{register, resolve_deps};
use crate::core::workspace::parse_workspace;
use crate::utils::build::{
    get_bool_with_profile, get_config_opt, get_config_str, get_language_with_profile,
    normalize_target_os, parse_version_info, profile_table, resolve_compiler,
    resolve_pkg_config_flags, resolve_tool,
};
use crate::utils::fs::{check_dir, find_project_root, with_dir};
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use glob::glob;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static BUILD_INTERRUPTED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER: Once = Once::new();

pub fn build(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr build [--debug | --release] [--target <triple>] [--force] [--clean] [--verbose]",
            BOLD_CYAN,
        );
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Compiles the project. Default profile is --debug.");
        println!();
        printc("OPTIONS:", BOLD_GREEN);
        println!("    --debug              Build with debug profile (default)");
        println!("    --release            Build with release profile");
        println!("    --target <triple>    Cross-compile for the given target");
        println!("    --force              Force a full rebuild");
        println!("    --clean              Clean before building");
        println!("    --verbose            Print detailed build output");
        println!("    --workspace <name>   Build a specific workspace member");
        return 0;
    }

    install_signal_handler();
    BUILD_INTERRUPTED.store(false, Ordering::SeqCst);
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
    let mut flags = match parse_build_run_flags(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    if flags.target.is_none() {
        let config_path = root.join("dcr.toml");
        if let Ok(config) = Config::open(config_path.to_str().unwrap())
            && !config.is_workspace_only()
        {
            let bt = get_build_string_with_profile(&config, "target", "debug");
            if !bt.is_empty() {
                flags.target = Some(bt);
            }
        }
    }
    if flags.target.is_none() {
        let default_target = if cfg!(target_os = "linux") {
            format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        )) {
            format!(
                "{}-unknown-{}",
                std::env::consts::ARCH,
                std::env::consts::OS
            )
        } else {
            "unknown".to_string()
        };
        flags.target = Some(default_target);
    }
    if flags.clean {
        let mut clean_args = Vec::new();
        clean_args.push(format!("--{}", flags.profile));
        let _ = clean(&clean_args);
    }
    match with_dir(&root, || {
        build_from_root(
            &root,
            &flags.profile,
            flags.target.as_deref(),
            flags.force,
            flags.verbose,
            flags.workspace.as_deref(),
        )
    }) {
        Ok(()) => 0,
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

fn install_signal_handler() {
    SIGNAL_HANDLER.call_once(|| {
        let _ = ctrlc::set_handler(|| {
            BUILD_INTERRUPTED.store(true, Ordering::SeqCst);
        });
    });
}

fn check_interrupted() -> Result<(), String> {
    if BUILD_INTERRUPTED.load(Ordering::SeqCst) {
        Err("Build interrupted".to_string())
    } else {
        Ok(())
    }
}

fn get_config_value_raw(
    config: &Config,
    section: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Option<toml::Value> {
    // Order: target.profile, profile.target, target, profile, base
    let keys = [
        target.map(|t| {
            format!(
                "{}.{}.{}.{}",
                section,
                normalize_target_os(t),
                profile,
                field
            )
        }),
        target.map(|t| {
            format!(
                "{}.{}.{}.{}",
                section,
                profile,
                normalize_target_os(t),
                field
            )
        }),
        target.map(|t| format!("{}.{}.{}", section, normalize_target_os(t), field)),
        Some(format!("{}.{}.{}", section, profile, field)),
        Some(format!("{}.{}", section, field)),
    ];
    for key in keys.into_iter().flatten() {
        if let Some(val) = config.get(&key) {
            return Some(val.clone());
        }
    }
    None
}

fn get_inherit(config: &Config, section: &str, profile: &str, target: Option<&str>) -> bool {
    get_config_value_raw(config, section, "inherit", profile, target)
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

fn get_config_value(
    config: &Config,
    section: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Option<String> {
    get_config_value_raw(config, section, field, profile, target)
        .and_then(|v| v.as_str().map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
}

fn get_string_with_profile_and_target(
    config: &Config,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> String {
    if get_inherit(config, "build", profile, target) {
        get_config_value(config, "build", field, profile, target)
            .unwrap_or_else(|| get_config_str(config, &format!("build.{field}")))
    } else {
        get_config_value(config, "build", field, profile, target).unwrap_or_default()
    }
}

pub fn get_build_string_with_profile(config: &Config, field: &str, profile: &str) -> String {
    get_string_with_profile_and_target(config, field, profile, None)
}

fn parse_string_array(value: &toml::Value, key: &str) -> Result<Vec<String>, String> {
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

fn get_list_with_profile_and_target(
    config: &Config,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Result<Vec<String>, String> {
    let inherit = get_inherit(config, "build", profile, target);
    let mut out = if inherit {
        get_config_list(config, &format!("build.{field}"))?
    } else {
        Vec::new()
    };
    // Custom from target/profile
    if let Some(val) = get_config_value_raw(config, "build", field, profile, target) {
        if let Some(_arr) = val.as_array() {
            let is_base = inherit
                && config
                    .get(&format!("build.{field}"))
                    .map(|v| v == &val)
                    .unwrap_or(false);
            if !is_base {
                let custom = parse_string_array(&val, &format!("build.{field}"))?;
                if inherit {
                    out.extend(custom);
                } else {
                    out = custom;
                }
            }
        }
    } else if inherit {
        // Legacy append
        if let Some(table) = profile_table(config, profile)
            && let Some(value) = table.get(field)
        {
            let mut extra = parse_string_array(value, &format!("build.{profile}.{field}"))?;
            out.append(&mut extra);
        }
        if let Some(t) = target {
            let normalized_t = normalize_target_os(t);
            if let Some(table) = profile_table(config, normalized_t)
                && let Some(value) = table.get(field)
            {
                let mut extra =
                    parse_string_array(value, &format!("build.{normalized_t}.{field}"))?;
                out.append(&mut extra);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::utils::build::normalize_target_os;

    #[test]
    fn test_normalize_target_os() {
        assert_eq!(normalize_target_os("linux"), "x86_64-unknown-linux-gnu");
        assert_eq!(normalize_target_os("macos"), "x86_64-apple-darwin");
        assert_eq!(normalize_target_os("windows"), "x86_64-pc-windows-msvc");
        assert_eq!(
            normalize_target_os("x86_64-unknown-linux-gnu"),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(normalize_target_os("unknown"), "unknown");
    }
}

fn get_list_with_profile(
    config: &Config,
    field: &str,
    profile: &str,
) -> Result<Vec<String>, String> {
    get_list_with_profile_and_target(config, field, profile, None)
}

fn get_targets(config: &Config, profile: &str) -> Result<Vec<String>, String> {
    get_list_with_profile(config, "targets", profile)
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

fn get_build_steps_from_value(value: &toml::Value, key: &str) -> Result<Vec<BuildStep>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array"))?;
    let mut out = Vec::new();
    for item in arr {
        let tbl = item
            .as_table()
            .ok_or_else(|| format!("{key} entries must be tables"))?;
        let name = tbl
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let input = tbl
            .get("in")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let output = tbl
            .get("out")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let cmd = tbl
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() || input.is_empty() || output.is_empty() || cmd.is_empty() {
            return Err(format!("{key} entries must include name, in, out, cmd"));
        }
        out.push(BuildStep {
            name,
            input,
            output,
            cmd,
        });
    }
    Ok(out)
}

fn get_build_steps_with_profile(
    config: &Config,
    field: &str,
    profile: &str,
) -> Result<Vec<BuildStep>, String> {
    if let Some(table) = profile_table(config, profile)
        && let Some(value) = table.get(field)
    {
        return get_build_steps_from_value(value, &format!("build.{profile}.{field}"));
    }
    get_build_steps(config, &format!("build.{field}"))
}

fn ensure_target_dirs(
    items: &[String],
    profile: &str,
    target_dir: Option<String>,
    skip_local_target: bool,
) {
    if !skip_local_target && !items.contains(&"target".to_string()) {
        let _ = fs::create_dir("./target");
    }
    if let Some(dir) = &target_dir {
        let _ = fs::create_dir_all(dir);
    } else {
        let default_dir = if cfg!(target_os = "linux") {
            let arch = std::env::consts::ARCH;
            format!("{arch}-unknown-linux-gnu/{profile}")
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        )) {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            format!("{arch}-unknown-{os}/{profile}")
        } else {
            profile.to_string()
        };
        let target_items = check_dir(Some("target")).unwrap_or_default();
        if !target_items.contains(&default_dir) {
            let _ = fs::create_dir_all(format!("./target/{default_dir}"));
        }
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

#[allow(unused_variables)]
fn build_from_root(
    root: &Path,
    profile: &str,
    target: Option<&str>,
    force: bool,
    verbose: bool,
    workspace: Option<&str>,
) -> Result<(), String> {
    let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;

    if config.is_workspace_only() {
        let ws = parse_workspace(&config, profile, target, root)?
            .ok_or_else(|| "Workspace root has no members defined".to_string())?;
        let members: Vec<_> = if let Some(filter_name) = workspace {
            ws.members
                .into_iter()
                .filter(|m| m.name == filter_name)
                .collect()
        } else {
            ws.members
        };
        if members.is_empty() {
            return Err("No matching workspace members to build".to_string());
        }
        for member in &members {
            build_project_at(
                &member.path,
                profile,
                target,
                &[],
                force,
                verbose,
                Some(root),
            )?;
        }
        return Ok(());
    }

    let project_name = get_config_str(&config, "package.name");
    let project_version = get_config_str(&config, "package.version");

    let targets_to_build: Vec<Option<String>> = if let Some(t) = target {
        vec![Some(normalize_target_os(t).to_string())]
    } else {
        let config_targets = get_targets(&config, profile)?;
        if config_targets.is_empty() {
            vec![None]
        } else {
            config_targets
                .into_iter()
                .map(|t| Some(normalize_target_os(&t).to_string()))
                .collect()
        }
    };

    let start_time = Instant::now();
    for (i, build_target) in targets_to_build.iter().enumerate() {
        check_interrupted()?;
        if targets_to_build.len() > 1 {
            println!(
                "    Building target {} of {}: {}",
                i + 1,
                targets_to_build.len(),
                build_target.as_ref().map_or("native", |t| t.as_str())
            );
        } else {
            println!(
                "    Building project `{}`\n    Profile: {}\n      Target: {}",
                colored(&project_name, BOLD_GREEN),
                colored(profile, BOLD_GREEN),
                colored(
                    build_target.as_ref().map_or("native", |t| t.as_str()),
                    BOLD_GREEN
                )
            );
        }
        if let Some(ws) = parse_workspace(&config, profile, build_target.as_deref(), root)? {
            if let Some(filter_name) = workspace {
                for member in ws.members.iter().filter(|m| m.name == filter_name) {
                    build_project_at(
                        &member.path,
                        profile,
                        build_target.as_deref(),
                        &[],
                        force,
                        verbose,
                        Some(root),
                    )?;
                }
            } else {
                build_workspace(
                    &ws,
                    profile,
                    build_target.as_deref(),
                    force,
                    verbose,
                    Some(root),
                )?;
                let excludes: Vec<std::path::PathBuf> =
                    ws.members.iter().map(|m| m.path.clone()).collect();
                build_project_at(
                    root,
                    profile,
                    build_target.as_deref(),
                    &excludes,
                    force,
                    verbose,
                    None,
                )?;
            }
        } else {
            build_project_at(
                root,
                profile,
                build_target.as_deref(),
                &[],
                force,
                verbose,
                None,
            )?;
        }
    }
    let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
    println!(
        "    {} Build completed successfully in {} seconds",
        colored("✔", BOLD_GREEN),
        colored(&elapsed.to_string(), BOLD_GREEN)
    );
    Ok(())
}

fn build_workspace(
    workspace: &crate::core::workspace::Workspace,
    profile: &str,
    target: Option<&str>,
    force: bool,
    verbose: bool,
    workspace_root: Option<&Path>,
) -> Result<(), String> {
    for member in &workspace.members {
        build_project_at(
            &member.path,
            profile,
            target,
            &[],
            force,
            verbose,
            workspace_root,
        )?;
    }
    Ok(())
}

fn build_project_at(
    project_root: &Path,
    profile: &str,
    target: Option<&str>,
    exclude_dirs: &[std::path::PathBuf],
    force: bool,
    verbose: bool,
    workspace_root: Option<&Path>,
) -> Result<(), String> {
    with_dir(project_root, || {
        check_interrupted()?;
        let items = check_dir(None).map_err(|_| "Failed to read project directory".to_string())?;
        if !items.contains(&"dcr.toml".to_string()) {
            return Err("dcr.toml file not found".to_string());
        }
        let mut config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;
        // Inherit build fields from workspace root if configured
        if let Some(root) = workspace_root {
            let has_inherit = config
                .get("build.inherit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if has_inherit {
                let root_path = root.join("dcr.toml");
                if root_path.is_file() {
                    let parent_config =
                        Config::open(&root_path.to_string_lossy()).map_err(|e| e.to_string())?;
                    config.merge_parent(&parent_config);
                }
            }
        }
        let project_name = get_config_str(&config, "package.name");
        let project_version = get_config_str(&config, "package.version");
        let build_target_config = get_build_string_with_profile(&config, "target", profile);
        let build_target = target.or(if build_target_config.is_empty() {
            None
        } else {
            Some(build_target_config.as_str())
        });
        let project_compiler =
            get_string_with_profile_and_target(&config, "compiler", profile, build_target);
        let build_language = get_language_with_profile(&config, profile)?;
        let build_standard =
            get_string_with_profile_and_target(&config, "standard", profile, build_target);
        let build_cxx_standard =
            get_string_with_profile_and_target(&config, "cxx_standard", profile, build_target);
        let build_kind = get_string_with_profile_and_target(&config, "kind", profile, build_target);
        let build_platform =
            get_string_with_profile_and_target(&config, "platform", profile, build_target);
        let mut build_type =
            get_string_with_profile_and_target(&config, "type", profile, build_target);
        if build_type.is_empty() {
            build_type = get_config_str(&config, "package.type");
        }
        let tc_cc = get_config_value(&config, "toolchain", "cc", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.cc"));
        let tc_cxx = get_config_value(&config, "toolchain", "cxx", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.cxx"));
        let tc_as = get_config_value(&config, "toolchain", "as", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.as"));
        let tc_ar = get_config_value(&config, "toolchain", "ar", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.ar"));
        let tc_ld = get_config_value(&config, "toolchain", "ld", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.ld"));
        let tc_uic = get_config_value(&config, "toolchain", "uic", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.uic"));
        let tc_moc = get_config_value(&config, "toolchain", "moc", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.moc"));
        let tc_rcc = get_config_value(&config, "toolchain", "rcc", profile, build_target)
            .or_else(|| get_config_opt(&config, "toolchain.rcc"));
        let mut build_cflags =
            get_list_with_profile_and_target(&config, "cflags", profile, build_target)?;
        let mut build_ldflags =
            get_list_with_profile_and_target(&config, "ldflags", profile, build_target)?;
        let build_ldscript =
            get_string_with_profile_and_target(&config, "ldscript", profile, build_target);
        if !build_ldscript.is_empty() {
            build_ldflags.push(format!("-T{}", build_ldscript));
        }

        // Новые поля: filename и extension
        let output_filename =
            get_string_with_profile_and_target(&config, "filename", profile, build_target);
        let output_extension =
            get_string_with_profile_and_target(&config, "extension", profile, build_target);
        let build_excludes =
            get_list_with_profile_and_target(&config, "exclude", profile, build_target)?;
        let build_includes =
            get_list_with_profile_and_target(&config, "include", profile, build_target)?;
        let build_roots =
            get_list_with_profile_and_target(&config, "roots", profile, build_target)?;
        let src_disable = get_bool_with_profile(&config, "src_disable", profile, false);
        let build_expects =
            get_list_with_profile_and_target(&config, "expect", profile, build_target)?;
        let pkg_configs =
            get_list_with_profile_and_target(&config, "pkg_config", profile, build_target)?;
        let build_generated =
            get_list_with_profile_and_target(&config, "generated", profile, build_target)?;
        let build_steps = get_build_steps_with_profile(&config, "steps", profile)?;
        let build_post_steps = get_build_steps_with_profile(&config, "post_steps", profile)?;

        let resolved_compiler = resolve_compiler(
            &build_language,
            &project_compiler,
            tc_cc.as_deref(),
            tc_cxx.as_deref(),
            tc_as.as_deref(),
        );
        if let Some(t) = build_target
            && !t.trim().is_empty()
            && resolved_compiler.to_lowercase().contains("clang")
        {
            let target_flag = format!("--target={}", t.trim());
            if !build_cflags
                .iter()
                .any(|f| f == &target_flag || f.starts_with("--target="))
            {
                build_cflags.insert(0, target_flag);
            }
        }
        let resolved_linker = resolve_tool("DCR_LD", tc_ld.as_deref());
        let resolved_archiver = resolve_tool("DCR_AR", tc_ar.as_deref());

        let out_dir_config = get_build_string_with_profile(&config, "out_dir", profile);
        let target_dir_binding = if !out_dir_config.is_empty() {
            out_dir_config
        } else {
            match workspace_root {
                Some(root) => {
                    let target_str = build_target
                        .map(normalize_target_os)
                        .filter(|t| !t.is_empty());
                    let rel: PathBuf = match target_str {
                        Some(ref t) => Path::new("target").join(t).join(profile),
                        None => {
                            let default_dir = if cfg!(target_os = "linux") {
                                let arch = std::env::consts::ARCH;
                                format!("{arch}-unknown-linux-gnu/{profile}")
                            } else {
                                profile.to_string()
                            };
                            Path::new("target").join(&default_dir)
                        }
                    };
                    root.join(&rel).to_string_lossy().to_string()
                }
                None => normalize_target(build_target.unwrap_or(""), profile).unwrap_or_default(),
            }
        };
        let target_dir = if target_dir_binding.is_empty() {
            None
        } else {
            Some(target_dir_binding.clone())
        };
        ensure_target_dirs(&items, profile, target_dir, workspace_root.is_some());

        let deps_table = config.get("dependencies").and_then(|v| v.as_table());
        let resolved = resolve_deps(&config, profile, build_target, project_root)?;

        // Registry dependencies are cached under the DCR registry root. Build
        // them as normal DCR projects before the current project is linked.
        if let Some(deps) = deps_table {
            for (name, value) in deps {
                if register::is_registry_dep(value) {
                    let pkg_info = register::resolve_package_from_registry(name)?;
                    let version = pkg_info
                        .get("latest_version")
                        .or_else(|| pkg_info.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let dep_root = register::package_root_from_registry_info(&pkg_info)?;
                    let include_dir = dep_root.join("target").join("include");
                    let lib_dir = dep_root.join("target").join("lib");

                    if !include_dir.exists() || !lib_dir.exists() {
                        print!(
                            "\r{:100}\r      {} {} v{}",
                            "",
                            colored("Building", BOLD_GREEN),
                            name,
                            version
                        );
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        if !dep_root.join("dcr.toml").is_file() {
                            return Err(format!(
                                "Registry dependency `{}` is missing dcr.toml at {}",
                                name,
                                dep_root.display()
                            ));
                        }
                        build_project_at(
                            &dep_root,
                            profile,
                            build_target,
                            &[],
                            force,
                            verbose,
                            None,
                        )?;
                        print!(
                            "\r{:100}\r       {} {} v{}",
                            "",
                            colored("Ready", BOLD_GREEN),
                            name,
                            version
                        );
                        println!();
                    } else {
                        println!(
                            "      {} {} v{}",
                            colored("Ready", BOLD_GREEN),
                            name,
                            version
                        );
                    }
                }
            }
        }

        let (resolved_cflags, resolved_ldflags) =
            resolve_pkg_config_flags(&pkg_configs, &build_cflags, &build_ldflags)?;
        let mut combined_excludes = Vec::new();
        for dir in exclude_dirs {
            combined_excludes.push(dir.clone());
        }
        let mut exclude_patterns = Vec::new();
        for raw in build_excludes {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalized = trimmed.replace('\\', "/");
            if common::has_glob_magic(&normalized) {
                exclude_patterns.push(normalized);
                continue;
            }
            let p = Path::new(trimmed);
            if p.is_absolute() {
                combined_excludes.push(p.to_path_buf());
                exclude_patterns.push(normalized);
            } else {
                combined_excludes.push(project_root.join(p));
                exclude_patterns.push(normalized);
            }
        }
        let mut combined_includes: Vec<String> = Vec::new();
        combined_includes.extend(exclude_patterns.iter().map(|v| format!("!{v}")));
        combined_includes.extend(build_includes.iter().map(|v| v.replace('\\', "/")));

        let mut source_roots: Vec<PathBuf> = Vec::new();
        for raw in &build_roots {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let p = Path::new(trimmed);
            if p.is_absolute() {
                source_roots.push(p.to_path_buf());
            } else {
                source_roots.push(project_root.join(p));
            }
        }
        if !src_disable && source_roots.is_empty() {
            source_roots.push(project_root.join("src"));
        }

        let mut merged_include_dirs = resolved.include_dirs.clone();
        for raw in &build_includes {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalized = trimmed.replace('\\', "/");
            if common::has_glob_magic(&normalized) {
                continue;
            }
            let p = Path::new(trimmed);
            let dir = if p.is_absolute() {
                p.to_path_buf()
            } else {
                project_root.join(p)
            };
            if dir.is_dir() {
                merged_include_dirs.push(dir.to_string_lossy().to_string());
            }
        }

        let version_info = parse_version_info(&project_version);
        let substitute = |s: &str| -> String {
            s.replace("{profile}", profile)
                .replace("{name}", &project_name)
                .replace("{version}", &version_info.full)
                .replace("{version_major}", &version_info.major)
                .replace("{version_minor}", &version_info.minor)
                .replace("{version_patch}", &version_info.patch)
                .replace("{version_suffix}", &version_info.suffix)
                .replace("{version_suffix_dash}", &version_info.suffix_dash)
        };
        let resolved_cflags: Vec<String> = resolved_cflags.iter().map(|f| substitute(f)).collect();
        let modified_ldflags: Vec<String> =
            resolved_ldflags.iter().map(|f| substitute(f)).collect();
        let merged_include_dirs: Vec<String> =
            merged_include_dirs.iter().map(|f| substitute(f)).collect();
        let ctx = BuildContext {
            profile,
            project_name: &project_name,
            compiler: &resolved_compiler,
            language: &build_language,
            standard: &build_standard,
            cxx_standard: &build_cxx_standard,
            target: build_target,
            target_dir: if target_dir_binding.is_empty() {
                None
            } else {
                Some(target_dir_binding.as_str())
            },
            kind: normalize_kind(&build_kind),
            platform: normalize_platform(&build_platform),
            linker: resolved_linker.as_deref(),
            archiver: resolved_archiver.as_deref(),
            package_type: if build_type.is_empty() {
                None
            } else {
                Some(build_type.as_str())
            },
            source_roots: &source_roots,
            exclude_dirs: &combined_excludes,
            include_paths: &combined_includes,
            include_dirs: &merged_include_dirs,
            lib_dirs: &resolved.lib_dirs,
            libs: &resolved.libs,
            cflags: &resolved_cflags,
            ldflags: &modified_ldflags,
            output_filename: if output_filename.is_empty() {
                None
            } else {
                Some(output_filename.as_str())
            },
            output_extension: if output_extension.is_empty() {
                None
            } else {
                Some(output_extension.as_str())
            },
            verbose,
        };
        if std::env::var("DCR_DEBUG").is_ok() {
            eprintln!("[dcr] debug: compiler={}", ctx.compiler);
            eprintln!("[dcr] debug: cflags={:?}", ctx.cflags);
            eprintln!("[dcr] debug: ldflags={:?}", ctx.ldflags);
            eprintln!("[dcr] debug: lib_dirs={:?}", ctx.lib_dirs);
            eprintln!("[dcr] debug: libs={:?}", ctx.libs);
        }
        let tool_execs = resolve_toolchain_execs(&tc_uic, &tc_moc, &tc_rcc, &pkg_configs);
        let step_flags =
            build_step_flags(&resolved_cflags, &resolved.include_dirs, &resolved_compiler);
        let step_vars = StepVars {
            profile,
            version: &version_info.full,
            version_major: &version_info.major,
            version_minor: &version_info.minor,
            version_patch: &version_info.patch,
            version_suffix: &version_info.suffix,
            version_suffix_dash: &version_info.suffix_dash,
        };
        let steps_dirty = build_steps_need_run(&build_steps, &step_vars)?;
        if steps_dirty {
            clean_generated_files(&build_generated)?;
            run_build_steps(&build_steps, &tool_execs, &step_flags, &step_vars)?;
        }
        let sources = collect_sources(&ctx)?;
        let headers = collect_header_files(&ctx, project_root)?;
        let lib_files = collect_lib_files(&ctx);
        let fingerprint = compute_build_fingerprint(&ctx, &sources, &headers, &lib_files)?;
        let mut skip = should_skip_build(&ctx, &fingerprint);
        if steps_dirty {
            skip = false;
        }
        let debug_enabled = std::env::var("DCR_DEBUG").is_ok();
        if force {
            skip = false;
        }
        if skip && !debug_enabled {
            return Ok(());
        }
        println!(
            "   {} {} v{}",
            colored("Compiling", BOLD_GREEN),
            project_name,
            project_version
        );
        if !skip {
            run_build(&ctx)?;
            check_interrupted()?;
            write_build_fingerprint(&ctx, &fingerprint)?;
        }
        if ctx.package_type == Some("lib") {
            package_library(&ctx, &headers)?;
        }
        let post_steps_dirty = build_steps_need_run(&build_post_steps, &step_vars)?;
        if post_steps_dirty {
            run_build_steps(&build_post_steps, &tool_execs, &step_flags, &step_vars)?;
        }
        verify_expectations(&build_expects, &step_vars)?;
        Ok(())
    })
}

fn package_library(ctx: &BuildContext, headers: &[PathBuf]) -> Result<(), String> {
    let target_root = if let Some(dir) = ctx.target_dir {
        Path::new(dir)
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("target"))
    } else {
        Path::new("target")
    };

    let include_dir = target_root.join("include");
    let lib_dir = target_root.join("lib");

    fs::create_dir_all(&include_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&lib_dir).map_err(|e| e.to_string())?;

    for header in headers {
        if let Some(name) = header.file_name() {
            let dest = include_dir.join(name);
            fs::copy(header, dest)
                .map_err(|e| format!("Failed to copy header {:?}: {}", header, e))?;
        }
    }

    let mut outputs = Vec::new();
    if ctx.kind == "staticlib" || ctx.kind == "any" {
        outputs.push(crate::platform::lib_path(
            ctx.profile,
            ctx.project_name,
            ctx.target_dir,
        ));
    }
    if ctx.kind == "sharedlib" || ctx.kind == "any" {
        outputs.push(crate::platform::shared_lib_path(
            ctx.profile,
            ctx.project_name,
            ctx.target_dir,
        ));
    }

    for output in outputs {
        let path = Path::new(&output);
        if path.exists()
            && let Some(name) = path.file_name()
        {
            let dest = lib_dir.join(name);
            fs::copy(path, dest).map_err(|e| format!("Failed to copy lib {:?}: {}", path, e))?;
        }
    }

    Ok(())
}

pub fn normalize_target(target: &str, profile: &str) -> Option<String> {
    let trimmed = normalize_target_os(target.trim());
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("target/{trimmed}/{profile}"))
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

struct ToolchainExecs {
    uic: String,
    moc: String,
    rcc: String,
}

fn resolve_toolchain_execs(
    uic: &Option<String>,
    moc: &Option<String>,
    rcc: &Option<String>,
    pkg_configs: &[String],
) -> ToolchainExecs {
    let qt_bins = resolve_qt_host_bins(pkg_configs);
    ToolchainExecs {
        uic: resolve_qt_tool(uic, qt_bins.as_deref(), "uic"),
        moc: resolve_qt_tool(moc, qt_bins.as_deref(), "moc"),
        rcc: resolve_qt_tool(rcc, qt_bins.as_deref(), "rcc"),
    }
}

fn resolve_qt_tool(configured: &Option<String>, qt_bins: Option<&Path>, tool: &str) -> String {
    if let Some(value) = configured {
        return value.clone();
    }
    if let Some(dir) = qt_bins {
        let candidate = dir.join(tool);
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
        if cfg!(target_os = "windows") {
            let candidate = dir.join(format!("{tool}.exe"));
            if candidate.is_file() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    if let Some(candidate) = detect_qt6_tool_variant(tool) {
        return candidate;
    }
    tool.to_string()
}

fn resolve_qt_host_bins(pkgs: &[String]) -> Option<PathBuf> {
    let qt_pkgs: Vec<&String> = pkgs.iter().filter(|p| p.starts_with("Qt6")).collect();
    if qt_pkgs.is_empty() {
        return None;
    }
    let preferred = ["Qt6Core", "Qt6Widgets", "Qt6Gui"];
    for name in preferred {
        if let Some(dir) = query_pkg_config_var(name, "host_bins") {
            return Some(dir);
        }
        if let Some(dir) = query_pkg_config_var(name, "libexecdir")
            && let Some(bin) = qt_bins_from_libexec(&dir)
        {
            return Some(bin);
        }
        if let Some(dir) = query_pkg_config_var(name, "bindir") {
            return Some(dir);
        }
    }
    for pkg in qt_pkgs {
        if let Some(dir) = query_pkg_config_var(pkg, "host_bins") {
            return Some(dir);
        }
        if let Some(dir) = query_pkg_config_var(pkg, "libexecdir")
            && let Some(bin) = qt_bins_from_libexec(&dir)
        {
            return Some(bin);
        }
        if let Some(dir) = query_pkg_config_var(pkg, "bindir") {
            return Some(dir);
        }
    }
    None
}

fn query_pkg_config_var(pkg: &str, var: &str) -> Option<PathBuf> {
    let output = std::process::Command::new("pkg-config")
        .arg(format!("--variable={var}"))
        .arg(pkg)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    if path.is_dir() { Some(path) } else { None }
}

fn qt_bins_from_libexec(libexec: &Path) -> Option<PathBuf> {
    for tool in ["moc", "uic", "rcc"] {
        if libexec.join(tool).is_file() {
            return Some(libexec.to_path_buf());
        }
        if cfg!(target_os = "windows") && libexec.join(format!("{tool}.exe")).is_file() {
            return Some(libexec.to_path_buf());
        }
    }
    let bin = libexec.join("bin");
    if bin.is_dir() { Some(bin) } else { None }
}

fn detect_qt6_tool_variant(tool: &str) -> Option<String> {
    [format!("{tool}6"), format!("{tool}-qt6")]
        .into_iter()
        .find(|candidate| is_on_path(candidate))
}

fn is_on_path(cmd: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[derive(Clone)]
struct BuildStep {
    name: String,
    input: String,
    output: String,
    cmd: String,
}

fn get_build_steps(config: &Config, key: &str) -> Result<Vec<BuildStep>, String> {
    let value = match config.get(key) {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    get_build_steps_from_value(value, key)
}

fn run_build_steps(
    steps: &[BuildStep],
    tools: &ToolchainExecs,
    step_flags: &str,
    vars: &StepVars,
) -> Result<(), String> {
    for step in steps {
        run_build_step(step, tools, step_flags, vars)?;
    }
    Ok(())
}

fn run_build_step(
    step: &BuildStep,
    tools: &ToolchainExecs,
    step_flags: &str,
    vars: &StepVars,
) -> Result<(), String> {
    let input_pattern = expand_step_value(&step.input, "", vars);
    let inputs = expand_glob(&input_pattern)?;
    if inputs.is_empty() {
        return Ok(());
    }
    let needs_stem = step.output.contains("{stem}");
    if inputs.len() > 1 && !needs_stem {
        return Err(format!(
            "build.steps '{}' output must include {{stem}} for multiple inputs",
            step.name
        ));
    }
    for input in inputs {
        if !input.is_file() {
            continue;
        }
        let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
        let out_path = PathBuf::from(expand_step_value(&step.output, stem, vars));
        if !should_run_step(&input, &out_path) {
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create step output dir: {err}"))?;
        }
        let cmd = substitute_step_cmd(&step.cmd, &input, &out_path, tools, step_flags, stem, vars);
        let status = run_shell_command(&cmd)
            .map_err(|err| format!("Failed to run step '{}': {err}", step.name))?;
        if !status.success() {
            return Err(format!("Step '{}' failed", step.name));
        }
    }
    Ok(())
}

fn clean_generated_files(patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        for path in expand_glob(pattern)? {
            if path.is_file() {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

fn expand_glob(pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let entries = glob(pattern).map_err(|err| format!("glob error: {err}"))?;
    for entry in entries {
        let path = entry.map_err(|err| format!("glob error: {err}"))?;
        out.push(path);
    }
    Ok(out)
}

fn verify_expectations(patterns: &[String], vars: &StepVars) -> Result<(), String> {
    for pattern in patterns {
        let expanded = expand_step_value(pattern, "", vars);
        let matches = expand_glob(&expanded)?;
        if matches.is_empty() {
            return Err(format!("Expected artifact not found: {expanded}"));
        }
    }
    Ok(())
}

fn should_run_step(input: &Path, output: &Path) -> bool {
    let in_time = fs::metadata(input).and_then(|m| m.modified());
    let out_time = fs::metadata(output).and_then(|m| m.modified());
    match (in_time, out_time) {
        (Ok(i), Ok(o)) => i > o,
        (Ok(_), Err(_)) => true,
        _ => true,
    }
}

fn substitute_step_cmd(
    template: &str,
    input: &Path,
    output: &Path,
    tools: &ToolchainExecs,
    step_flags: &str,
    stem: &str,
    vars: &StepVars,
) -> String {
    template
        .replace("{in}", &input.to_string_lossy())
        .replace("{out}", &output.to_string_lossy())
        .replace("{uic}", &tools.uic)
        .replace("{moc}", &tools.moc)
        .replace("{rcc}", &tools.rcc)
        .replace("{cflags}", step_flags)
        .replace("{stem}", stem)
        .replace("{profile}", vars.profile)
        .replace("{version}", vars.version)
        .replace("{version_major}", vars.version_major)
        .replace("{version_minor}", vars.version_minor)
        .replace("{version_patch}", vars.version_patch)
        .replace("{version_suffix}", vars.version_suffix)
        .replace("{version_suffix_dash}", vars.version_suffix_dash)
}

fn build_step_flags(cflags: &[String], include_dirs: &[String], compiler: &str) -> String {
    let mut out = Vec::new();
    let msvc_style = is_msvc_compiler(compiler) || cflags.iter().any(|f| f.starts_with('/'));
    for flag in cflags {
        if flag.starts_with("-I") || flag.starts_with("-D") {
            out.push(flag.clone());
        }
        if flag.starts_with("/I") || flag.starts_with("/D") {
            out.push(flag.clone());
        }
        if msvc_style && flag.starts_with("-D") {
            out.push(format!("/D{}", flag.trim_start_matches("-D")));
        }
    }
    for dir in include_dirs {
        out.push(format!("-I{dir}"));
        if msvc_style {
            out.push(format!("/I{dir}"));
        }
    }
    let mut dedup = Vec::new();
    for item in out {
        if !dedup.contains(&item) {
            dedup.push(item);
        }
    }
    dedup
        .into_iter()
        .map(quote_step_arg)
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_step_arg(arg: String) -> String {
    if !arg.chars().any(|c| c.is_whitespace() || c == '"') {
        return arg;
    }
    let escaped = arg.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn is_msvc_compiler(compiler: &str) -> bool {
    let lower = compiler.to_lowercase();
    lower.contains("cl.exe")
        || lower == "cl"
        || lower.contains("clang-cl")
        || lower.contains("msvc")
}

fn run_shell_command(cmd: &str) -> Result<std::process::ExitStatus, std::io::Error> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .status()
    } else {
        std::process::Command::new("sh").arg("-c").arg(cmd).status()
    }
}

fn build_steps_need_run(steps: &[BuildStep], vars: &StepVars) -> Result<bool, String> {
    for step in steps {
        let input_pattern = expand_step_value(&step.input, "", vars);
        let inputs = expand_glob(&input_pattern)?;
        if inputs.is_empty() {
            continue;
        }
        let needs_stem = step.output.contains("{stem}");
        if inputs.len() > 1 && !needs_stem {
            return Err(format!(
                "build.steps '{}' output must include {{stem}} for multiple inputs",
                step.name
            ));
        }
        for input in inputs {
            if !input.is_file() {
                continue;
            }
            let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
            let out_path = PathBuf::from(expand_step_value(&step.output, stem, vars));
            if should_run_step(&input, &out_path) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn expand_step_value(template: &str, stem: &str, vars: &StepVars) -> String {
    template
        .replace("{stem}", stem)
        .replace("{profile}", vars.profile)
        .replace("{version}", vars.version)
        .replace("{version_major}", vars.version_major)
        .replace("{version_minor}", vars.version_minor)
        .replace("{version_patch}", vars.version_patch)
        .replace("{version_suffix}", vars.version_suffix)
        .replace("{version_suffix_dash}", vars.version_suffix_dash)
}

struct StepVars<'a> {
    profile: &'a str,
    version: &'a str,
    version_major: &'a str,
    version_minor: &'a str,
    version_patch: &'a str,
    version_suffix: &'a str,
    version_suffix_dash: &'a str,
}

fn compute_build_fingerprint(
    ctx: &BuildContext,
    sources: &[String],
    headers: &[std::path::PathBuf],
    lib_files: &[std::path::PathBuf],
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(ctx.profile.as_bytes());
    hasher.update(ctx.project_name.as_bytes());
    hasher.update(ctx.compiler.as_bytes());
    hasher.update(ctx.language.as_bytes());
    hasher.update(ctx.standard.as_bytes());
    hasher.update(ctx.kind.as_bytes());
    if let Some(v) = ctx.target_dir {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.platform {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.linker {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.archiver {
        hasher.update(v.as_bytes());
    }
    for value in ctx.include_dirs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.lib_dirs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.libs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.cflags {
        hasher.update(value.as_bytes());
    }
    for value in ctx.ldflags {
        hasher.update(value.as_bytes());
    }
    let toml =
        fs::read_to_string("dcr.toml").map_err(|err| format!("Failed to read dcr.toml: {err}"))?;
    hasher.update(toml.as_bytes());
    if let Ok(lock) = fs::read_to_string("dcr.lock") {
        hasher.update(lock.as_bytes());
    }
    for source in sources {
        let path = Path::new(source);
        update_hasher_with_file(&mut hasher, path)?;
    }
    for header in headers {
        update_hasher_with_file(&mut hasher, header)?;
    }
    for lib in lib_files {
        update_hasher_with_file(&mut hasher, lib)?;
    }
    Ok(to_hex(&hasher.finalize()))
}

fn should_skip_build(ctx: &BuildContext, fingerprint: &str) -> bool {
    let output = build_output_path(ctx);
    if !Path::new(&output).is_file() {
        return false;
    }
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    let cached = fs::read_to_string(cache_path).unwrap_or_default();
    cached.trim() == fingerprint
}

fn write_build_fingerprint(ctx: &BuildContext, fingerprint: &str) -> Result<(), String> {
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Failed to create cache dir: {err}"))?;
    }
    fs::write(cache_path, format!("{fingerprint}\n"))
        .map_err(|err| format!("Failed to write cache: {err}"))
}

fn build_cache_path(profile: &str, target_dir: Option<&str>) -> std::path::PathBuf {
    match target_dir {
        Some(dir) => Path::new(dir).join(".dcr-build.hash"),
        None => Path::new("./target").join(profile).join(".dcr-build.hash"),
    }
}

fn build_output_path(ctx: &BuildContext) -> String {
    let name = ctx.output_filename.unwrap_or(ctx.project_name);
    let ext = ctx.output_extension.unwrap_or("");

    let final_name = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    };

    if ctx.kind == "staticlib" {
        return crate::platform::lib_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "sharedlib" {
        return crate::platform::shared_lib_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "efi" {
        return crate::platform::efi_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "elf" {
        return crate::platform::elf_path(ctx.profile, &final_name, ctx.target_dir);
    }
    crate::platform::bin_path(ctx.profile, &final_name, ctx.target_dir)
}

fn collect_header_files(
    ctx: &BuildContext,
    project_root: &Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut out = Vec::new();
    let mut roots = Vec::new();
    if ctx.source_roots.is_empty() {
        roots.push(project_root.join("src"));
    } else {
        roots.extend(ctx.source_roots.iter().cloned());
    }
    for dir in ctx.include_dirs {
        roots.push(Path::new(dir).to_path_buf());
    }
    for root in roots {
        if !root.exists() {
            continue;
        }
        collect_header_files_rec(&root, &mut out, ctx.exclude_dirs, ctx.include_paths)?;
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn collect_header_files_rec(
    dir: &Path,
    out: &mut Vec<std::path::PathBuf>,
    exclude_dirs: &[std::path::PathBuf],
    include_paths: &[String],
) -> Result<(), String> {
    if common::is_excluded(dir, exclude_dirs, include_paths) && include_paths.is_empty() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| format!("read_dir error: {err}"))? {
        let entry = entry.map_err(|err| format!("read_dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            if common::is_excluded(&path, exclude_dirs, include_paths) && include_paths.is_empty() {
                continue;
            }
            collect_header_files_rec(&path, out, exclude_dirs, include_paths)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if common::is_excluded(&path, exclude_dirs, include_paths) {
            continue;
        }
        if is_header_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_header_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
    matches!(ext, "h" | "hpp" | "hh" | "hxx" | "inc")
}

fn collect_lib_files(ctx: &BuildContext) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for dir in ctx.lib_dirs {
        let dir_path = Path::new(dir);
        for lib in ctx.libs {
            for candidate in lib_candidates(lib) {
                let path = dir_path.join(candidate);
                if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out
}

fn lib_candidates(name: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        return vec![format!("{name}.lib")];
    }
    if cfg!(target_os = "macos") {
        return vec![
            format!("lib{name}.a"),
            format!("lib{name}.dylib"),
            format!("lib{name}.so"),
        ];
    }
    vec![
        format!("lib{name}.a"),
        format!("lib{name}.so"),
        format!("lib{name}.so.0"),
    ]
}

fn update_hasher_with_file(hasher: &mut Sha256, path: &Path) -> Result<(), String> {
    hasher.update(path.to_string_lossy().as_bytes());
    let meta = fs::metadata(path).map_err(|err| format!("source read error: {err}"))?;
    hasher.update(meta.len().to_le_bytes());
    if let Ok(modified) = meta.modified()
        && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
    {
        hasher.update(duration.as_nanos().to_le_bytes());
    }
    Ok(())
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}
