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
    cmd.arg("-o")
        .arg(platform::bin_path(ctx.profile, ctx.project_name));

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
    let entries = fs::read_dir("./src").map_err(|err| format!("src dir error: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("src dir error: {err}"))?;
        let path = entry.path();
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
            sources.push(file);
        }
    }
    sources.sort();
    if sources.is_empty() {
        return Err("No source files found in ./src".to_string());
    }
    Ok(sources)
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
