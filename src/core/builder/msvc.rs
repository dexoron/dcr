use crate::core::builder::BuildContext;
use crate::platform;
use std::fs;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = if ctx.compiler.is_empty() {
        "cl"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let mut cmd = Command::new(compiler);
    let sources = collect_sources(ctx.language)?;

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

    if !ctx.standard.is_empty() {
        let std_flag = msvc_standard_flag(ctx.language, ctx.standard)?;
        cmd.arg(std_flag);
    }

    for flag in default_flags(ctx.profile) {
        cmd.arg(flag);
    }

    for source in &sources {
        cmd.arg(source);
    }
    cmd.arg(format!(
        "/Fe:{}",
        platform::bin_path(ctx.profile, ctx.project_name)
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

fn default_flags(profile: &str) -> &'static [&'static str] {
    match profile {
        "release" => &["/O2", "/DNDEBUG"],
        "debug" => &["/Od", "/Zi", "/W4", "/DDEBUG", "/Oy-"],
        _ => &[],
    }
}
