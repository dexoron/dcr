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
use crate::core::build::common;
use crate::platform;
use crate::utils::build::is_bare_metal_target;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Returns the artifact directory path based on the build context.
pub(crate) fn artifact_dir(ctx: &BuildContext) -> PathBuf {
    match ctx.target_dir {
        Some(dir) => Path::new(dir).to_path_buf(),
        None => Path::new("./target").join(ctx.profile),
    }
}

/// Resolves the artifact path for a given kind/profile/name.
///
/// # Parameters
/// - `kind`: Artifact kind (`bin`, `staticlib`, `sharedlib`, `flat-bin`, `efi`, `elf`, …).
/// - `profile`: Build profile (`debug` / `release` / custom).
/// - `project_name`: Default basename when `output_filename` is unset.
/// - `target_dir`: Optional artifact directory override.
/// - `output_filename` / `output_extension`: Optional name/ext overrides.
///
/// # Returns
/// Relative/absolute path string, or `None` for `none` / `custom` kinds.
pub fn resolve_artifact_path(
    kind: &str,
    profile: &str,
    project_name: &str,
    target_dir: Option<&str>,
    output_filename: Option<&str>,
    output_extension: Option<&str>,
) -> Option<String> {
    if crate::utils::build::is_flat_bin(kind) {
        let name = output_filename.unwrap_or(project_name);
        let ext = output_extension.unwrap_or("bin");
        let file = if ext.is_empty() {
            name.to_string()
        } else {
            format!("{name}.{ext}")
        };
        let dir = match target_dir {
            Some(dir) => Path::new(dir).to_path_buf(),
            None => Path::new("./target").join(profile),
        };
        return Some(dir.join(file).to_string_lossy().to_string());
    }

    if matches!(kind, "none" | "custom") {
        return None;
    }

    let name = output_filename.unwrap_or(project_name);
    let ext = output_extension.unwrap_or("");
    let final_name = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    };

    let path = match kind {
        "staticlib" => platform::lib_path(profile, &final_name, target_dir),
        "sharedlib" => platform::shared_lib_path(profile, &final_name, target_dir),
        "efi" => platform::efi_path(profile, &final_name, target_dir),
        "elf" => platform::elf_path(profile, &final_name, target_dir),
        _ => platform::bin_path(profile, &final_name, target_dir),
    };
    Some(path)
}

/// Converts a relative or absolute path to an absolute path by joining with project root if needed.
pub fn absolute_artifact_path(project_root: &Path, relative_or_abs: &str) -> PathBuf {
    let p = Path::new(relative_or_abs);
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        let stripped = relative_or_abs
            .strip_prefix("./")
            .unwrap_or(relative_or_abs);
        project_root.join(stripped)
    };
    crate::utils::fs::canonicalize_path(&joined)
}

/// Returns the output path for a flat binary artifact.
pub(crate) fn flat_output_path(ctx: &BuildContext) -> String {
    let name = ctx.output_filename.unwrap_or(ctx.project_name);
    let ext = ctx.output_extension.unwrap_or("bin");
    let file = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{name}.{ext}")
    };
    artifact_dir(ctx).join(file).to_string_lossy().to_string()
}

/// Returns the output path for a flat binary artifact from a source file.
pub(crate) fn flat_source_output_path(ctx: &BuildContext, source: &str) -> String {
    let stem = Path::new(source)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    let ext = ctx.output_extension.unwrap_or("bin");
    let file = if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{ext}")
    };
    artifact_dir(ctx).join(file).to_string_lossy().to_string()
}

// Helper to find a suitable objcopy tool in PATH
fn resolve_objcopy() -> Result<String, String> {
    for candidate in ["llvm-objcopy", "objcopy", "gobjcopy"] {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }
    }
    Err(
        "flat-bin requires llvm-objcopy, objcopy, or gobjcopy in PATH \
         (install binutils or LLVM tools)"
            .to_string(),
    )
}

