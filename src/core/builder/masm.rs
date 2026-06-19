use crate::core::builder::BuildContext;
use crate::core::builder::asm;
use crate::core::builder::common;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly(ctx, "MASM", "ml", &["asm"], build_object)
}

pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["asm"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    let mut cmd = Command::new(assembler);
    cmd.arg("/nologo")
        .arg("/c")
        .arg("/Fo")
        .arg(obj_path)
        .arg(source);

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}
