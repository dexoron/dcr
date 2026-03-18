use crate::core::builder::BuildContext;
use crate::core::builder::common;
use crate::platform;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let lang = ctx.language.to_lowercase();
    if lang.split('+').any(|p| p.trim() != "asm") {
        return Err("GAS backend requires build.language = \"asm\"".to_string());
    }
    let assembler = if ctx.compiler.is_empty() {
        "as"
    } else {
        ctx.compiler
    };
    let start_time = Instant::now();
    let sources = collect_sources(ctx)?;
    let obj_dir = Path::new("./target").join(ctx.profile).join("obj");
    let objects = build_objects(assembler, &sources, &obj_dir, ctx, "o")?;

    if ctx.kind == "staticlib" {
        let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
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
    let mut objects = Vec::new();
    for source in sources {
        let obj_path = common::object_path(obj_dir, source, obj_ext);
        if let Some(parent) = Path::new(&obj_path).parent() {
            fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
        }
        if common::needs_rebuild(source, &obj_path) {
            let mut cmd = Command::new(assembler);
            cmd.arg(source).arg("-o").arg(&obj_path);
            for flag in ctx.cflags {
                cmd.arg(flag);
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
