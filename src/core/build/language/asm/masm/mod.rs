use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use crate::core::build::language::asm::common as asm;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Builds MASM (`.asm`) sources via `ml`.
///
/// For `flat-bin`, intermediates are converted with objcopy after assembly.
/// Returns elapsed seconds or an error.
pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly_via_objcopy(ctx, "MASM", "ml", &["asm"], build_object)
}

/// Collects MASM assembly source files from the given source roots.
pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["asm"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

/// Builds a single MASM object file from the given source.
///
/// Creates the parent directory for the output object if it doesn't exist,
/// skips the build if the object is already up to date, constructs the ml
/// command line with flags, prints the command in debug mode if enabled,
/// and executes it.
fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        // Ensure the parent directory exists for the object file
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    if !common::needs_rebuild(source, obj_path) {
        // Skip rebuild if the object file is already current
        return Ok(());
    }

    let mut cmd = Command::new(assembler);
    cmd.arg("/nologo")
        .arg("/c")
        .arg("/Fo")
        .arg(obj_path)
        .arg(source);

    for flag in crate::core::build::language::asm::common::filter_asm_flags(ctx.cflags) {
        // Add filtered compiler flags to the command
        cmd.arg(flag);
    }

    if ctx.verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }

    common::run_command_sync_output(&mut cmd)
}
