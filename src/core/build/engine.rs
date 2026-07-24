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

use crate::core::build::builder::{BuildContext, build as build_project, collect_sources};
use crate::core::build::cache::{
    collect_header_files, collect_lib_files, compute_build_fingerprint, should_skip_build,
    write_build_fingerprint,
};
use crate::core::build::common;
use crate::core::build::report::{
    BuildError, BuildEvent, BuildOutcome, BuildReporter, BuildRequest,
};
use crate::core::build::resolve::{
    get_build_string_with_profile, get_config_value, get_lang_list, get_lang_string,
    get_list_with_profile_and_target, get_string_with_profile_and_target, get_targets,
};
use crate::core::build::steps::{
    StepVars, build_step_flags, build_steps_need_run, clean_generated_files,
    get_build_steps_with_profile, run_build_steps, verify_expectations,
};
use crate::core::build_config::Config;
use crate::core::deps::{register, resolve_deps};
use crate::core::workspace::parse_workspace;
use crate::utils::build::{
    get_bool_with_profile, get_config_opt, get_config_str, get_language_with_profile,
    is_bare_metal_target, normalize_kind, normalize_platform, normalize_target,
    normalize_target_os, parse_version_info, prepend_clang_target_flag, primary_language,
    resolve_compiler, resolve_pkg_config_flags, resolve_tool, substitute_vars,
};
use crate::utils::fs::check_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

fn ensure_target_dirs(
    project_root: &Path,
    items: &[String],
    profile: &str,
    target_dir: Option<String>,
    skip_local_target: bool,
) {
    if !skip_local_target && !items.contains(&"target".to_string()) {
        let _ = fs::create_dir_all(project_root.join("target"));
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
        let target_path = project_root.join("target");
        let target_items = check_dir(target_path.to_str()).unwrap_or_default();
        if !target_items.contains(&default_dir) {
            let _ = fs::create_dir_all(target_path.join(&default_dir));
        }
    }
}

fn run_compile(
    ctx: &BuildContext,
    _cancel: &Arc<AtomicBool>,
    _rep: &mut dyn BuildReporter,
) -> Result<f64, String> {
    let start_time = Instant::now();
    let result = build_project(ctx);
    common::finish_progress_line();
    match result {
        Ok(times) => {
            let times = if times == 0.0 {
                ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0
            } else {
                times
            };
            Ok(times)
        }
        Err(e) => Err(e),
    }
}

// Library entry point: run a build described by `req`, streaming progress to
// `rep`. The CLI wraps this with a CliReporter; TUI/GUI/MCP plug in their own.
pub fn run_build(
    root: &Path,
    req: &BuildRequest,
    rep: &mut dyn BuildReporter,
) -> Result<BuildOutcome, BuildError> {
    let secs = build_all(
        root,
        &req.profile,
        req.target.as_deref(),
        req.force,
        req.verbose,
        req.workspace.as_deref(),
        &req.cancel,
        rep,
    )
    .map_err(BuildError::from)?;
    Ok(BuildOutcome { secs })
}

#[allow(unused_variables, clippy::too_many_arguments)]
fn build_all(
    root: &Path,
    profile: &str,
    target: Option<&str>,
    force: bool,
    verbose: bool,
    workspace: Option<&str>,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
) -> Result<f64, String> {
    let config =
        Config::open(root.join("dcr.toml").to_str().unwrap()).map_err(|err| err.to_string())?;

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
                cancel,
                &mut *rep,
            )?;
        }
        if let Some(archive) = &config.typed().archive {
            #[cfg(feature = "archive")]
            {
                super::archive::pack_archive(root, archive, profile)?;
            }
            #[cfg(not(feature = "archive"))]
            {
                let _ = (archive, profile);
                return Err(
                    "disk image packing requires building dcr with --features archive".to_string(),
                );
            }
        }
        return Ok(0.0);
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
        if cancel.load(Ordering::SeqCst) {
            return Err("Build interrupted".to_string());
        }
        let target_label = build_target.as_ref().map_or("native", |t| t.as_str());
        if targets_to_build.len() > 1 {
            rep.on_event(BuildEvent::TargetStart {
                index: i + 1,
                total: targets_to_build.len(),
                target: target_label,
            });
        } else {
            rep.on_event(BuildEvent::ProjectStart {
                name: &project_name,
                profile,
                target: target_label,
            });
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
                        cancel,
                        &mut *rep,
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
                    cancel,
                    &mut *rep,
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
                    cancel,
                    &mut *rep,
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
                cancel,
                &mut *rep,
            )?;
        }
    }
    let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
    rep.on_event(BuildEvent::Finished { secs: elapsed });
    Ok(elapsed)
}

