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
use crate::cli::r#gen::{ProjectInfo, write_dcr_metadata};
use crate::core::build::builder::artifact::{absolute_artifact_path, resolve_artifact_path};
use crate::core::build::common;
use crate::core::build::{BuildEvent, BuildReporter, BuildRequest, run_build};
use crate::core::build_config::Config;
use crate::utils::build::{
    get_config_opt, get_config_str, get_language_with_profile_or_default, get_string_with_profile,
    normalize_kind, resolve_artifact_target_dir, resolve_compiler,
};
use crate::utils::fs::{canonicalize_path, find_project_root};
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, BOLD_YELLOW, colored, printc};
use std::io::IsTerminal;
use std::path::Path;
use std::sync::{Arc, atomic::AtomicBool};

pub use crate::core::build::get_build_string_with_profile;

/// Formats a left-aligned status verb with the given style for CLI output.
fn status(verb: &str, style: &str) -> String {
    colored(&format!("{verb:<9}"), style)
}

/// CLI build reporter that prints human-readable progress to stderr/stdout.
struct CliReporter;

impl CliReporter {
    /// Prints a single status line with a styled verb and free-form rest text.
    ///
    /// # Parameters
    /// - `verb`: Short status label (e.g. `"target"`, `"dep"`).
    /// - `style`: Color/style constant for the verb.
    /// - `rest`: Remaining message text after the verb.
    fn line(&self, verb: &str, style: &str, rest: &str) {
        eprintln!("  {} {}", status(verb, style), rest);
    }
}

impl BuildReporter for CliReporter {
    /// Handles build events by printing progress, status lines, and compiler output.
    ///
    /// # Parameters
    /// - `event`: Build pipeline event to report to the user.
    fn on_event(&mut self, event: BuildEvent<'_>) {
        match event {
            BuildEvent::TargetStart {
                index,
                total,
                target,
            } => {
                common::finish_progress_line();
                self.line("target", BOLD_CYAN, &format!("{index}/{total}  {target}"));
            }
            BuildEvent::ProjectStart {
                name,
                profile,
                target,
            } => {
                common::finish_progress_line();
                self.line(
                    "project",
                    BOLD_CYAN,
                    &format!("{name}  ({profile}, {target})"),
                );
            }
            BuildEvent::DepBuilding { name, version } => {
                common::finish_progress_line();
                self.line("dep", BOLD_YELLOW, &format!("{name} v{version}"));
            }
            BuildEvent::DepReady {
                name,
                version,
                rebuilt,
            } => {
                common::finish_progress_line();
                // Only announce deps that were actually rebuilt this run.
                if rebuilt {
                    self.line("ready", BOLD_GREEN, &format!("{name} v{version}"));
                }
            }
            BuildEvent::Compiling { name, version } => {
                common::finish_progress_line();
                let label = format!("  {} {} v{}", status("compile", BOLD_GREEN), name, version);
                // Terminals use an in-place progress line; non-TTY gets a plain log line.
                common::set_progress_label(Some(label.clone()));
                if !std::io::stderr().is_terminal() {
                    eprintln!("{label}");
                }
            }
            BuildEvent::Packing { path } => {
                common::finish_progress_line();
                self.line("pack", BOLD_CYAN, path);
            }
            BuildEvent::Finished { secs } => {
                common::finish_progress_line();
                self.line("done", BOLD_GREEN, &format!("in {secs}s"));
            }
            BuildEvent::CompilerOutput { stream, text } => {
                // Pause the progress spinner so compiler text is not overwritten.
                common::interrupt_progress_for_output();
                if stream == "stderr" {
                    eprint!("{text}");
                } else {
                    print!("{text}");
                }
            }
        }
    }
}

/// Runs the project build command from the CLI.
///
/// Parses flags, locates the project root, optionally cleans, then invokes the
/// core build pipeline. On success, may write `.dcr` metadata and print the
/// main artifact path when requested.
///
/// # Parameters
/// - `args`: Tokens after `dcr build` (profile, target, force, clean, etc.).
///
/// # Returns
/// Process exit code: `0` on success or help, non-zero on failure.
pub fn build(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr build [--debug | --release] [--target <triple>] [--force] [--clean] [--verbose] [--print-artifact-path]",
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
        println!(
            "    --print-artifact-path  Print absolute main artifact path on stdout after success"
        );
        return 0;
    }

    // Shared cancel flag so Ctrl-C can stop an in-progress build cooperatively.
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();
    ctrlc::set_handler(move || {
        cancel_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .ok();
    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };
    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => canonicalize_path(&dir),
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

    // Prefer CLI --target; otherwise fall back to profile target from dcr.toml.
    if flags.target.is_none() {
        let config_path = root.join("dcr.toml");
        if let Ok(config) = Config::open(config_path.to_str().unwrap())
            && !config.is_workspace_only()
        {
            let bt = get_build_string_with_profile(&config, "target", &flags.profile);
            if !bt.is_empty() {
                flags.target = Some(bt);
            }
        }
    }
    if flags.clean {
        let mut clean_args = Vec::new();
        clean_args.push(format!("--{}", flags.profile));
        let _ = clean(&clean_args);
    }

    let req = BuildRequest {
        profile: flags.profile.clone(),
        target: flags.target.clone(),
        force: flags.force,
        verbose: flags.verbose,
        workspace: flags.workspace.clone(),
        cancel,
    };
    let mut reporter = CliReporter;
    match run_build(&root, &req, &mut reporter) {
        Ok(_) => {
            // Persist IDE/metadata and optionally echo the main artifact path.
            if let Some(info) =
                resolve_post_build_info(&root, &flags.profile, flags.target.as_deref())
            {
                let _ = write_dcr_metadata(&root, &info);
                if flags.print_artifact_path
                    && let Some(ref path) = info.artifact_path
                {
                    println!("{path}");
                }
            } else if flags.print_artifact_path
                && let Some(path) =
                    fallback_artifact_path(&root, &flags.profile, flags.target.as_deref())
            {
                println!("{path}");
            }
            0
        }
        Err(err) => {
            common::finish_progress_line();
            error(&err.message);
            1
        }
    }
}

