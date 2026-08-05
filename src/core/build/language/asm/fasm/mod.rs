use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use crate::core::build::language::asm::common as asm;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Builds an assembly file using the FASM assembler.
///
/// Delegates to the shared assembly build logic with FASM configuration.
pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly(ctx, "FASM", "fasm", &["asm", "fasm"], build_object)
}

/// Collects all source files for assembly files using FASM.
pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["asm", "fasm"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

/// Assembles one FASM source into an object file.
fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    // Skip rebuilding if the object is up to date
    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    let mut cmd = Command::new(assembler);
    cmd.arg(source).arg(obj_path);

    for flag in crate::core::build::language::asm::common::filter_asm_flags(ctx.cflags) {
        cmd.arg(flag);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}
