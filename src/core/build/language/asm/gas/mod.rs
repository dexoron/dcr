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
use crate::core::build::language::asm::common as asm;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Builds GAS (`.s`) sources via the GNU Assembler.
///
/// Resolves `as` on PATH and delegates to the shared assembly builder. For
/// `flat-bin`, intermediates are converted with objcopy after assembly.
pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    // Use full path to as
    let as_path = std::process::Command::new("which")
        .arg("as")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "as".to_string());
    asm::build_assembly_via_objcopy(ctx, "GAS", &as_path, &["s"], build_object)
}

/// Collects assembly source files with .s extension from the build context's source roots.
///
/// Excludes specified directories and respects include paths for filtering.
pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["s"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

/// Assembles a single .s source file into an object file using the provided assembler.
///
/// Creates parent directories for the output object, skips if up-to-date,
/// runs the assembler with flags from the build context, and handles verbose output.
fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    // Check if the object file is already up to date to avoid unnecessary rebuilds.
    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    // Prepare the assembler command with source file and output path.
    let mut cmd = Command::new(assembler);
    cmd.arg(source).arg("-o").arg(obj_path);

    for flag in crate::core::build::language::asm::common::filter_asm_flags(ctx.cflags) {
        cmd.arg(flag);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}
