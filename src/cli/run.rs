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

use crate::cli::build::build;
use crate::cli::flags::parse_build_run_flags;
use crate::core::config::Config;
use crate::core::runner::run_binary;
use crate::utils::build::{default_target_triple, normalize_target_os, parse_version_info};
use crate::utils::fs::find_project_root;
use crate::utils::fs::with_dir;
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use std::path::Path;
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
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr run [--debug | --release] [--target <triple>] [--force] [--clean] [--verbose]",
            BOLD_CYAN,
        );
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Builds and runs the project. Only available for kind = \"bin\".");
        println!();
        printc("OPTIONS:", BOLD_GREEN);
        println!("    --debug              Run with debug profile (default)");
        println!("    --release            Run with release profile");
        println!("    --target <triple>    Cross-compile for the given target");
        println!("    --force              Force a full rebuild");
        println!("    --clean              Clean before building");
        println!("    --verbose            Print detailed build output");
        return 0;
    }

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
    let flags = match parse_build_run_flags(args) {
        Ok(v) => v,
        Err(_) => return 1,
    };

    // Handle workspace-only root: delegate to workspace member
    if config.is_workspace_only() {
        let ws = match crate::core::workspace::parse_workspace(
            &config,
            &flags.profile,
            flags.target.as_deref(),
            &root,
        ) {
            Ok(Some(ws)) => ws,
            Ok(None) => {
                error("Workspace root has no members defined");
                return 1;
            }
            Err(e) => {
                error(&e);
                return 1;
            }
        };
        let member = match &flags.workspace {
            Some(name) => ws.members.iter().find(|m| m.name == *name),
            None => ws.main_member(),
        };
        let member = match member {
            Some(m) => m,
            None => {
                if let Some(name) = &flags.workspace {
                    error(&format!("Workspace member '{name}' not found"));
                } else {
                    error("No workspace member to run (set `main = true` on one member)");
                }
                return 1;
            }
        };
        // Build and run from the member's directory
        return match with_dir(&member.path, || {
            run_project(&member.path, &flags, Some(root.as_path()))
        }) {
            Ok(code) => code,
            Err(e) => {
                error(&e);
                1
            }
        };
    }

    match run_project(&root, &flags, None) {
        Ok(code) => code,
        Err(e) => {
            error(&e);
            1
        }
    }
}

fn run_project(
    root: &Path,
    flags: &crate::cli::flags::BuildRunFlags,
    workspace_root: Option<&Path>,
) -> Result<i32, String> {
    let config =
        Config::open(root.join("dcr.toml").to_str().unwrap()).map_err(|err| err.to_string())?;

    let project_name: &str = config
        .get("package.name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // If no target specified, use default host target for target-specific config
    let mut target = flags.target.clone();
    if target.is_none() {
        target = Some(default_target_triple());
    }

    let build_kind = config
        .get(&format!("build.{}.kind", flags.profile))
        .and_then(|v| v.as_str())
        .or_else(|| config.get("build.kind").and_then(|v| v.as_str()))
        .unwrap_or("");

    let out_dir =
        crate::cli::build::get_build_string_with_profile(&config, "out_dir", &flags.profile);
    let normalized_target_dir = if !out_dir.is_empty() {
        Some(out_dir)
    } else {
        match workspace_root {
            Some(wr) => target
                .as_ref()
                .and_then(|t| crate::cli::build::normalize_target(t, &flags.profile))
                .map(|rel| wr.join(&rel).to_string_lossy().to_string()),
            None => target
                .as_ref()
                .and_then(|t| crate::cli::build::normalize_target(t, &flags.profile)),
        }
    };

    let version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let run_cmd = get_run_cmd(&config, &flags.profile, target.as_deref(), version);

    let kind = build_kind.trim();
    if run_cmd.is_none()
        && (kind == "staticlib" || kind == "sharedlib" || kind == "efi" || kind == "elf")
    {
        return Err("Cannot run library build".to_string());
    }

    let build_status = build(&args_for_build(flags));
    let bin_path = crate::platform::bin_path(
        &flags.profile,
        project_name,
        normalized_target_dir.as_deref(),
    );
    if build_status == 0 {
        if let Some(cmd) = run_cmd {
            println!("\n    {} {}", colored("Running", BOLD_GREEN), cmd);
            println!("--------------------------------");
            return Ok(run_shell(&cmd));
        }
        println!("\n    {} {}", colored("Running", BOLD_GREEN), bin_path);
        println!("--------------------------------");
        return Ok(run_binary(
            project_name,
            &flags.profile,
            normalized_target_dir.as_deref(),
        ));
    }

    let fallback_code = if let Some(cmd) = run_cmd {
        run_shell(&cmd)
    } else {
        run_binary(
            project_name,
            &flags.profile,
            normalized_target_dir.as_deref(),
        )
    };
    if fallback_code != 1 {
        return Ok(fallback_code);
    }

    Err("Fix errors in the code to run the project".to_string())
}

fn args_for_build(flags: &crate::cli::flags::BuildRunFlags) -> Vec<String> {
    let mut args = Vec::new();
    args.push(format!("--{}", flags.profile));
    if let Some(ref target) = flags.target {
        args.push("--target".to_string());
        args.push(target.clone());
    }
    if let Some(ref name) = flags.workspace {
        args.push("--workspace".to_string());
        args.push(name.clone());
    }
    if flags.verbose {
        args.push("--verbose".to_string());
    }
    args
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
