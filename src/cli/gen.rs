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

use crate::core::build::builder::BuildContext;
use crate::core::build::builder::artifact::{absolute_artifact_path, resolve_artifact_path};
use crate::core::build::builder::collect_sources;
use crate::core::build::common;
use crate::core::build_config::Config;
use crate::core::workspace::parse_workspace;
use crate::utils::build::{
    get_bool_with_profile, get_config_opt, get_config_str, get_language_with_profile_or_default,
    get_list_with_profile, get_string_with_profile, normalize_kind, normalize_platform,
    resolve_artifact_target_dir, resolve_compiler, resolve_pkg_config_flags_lossy,
};
use crate::utils::fs::{
    absolute_join, atomic_write, canonicalize_path, ensure_dcr_dir, find_project_root,
};
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, printc};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Path-dep include/lib/libs metadata for `dcr gen` (project may not be built yet).
///
/// Explicit include/lib lists may point at missing paths; default `include`/`lib`
/// candidates are added only when those directories exist.
struct GenDeps {
    include_dirs: Vec<String>,
    lib_dirs: Vec<String>,
    libs: Vec<String>,
}

/// Collects path-dependency include/lib/libs from `dcr.toml` for code generation.
///
/// Skips `system` deps and non-path entries. Falls back to common include/lib
/// directory names only when those directories exist.
fn resolve_deps_for_gen(config: &Config, profile: &str, project_root: &Path) -> GenDeps {
    let deps_val = match config.get("dependencies") {
        Some(v) => v,
        None => {
            return GenDeps {
                include_dirs: vec![],
                lib_dirs: vec![],
                libs: vec![],
            };
        }
    };
    let deps_table = match deps_val.as_table() {
        Some(t) => t,
        None => {
            return GenDeps {
                include_dirs: vec![],
                lib_dirs: vec![],
                libs: vec![],
            };
        }
    };

    let mut include_dirs = Vec::new();
    let mut lib_dirs = Vec::new();
    let mut libs = Vec::new();

    for (name, value) in deps_table {
        let tbl = match value.as_table() {
            Some(t) => t,
            None => continue,
        };
        // skip system deps
        if tbl.get("system").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let path_raw = match tbl.get("path").and_then(|v| v.as_str()) {
            Some(p) => p.replace("{profile}", profile),
            None => continue,
        };
        let dep_path = {
            let p = Path::new(&path_raw);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                project_root.join(p)
            }
        };

        // include dirs — use explicit list or fall back to <dep>/include if it exists
        let include_raws: Option<Vec<String>> =
            tbl.get("include").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.replace("{profile}", profile))
                    .collect()
            });

        if let Some(raws) = include_raws {
            for r in raws {
                let p = Path::new(&r);
                let full = if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    dep_path.join(p)
                };
                include_dirs.push(full.to_string_lossy().to_string());
            }
        } else {
            let candidate = dep_path.join("include");
            if candidate.exists() {
                include_dirs.push(candidate.to_string_lossy().to_string());
            }
        }

        // lib dirs — use explicit list or fall back to <dep>/lib (best-effort, may not exist yet)
        let lib_raws: Option<Vec<String>> = tbl.get("lib").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.replace("{profile}", profile))
                .collect()
        });

        if let Some(raws) = lib_raws {
            for r in raws {
                let p = Path::new(&r);
                let full = if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    dep_path.join(p)
                };
                // Include even if it doesn't exist yet — for IntelliSense purposes
                lib_dirs.push(full.to_string_lossy().to_string());
            }
        } else {
            for default in &["lib", "lib64"] {
                let candidate = dep_path.join(default);
                if candidate.exists() {
                    lib_dirs.push(candidate.to_string_lossy().to_string());
                    break;
                }
            }
        }

        // libs
        let libs_raws: Option<Vec<String>> =
            tbl.get("libs").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });
        match libs_raws {
            Some(ls) if !ls.is_empty() => libs.extend(ls),
            _ => libs.push(name.clone()),
        }
    }

    GenDeps {
        include_dirs,
        lib_dirs,
        libs,
    }
}

// ── public API ───────────────────────────────────────────────────────────────

/// Per-project (or workspace member) metadata collected for `dcr gen`.
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
    pub profile: String,
    pub language: String,
    pub standard: String,
    pub cxx_standard: String,
    pub compiler: String,
    pub kind: String,
    pub target: String,
    pub target_dir: Option<String>,
    pub artifact_path: Option<String>,
    pub artifact_kind: String,
    pub sources: Vec<String>,
    pub include_dirs: Vec<String>,
    pub lib_dirs: Vec<String>,
    pub libs: Vec<String>,
    pub cflags: Vec<String>,
    pub ldflags: Vec<String>,
    pub source_roots: Vec<String>,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub output_filename: Option<String>,
    pub output_extension: Option<String>,
    pub out_dir: String,
    pub debugger: String,
    pub moc: Option<String>,
    pub uic: Option<String>,
    pub rcc: Option<String>,
    pub workspace_root: Option<PathBuf>,
}

/// Entry point for `dcr gen`: dispatches to project-info / compile-commands / vscode / clion.
///
/// # Parameters
/// - `args`: First token is the subcommand; the rest are subcommand flags.
///
/// # Returns
/// Process exit code from the chosen subcommand (`0` for help).
pub fn r#gen(args: &[String]) -> i32 {
    let subcommand = match args.first() {
        Some(s) if s == "--help" => {
            printc("USAGE:", BOLD_GREEN);
            printc("    dcr gen <subcommand>", BOLD_CYAN);
            println!();
            printc("DESCRIPTION:", BOLD_GREEN);
            println!("    Generates IDE and tooling integration files.");
            println!();
            printc("SUBCOMMANDS:", BOLD_GREEN);
            println!("    project-info      Print project metadata as JSON");
            println!("    compile-commands  Generate .dcr/compile_commands.json");
            println!("    vscode            Generate .vscode/ integration files");
            println!("    clion             Generate .idea/ integration files");
            println!();
            printc("OPTIONS:", BOLD_GREEN);
            println!("    --debug | --release   Profile (default: debug)");
            println!("    --quiet | -q          Suppress success messages on stdout");
            return 0;
        }
        Some(s) => s.as_str(),
        None => {
            printc("USAGE:", BOLD_GREEN);
            printc("    dcr gen <subcommand>", BOLD_CYAN);
            println!();
            printc("SUBCOMMANDS:", BOLD_GREEN);
            println!("    project-info      Print project metadata as JSON");
            println!("    compile-commands  Generate .dcr/compile_commands.json");
            println!("    vscode            Generate .vscode/ integration files");
            println!("    clion             Generate .idea/ integration files");
            return 1;
        }
    };

    let rest = &args[1..];

    match subcommand {
        "project-info" => gen_project_info(rest),
        "compile-commands" => gen_compile_commands(rest),
        "vscode" => gen_vscode(rest),
        "clion" => gen_clion(rest),
        _ => {
            error(&format!("Unknown gen subcommand: {subcommand}"));
            1
        }
    }
}