/// Converts an object/ELF to a raw binary via `objcopy -O binary` (skips if up to date).
pub(crate) fn objcopy_binary(ctx: &BuildContext, input: &str, output: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(output).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("output dir error: {e}"))?;
    }
    if !common::needs_rebuild(input, output) {
        return Ok(());
    }
    let tool = resolve_objcopy()?;
    let mut cmd = Command::new(&tool);
    cmd.arg("-O").arg("binary").arg(input).arg(output);
    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    common::run_command_sync_output(&mut cmd)
}

/// Links objects into an intermediate ELF, then converts it to a flat binary via objcopy.
pub(crate) fn link_flat_binary(
    ctx: &BuildContext,
    objects: &[String],
    default_linker: &str,
    start_time: Instant,
) -> Result<f64, String> {
    let out_path = flat_output_path(ctx);
    if !common::needs_link(objects, &out_path) {
        return Ok(common::elapsed_secs(start_time));
    }

    let intermediate = {
        let dir = match ctx.target_dir {
            Some(dir) => Path::new(dir).join("obj"),
            None => Path::new("./target").join(ctx.profile).join("obj"),
        };
        std::fs::create_dir_all(&dir).map_err(|e| format!("obj dir error: {e}"))?;
        dir.join(format!("{}.flat.elf", ctx.project_name))
            .to_string_lossy()
            .to_string()
    };

    let mut cmd = Command::new(ctx.linker.unwrap_or(default_linker));
    cmd.arg("-nostdlib").arg("-static");
    for obj in objects {
        cmd.arg(obj);
    }
    for dir in ctx.lib_dirs {
        cmd.arg(format!("-L{dir}"));
    }
    for lib in ctx.libs {
        cmd.arg(format!("-l{lib}"));
    }
    for flag in ctx.ldflags {
        cmd.arg(flag);
    }
    cmd.arg("-o").arg(&intermediate);

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    let program = cmd.get_program().to_string_lossy().into_owned();
    let output = cmd.output().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            format!("linker not found: {program} (check [toolchain].ld / PATH)")
        } else {
            format!("flat-bin link failed: {err}")
        }
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "flat-bin link failed:\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }

    objcopy_binary(ctx, &intermediate, &out_path)?;
    Ok(common::elapsed_secs(start_time))
}

/// Archives objects into a static library.
pub(crate) fn archive_static(
    ctx: &BuildContext,
    objects: &[String],
    start_time: Instant,
) -> Result<f64, String> {
    let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
    if !common::needs_link(objects, &lib_path) {
        return Ok(common::elapsed_secs(start_time));
    }

    let archiver = ctx
        .archiver
        .unwrap_or_else(|| default_archiver(ctx.compiler));

    let mut cmd = Command::new(archiver);
    if cfg!(target_os = "windows") && is_msvc_lib(archiver) {
        cmd.arg("/nologo").arg(format!("/OUT:{lib_path}"));
    } else {
        cmd.arg("rcs").arg(&lib_path);
    }
    for obj in objects {
        cmd.arg(obj);
    }
    run(ctx, cmd, start_time)
}

/// Links objects into a binary or library based on artifact kind.
pub(crate) fn link_binary(
    ctx: &BuildContext,
    objects: &[String],
    default_linker: &str,
    start_time: Instant,
) -> Result<f64, String> {
    if crate::utils::build::is_flat_bin(ctx.kind) {
        return link_flat_binary(ctx, objects, default_linker, start_time);
    }

    let mut cmd = Command::new(ctx.linker.unwrap_or(default_linker));
    if ctx.kind == "sharedlib" {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
    }
    if ctx.kind == "efi" {
        cmd.arg("-shared");
        cmd.arg("-nostdlib");
        cmd.arg("-Wl,-dll");
        cmd.arg("-Wl,--subsystem,10");
    } else if ctx.freestanding || is_bare_metal_target(ctx.target) {
        cmd.arg("-nostdlib");
        cmd.arg("-static");
    }
    for obj in objects {
        cmd.arg(obj);
    }
    for dir in ctx.lib_dirs {
        cmd.arg(format!("-L{dir}"));
    }
    for lib in ctx.libs {
        cmd.arg(format!("-l{lib}"));
    }
    for flag in ctx.ldflags {
        cmd.arg(flag);
    }

    let name = ctx.output_filename.unwrap_or(ctx.project_name);
    let ext = ctx.output_extension.unwrap_or("");
    let final_name = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    };

    let out_path = if ctx.kind == "sharedlib" {
        platform::shared_lib_path(ctx.profile, &final_name, ctx.target_dir)
    } else if ctx.kind == "efi" {
        platform::efi_path(ctx.profile, &final_name, ctx.target_dir)
    } else if ctx.kind == "elf" {
        platform::elf_path(ctx.profile, &final_name, ctx.target_dir)
    } else {
        platform::bin_path(ctx.profile, &final_name, ctx.target_dir)
    };

    if !common::needs_link(objects, &out_path) {
        return Ok(common::elapsed_secs(start_time));
    }
    cmd.arg("-o").arg(out_path);
    run(ctx, cmd, start_time)
}

