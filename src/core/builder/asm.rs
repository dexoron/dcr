use crate::core::builder::BuildContext;
use crate::core::builder::common;
use crate::platform;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build_assembly<F>(
    ctx: &BuildContext,
    backend_name: &str,
    default_assembler: &str,
    extensions: &[&str],
    build_object: F,
) -> Result<f64, String>
where
    F: Fn(&str, &str, &str, &BuildContext) -> Result<(), String> + Sync,
{
    let lang = ctx.language.to_lowercase();
    if lang.split(',').any(|p| p.trim() != "asm") {
        return Err(format!(
            "{backend_name} backend requires build.language = \"asm\""
        ));
    }

    let assembler = if ctx.compiler.is_empty() {
        default_assembler
    } else {
        ctx.compiler
    };

    let start_time = Instant::now();
    let sources = common::collect_sources(
        ctx.source_roots,
        extensions,
        ctx.exclude_dirs,
        ctx.include_paths,
    )?;
    let obj_dir = match ctx.target_dir {
        Some(dir) => Path::new(dir).join("obj"),
        None => Path::new("./target").join(ctx.profile).join("obj"),
    };
    let objects = build_objects_common(assembler, &sources, &obj_dir, ctx, &build_object)?;

    if ctx.kind == "staticlib" {
        return archive_staticlib(ctx, &objects, start_time);
    }

    link_objects(ctx, &objects, start_time)
}

fn build_objects_common<F>(
    assembler: &str,
    sources: &[String],
    obj_dir: &Path,
    ctx: &BuildContext,
    build_object: &F,
) -> Result<Vec<String>, String>
where
    F: Fn(&str, &str, &str, &BuildContext) -> Result<(), String> + Sync,
{
    let objects: Vec<String> = sources
        .iter()
        .map(|s| common::object_path(obj_dir, s, "o"))
        .collect();

    common::parallel_build(
        sources.len(),
        |i| build_object(assembler, &sources[i], &objects[i], ctx),
        ctx.codegen_units,
    )?;

    Ok(objects)
}

fn archive_staticlib(
    ctx: &BuildContext,
    objects: &[String],
    start_time: Instant,
) -> Result<f64, String> {
    let lib_path = platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
    if !common::needs_link(objects, &lib_path) {
        return Ok(common::elapsed_secs(start_time));
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
    for obj in objects {
        cmd.arg(obj);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    match cmd.status() {
        Ok(status) if status.success() => Ok(common::elapsed_secs(start_time)),
        Ok(_) => Err("Build failed".to_string()),
        Err(err) => Err(format!("Build failed: {err}")),
    }
}

fn link_objects(
    ctx: &BuildContext,
    objects: &[String],
    start_time: Instant,
) -> Result<f64, String> {
    let mut cmd = Command::new(ctx.linker.unwrap_or("cc"));
    if ctx.kind == "sharedlib" {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
    }

    for obj in objects {
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

    if !common::needs_link(objects, &out_path) {
        return Ok(common::elapsed_secs(start_time));
    }

    cmd.arg("-o").arg(out_path);

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    match cmd.status() {
        Ok(status) if status.success() => Ok(common::elapsed_secs(start_time)),
        Ok(_) => Err("Build failed".to_string()),
        Err(err) => Err(format!("Build failed: {err}")),
    }
}