// ── shared: collect per-project data ─────────────────────────────────────────

/// Collects [`ProjectInfo`] with cwd set to the project root so relative paths resolve.
fn collect_project_info(
    root: &Path,
    profile: &str,
    workspace_root: Option<&Path>,
) -> Result<ProjectInfo, String> {
    let prev = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(root).map_err(|e| e.to_string())?;
    let result = collect_project_info_inner(root, profile, workspace_root);
    let _ = std::env::set_current_dir(prev);
    result
}

/// Reads `dcr.toml` and resolves compiler, deps, sources, and artifact paths for one project.
///
/// Called once per project/member from [`collect_all`] (not recursive).
fn collect_project_info_inner(
    root: &Path,
    profile: &str,
    workspace_root: Option<&Path>,
) -> Result<ProjectInfo, String> {
    let root = canonicalize_path(root);
    let config = Config::open("./dcr.toml").map_err(|e| e.to_string())?;

    let name = get_config_str(&config, "package.name");
    let version = get_config_str(&config, "package.version");
    let language = get_language_with_profile_or_default(&config, profile);
    let standard = get_string_with_profile(&config, "standard", profile);
    let cxx_standard = get_string_with_profile(&config, "cxx_standard", profile);
    let compiler_s = get_string_with_profile(&config, "compiler", profile);
    let kind_raw = get_string_with_profile(&config, "kind", profile);
    let kind = normalize_kind(&kind_raw).to_string();
    let build_target = get_string_with_profile(&config, "target", profile);
    let platform = get_string_with_profile(&config, "platform", profile);
    let output_filename_s = get_string_with_profile(&config, "filename", profile);
    let output_extension_s = get_string_with_profile(&config, "extension", profile);
    let output_filename = if output_filename_s.is_empty() {
        None
    } else {
        Some(output_filename_s)
    };
    let output_extension = if output_extension_s.is_empty() {
        None
    } else {
        Some(output_extension_s)
    };

    let tc_cc = get_config_opt(&config, "toolchain.cc");
    let tc_cxx = get_config_opt(&config, "toolchain.cxx");
    let tc_as = get_config_opt(&config, "toolchain.as");
    let tc_ar = get_config_opt(&config, "toolchain.ar");
    let tc_ld = get_config_opt(&config, "toolchain.ld");
    let tc_moc = get_config_opt(&config, "toolchain.moc");
    let tc_uic = get_config_opt(&config, "toolchain.uic");
    let tc_rcc = get_config_opt(&config, "toolchain.rcc");
    let tc_debugger = get_config_opt(&config, "toolchain.debugger");

    let base_cflags = get_list_with_profile(&config, "cflags", profile);
    let base_ldflags = get_list_with_profile(&config, "ldflags", profile);
    let build_excludes = get_list_with_profile(&config, "exclude", profile);
    let build_includes = get_list_with_profile(&config, "include", profile);
    let build_roots = get_list_with_profile(&config, "roots", profile);
    let src_disable = get_bool_with_profile(&config, "src_disable", profile, false);
    let pkg_configs = get_list_with_profile(&config, "pkg_config", profile);

    let resolved_compiler = resolve_compiler(
        &language,
        &compiler_s,
        tc_cc.as_deref(),
        tc_cxx.as_deref(),
        tc_as.as_deref(),
    );

    let resolved_linker = tc_ld.or_else(|| {
        std::env::var("DCR_LD")
            .ok()
            .filter(|v| !v.trim().is_empty())
    });
    let resolved_archiver = tc_ar.or_else(|| {
        std::env::var("DCR_AR")
            .ok()
            .filter(|v| !v.trim().is_empty())
    });

    let resolved = resolve_deps_for_gen(&config, profile, &root);
    let (resolved_cflags, resolved_ldflags) =
        resolve_pkg_config_flags_lossy(&pkg_configs, &base_cflags, &base_ldflags);

    // Build exclude/include pattern lists (same logic as cli::build)
    let mut combined_excludes: Vec<PathBuf> = Vec::new();
    let mut exclude_patterns: Vec<String> = Vec::new();
    for raw in &build_excludes {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        let norm = t.replace('\\', "/");
        let p = Path::new(t);
        if p.is_absolute() {
            combined_excludes.push(p.to_path_buf());
        } else {
            combined_excludes.push(root.join(p));
        }
        exclude_patterns.push(norm);
    }

    let mut combined_includes: Vec<String> = Vec::new();
    combined_includes.extend(exclude_patterns.iter().map(|v| format!("!{v}")));
    combined_includes.extend(build_includes.iter().map(|v| v.replace('\\', "/")));

    // Source roots
    let mut source_roots: Vec<PathBuf> = Vec::new();
    for raw in &build_roots {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        let p = Path::new(t);
        source_roots.push(if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        });
    }
    if !src_disable && source_roots.is_empty() {
        source_roots.push(root.join("src"));
    }

    // Merge include dirs (dep include dirs + any include globs that are directories)
    let mut merged_include_dirs = resolved.include_dirs.clone();
    for raw in &build_includes {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        let norm = t.replace('\\', "/");
        if common::has_glob_magic(&norm) {
            continue;
        }
        let p = Path::new(t);
        let dir = if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        };
        if dir.is_dir() {
            merged_include_dirs.push(dir.to_string_lossy().to_string());
        }
    }

    let out_dir_config = get_string_with_profile(&config, "out_dir", profile);
    let has_explicit = !build_target.trim().is_empty();
    let target_dir_binding = resolve_artifact_target_dir(
        &root,
        workspace_root,
        profile,
        &build_target,
        &out_dir_config,
        has_explicit,
    );
    let target_dir_opt = if target_dir_binding.is_empty() {
        None
    } else {
        Some(target_dir_binding.as_str())
    };

    let ctx = BuildContext {
        profile,
        project_name: &name,
        compiler: &resolved_compiler,
        language: &language,
        standard: &standard,
        cxx_standard: &cxx_standard,
        target: if build_target.is_empty() {
            None
        } else {
            Some(build_target.as_str())
        },
        target_dir: target_dir_opt,
        kind: kind.as_str(),
        platform: normalize_platform(&platform),
        linker: resolved_linker.as_deref(),
        archiver: resolved_archiver.as_deref(),
        moc: tc_moc.as_deref(),
        uic: tc_uic.as_deref(),
        rcc: tc_rcc.as_deref(),
        package_type: None,
        freestanding: false,
        panic_abort: false,
        codegen_units: 0,
        source_roots: &source_roots,
        exclude_dirs: &combined_excludes,
        include_paths: &combined_includes,
        include_dirs: &merged_include_dirs,
        lib_dirs: &resolved.lib_dirs,
        libs: &resolved.libs,
        cflags: &resolved_cflags,
        ldflags: &resolved_ldflags,
        output_filename: output_filename.as_deref(),
        output_extension: output_extension.as_deref(),
        verbose: false,
        qt: false,
    };

    let sources = collect_sources(&ctx).map_err(|e| format!("Failed to collect sources: {e}"))?;

    // Convert relative source paths to absolute
    let abs_sources: Vec<String> = sources
        .iter()
        .map(|s| {
            absolute_join(&root, Path::new(s))
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let abs_include_dirs: Vec<String> = merged_include_dirs
        .iter()
        .map(|d| {
            absolute_join(&root, Path::new(d))
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let abs_lib_dirs: Vec<String> = resolved
        .lib_dirs
        .iter()
        .map(|d| {
            absolute_join(&root, Path::new(d))
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let abs_source_roots: Vec<String> = source_roots
        .iter()
        .map(|p| canonicalize_path(p).to_string_lossy().to_string())
        .collect();

    let abs_target_dir = {
        let p = Path::new(&target_dir_binding);
        let path = if p.is_absolute() {
            canonicalize_path(p)
        } else {
            absolute_join(&root, p)
        };
        path.to_string_lossy().into_owned()
    };

    let relative_artifact = resolve_artifact_path(
        &kind,
        profile,
        &name,
        Some(&abs_target_dir),
        output_filename.as_deref(),
        output_extension.as_deref(),
    );

    let artifact_path = relative_artifact.map(|rel| {
        absolute_artifact_path(&root, &rel)
            .to_string_lossy()
            .to_string()
    });

    let debugger = resolve_debugger(tc_debugger.as_deref());
    let ws_root = workspace_root.map(canonicalize_path);

    Ok(ProjectInfo {
        name,
        version,
        root,
        profile: profile.to_string(),
        language,
        standard,
        cxx_standard,
        compiler: resolved_compiler,
        kind: kind.clone(),
        target: build_target,
        target_dir: Some(abs_target_dir),
        artifact_path,
        artifact_kind: kind,
        sources: abs_sources,
        include_dirs: abs_include_dirs,
        lib_dirs: abs_lib_dirs,
        libs: resolved.libs,
        cflags: resolved_cflags,
        ldflags: resolved_ldflags,
        source_roots: abs_source_roots,
        include_globs: build_includes,
        exclude_globs: build_excludes,
        output_filename,
        output_extension,
        out_dir: out_dir_config,
        debugger,
        moc: tc_moc.and_then(|v| resolve_tool_path(&v)),
        uic: tc_uic.and_then(|v| resolve_tool_path(&v)),
        rcc: tc_rcc.and_then(|v| resolve_tool_path(&v)),
        workspace_root: ws_root,
    })
}

fn find_in_path(name: &str) -> Option<String> {
    if name.contains('/') || name.contains('\\') {
        let p = Path::new(name);
        if p.exists() {
            return Some(canonicalize_path(p).to_string_lossy().to_string());
        }
        return Some(name.to_string());
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(canonicalize_path(&candidate).to_string_lossy().to_string());
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{name}.exe"));
            if with_exe.is_file() {
                return Some(canonicalize_path(&with_exe).to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Resolves a tool name (e.g. moc, uic, rcc) for generated metadata.
///
/// Prefers a PATH hit via [`find_in_path`]; otherwise returns the trimmed name
/// as given (absolute or relative). Empty input yields `None`.
fn resolve_tool_path(name: &str) -> Option<String> {
    let t = name.trim();
    if t.is_empty() {
        return None;
    }
    find_in_path(t).or_else(|| Some(t.to_string()))
}

/// Picks a debugger name: configured value if set, else first of lldb/gdb on PATH,
/// else `cppvsdbg` on Windows or `lldb` elsewhere (even if not on PATH).
fn resolve_debugger(configured: Option<&str>) -> String {
    if let Some(d) = configured {
        let t = d.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    for candidate in ["lldb", "gdb"] {
        if find_in_path(candidate).is_some() {
            return candidate.to_string();
        }
    }
    if cfg!(windows) {
        "cppvsdbg".to_string()
    } else {
        "lldb".to_string()
    }
}

fn utc_now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let tod = secs % 86400;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Converts days since the Unix epoch to `(year, month, day)` (proleptic Gregorian).
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

/// Writes `.dcr/build-info.json` and `.dcr/toolchain.json` from project metadata.
///
/// # Parameters
/// - `root`: Project root (creates `.dcr/` if needed).
/// - `info`: Collected project metadata (compiler, artifact, toolchain tools).
///
/// # Returns
/// `Ok(())` on success, or an error string if directory/file writes fail.
pub fn write_dcr_metadata(root: &Path, info: &ProjectInfo) -> Result<(), String> {
    let dcr = ensure_dcr_dir(root).map_err(|e| format!("Failed to create .dcr/: {e}"))?;

    let artifact_json = match &info.artifact_path {
        Some(p) => json_str(p),
        None => "null".to_string(),
    };
    let target_dir_json = match &info.target_dir {
        Some(p) => json_str(p),
        None => "null".to_string(),
    };

    let build_info = format!(
        r#"{{
  "schemaVersion": 1,
  "generatedAt": {generated},
  "profile": {profile},
  "compiler": {compiler},
  "target": {target},
  "target_dir": {target_dir},
  "artifact_path": {artifact},
  "artifact_kind": {kind}
}}
"#,
        generated = json_str(&utc_now_iso()),
        profile = json_str(&info.profile),
        compiler = json_str(&info.compiler),
        target = json_str(&info.target),
        target_dir = target_dir_json,
        artifact = artifact_json,
        kind = json_str(&info.artifact_kind),
    );
    atomic_write(&dcr.join("build-info.json"), build_info.as_bytes())
        .map_err(|e| format!("Failed to write build-info.json: {e}"))?;

    let compiler_path = find_in_path(&info.compiler).unwrap_or_else(|| info.compiler.clone());
    let debugger_path = find_in_path(&info.debugger).unwrap_or_else(|| info.debugger.clone());

    let mut toolchain = String::from("{\n");
    toolchain.push_str("  \"schemaVersion\": 1,\n");
    toolchain.push_str(&format!("  \"compiler\": {},\n", json_str(&info.compiler)));
    toolchain.push_str(&format!(
        "  \"compilerPath\": {},\n",
        json_str(&compiler_path)
    ));
    toolchain.push_str(&format!("  \"debugger\": {},\n", json_str(&info.debugger)));
    toolchain.push_str(&format!("  \"debuggerPath\": {}", json_str(&debugger_path)));
    if let Some(ref m) = info.moc {
        toolchain.push_str(",\n");
        toolchain.push_str(&format!("  \"moc\": {}", json_str(m)));
    }
    if let Some(ref u) = info.uic {
        toolchain.push_str(",\n");
        toolchain.push_str(&format!("  \"uic\": {}", json_str(u)));
    }
    if let Some(ref r) = info.rcc {
        toolchain.push_str(",\n");
        toolchain.push_str(&format!("  \"rcc\": {}", json_str(r)));
    }
    toolchain.push_str("\n}\n");
    atomic_write(&dcr.join("toolchain.json"), toolchain.as_bytes())
        .map_err(|e| format!("Failed to write toolchain.json: {e}"))?;

    Ok(())
}

fn ensure_clangd(root: &Path, quiet: bool) {
    let path = root.join(".clangd");
    let marker = "# managed by dcr gen";
    let desired = format!("{marker}\nCompileFlags:\n  CompilationDatabase: .dcr\n");
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content)
                if content.contains(marker) || content.contains("CompilationDatabase: .dcr") =>
            {
                return;
            }
            Ok(_) => {
                if !quiet {
                    eprintln!(
                        "Warning: .clangd already exists and is not managed by dcr; \
                         set CompilationDatabase to .dcr manually if needed"
                    );
                }
                return;
            }
            Err(_) => return,
        }
    }
    let _ = std::fs::write(&path, desired);
}

/// Collects [`ProjectInfo`] for the root package and workspace members (skips failed members).
fn collect_all(root: &Path, profile: &str) -> Result<Vec<ProjectInfo>, String> {
    let root = canonicalize_path(root);
    let config = {
        let prev = std::env::current_dir().map_err(|e| e.to_string())?;
        std::env::set_current_dir(&root).map_err(|e| e.to_string())?;
        let cfg = Config::open("./dcr.toml").map_err(|e| e.to_string());
        let _ = std::env::set_current_dir(prev);
        cfg?
    };

    let mut all = Vec::new();
    let is_workspace = parse_workspace(&config, profile, None, &root)
        .ok()
        .flatten()
        .is_some();
    let ws_root = if is_workspace {
        Some(root.as_path())
    } else {
        None
    };

    if let Ok(Some(ws)) = parse_workspace(&config, profile, None, &root) {
        for member in &ws.members {
            let member_path = canonicalize_path(&member.path);
            match collect_project_info(&member_path, profile, ws_root) {
                Ok(info) => all.push(info),
                Err(e) => eprintln!(
                    "Warning: skipping workspace member {}: {e}",
                    member_path.display()
                ),
            }
        }
    }

    if !is_workspace || !config.is_workspace_only() {
        match collect_project_info(&root, profile, ws_root) {
            Ok(info) => all.push(info),
            Err(e) if is_workspace => {
                eprintln!("Warning: skipping workspace root package: {e}");
            }
            Err(e) => return Err(e),
        }
    }

    if all.is_empty() {
        return Err("No project members found".to_string());
    }

    Ok(all)
}

// ── dcr gen project-info ─────────────────────────────────────────────────────

/// `dcr gen project-info`: print project metadata JSON; writes `.dcr` meta from the last entry.
fn gen_project_info(args: &[String]) -> i32 {
    let (root, profile, quiet) = match parse_gen_args(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    let all = match collect_all(&root, &profile) {
        Ok(v) => v,
        Err(e) => {
            error(&e);
            return 1;
        }
    };

    if let Some(info) = all.last()
        && let Err(e) = write_dcr_metadata(&root, info)
        && !quiet
    {
        eprintln!("Warning: {e}");
    }
    ensure_clangd(&root, quiet);

    print!("[");
    for (i, info) in all.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        println!();
        print!("{}", project_info_to_json(info));
    }
    println!();
    println!("]");
    0
}

/// Pretty multi-line JSON object for one [`ProjectInfo`] (used inside a JSON array).
fn project_info_to_json(info: &ProjectInfo) -> String {
    let mut out = String::new();
    out.push_str("  {\n");
    out.push_str(&format!("    \"name\": {},\n", json_str(&info.name)));
    out.push_str(&format!("    \"version\": {},\n", json_str(&info.version)));
    out.push_str(&format!(
        "    \"root\": {},\n",
        json_str(&info.root.to_string_lossy())
    ));
    out.push_str(&format!("    \"profile\": {},\n", json_str(&info.profile)));
    out.push_str(&format!(
        "    \"language\": {},\n",
        json_str(&info.language)
    ));
    out.push_str(&format!(
        "    \"standard\": {},\n",
        json_str(&info.standard)
    ));
    out.push_str(&format!(
        "    \"cxx_standard\": {},\n",
        json_str(&info.cxx_standard)
    ));
    out.push_str(&format!(
        "    \"compiler\": {},\n",
        json_str(&info.compiler)
    ));
    out.push_str(&format!("    \"kind\": {},\n", json_str(&info.kind)));
    out.push_str(&format!("    \"target\": {},\n", json_str(&info.target)));
    out.push_str(&format!(
        "    \"target_dir\": {},\n",
        match &info.target_dir {
            Some(p) => json_str(p),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "    \"artifact_path\": {},\n",
        match &info.artifact_path {
            Some(p) => json_str(p),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "    \"artifact_kind\": {},\n",
        json_str(&info.artifact_kind)
    ));
    out.push_str(&format!(
        "    \"sources\": {},\n",
        json_str_array(&info.sources)
    ));
    out.push_str(&format!(
        "    \"include_dirs\": {},\n",
        json_str_array(&info.include_dirs)
    ));
    out.push_str(&format!(
        "    \"lib_dirs\": {},\n",
        json_str_array(&info.lib_dirs)
    ));
    out.push_str(&format!("    \"libs\": {},\n", json_str_array(&info.libs)));
    out.push_str(&format!(
        "    \"cflags\": {},\n",
        json_str_array(&info.cflags)
    ));
    out.push_str(&format!(
        "    \"ldflags\": {},\n",
        json_str_array(&info.ldflags)
    ));
    out.push_str(&format!(
        "    \"source_roots\": {},\n",
        json_str_array(&info.source_roots)
    ));
    out.push_str(&format!(
        "    \"include_globs\": {},\n",
        json_str_array(&info.include_globs)
    ));
    out.push_str(&format!(
        "    \"exclude_globs\": {}\n",
        json_str_array(&info.exclude_globs)
    ));
    out.push_str("  }");
    out
}

// ── dcr gen compile-commands ─────────────────────────────────────────────────

/// `dcr gen compile-commands`: write `.dcr/compile_commands.json`.
fn gen_compile_commands(args: &[String]) -> i32 {
    let (root, profile, quiet) = match parse_gen_args(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    let all = match collect_all(&root, &profile) {
        Ok(v) => v,
        Err(e) => {
            error(&e);
            return 1;
        }
    };

    gen_compile_commands_inner(&root, &profile, &all, quiet)
}

/// Writes compile_commands.json, build metadata, and ensures `.clangd`.
///
/// Used by `compile-commands`, `vscode`, and `clion`.
fn gen_compile_commands_inner(root: &Path, profile: &str, all: &[ProjectInfo], quiet: bool) -> i32 {
    if let Err(e) = ensure_dcr_dir(root) {
        error(&format!("Failed to create .dcr/: {e}"));
        return 1;
    }

    let entries = build_compile_commands(all, profile);
    let out_path = root.join(".dcr").join("compile_commands.json");
    match atomic_write(&out_path, entries.as_bytes()) {
        Ok(_) => {
            if let Some(info) = all.last()
                && let Err(e) = write_dcr_metadata(root, info)
                && !quiet
            {
                eprintln!("Warning: {e}");
            }
            ensure_clangd(root, quiet);
            if !quiet {
                println!("Generated {}", out_path.display());
            }
            0
        }
        Err(e) => {
            error(&format!("Failed to write compile_commands.json: {e}"));
            1
        }
    }
}

/// Builds a clangd/cpptools `compile_commands.json` array (one entry per source file).
fn build_compile_commands(projects: &[ProjectInfo], profile: &str) -> String {
    let mut out = String::from("[\n");
    let mut first = true;

    for info in projects {
        for source in &info.sources {
            if !first {
                out.push_str(",\n");
            }
            first = false;

            let command = build_compile_command(info, source, profile);
            out.push_str("  {\n");
            out.push_str(&format!(
                "    \"directory\": {},\n",
                json_str(&info.root.to_string_lossy())
            ));
            out.push_str(&format!("    \"file\": {},\n", json_str(source)));
            out.push_str(&format!(
                "    \"arguments\": {}\n",
                json_str_array(&command)
            ));
            out.push_str("  }");
        }
    }

    out.push_str("\n]\n");
    out
}

/// Assembles one compiler argv: compiler, `-c`, optional `-x` asm, source/obj paths,
/// `-fPIC`, `-std=`, profile defaults, cflags, and include paths.
fn build_compile_command(info: &ProjectInfo, source: &str, profile: &str) -> Vec<String> {
    let mut cmd: Vec<String> = Vec::new();
    let compiler = if info.compiler.is_empty() {
        "cc"
    } else {
        &info.compiler
    };
    cmd.push(compiler.to_string());
    cmd.push("-c".to_string());

    // ASM x flag — must be before source file
    if let Some(flag) = common::asm_lang_flag(source) {
        cmd.push("-x".to_string());
        cmd.push(flag.to_string());
    }

    let abs_source = absolute_join(&info.root, Path::new(source));
    cmd.push(abs_source.to_string_lossy().to_string());

    // Object path (for -o, approximate — not critical for IntelliSense)

    let obj_base = info
        .target_dir
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| info.root.join("target").join(profile));
    let obj_dir = obj_base.join("obj");
    let obj_path = {
        let p = Path::new(source);
        let rel = strip_src_prefix(p);
        absolute_join(&info.root, &obj_dir.join(rel).with_extension("o"))
            .to_string_lossy()
            .to_string()
    };
    cmd.push("-o".to_string());
    cmd.push(obj_path);

    if info.kind == "sharedlib" {
        cmd.push("-fPIC".to_string());
    }

    // -std=
    if !info.standard.is_empty() && info.language.to_lowercase() != "asm" {
        let ext = Path::new(source)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let is_cpp = matches!(ext, "cpp" | "cxx" | "cc");
        if is_cpp && !info.cxx_standard.is_empty() {
            cmd.push(format!("-std={}", info.cxx_standard));
        } else if !is_cpp {
            cmd.push(format!("-std={}", info.standard));
        }
    }

    // Default profile flags (same idea as default_profile_flags)
    match profile {
        "release" => {
            cmd.push("-O3".to_string());
            cmd.push("-DNDEBUG".to_string());
        }
        "debug" => {
            cmd.push("-O0".to_string());
            cmd.push("-g".to_string());
            cmd.push("-Wall".to_string());
            cmd.push("-Wextra".to_string());
            cmd.push("-fno-omit-frame-pointer".to_string());
            cmd.push("-DDCR_DEBUG".to_string());
        }
        _ => {}
    }

    for flag in &info.cflags {
        // Expand relative -I paths to absolute so clangd/cpptools work
        // regardless of their working directory.
        if let Some(rel) = flag.strip_prefix("-I") {
            let abs = absolute_join(&info.root, Path::new(rel));
            cmd.push(format!("-I{}", abs.to_string_lossy()));
        } else {
            cmd.push(flag.clone());
        }
    }
    for dir in &info.include_dirs {
        let abs = absolute_join(&info.root, Path::new(dir));
        cmd.push(format!("-I{}", abs.to_string_lossy()));
    }

    cmd
}

fn strip_src_prefix(p: &Path) -> PathBuf {
    // Try to strip leading ./src or src
    let s = p.to_string_lossy();
    let trimmed = s.trim_start_matches("./");
    let without_src = trimmed
        .strip_prefix("src/")
        .unwrap_or(trimmed)
        .strip_prefix("src\\")
        .unwrap_or(trimmed);
    PathBuf::from(without_src)
}

// ── dcr gen vscode ───────────────────────────────────────────────────────────

/// `dcr gen vscode`: compile_commands plus `.vscode/` tasks, launch, settings, extensions.
fn gen_vscode(args: &[String]) -> i32 {
    let (root, profile, quiet) = match parse_gen_args(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    // Collect project info once for tasks/launch and compile-commands
    let all = match collect_all(&root, &profile) {
        Ok(v) => v,
        Err(e) => {
            error(&e);
            return 1;
        }
    };

    // 1. Generate compile_commands.json
    let cc_code = gen_compile_commands_inner(&root, &profile, &all, quiet);
    if cc_code != 0 {
        return cc_code;
    }

    let vscode_dir = root.join(".vscode");
    if let Err(e) = std::fs::create_dir_all(&vscode_dir) {
        error(&format!("Failed to create .vscode/: {e}"));
        return 1;
    }

    // tasks.json
    if let Err(e) = std::fs::write(vscode_dir.join("tasks.json"), gen_tasks_json()) {
        error(&format!("Failed to write tasks.json: {e}"));
        return 1;
    }
    println!("Generated {}", vscode_dir.join("tasks.json").display());

    // launch.json — one entry per binary target
    let launch = gen_launch_json(&all, &root);
    if let Err(e) = std::fs::write(vscode_dir.join("launch.json"), launch) {
        error(&format!("Failed to write launch.json: {e}"));
        return 1;
    }
    println!("Generated {}", vscode_dir.join("launch.json").display());

    // settings.json (clangd compile-commands-dir)
    let settings = gen_settings_json(&root);
    if let Err(e) = std::fs::write(vscode_dir.join("settings.json"), settings) {
        error(&format!("Failed to write settings.json: {e}"));
        return 1;
    }
    println!("Generated {}", vscode_dir.join("settings.json").display());

    // extensions.json — disable cpptools, recommend clangd
    if let Err(e) = std::fs::write(vscode_dir.join("extensions.json"), gen_extensions_json()) {
        error(&format!("Failed to write extensions.json: {e}"));
        return 1;
    }
    println!("Generated {}", vscode_dir.join("extensions.json").display());

    0
}

fn gen_extensions_json() -> String {
    r#"{
  "recommendations": [
    "llvm-vs-code-extensions.vscode-clangd",
    "vadimcn.vscode-lldb"
  ],
  "unwantedRecommendations": [
    "ms-vscode.cpptools",
    "ms-vscode.cpptools-extension-pack",
    "ms-vscode.cpptools-themes"
  ]
}
"#
    .to_string()
}

fn gen_tasks_json() -> String {
    r#"{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "dcr: build (debug)",
      "type": "shell",
      "command": "dcr build --debug",
      "group": {
        "kind": "build",
        "isDefault": true
      },
      "problemMatcher": ["$gcc"],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "dcr: build (release)",
      "type": "shell",
      "command": "dcr build --release",
      "group": "build",
      "problemMatcher": ["$gcc"],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "dcr: run (debug)",
      "type": "shell",
      "command": "dcr run --debug",
      "group": {
        "kind": "test",
        "isDefault": true
      },
      "problemMatcher": ["$gcc"],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "dcr: run (release)",
      "type": "shell",
      "command": "dcr run --release",
      "group": "test",
      "problemMatcher": ["$gcc"],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "dcr: clean",
      "type": "shell",
      "command": "dcr clean --all",
      "group": "none",
      "problemMatcher": [],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "dcr: gen compile-commands",
      "type": "shell",
      "command": "dcr gen compile-commands",
      "group": "none",
      "problemMatcher": [],
      "presentation": { "reveal": "always", "panel": "shared" }
    }
  ]
}
"#
    .to_string()
}

/// Absolute program path for a launch config; reuses `artifact_path` when profiles match,
/// otherwise recomputes from kind/target/out_dir/filename.
fn resolve_launch_program(info: &ProjectInfo, profile: &str) -> String {
    if info.profile == profile
        && let Some(ref p) = info.artifact_path
    {
        return p.clone();
    }
    let has_explicit = !info.target.trim().is_empty();
    let target_dir = resolve_artifact_target_dir(
        &info.root,
        info.workspace_root.as_deref(),
        profile,
        &info.target,
        &info.out_dir,
        has_explicit,
    );
    let rel = resolve_artifact_path(
        &info.kind,
        profile,
        &info.name,
        Some(&target_dir),
        info.output_filename.as_deref(),
        info.output_extension.as_deref(),
    );
    match rel {
        Some(r) => absolute_artifact_path(&info.root, &r)
            .to_string_lossy()
            .to_string(),
        None => {
            let name = if cfg!(windows) && !info.name.to_ascii_lowercase().ends_with(".exe") {
                format!("{}.exe", info.name)
            } else {
                info.name.clone()
            };
            Path::new(&target_dir)
                .join(name)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// VS Code debug type from debugger name: `cppdbg`, `cppvsdbg`, or `lldb`.
fn debug_type_for(info: &ProjectInfo) -> &'static str {
    let d = info.debugger.to_lowercase();
    if d.contains("gdb") || d == "cppdbg" {
        "cppdbg"
    } else if cfg!(windows) && (d.contains("vs") || d == "cppvsdbg") {
        "cppvsdbg"
    } else {
        "lldb"
    }
}

/// `launch.json` content: one debug config per `bin` target for debug and release.
fn gen_launch_json(projects: &[ProjectInfo], _root: &Path) -> String {
    let mut configs = Vec::new();

    for info in projects {
        if info.kind != "bin" {
            continue;
        }

        // binary expected at info.root/target/<profile>/<name> (account for member projects)
        let debug_bin = resolve_launch_program(info, "debug");
        let release_bin = resolve_launch_program(info, "release");
        let dtype = debug_type_for(info);

        let debug_entry = format!(
            r#"    {{
      "name": {name},
      "type": {dtype},
      "request": "launch",
      "program": {prog},
      "args": [],
      "stopOnEntry": false,
      "cwd": {cwd},
      "terminal": "integrated",
      "preLaunchTask": "dcr: build (debug)"
    }}"#,
            name = json_str(&format!("{} (debug)", info.name)),
            dtype = json_str(dtype),
            prog = json_str(&debug_bin),
            cwd = json_str(&info.root.to_string_lossy()),
        );

        let release_entry = format!(
            r#"    {{
      "name": {name},
      "type": {dtype},
      "request": "launch",
      "program": {prog},
      "args": [],
      "stopOnEntry": false,
      "cwd": {cwd},
      "terminal": "integrated",
      "preLaunchTask": "dcr: build (release)"
    }}"#,
            name = json_str(&format!("{} (release)", info.name)),
            dtype = json_str(dtype),
            prog = json_str(&release_bin),
            cwd = json_str(&info.root.to_string_lossy()),
        );

        configs.push(debug_entry);
        configs.push(release_entry);
    }

    if configs.is_empty() {
        // no binary targets — emit a placeholder
        configs.push(
            r#"    {
      "name": "(placeholder — no binary targets found)",
      "type": "lldb",
      "request": "launch",
      "program": "",
      "cwd": "${workspaceFolder}"
    }"#
            .to_string(),
        );
    }

    format!(
        "{{\n  \"version\": \"0.2.0\",\n  \"configurations\": [\n{}\n  ]\n}}\n",
        configs.join(",\n")
    )
}

fn gen_settings_json(root: &Path) -> String {
    let cc_dir = root.join(".dcr").to_string_lossy().to_string();
    format!(
        r#"{{
  "clangd.arguments": [
    "--compile-commands-dir={cc_dir}",
    "--header-insertion=never",
    "--clang-tidy=false"
  ],
  "C_Cpp.intelliSenseEngine": "disabled",
  "C_Cpp.autocomplete": "disabled",
  "C_Cpp.errorSquiggles": "disabled",
  "C_Cpp.hover": "disabled"
}}
"#
    )
}