/// Collects project and artifact metadata after a successful build.
///
/// Reads `dcr.toml`, resolves language/compiler/kind/target/output settings for
/// the given profile, and computes absolute artifact and target directory paths.
///
/// # Parameters
/// - `root`: Project root with `dcr.toml`.
/// - `profile`: Build profile used for the just-finished build.
/// - `cli_target`: Optional target from the CLI (overrides config when set).
///
/// # Returns
/// `Some(ProjectInfo)` for a normal package; `None` for workspace-only roots
/// or when required config/paths cannot be resolved.
fn resolve_post_build_info(
    root: &Path,
    profile: &str,
    cli_target: Option<&str>,
) -> Option<ProjectInfo> {
    let config = Config::open(root.join("dcr.toml").to_str()?).ok()?;
    if config.is_workspace_only() {
        return None;
    }
    let name = get_config_str(&config, "package.name");
    let version = get_config_str(&config, "package.version");
    let language = get_language_with_profile_or_default(&config, profile);
    let standard = get_string_with_profile(&config, "standard", profile);
    let cxx_standard = get_string_with_profile(&config, "cxx_standard", profile);
    let compiler_s = get_string_with_profile(&config, "compiler", profile);
    let kind = normalize_kind(&get_string_with_profile(&config, "kind", profile)).to_string();
    let build_target = cli_target
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| get_string_with_profile(&config, "target", profile));
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
    let resolved_compiler = resolve_compiler(
        &language,
        &compiler_s,
        tc_cc.as_deref(),
        tc_cxx.as_deref(),
        tc_as.as_deref(),
    );
    let out_dir = get_string_with_profile(&config, "out_dir", profile);
    // Explicit target (CLI or config) changes how the artifact directory is laid out.
    let has_explicit = cli_target.is_some_and(|t| !t.is_empty()) || !build_target.trim().is_empty();
    let target_dir =
        resolve_artifact_target_dir(root, None, profile, &build_target, &out_dir, has_explicit);
    let rel = resolve_artifact_path(
        &kind,
        profile,
        &name,
        Some(&target_dir),
        output_filename.as_deref(),
        output_extension.as_deref(),
    )?;
    let artifact_path = absolute_artifact_path(root, &rel)
        .to_string_lossy()
        .to_string();
    let abs_target_dir = {
        let p = Path::new(&target_dir);
        let path = if p.is_absolute() {
            canonicalize_path(p)
        } else {
            canonicalize_path(&root.join(p))
        };
        path.to_string_lossy().into_owned()
    };
    Some(ProjectInfo {
        name,
        version,
        root: root.to_path_buf(),
        profile: profile.to_string(),
        language,
        standard,
        cxx_standard,
        compiler: resolved_compiler,
        kind: kind.clone(),
        target: build_target,
        target_dir: Some(abs_target_dir),
        artifact_path: Some(artifact_path),
        artifact_kind: kind,
        sources: vec![],
        include_dirs: vec![],
        lib_dirs: vec![],
        libs: vec![],
        cflags: vec![],
        ldflags: vec![],
        source_roots: vec![],
        include_globs: vec![],
        exclude_globs: vec![],
        output_filename,
        output_extension,
        out_dir,
        debugger: "lldb".to_string(),
        moc: None,
        uic: None,
        rcc: None,
        workspace_root: None,
    })
}

/// Best-effort absolute main artifact path when full post-build info is unavailable.
///
/// Assumes a binary (`"bin"`) kind and uses the package name plus profile,
/// target, and `out_dir` settings from config.
fn fallback_artifact_path(root: &Path, profile: &str, target: Option<&str>) -> Option<String> {
    let config = Config::open(root.join("dcr.toml").to_str()?).ok()?;
    let name = get_config_str(&config, "package.name");
    if name.is_empty() {
        return None;
    }
    let build_target = target.unwrap_or("").to_string();
    let out_dir = get_string_with_profile(&config, "out_dir", profile);
    let has_explicit = !build_target.trim().is_empty();
    let target_dir =
        resolve_artifact_target_dir(root, None, profile, &build_target, &out_dir, has_explicit);
    let rel = resolve_artifact_path("bin", profile, &name, Some(&target_dir), None, None)?;
    Some(
        absolute_artifact_path(root, &rel)
            .to_string_lossy()
            .to_string(),
    )
}
