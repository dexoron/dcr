use crate::core::build::builder::BuildContext;
use crate::core::build::builder::artifact;
use crate::core::build::common;
use std::path::Path;
use std::time::Instant;

pub fn filter_asm_flags(flags: &[String]) -> Vec<String> {
    flags
        .iter()
        .filter(|f| {
            let f = f.as_str();
            f.starts_with("-I") || f.starts_with("-i") || f == "-g"
        })
        .cloned()
        .collect()
}

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
        return artifact::archive_static(ctx, &objects, start_time);
    }

    artifact::link_binary(ctx, &objects, "cc", start_time)
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