// ── dcr gen clion ─────────────────────────────────────────────────────────────

/// `dcr gen clion`: compile_commands plus `.idea/` tools, targets, misc, run configs.
fn gen_clion(args: &[String]) -> i32 {
    let (root, profile, quiet) = match parse_gen_args(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    // collect project info
    let all = match collect_all(&root, &profile) {
        Ok(v) => v,
        Err(e) => {
            error(&e);
            return 1;
        }
    };

    // 1. compile_commands.json
    let cc_code = gen_compile_commands_inner(&root, &profile, &all, quiet);
    if cc_code != 0 {
        return cc_code;
    }

    let idea_dir = root.join(".idea");
    if let Err(e) = std::fs::create_dir_all(&idea_dir) {
        error(&format!("Failed to create .idea/: {e}"));
        return 1;
    }
    let run_configs_dir = idea_dir.join("runConfigurations");
    if let Err(e) = std::fs::create_dir_all(&run_configs_dir) {
        error(&format!("Failed to create .idea/runConfigurations/: {e}"));
        return 1;
    }

    // externalTools.xml
    let ext_tools = gen_clion_external_tools();
    if let Err(e) = std::fs::write(idea_dir.join("externalTools.xml"), ext_tools) {
        error(&format!("Failed to write externalTools.xml: {e}"));
        return 1;
    }
    println!("Generated {}", idea_dir.join("externalTools.xml").display());

    // customTargets.xml
    let targets = gen_clion_custom_targets();
    if let Err(e) = std::fs::write(idea_dir.join("customTargets.xml"), targets) {
        error(&format!("Failed to write customTargets.xml: {e}"));
        return 1;
    }
    println!("Generated {}", idea_dir.join("customTargets.xml").display());

    // misc.xml — point CLion at compile_commands.json
    let misc = gen_clion_misc_xml(&root);
    if let Err(e) = std::fs::write(idea_dir.join("misc.xml"), misc) {
        error(&format!("Failed to write misc.xml: {e}"));
        return 1;
    }
    println!("Generated {}", idea_dir.join("misc.xml").display());

    // .idea/.gitignore
    let gitignore = "# CLion generated files\nworkspace.xml\n*.iml\n";
    if let Err(e) = std::fs::write(idea_dir.join(".gitignore"), gitignore) {
        error(&format!("Failed to write .idea/.gitignore: {e}"));
        return 1;
    }
    println!("Generated {}", idea_dir.join(".gitignore").display());

    // runConfigurations/<name>.xml — one per binary
    for info in &all {
        if info.kind != "bin" {
            continue;
        }
        let xml = gen_clion_run_config(info, &root, &profile);
        let fname = format!("{}.xml", sanitize_filename(&info.name));
        let path = run_configs_dir.join(&fname);
        if let Err(e) = std::fs::write(&path, xml) {
            error(&format!("Failed to write runConfigurations/{fname}: {e}"));
            return 1;
        }
        println!("Generated {}", path.display());
    }

    0
}

fn gen_clion_external_tools() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="ExternalToolsComponent">
    <tools name="DCR">
      <tool name="Build Debug"
            description="dcr build --debug"
            showInMainMenu="true"
            showInEditor="false"
            showInProject="false"
            showInSearchPopup="false"
            disabled="false"
            useConsole="true"
            showConsoleOnStdOut="false"
            showConsoleOnStdErr="true"
            synchronizeAfterRun="true">
        <exec>
          <option name="COMMAND" value="dcr" />
          <option name="PARAMETERS" value="build --debug" />
          <option name="WORKING_DIRECTORY" value="$ProjectFileDir$" />
        </exec>
      </tool>
      <tool name="Build Release"
            description="dcr build --release"
            showInMainMenu="true"
            showInEditor="false"
            showInProject="false"
            showInSearchPopup="false"
            disabled="false"
            useConsole="true"
            showConsoleOnStdOut="false"
            showConsoleOnStdErr="true"
            synchronizeAfterRun="true">
        <exec>
          <option name="COMMAND" value="dcr" />
          <option name="PARAMETERS" value="build --release" />
          <option name="WORKING_DIRECTORY" value="$ProjectFileDir$" />
        </exec>
      </tool>
      <tool name="Clean"
            description="dcr clean"
            showInMainMenu="true"
            showInEditor="false"
            showInProject="false"
            showInSearchPopup="false"
            disabled="false"
            useConsole="true"
            showConsoleOnStdOut="false"
            showConsoleOnStdErr="true"
            synchronizeAfterRun="true">
        <exec>
          <option name="COMMAND" value="dcr" />
          <option name="PARAMETERS" value="clean" />
          <option name="WORKING_DIRECTORY" value="$ProjectFileDir$" />
        </exec>
      </tool>
      <tool name="Gen Compile Commands"
            description="dcr gen compile-commands"
            showInMainMenu="true"
            showInEditor="false"
            showInProject="false"
            showInSearchPopup="false"
            disabled="false"
            useConsole="true"
            showConsoleOnStdOut="false"
            showConsoleOnStdErr="true"
            synchronizeAfterRun="true">
        <exec>
          <option name="COMMAND" value="dcr" />
          <option name="PARAMETERS" value="gen compile-commands" />
          <option name="WORKING_DIRECTORY" value="$ProjectFileDir$" />
        </exec>
      </tool>
    </tools>
  </component>
</project>
"#
    .to_string()
}

