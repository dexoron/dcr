use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use crate::core::build::language::Language;
use crate::core::build::language::asm::common as asm;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents the LLVM IR language implementation for the build system.
pub struct LlvmIr;

impl Language for LlvmIr {
    /// Returns the unique identifier for the LLVM IR language.
    fn id(&self) -> &'static str {
        "llvm_ir"
    }

    /// Returns the list of file extensions recognized for this language.
    fn extensions(&self) -> &'static [&'static str] {
        &["ll"]
    }

    /// Determines whether the provided token matches the LLVM IR language.
    fn matches_token(&self, token: &str) -> bool {
        matches!(token.to_lowercase().as_str(), "llvm_ir" | "llvm-ir" | "ll")
    }
}

/// Builds LLVM IR source files into object files using llc.
pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly_via_objcopy(ctx, "LLVM-IR", "llc", &["ll"], build_object)
}

/// Collects source files for LLVM IR compilation from the given context.
pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["ll"],
        ctx.exclude_dirs,
        ctx.include_paths,
    )
}

/// Builds a single LLVM IR source file into an object file.
fn build_object(
    assembler: &str,
    source: &str,
    obj_path: &str,
    ctx: &BuildContext,
) -> Result<(), String> {
    if let Some(parent) = Path::new(obj_path).parent() {
        // Create parent directory if it does not exist
        fs::create_dir_all(parent).map_err(|err| format!("obj dir error: {err}"))?;
    }

    if !common::needs_rebuild(source, obj_path) {
        return Ok(());
    }

    let mut cmd = Command::new(assembler);
    cmd.arg("-filetype=obj").arg(source).arg("-o").arg(obj_path);

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    common::run_command_sync_output(&mut cmd)
}
