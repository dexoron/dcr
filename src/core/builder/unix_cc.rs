use crate::core::builder::BuildContext;
use crate::core::builder::common;
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
    let extensions = source_extensions(ctx.language);
    let sources = common::collect_sources(&extensions, ctx.exclude_dirs)?;
    let obj_dir = Path::new("./target").join(ctx.profile).join("obj");
    let objects = build_objects(compiler, &sources, &obj_dir, ctx, "o")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
        let mut cmd = Command::new(ctx.archiver.unwrap_or("ar"));
        cmd.arg("rcs").arg(&lib_path);
        for obj in &objects {
            cmd.arg(obj);
        }
        if std::env::var("DCR_DEBUG").is_ok() {
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

    let mut cmd = Command::new(ctx.linker.unwrap_or(compiler));
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
    let out_path = if ctx.kind == "sharedlib" {
        platform::shared_lib_path(ctx.profile, ctx.project_name, ctx.target_dir)
    } else {
        platform::bin_path(ctx.profile, ctx.project_name, ctx.target_dir)
    };
    cmd.arg("-o").arg(out_path);

    if std::env::var("DCR_DEBUG").is_ok() {
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
    let extensions = source_extensions(ctx.language);
    common::collect_sources(&extensions, ctx.exclude_dirs)
}

fn source_extensions(language: &str) -> Vec<&str> {
    let lang = language.to_lowercase();
    match lang.as_str() {
        "c" => vec!["c"],
        "c++" | "cpp" | "cxx" => vec!["cpp", "cxx", "cc"],
        "asm" => vec!["s", "S", "asm"],
        _ => vec!["c"],
    }
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
            "-DDCR_DEBUG",
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
        let obj_path = common::object_path(obj_dir, source, obj_ext);
        if let Some(parent) = Path::new(&obj_path).parent() {
            fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
        }
        if common::needs_rebuild(source, &obj_path) {
            let mut cmd = Command::new(compiler);
            cmd.arg("-c").arg(source).arg("-o").arg(&obj_path);
            if ctx.kind == "sharedlib" {
                cmd.arg("-fPIC");
            }
            if let Some(flag) = asm_lang_flag(source) {
                cmd.arg("-x").arg(flag);
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
            let d_path = Path::new(&obj_path).with_extension("d");
            cmd.arg("-MMD").arg("-MF").arg(&d_path);
            if std::env::var("DCR_DEBUG").is_ok() {
                eprintln!("[dcr] {:?}", cmd);
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

fn asm_lang_flag(source: &str) -> Option<&'static str> {
    let ext = Path::new(source).extension().and_then(|v| v.to_str())?;
    match ext {
        "S" => Some("assembler-with-cpp"),
        "s" | "asm" => Some("assembler"),
        _ => None,
    }
}
