use crate::core::builder::BuildContext;
use crate::platform;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = if ctx.compiler.is_empty() {
        "cc"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let sources = collect_sources(ctx.language)?;
    let obj_dir = Path::new("./target").join(ctx.profile).join("obj");
    let objects = build_objects(compiler, &sources, &obj_dir, ctx, "o")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
        let mut cmd = Command::new("ar");
        cmd.arg("rcs").arg(&lib_path);
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
    for flag in default_flags(ctx.profile) {
        cmd.arg(flag);
    }
    for flag in ctx.cflags {
        cmd.arg(flag);
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
    let out_path = if ctx.kind == "sharedlib" {
        platform::shared_lib_path(ctx.profile, ctx.project_name, ctx.target_dir)
    } else {
        platform::bin_path(ctx.profile, ctx.project_name, ctx.target_dir)
    };
    cmd.arg("-o").arg(out_path);

    match cmd.status() {
        Ok(status) if status.success() => {
            let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
            Ok(elapsed)
        }
        Ok(_) => Err("Build failed".to_string()),
        Err(err) => Err(format!("Build failed: {err}")),
    }
}

fn collect_sources(language: &str) -> Result<Vec<String>, String> {
    let lang = language.to_lowercase();
    let mut sources = Vec::new();
    collect_sources_rec("./src", &lang, &mut sources)?;
    sources.sort();
    if sources.is_empty() {
        return Err("No source files found in ./src".to_string());
    }
    Ok(sources)
}

fn collect_sources_rec(dir: &str, lang: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| format!("src dir error: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("src dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_sources_rec(&path.to_string_lossy(), lang, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file = path.to_string_lossy().to_string();
        let allowed = (lang == "c" && ext == "c")
            || ((lang == "c++" || lang == "cpp" || lang == "cxx")
                && (ext == "cpp" || ext == "cxx" || ext == "cc"))
            || (lang == "asm" && (ext == "s" || ext == "asm"));
        if allowed {
            out.push(file);
        }
    }
    Ok(())
}

fn default_flags(profile: &str) -> &'static [&'static str] {
    match profile {
        "release" => &["-O3", "-DNDEBUG", "-march=native"],
        "debug" => &[
            "-O0",
            "-g",
            "-Wall",
            "-Wextra",
            "-fno-omit-frame-pointer",
            "-DDEBUG",
        ],
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
        let obj_path = object_path(obj_dir, source, obj_ext);
        if let Some(parent) = Path::new(&obj_path).parent() {
            fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
        }
        if needs_rebuild(source, &obj_path) {
            let mut cmd = Command::new(compiler);
            cmd.arg("-c").arg(source).arg("-o").arg(&obj_path);
            if ctx.kind == "sharedlib" {
                cmd.arg("-fPIC");
            }
            if let Some(platform) = ctx.platform
                && !platform.trim().is_empty()
            {
                cmd.arg(format!("-march={}", platform));
            }
            if !ctx.standard.is_empty() && ctx.language.to_lowercase() != "asm" {
                cmd.arg(format!("-std={}", ctx.standard));
            }
            for flag in default_flags(ctx.profile) {
                cmd.arg(flag);
            }
            for flag in ctx.cflags {
                cmd.arg(flag);
            }
            for dir in ctx.include_dirs {
                cmd.arg(format!("-I{dir}"));
            }
            match cmd.status() {
                Ok(status) if status.success() => {}
                Ok(_) => return Err("Build failed".to_string()),
                Err(err) => return Err(format!("Build failed: {err}")),
            }
        }
        objects.push(obj_path);
    }
    Ok(objects)
}

fn object_path(obj_dir: &Path, source: &str, obj_ext: &str) -> String {
    let src_path = Path::new(source);
    let rel = src_path
        .strip_prefix("./src")
        .or_else(|_| src_path.strip_prefix("src"))
        .unwrap_or(src_path);
    let mut out = obj_dir.join(rel);
    out.set_extension(obj_ext.trim_start_matches('.'));
    out.to_string_lossy().to_string()
}

fn needs_rebuild(source: &str, object: &str) -> bool {
    let src_time = fs::metadata(source).and_then(|m| m.modified());
    let obj_time = fs::metadata(object).and_then(|m| m.modified());
    match (src_time, obj_time) {
        (Ok(s), Ok(o)) => s > o,
        (Ok(_), Err(_)) => true,
        _ => true,
    }
}
