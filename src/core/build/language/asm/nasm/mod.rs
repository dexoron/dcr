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

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly(ctx, "NASM", "nasm", &["asm"], build_object)
}

pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    // NASM handles .asm only (not .s which is GAS syntax, not .S which is GCC preprocessed)
    common::collect_sources(
        ctx.source_roots,
        &["asm"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    let format = if crate::utils::build::is_flat_bin(ctx.kind) {
        "bin"
    } else {
        nasm_format(ctx.platform)
    };

    let mut cmd = Command::new(assembler);
    cmd.arg("-f")
        .arg(format)
        .arg(source)
        .arg("-o")
        .arg(obj_path);

    for flag in crate::core::build::language::asm::common::filter_asm_flags(ctx.cflags) {
        cmd.arg(flag);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}

fn nasm_format(platform: Option<&str>) -> &'static str {
    if let Some(p) = platform {
        let p = p.to_lowercase().replace('-', "_");
        if p == "x86" || (p.starts_with('i') && p.ends_with("86") && p.len() == 4) {
            #[cfg(target_os = "macos")]
            {
                return "macho32";
            }
            #[cfg(target_os = "windows")]
            {
                return "win32";
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                return "elf32";
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        "elf64"
    }
    #[cfg(target_os = "macos")]
    {
        "macho64"
    }
    #[cfg(target_os = "windows")]
    {
        "win64"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "elf64"
    }
}
