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
use crate::core::build::builder::artifact;
use crate::core::build::common;
use crate::utils::build::is_bare_metal_target;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = if ctx.compiler.is_empty() {
        "cc"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let obj_dir = match ctx.target_dir {
        Some(dir) => Path::new(dir).join("obj"),
        None => Path::new("./target").join(ctx.profile).join("obj"),
    };
    let qt_include_path = if ctx.qt {
        crate::core::build::language::cxx::qt::process_qt(ctx, &obj_dir)?
    } else {
        None
    };

    let mut all_source_roots = ctx.source_roots.to_vec();
    if let Some(qt_path) = &qt_include_path {
        all_source_roots.push(qt_path.clone());
    }

    let extensions = source_extensions(ctx.language);
    let sources = common::collect_sources(
        &all_source_roots,
        &extensions,
        ctx.exclude_dirs,
        ctx.include_paths,
    )?;
    let objects = build_objects(compiler, &sources, &obj_dir, ctx, "o", qt_include_path)?;

    if ctx.kind == "staticlib" {
        return artifact::archive_static(ctx, &objects, start_time);
    }

    artifact::link_binary(ctx, &objects, compiler, start_time)
}

pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    let extensions = source_extensions(ctx.language);
    common::collect_sources(
        ctx.source_roots,
        &extensions,
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

fn source_extensions(language: &str) -> Vec<&'static str> {
    crate::core::build::common::source_extensions(language)
}

fn default_flags(profile: &str) -> &'static [&'static str] {
    crate::utils::build::default_profile_flags(profile)
}

fn build_objects(
    compiler: &str,
    sources: &[String],
    obj_dir: &Path,
    ctx: &BuildContext,
    obj_ext: &str,
    qt_include_dir: Option<PathBuf>,
) -> Result<Vec<String>, String> {
    let objects: Vec<String> = sources
        .iter()
        .map(|s| common::object_path(obj_dir, s, obj_ext))
        .collect();

    common::parallel_build(
        sources.len(),
        |i| {
            build_object(
                compiler,
                &sources[i],
                &objects[i],
                ctx,
                qt_include_dir.as_deref(),
            )
        },
        ctx.codegen_units,
    )?;

    Ok(objects)
}

fn build_object(
    compiler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
    qt_include_dir: Option<&Path>,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    let mut cmd = Command::new(compiler);
    cmd.arg("-c");

    if let Some(flag) = asm_lang_flag(source) {
        cmd.arg("-x").arg(flag);
    }

    cmd.arg(source).arg("-o").arg(obj_path);

    if ctx.kind == "sharedlib" {
        cmd.arg("-fPIC");
    }

    if (ctx.freestanding || is_bare_metal_target(ctx.target))
        && ctx.language.to_lowercase() != "asm"
    {
        cmd.arg("-ffreestanding");
    }

    if ctx.panic_abort && ctx.language.to_lowercase() != "asm" {
        if ctx.language.contains("++")
            || ctx.language.contains("cpp")
            || ctx.language.contains("cxx")
        {
            cmd.arg("-fno-exceptions");
        }
        cmd.arg("-fno-unwind-tables");
        cmd.arg("-fno-asynchronous-unwind-tables");
    }

    if let Some(platform) = ctx.platform
        && !platform.trim().is_empty()
    {
        cmd.arg(format!("-march={}", platform));
    }

    if !ctx.standard.is_empty() && ctx.language.to_lowercase() != "asm" {
        let ext = Path::new(source)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let is_cpp = matches!(ext, "cpp" | "cxx" | "cc");
        if is_cpp && !ctx.cxx_standard.is_empty() {
            cmd.arg(format!("-std={}", ctx.cxx_standard));
        } else if !is_cpp {
            cmd.arg(format!("-std={}", ctx.standard));
        }
    }

    let use_dcr_defaults =
        ctx.cflags.is_empty() && !(ctx.freestanding || is_bare_metal_target(ctx.target));
    if use_dcr_defaults {
        for flag in default_flags(ctx.profile) {
            cmd.arg(flag);
        }
    }

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    for dir in ctx.include_dirs {
        cmd.arg(format!("-I{dir}"));
    }

    if let Some(path) = qt_include_dir {
        cmd.arg(format!("-I{}", path.display()));
    }

    let d_path = Path::new(obj_path).with_extension("d");
    cmd.arg("-MMD").arg("-MF").arg(&d_path);

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}

fn asm_lang_flag(source: &str) -> Option<&'static str> {
    crate::core::build::common::asm_lang_flag(source)
}