/// Returns the default archiver name based on compiler and platform.
fn default_archiver(compiler: &str) -> &'static str {
    let c = compiler.to_lowercase();
    if cfg!(target_os = "windows") {
        if c.contains("cl") && !c.contains("clang") {
            "lib"
        } else if c.contains("clang") {
            "llvm-ar"
        } else {
            "ar"
        }
    } else {
        "ar"
    }
}

/// Checks if the archiver is an MSVC lib tool.
fn is_msvc_lib(archiver: &str) -> bool {
    let a = archiver.to_lowercase();
    a == "lib" || a == "lib.exe" || a.ends_with("\\lib.exe") || a.ends_with("/lib.exe")
}

/// Runs a command and handles success/failure with timing.
fn run(ctx: &BuildContext, mut cmd: Command, start_time: Instant) -> Result<f64, String> {
    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    let program = cmd.get_program().to_string_lossy().into_owned();
    let output = cmd.output().map_err(|err| {
        let msg = err.to_string();
        if err.kind() == std::io::ErrorKind::NotFound {
            format!("linker not found: {program} (check [toolchain].ld / PATH)")
        } else {
            format!("Build failed: {msg}")
        }
    })?;
    if output.status.success() {
        Ok(common::elapsed_secs(start_time))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("Build failed:\nstdout: {stdout}\nstderr: {stderr}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default `bin` kind resolves via platform bin path.
    #[test]
    fn resolve_bin_default() {
        let p = resolve_artifact_path("bin", "debug", "hello", None, None, None).unwrap();
        assert_eq!(p, platform::bin_path("debug", "hello", None));
    }

    /// Custom `target_dir` places the binary under that directory.
    #[test]
    fn resolve_bin_custom_target_dir() {
        let p = resolve_artifact_path("bin", "debug", "hello", Some("/out"), None, None).unwrap();
        let name = if cfg!(windows) { "hello.exe" } else { "hello" };
        let expected = Path::new("/out").join(name);
        assert_eq!(p, expected.to_string_lossy());
    }

    /// `staticlib` kind resolves to a platform static library path.
    #[test]
    fn resolve_staticlib() {
        let p = resolve_artifact_path("staticlib", "release", "mylib", Some("dist"), None, None)
            .unwrap();
        let name = if cfg!(windows) {
            "mylib.lib"
        } else {
            "libmylib.a"
        };
        let expected = Path::new("dist").join(name);
        assert_eq!(p, expected.to_string_lossy());
    }

    /// Custom output filename and extension override the default name.
    #[test]
    fn resolve_with_filename_ext() {
        let p = resolve_artifact_path(
            "bin",
            "debug",
            "pkg",
            Some("/out"),
            Some("KERNEL"),
            Some("ELF"),
        )
        .unwrap();
        assert_eq!(
            p,
            std::path::Path::new("/out")
                .join("KERNEL.ELF")
                .to_string_lossy()
        );
    }

    /// `none` kind has no artifact path.
    #[test]
    fn resolve_none_kind() {
        assert!(resolve_artifact_path("none", "debug", "x", None, None, None).is_none());
    }
}
