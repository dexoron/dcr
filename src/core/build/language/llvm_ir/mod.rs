use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use crate::core::build::language::Language;
use crate::core::build::language::asm::common as asm;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct LlvmIr;

impl Language for LlvmIr {
    fn id(&self) -> &'static str {
        "llvm_ir"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["ll"]
    }
    fn matches_token(&self, token: &str) -> bool {
        matches!(token.to_lowercase().as_str(), "llvm_ir" | "llvm-ir" | "ll")
    }
}

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    asm::build_assembly(ctx, "LLVM-IR", "llc", &["ll"], build_object)
}

pub(crate) fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    common::collect_sources(
        ctx.source_roots,
        &["ll"],
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
    cmd.arg("-filetype=obj").arg(source).arg("-o").arg(obj_path);

    for flag in ctx.cflags {
        cmd.arg(flag);
    }

    common::run_command_sync_output(&mut cmd)
}