#[allow(clippy::too_many_arguments)]
fn build_workspace(
    workspace: &crate::core::workspace::Workspace,
    profile: &str,
    target: Option<&str>,
    force: bool,
    verbose: bool,
    workspace_root: Option<&Path>,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
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
            cancel,
            &mut *rep,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_project_at(
    project_root: &Path,
    profile: &str,
    target: Option<&str>,
    exclude_dirs: &[std::path::PathBuf],
    force: bool,
    verbose: bool,
    workspace_root: Option<&Path>,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        return Err("Build interrupted".to_string());
    }
    let items = check_dir(project_root.to_str())
        .map_err(|_| "Failed to read project directory".to_string())?;
    if !items.contains(&"dcr.toml".to_string()) {
        return Err("dcr.toml file not found".to_string());
    }
    let config_path = project_root.join("dcr.toml");
    let mut config = Config::open(&config_path.to_string_lossy()).map_err(|err| err.to_string())?;
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
    let build_language = get_language_with_profile(&config, profile)?;
    let build_qt = get_bool_with_profile(&config, "qt", profile, false);
    let lang_key = primary_language(&build_language);
    let project_compiler = get_lang_string(&config, &lang_key, "compiler", profile, build_target)
        .unwrap_or_else(|| {
            get_string_with_profile_and_target(&config, "compiler", profile, build_target)
        });
    let build_standard = get_lang_string(&config, &lang_key, "standard", profile, build_target)
        .unwrap_or_else(|| {
            get_string_with_profile_and_target(&config, "standard", profile, build_target)
        });
    let build_cxx_standard =
        get_lang_string(&config, &lang_key, "cxx_standard", profile, build_target).unwrap_or_else(
            || get_string_with_profile_and_target(&config, "cxx_standard", profile, build_target),
        );
    let build_kind = get_string_with_profile_and_target(&config, "kind", profile, build_target);
    let build_platform =
        get_string_with_profile_and_target(&config, "platform", profile, build_target);
    let mut build_type = get_string_with_profile_and_target(&config, "type", profile, build_target);
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
    let build_roots = get_list_with_profile_and_target(&config, "roots", profile, build_target)?;
    let src_disable = get_bool_with_profile(&config, "src_disable", profile, false);
    let freestanding = get_bool_with_profile(&config, "freestanding", profile, false);
    let lto = get_bool_with_profile(&config, "lto", profile, false);
    let strip = get_bool_with_profile(&config, "strip", profile, false);
    let opt_level = get_string_with_profile_and_target(&config, "opt_level", profile, build_target);
    let debug = get_bool_with_profile(&config, "debug", profile, profile != "release");
    let warnings = get_list_with_profile_and_target(&config, "warnings", profile, build_target)?;
    let panic_val = get_string_with_profile_and_target(&config, "panic", profile, build_target);
    let panic_abort = panic_val == "abort";
    let codegen_units: usize =
        get_string_with_profile_and_target(&config, "codegen-units", profile, build_target)
            .parse()
            .unwrap_or(0);

    if build_cflags.is_empty() && !freestanding && !is_bare_metal_target(build_target) {
        if !opt_level.is_empty() {
            build_cflags.push(format!("-O{opt_level}"));
        } else {
            build_cflags.push(match profile {
                "release" => "-O3".to_string(),
                _ => "-O0".to_string(),
            });
        }
        if debug {
            build_cflags.push("-g".to_string());
        }
        if warnings.is_empty() {
            if profile == "debug" {
                build_cflags.push("-Wall".to_string());
                build_cflags.push("-Wextra".to_string());
            }
        } else {
            for w in &warnings {
                build_cflags.push(format!("-W{w}"));
            }
        }
        match profile {
            "debug" => {
                build_cflags.push("-fno-omit-frame-pointer".to_string());
                build_cflags.push("-DDCR_DEBUG".to_string());
            }
            "release" => {
                build_cflags.push("-DNDEBUG".to_string());
            }
            _ => {}
        }
    }

    for flag in get_lang_list(&config, &lang_key, "flags", profile, build_target)? {
        build_cflags.push(flag);
    }

    if lto {
        build_cflags.push("-flto".to_string());
        build_ldflags.push("-flto".to_string());
    }
    if strip {
        build_ldflags.push("-s".to_string());
    }

    let build_expects = get_list_with_profile_and_target(&config, "expect", profile, build_target)?;
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
    let resolved_linker = resolve_tool("DCR_LD", tc_ld.as_deref());
    prepend_clang_target_flag(&mut build_cflags, build_target, &resolved_compiler);
    prepend_clang_target_flag(
        &mut build_ldflags,
        build_target,
        resolved_linker.as_deref().unwrap_or(&resolved_compiler),
    );
    let resolved_archiver = resolve_tool("DCR_AR", tc_ar.as_deref());

    let out_dir_config = get_build_string_with_profile(&config, "out_dir", profile);
    let target_dir_binding = if !out_dir_config.is_empty() {
        let p = Path::new(&out_dir_config);
        if p.is_absolute() {
            out_dir_config
        } else {
            project_root.join(p).to_string_lossy().to_string()
        }
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
                        Path::new("target").join(&default_dir)
                    }
                };
                root.join(&rel).to_string_lossy().to_string()
            }
            None => {
                let base_dir =
                    if let Some(rel) = normalize_target(build_target.unwrap_or(""), profile) {
                        PathBuf::from(rel)
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
                        Path::new("target").join(&default_dir)
                    };
                project_root.join(base_dir).to_string_lossy().to_string()
            }
        }
    };
    let target_dir = if target_dir_binding.is_empty() {
        None
    } else {
        Some(target_dir_binding.clone())
    };
    ensure_target_dirs(
        project_root,
        &items,
        profile,
        target_dir,
        workspace_root.is_some(),
    );

    let mut ws_include_dirs = Vec::new();
    let mut ws_lib_dirs = Vec::new();
    let mut ws_libs = Vec::new();

    if let Some(wroot) = workspace_root {
        let wroot_toml = wroot.join("dcr.toml");
        if wroot_toml.is_file()
            && let Ok(wroot_config) = Config::open(&wroot_toml.to_string_lossy())
            && let Ok(Some(ws)) =
                crate::core::workspace::parse_workspace(&wroot_config, profile, build_target, wroot)
        {
            let current_abs = project_root
                .canonicalize()
                .unwrap_or_else(|_| project_root.to_path_buf());
            if let Some(current_member) = ws
                .members
                .iter()
                .find(|m| m.path.canonicalize().unwrap_or_else(|_| m.path.clone()) == current_abs)
            {
                for dep_name in &current_member.deps {
                    if let Some(dep_member) = ws.members.iter().find(|m| &m.name == dep_name) {
                        let dep_inc = dep_member.path.join("include");
                        let dep_src = dep_member.path.join("src");
                        let dep_target_inc = dep_member.path.join("target").join("include");
                        let ws_target_inc = wroot.join("target").join("include");
                        if dep_inc.is_dir() {
                            ws_include_dirs.push(dep_inc.to_string_lossy().to_string());
                        }
                        if dep_src.is_dir() {
                            ws_include_dirs.push(dep_src.to_string_lossy().to_string());
                        }
                        if dep_target_inc.is_dir() {
                            ws_include_dirs.push(dep_target_inc.to_string_lossy().to_string());
                        }
                        if ws_target_inc.is_dir() {
                            ws_include_dirs.push(ws_target_inc.to_string_lossy().to_string());
                        }
                        let dep_target_lib = dep_member.path.join("target").join("lib");
                        if dep_target_lib.is_dir() {
                            ws_lib_dirs.push(dep_target_lib.to_string_lossy().to_string());
                        }
                        let target_str = build_target
                            .map(normalize_target_os)
                            .filter(|t| !t.is_empty());
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
                        let rel = match target_str {
                            Some(ref t) => Path::new("target").join(t).join(profile),
                            None => Path::new("target").join(&default_dir),
                        };
                        let dep_build_lib1 = dep_member.path.join(&rel);
                        let dep_build_lib2 = wroot.join(&rel);
                        if dep_build_lib1.is_dir() {
                            ws_lib_dirs.push(dep_build_lib1.to_string_lossy().to_string());
                        }
                        if dep_build_lib2.is_dir() {
                            ws_lib_dirs.push(dep_build_lib2.to_string_lossy().to_string());
                        }
                        let dep_toml = dep_member.path.join("dcr.toml");
                        let is_lib = if dep_toml.is_file() {
                            if let Ok(dep_config) = Config::open(&dep_toml.to_string_lossy()) {
                                let kind =
                                    get_build_string_with_profile(&dep_config, "kind", profile);
                                kind == "staticlib" || kind == "sharedlib"
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        if is_lib {
                            ws_libs.push(dep_member.name.clone());
                        }
                    }
                }
            }
        }
    }

    let deps_table = config.get("dependencies").and_then(|v| v.as_table());
    let mut resolved = resolve_deps(&config, profile, build_target, project_root)?;
    resolved.include_dirs.extend(ws_include_dirs);
    resolved.lib_dirs.extend(ws_lib_dirs);
    resolved.libs.extend(ws_libs);

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
                    rep.on_event(BuildEvent::DepBuilding { name, version });
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
                        cancel,
                        &mut *rep,
                    )?;
                    rep.on_event(BuildEvent::DepReady {
                        name,
                        version,
                        rebuilt: true,
                    });
                } else {
                    rep.on_event(BuildEvent::DepReady {
                        name,
                        version,
                        rebuilt: false,
                    });
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
    let resolved_cflags: Vec<String> = resolved_cflags
        .iter()
        .map(|f| substitute_vars(f, &version_info, profile, &project_name))
        .map(|f| absolutize_flag_path(&f, project_root, &["-isystem", "-idirafter", "-I", "-i"]))
        .collect();
    let modified_ldflags: Vec<String> = resolved_ldflags
        .iter()
        .map(|f| substitute_vars(f, &version_info, profile, &project_name))
        .map(|f| absolutize_flag_path(&f, project_root, &["-L", "-T"]))
        .collect();
    let merged_include_dirs: Vec<String> = merged_include_dirs
        .iter()
        .map(|f| substitute_vars(f, &version_info, profile, &project_name))
        .collect();
    let tool_execs = resolve_toolchain_execs(&tc_uic, &tc_moc, &tc_rcc, &pkg_configs);
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
        moc: Some(&tool_execs.moc),
        uic: Some(&tool_execs.uic),
        rcc: Some(&tool_execs.rcc),
        package_type: if build_type.is_empty() {
            None
        } else {
            Some(build_type.as_str())
        },
        freestanding,
        panic_abort,
        codegen_units,
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
        qt: build_qt,
    };
    let step_flags = build_step_flags(&resolved_cflags, &resolved.include_dirs, &resolved_compiler);
    let step_vars = StepVars {
        profile,
        version: &version_info.full,
        version_major: &version_info.major,
        version_minor: &version_info.minor,
        version_patch: &version_info.patch,
        version_suffix: &version_info.suffix,
        version_suffix_dash: &version_info.suffix_dash,
    };
    let mut steps_dirty = build_steps_need_run(&build_steps, &step_vars)?;
    if force {
        steps_dirty = true;
    }
    if steps_dirty {
        clean_generated_files(&build_generated)?;
        run_build_steps(
            &build_steps,
            &tool_execs,
            &step_flags,
            &step_vars,
            cancel,
            rep,
        )?;
    }
    let sources = collect_sources(&ctx)?;
    let headers = collect_header_files(&ctx, project_root)?;
    let lib_files = collect_lib_files(&ctx);
    let fingerprint = compute_build_fingerprint(&ctx, &sources, &headers, &lib_files)?;
    let mut skip = should_skip_build(&ctx, &fingerprint);
    let debug_enabled = std::env::var("DCR_DEBUG").is_ok();
    if steps_dirty {
        skip = false;
    }
    if force {
        skip = false;
    }
    if skip && !debug_enabled {
        return Ok(());
    }
    if !skip {
        rep.on_event(BuildEvent::Compiling {
            name: &project_name,
            version: &project_version,
        });
        run_compile(&ctx, cancel, rep)?;
        if cancel.load(Ordering::SeqCst) {
            return Err("Build interrupted".to_string());
        }
        write_build_fingerprint(&ctx, &fingerprint)?;
    }
    if ctx.package_type == Some("lib") || ctx.kind == "staticlib" || ctx.kind == "sharedlib" {
        package_library(&ctx, &headers, project_root)?;
    }
    let mut post_steps_dirty = build_steps_need_run(&build_post_steps, &step_vars)?;
    if force {
        post_steps_dirty = true;
    }
    if post_steps_dirty {
        run_build_steps(
            &build_post_steps,
            &tool_execs,
            &step_flags,
            &step_vars,
            cancel,
            rep,
        )?;
    }
    verify_expectations(&build_expects, &step_vars)?;
    if let Some(archive) = &config.typed().archive {
        let out = archive.output.replace("{profile}", profile);
        rep.on_event(BuildEvent::Packing { path: &out });
        #[cfg(feature = "archive")]
        {
            super::archive::pack_archive(project_root, archive, profile)?;
        }
        #[cfg(not(feature = "archive"))]
        {
            let _ = (project_root, archive, profile, out);
            return Err(
                "disk image packing requires building dcr with --features archive".to_string(),
            );
        }
    }
    Ok(())
}

