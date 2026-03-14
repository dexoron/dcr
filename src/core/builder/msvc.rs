use crate::core::builder::BuildContext;
use crate::core::builder::common;
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
    if ctx.language.to_lowercase() == "asm" {
        return Err("MSVC backend does not support build.language = \"asm\"".to_string());
    }
    let start_time = Instant::now();
    let sources = collect_sources(ctx)?;
    let obj_dir = Path::new("./target").join(ctx.profile).join("obj");
    let objects = build_objects(compiler, &sources, &obj_dir, ctx, "obj")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
        let mut cmd = Command::new(ctx.archiver.unwrap_or("lib"));
        cmd.arg("/nologo").arg(format!("/OUT:{lib_path}"));
        for obj in &objects {
            cmd.arg(obj);
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
    for flag in default_flags(ctx.profile) {
        cmd.arg(flag);
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
    } else {
        platform::bin_path(ctx.profile, ctx.project_name, ctx.target_dir)
    };
    cmd.arg(format!("/Fe:{out_path}"));

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
    let extensions = source_extensions(ctx.language);
    common::collect_sources(&extensions, ctx.exclude_dirs)
}

fn source_extensions(language: &str) -> Vec<&str> {
    let lang = language.to_lowercase();
    match lang.as_str() {
        "c" => vec!["c"],
        "c++" | "cpp" | "cxx" => vec!["cpp", "cxx", "cc"],
        _ => vec!["c"],
    }
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
        "debug" => &["/Od", "/Zi", "/W4", "/DDEBUG", "/Oy-"],
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
    let mut objects = Vec::new();
    for source in sources {
        let obj_path = common::object_path(obj_dir, source, obj_ext);
        if let Some(parent) = Path::new(&obj_path).parent() {
            fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
        }
        if common::needs_rebuild(source, &obj_path) {
            let mut cmd = Command::new(compiler);
            cmd.arg("/nologo");
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
            if !ctx.standard.is_empty() && ctx.language.to_lowercase() != "asm" {
                let std_flag = msvc_standard_flag(ctx.language, ctx.standard)?;
                cmd.arg(std_flag);
            }
            if let Some(flag) = msvc_arch_flag(ctx.platform) {
                cmd.arg(flag);
            }
            for flag in default_flags(ctx.profile) {
                cmd.arg(flag);
            }
            for flag in ctx.cflags {
                cmd.arg(flag);
            }
            for dir in ctx.include_dirs {
                cmd.arg(format!("/I{dir}"));
            }
            cmd.arg("/c").arg(source).arg(format!("/Fo:{}", obj_path));
            cmd.arg("/showIncludes");

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

            if !output.status.success() {
                eprint!("{}", clean_stdout);
                eprint!("{}", stderr);
                return Err("Build failed".to_string());
            } else {
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
            }

            let d_path = Path::new(&obj_path).with_extension("d");
            let mut d_content = format!("{}: \\\n", obj_path.replace('\\', "/"));
            for h in headers {
                let escaped = h.replace('\\', "/").replace(" ", "\\ ");
                d_content.push_str(&format!("  {} \\\n", escaped));
            }
            std::fs::write(&d_path, d_content).map_err(|err| format!("d file error: {err}"))?;
        }
        objects.push(obj_path);
    }
    Ok(objects)
}