fn gen_clion_custom_targets() -> String {
    // Fixed UUIDs for CLion custom targets
    let uuid = "dcr00000-0000-0000-0000-000000000001";
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="CLionExternalBuildManager">
    <target id="{uuid}"
            name="dcr: build (debug)"
            defaultType="TOOL">
      <build type="TOOL">
        <tool actionId="Tool_DCR_Build Debug" />
      </build>
      <clean type="TOOL">
        <tool actionId="Tool_DCR_Clean" />
      </clean>
    </target>
    <target id="dcr00000-0000-0000-0000-000000000002"
            name="dcr: build (release)"
            defaultType="TOOL">
      <build type="TOOL">
        <tool actionId="Tool_DCR_Build Release" />
      </build>
      <clean type="TOOL">
        <tool actionId="Tool_DCR_Clean" />
      </clean>
    </target>
  </component>
</project>
"#
    )
}

fn gen_clion_misc_xml(root: &Path) -> String {
    let cc_path = root.join(".dcr").join("compile_commands.json");
    let cc = xml_escape(&cc_path.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="CMakeWorkspace" PROJECT_DIR="$PROJECT_DIR$" />
  <component name="CompDBWorkspace" projectDir="$PROJECT_DIR$">
    <customCompileCommandsPath>{cc}</customCompileCommandsPath>
  </component>
</project>
"#
    )
}

