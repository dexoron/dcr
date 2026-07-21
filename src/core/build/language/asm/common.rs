use crate::core::build::builder::BuildContext;
use crate::core::build::builder::artifact;
use crate::core::build::common;
use crate::utils::build::{is_compile_only, is_flat_bin};
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
    build_assembly_with_mode(
        ctx,
        backend_name,
        default_assembler,
        extensions,
        FlatEmit::Direct,
        build_object,
    )
}

pub fn build_assembly_via_objcopy<F>(
    ctx: &BuildContext,
    backend_name: &str,
    default_assembler: &str,
    extensions: &[&str],
    build_object: F,
) -> Result<f64, String>
where
    F: Fn(&str, &str, &str, &BuildContext) -> Result<(), String> + Sync,
{
    build_assembly_with_mode(
        ctx,
        backend_name,
        default_assembler,
        extensions,
        FlatEmit::ViaObjcopy,
        build_object,
    )
}

#[derive(Clone, Copy)]
enum FlatEmit {
    Direct,
    ViaObjcopy,
}

fn build_assembly_with_mode<F>(
    ctx: &BuildContext,
    backend_name: &str,
    default_assembler: &str,
    extensions: &[&str],
    flat_mode: FlatEmit,
    build_object: F,
) -> Result<f64, String>
where
    F: Fn(&str, &str, &str, &BuildContext) -> Result<(), String> + Sync,
{
    let lang = ctx.language.to_lowercase();
    if lang.split(',').any(|p| {
        let t = p.trim();
        t != "asm" && t != "llvm_ir" && t != "llvm-ir" && t != "ll"
    }) {
        return Err(format!(
            "{backend_name} backend requires build.language = \"asm\" (or llvm_ir for LLC)"
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

    let flat = is_flat_bin(ctx.kind);
    let obj_dir = match ctx.target_dir {
        Some(dir) => Path::new(dir).join("obj"),
        None => Path::new("./target").join(ctx.profile).join("obj"),
    };

    let objects = match (flat, flat_mode) {
        (true, FlatEmit::Direct) => {
            let outs: Vec<String> = sources
                .iter()
                .map(|s| artifact::flat_source_output_path(ctx, s))
                .collect();
            common::parallel_build(
                sources.len(),
                |i| build_object(assembler, &sources[i], &outs[i], ctx),
                ctx.codegen_units,
            )?;
            outs
        }
        (true, FlatEmit::ViaObjcopy) => {
            let intermediates: Vec<String> = sources
                .iter()
                .map(|s| common::object_path(&obj_dir, s, "o"))
                .collect();
            common::parallel_build(
                sources.len(),
                |i| build_object(assembler, &sources[i], &intermediates[i], ctx),
                ctx.codegen_units,
            )?;
            let mut outs = Vec::with_capacity(sources.len());
            for (i, src) in sources.iter().enumerate() {
                let out = artifact::flat_source_output_path(ctx, src);
                artifact::objcopy_binary(ctx, &intermediates[i], &out)?;
                outs.push(out);
            }
            outs
        }
        (false, _) => build_objects_common(assembler, &sources, &obj_dir, ctx, &build_object)?,
    };

    if is_compile_only(ctx.kind) {
        return Ok(common::elapsed_secs(start_time));
    }

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
