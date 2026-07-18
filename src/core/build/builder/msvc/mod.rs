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
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = if ctx.compiler.is_empty() {
        "cl"
    } else {
        ctx.compiler
    };
    let lang = ctx.language.to_lowercase();
    if lang.contains("asm") {
        return Err("MSVC backend does not support build.language with asm".to_string());
    }
    let start_time = Instant::now();
    let sources = collect_sources(ctx)?;
    let obj_dir = match ctx.target_dir {
        Some(dir) => Path::new(dir).join("obj"),
        None => Path::new("./target").join(ctx.profile).join("obj"),
    };
    let objects = build_objects(compiler, &sources, &obj_dir, ctx, "obj")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
        if !common::needs_link(&objects, &lib_path) {
            let elapsed = common::elapsed_secs(start_time);
            return Ok(elapsed);
        }
        let mut cmd = Command::new(ctx.archiver.unwrap_or("lib"));
        cmd.arg("/nologo").arg(format!("/OUT:{lib_path}"));
        for obj in &objects {
            cmd.arg(obj);
        }
        if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
            eprintln!("[dcr] {:?}", cmd);
        }
        match cmd.status() {
            Ok(status) if status.success() => {
                let elapsed = common::elapsed_secs(start_time);
                return Ok(elapsed);
            }
            Ok(_) => return Err("Build failed".to_string()),
            Err(err) => return Err(format!("Build failed: {err}")),
        }
    }

    let mut cmd = Command::new(compiler);
    cmd.arg("/nologo");
    if ctx.kind == "sharedlib" {
        cmd.arg("/LD");
    }
    match ctx.language.to_lowercase().as_str() {
        "c" => {
            cmd.arg("/TC");
        }
        "c++" | "cpp" | "cxx" => {
            cmd.arg("/TP");
        }
        _ => {
            return Err("Unsupported language".to_string());
        }
    }

    if !ctx.standard.is_empty() {
        let std_flag = msvc_standard_flag(ctx.language, ctx.standard)?;
        cmd.arg(std_flag);
    }

    for obj in &objects {
        cmd.arg(obj);
    }
    if ctx.cflags.is_empty() {
        for flag in default_flags(ctx.profile) {
            cmd.arg(flag);
        }
    }
    for flag in ctx.cflags {
        cmd.arg(flag);
    }
    for dir in ctx.lib_dirs {
        cmd.arg(format!("/LIBPATH:{dir}"));
    }
    for lib in ctx.libs {
        if lib.to_lowercase().ends_with(".lib") {
            cmd.arg(lib);
        } else {
            cmd.arg(format!("{lib}.lib"));
        }
    }
    if !ctx.ldflags.is_empty() {
        cmd.arg("/link");
        for flag in ctx.ldflags {
            cmd.arg(flag);
        }
    }
    let out_path = if ctx.kind == "sharedlib" {
        platform::shared_lib_path(ctx.profile, ctx.project_name, ctx.target_dir)
    } else if ctx.kind == "elf" {
        platform::elf_path(ctx.profile, ctx.project_name, ctx.target_dir)
    } else {
        platform::bin_path(ctx.profile, ctx.project_name, ctx.target_dir)
    };

    if !common::needs_link(&objects, &out_path) {
        let elapsed = common::elapsed_secs(start_time);
        return Ok(elapsed);
    }
    cmd.arg("-o").arg(out_path);

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    match cmd.status() {
        Ok(status) if status.success() => {
            let elapsed = common::elapsed_secs(start_time);
            Ok(elapsed)
        }
        Ok(_) => Err("Build failed".to_string()),
        Err(err) => Err(format!("Build failed: {err}")),
    }
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

fn msvc_standard_flag(language: &str, standard: &str) -> Result<String, String> {
    let lang = language.to_lowercase();
    let std = standard.to_lowercase();
    if lang == "c" {
        return match std.as_str() {
            "c11" => Ok("/std:c11".to_string()),
            "c17" => Ok("/std:c17".to_string()),
            _ => Err("Unsupported C standard for MSVC".to_string()),
        };
    }
    if lang == "c++" || lang == "cpp" || lang == "cxx" {
        return match std.as_str() {
            "c++11" => Ok("/std:c++11".to_string()),
            "c++14" => Ok("/std:c++14".to_string()),
            "c++17" => Ok("/std:c++17".to_string()),
            "c++20" => Ok("/std:c++20".to_string()),
            "c++23" => Ok("/std:c++latest".to_string()),
            _ => Err("Unsupported C++ standard for MSVC".to_string()),
        };
    }
    Err("Unsupported language".to_string())
}