fn absolutize_flag_path(flag: &str, project_root: &Path, prefixes: &[&str]) -> String {
    for prefix in prefixes {
        if let Some(rest) = flag.strip_prefix(prefix) {
            if rest.is_empty() {
                return flag.to_string();
            }
            if rest.starts_with('/') || rest.starts_with('@') {
                return flag.to_string();
            }
            if rest.len() >= 2 && rest.as_bytes()[1] == b':' {
                return flag.to_string();
            }
            let abs = project_root.join(rest);
            let path = abs.to_string_lossy().replace('\\', "/");
            return format!("{prefix}{path}");
        }
    }
    flag.to_string()
}

fn find_target_root(dir: &str, fallback: &Path) -> PathBuf {
    let p = Path::new(dir);
    for ancestor in p.ancestors() {
        if ancestor.file_name().and_then(|n| n.to_str()) == Some("target") {
            return ancestor.to_path_buf();
        }
    }
    fallback.to_path_buf()
}

fn package_library(
    ctx: &BuildContext,
    headers: &[PathBuf],
    project_root: &Path,
) -> Result<(), String> {
    let target_root = if let Some(dir) = ctx.target_dir {
        find_target_root(dir, &project_root.join("target"))
    } else {
        project_root.join("target")
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

pub(crate) struct ToolchainExecs {
    pub(crate) uic: String,
    pub(crate) moc: String,
    pub(crate) rcc: String,
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
    if Path::new(cmd).is_file() {
        return true;
    }
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}

#[cfg(test)]
mod tests {
    use super::absolutize_flag_path;
    use std::path::Path;

    #[test]
    fn absolutize_prefers_long_include_prefixes() {
        let root = Path::new("/pkg");
        let prefs = ["-isystem", "-idirafter", "-I", "-i"];
        assert_eq!(
            absolutize_flag_path("-isystemvendor/inc", root, &prefs),
            "-isystem/pkg/vendor/inc"
        );
        assert_eq!(
            absolutize_flag_path("-idiraftervendor/old", root, &prefs),
            "-idirafter/pkg/vendor/old"
        );
        assert_eq!(
            absolutize_flag_path("-I../inc", root, &prefs),
            "-I/pkg/../inc"
        );
        assert_eq!(
            absolutize_flag_path("-i../inc", root, &prefs),
            "-i/pkg/../inc"
        );
        assert_eq!(
            absolutize_flag_path("-I/usr/include", root, &prefs),
            "-I/usr/include"
        );
    }
}
