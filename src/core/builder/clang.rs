use crate::core::builder::BuildContext;
use crate::platform;
use std::fs;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = if ctx.compiler.is_empty() {
        "cc"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let mut cmd = Command::new(compiler);
    let sources = collect_sources(ctx.language)?;
    for source in &sources {
        cmd.arg(source);
    }
    if !ctx.standard.is_empty() {
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
    for dir in ctx.lib_dirs {
        cmd.arg(format!("-L{dir}"));
    }
    for lib in ctx.libs {
        cmd.arg(format!("-l{lib}"));
    }
    for flag in ctx.ldflags {
        cmd.arg(flag);
    }
    cmd.arg("-o").arg(platform::bin_path(
        ctx.profile,
        ctx.project_name,
        ctx.target_dir,
    ));

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
                && (ext == "cpp" || ext == "cxx" || ext == "cc"));
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