fn msvc_arch_flag(platform: Option<&str>) -> Option<&'static str> {
    let raw = platform?.trim();
    if raw.is_empty() {
        return None;
    }
    let p = raw.to_lowercase().replace('-', "_");
    if p == "x86" || (p.starts_with('i') && p.ends_with("86") && p.len() == 4) {
        return Some("/arch:IA32");
    }
    match p.as_str() {
        "sse2" => Some("/arch:SSE2"),
        "avx" => Some("/arch:AVX"),
        "avx2" => Some("/arch:AVX2"),
        _ => None,
    }
}

fn default_flags(profile: &str) -> &'static [&'static str] {
    match profile {
        "release" => &["/O2", "/DNDEBUG"],
        "debug" => &["/Od", "/Zi", "/W4", "/DDCR_DEBUG", "/Oy-"],
        _ => &[],
    }
}

fn build_objects(
    compiler: &str,
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
        |i| build_object(compiler, &sources[i], &objects[i], ctx),
        ctx.codegen_units,
    )?;

    Ok(objects)
}

fn build_object(
    compiler: &str,
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

    let mut cmd = Command::new(compiler);
    cmd.arg("/nologo");

    let ext = Path::new(source)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let is_cpp = matches!(ext, "cpp" | "cxx" | "cc");
    if is_cpp {
        cmd.arg("/TP");
    } else {
        cmd.arg("/TC");
    }

    let std_val = if is_cpp && !ctx.cxx_standard.is_empty() {
        ctx.cxx_standard
    } else if !is_cpp && !ctx.standard.is_empty() {
        ctx.standard
    } else {
        ""
    };
    if !std_val.is_empty() {
        let std_flag = msvc_standard_flag(if is_cpp { "c++" } else { "c" }, std_val)?;
        cmd.arg(std_flag);
    }

    if let Some(flag) = msvc_arch_flag(ctx.platform) {
        cmd.arg(flag);
    }

    if ctx.cflags.is_empty() {
        for flag in default_flags(ctx.profile) {
            cmd.arg(flag);
        }
    }

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    for dir in ctx.include_dirs {
        cmd.arg(format!("/I{dir}"));
    }

    cmd.arg("/c").arg(source).arg(format!("/Fo:{}", obj_path));
    cmd.arg("/showIncludes");

    if std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    let output = cmd.output().map_err(|err| format!("Build failed: {err}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut headers = Vec::new();
    let mut clean_stdout = String::new();

    for line in stdout.lines() {
        if let Some(stripped) = line.strip_prefix("Note: including file:") {
            headers.push(stripped.trim().to_string());
        } else if let Some(stripped) = line.strip_prefix("Примечание: включение файла:")
        {
            headers.push(stripped.trim().to_string());
        } else {
            clean_stdout.push_str(line);
            clean_stdout.push('\n');
        }
    }

    let _lock = common::get_output_lock().lock().unwrap();

    if !output.status.success() {
        eprint!("{}", clean_stdout);
        eprint!("{}", stderr);
        return Err("Build failed".to_string());
    }

    let trimmed_out = clean_stdout.trim();
    let trimmed_err = stderr.trim();
    let src_filename = Path::new(source)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("");

    if !trimmed_out.is_empty() && trimmed_out != src_filename {
        print!("{}", clean_stdout);
    }

    if !trimmed_err.is_empty() {
        eprintln!("{}", trimmed_err);
    }

    let d_path = Path::new(obj_path).with_extension("d");
    let mut d_content = format!("{}: \\\n", obj_path.replace('\\', "/"));
    for h in headers {
        let escaped = h.replace('\\', "/").replace(" ", "\\ ");
        d_content.push_str(&format!("  {} \\\n", escaped));
    }
    fs::write(&d_path, d_content).map_err(|err| format!("d file error: {err}"))?;

    Ok(())
}
