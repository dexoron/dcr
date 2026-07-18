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
use std::process::Command;
use std::time::Instant;

pub(crate) fn archive_static(
    ctx: &BuildContext,
    objects: &[String],
    start_time: Instant,
) -> Result<f64, String> {
    let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
    if !common::needs_link(objects, &lib_path) {
        return Ok(common::elapsed_secs(start_time));
    }

    let archiver = ctx.archiver.unwrap_or(if cfg!(target_os = "windows") {
        "lib"
    } else {
        "ar"
    });

    let mut cmd = Command::new(archiver);
    if cfg!(target_os = "windows") && archiver == "lib" {
        cmd.arg("/nologo").arg(format!("/OUT:{lib_path}"));
    } else {
        cmd.arg("rcs").arg(&lib_path);
    }
    for obj in objects {
        cmd.arg(obj);
    }
    run(ctx, cmd, start_time)
}

pub(crate) fn link_binary(
    ctx: &BuildContext,
    objects: &[String],
    default_linker: &str,
    start_time: Instant,
) -> Result<f64, String> {
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

fn run(ctx: &BuildContext, mut cmd: Command, start_time: Instant) -> Result<f64, String> {
    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    let output = cmd.output().map_err(|err| format!("Build failed: {err}"))?;
    if output.status.success() {
        Ok(common::elapsed_secs(start_time))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("Build failed:\nstdout: {stdout}\nstderr: {stderr}"))
    }
}