/// XML for one CLion run configuration under `.idea/runConfigurations/`.
fn gen_clion_run_config(info: &ProjectInfo, _root: &Path, profile: &str) -> String {
    let bin_path = resolve_launch_program(info, profile);
    let bin = xml_escape(&bin_path);
    let target = if profile == "release" {
        "dcr: build (release)"
    } else {
        "dcr: build (debug)"
    };
    let target_esc = xml_escape(target);
    let name_esc = xml_escape(&format!("{} ({})", info.name, profile));
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<component name="ProjectRunConfigurationManager">
  <configuration default="false"
                 name="{name_esc}"
                 type="CLionExternalRunConfiguration"
                 factoryName="Application">
    <build target="{target_esc}" />
    <executable path="{bin}" />
    <workingDirectory value="$PROJECT_DIR$" />
    <envs />
    <method v="2">
      <option name="CLionExternalBuildTargetBeforeRunTask" enabled="true" />
    </method>
  </configuration>
</component>
"#
    )
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Parses `--debug` / `--release` / `--quiet` and resolves the project root.
fn parse_gen_args(args: &[String]) -> Result<(PathBuf, String, bool), i32> {
    let mut profile = "debug".to_string();
    let mut quiet = false;
    for arg in args {
        match arg.as_str() {
            "--debug" => profile = "debug".to_string(),
            "--release" => profile = "release".to_string(),
            "--quiet" | "-q" => quiet = true,
            _ => {}
        }
    }

    let start = std::env::current_dir().map_err(|_| {
        error("Failed to determine current directory");
        1i32
    })?;

    let root = match find_project_root(&start) {
        Ok(Some(r)) => canonicalize_path(&r),
        Ok(None) => {
            error("dcr.toml not found");
            return Err(1);
        }
        Err(_) => {
            error("Failed to find project root");
            return Err(1);
        }
    };

    Ok((root, profile, quiet))
}

fn json_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            c if c.is_control() => {
                // Escape other control characters as unicode escapes
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn json_str_array(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| json_str(s)).collect();
    format!("[{}]", inner.join(", "))
}
