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

use crate::core::builder::BuildContext;
use crate::core::builder::common;
use crate::platform;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let lang = ctx.language.to_lowercase();
    if lang.split(',').any(|p| p.trim() != "asm") {
        return Err("GAS backend requires build.language = \"asm\"".to_string());
    }
    let assembler = if ctx.compiler.is_empty() {
        "as"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let sources = collect_sources(ctx)?;
    let obj_dir = match ctx.target_dir {
        Some(dir) => Path::new(dir).join("obj"),
        None => Path::new("./target").join(ctx.profile).join("obj"),
    };
    let objects = build_objects(assembler, &sources, &obj_dir, ctx, "o")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
        if !common::needs_link(&objects, &lib_path) {
            let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
            return Ok(elapsed);
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
        for obj in &objects {
            cmd.arg(obj);
        }
        if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
            eprintln!("[dcr] {:?}", cmd);
        }
        match cmd.status() {
            Ok(status) if status.success() => {
                let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
                return Ok(elapsed);
            }
            Ok(_) => return Err("Build failed".to_string()),
            Err(err) => return Err(format!("Build failed: {err}")),
        }
    }

    let mut cmd = Command::new(ctx.linker.unwrap_or("cc"));
    if ctx.kind == "sharedlib" {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
    }
    for obj in &objects {
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
    } else if ctx.kind == "elf" {
        platform::elf_path(ctx.profile, &final_name, ctx.target_dir)
    } else {
        platform::bin_path(ctx.profile, &final_name, ctx.target_dir)
    };

    if !common::needs_link(&objects, &out_path) {
        let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
        return Ok(elapsed);
    }
    cmd.arg("-o").arg(out_path);

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    match cmd.status() {
        Ok(status) if status.success() => {
            let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
            Ok(elapsed)
        }
        Ok(_) => Err("Build failed".to_string()),
        Err(err) => Err(format!("Build failed: {err}")),
    }
}

pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    // GAS handles only lowercase .s files
    common::collect_sources(
        ctx.source_roots,
        &["s"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

fn build_objects(
    assembler: &str,
    sources: &[String],
    obj_dir: &Path,
    ctx: &BuildContext,
    obj_ext: &str,
) -> Result<Vec<String>, String> {
    let objects: Vec<String> = sources
        .iter()
        .map(|s| common::object_path(obj_dir, s, obj_ext))
        .collect();

    common::parallel_build(
        sources.len(),
        |i| build_object(assembler, &sources[i], &objects[i], ctx),
        ctx.codegen_units,
    )?;

    Ok(objects)
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

    let mut cmd = Command::new(assembler);
    cmd.arg(source).arg("-o").arg(obj_path);

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    common::run_command_sync_output(&mut cmd)
}
