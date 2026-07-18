// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod artifact;
pub mod cc_common;
pub mod msvc;

use crate::core::build::language::asm::{fasm, gas, masm, nasm};
use crate::core::build::language::llvm_ir;

pub struct BuildContext<'a> {
    pub profile: &'a str,
    pub project_name: &'a str,
    pub compiler: &'a str,
    pub language: &'a str,
    pub standard: &'a str,
    pub cxx_standard: &'a str,
    pub target: Option<&'a str>,
    pub target_dir: Option<&'a str>,
    pub kind: &'a str,
    pub platform: Option<&'a str>,
    pub linker: Option<&'a str>,
    pub archiver: Option<&'a str>,
    pub moc: Option<&'a str>,
    pub uic: Option<&'a str>,
    pub rcc: Option<&'a str>,
    pub package_type: Option<&'a str>,
    pub freestanding: bool,
    pub panic_abort: bool,
    pub codegen_units: usize,
    pub source_roots: &'a [std::path::PathBuf],
    pub exclude_dirs: &'a [std::path::PathBuf],
    pub include_paths: &'a [String],
    pub include_dirs: &'a [String],
    pub lib_dirs: &'a [String],
    pub libs: &'a [String],
    pub cflags: &'a [String],
    pub ldflags: &'a [String],
    pub output_filename: Option<&'a str>,
    pub output_extension: Option<&'a str>,
    pub verbose: bool,
    pub qt: bool,
}

pub trait Builder {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn build(&self, ctx: &BuildContext) -> Result<f64, String>;
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String>;
}

struct Cc;
struct Msvc;
struct Nasm;
struct Gas;
struct Masm;
struct Fasm;
struct Llc;

impl Builder for Cc {
    fn id(&self) -> &'static str {
        "cc"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        cc_common::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        cc_common::collect_sources(ctx)
    }
}

impl Builder for Msvc {
    fn id(&self) -> &'static str {
        "msvc"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        msvc::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        msvc::collect_sources(ctx)
    }
}

impl Builder for Nasm {
    fn id(&self) -> &'static str {
        "nasm"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        nasm::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        nasm::collect_sources(ctx)
    }
}

impl Builder for Gas {
    fn id(&self) -> &'static str {
        "gas"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        gas::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        gas::collect_sources(ctx)
    }
}

impl Builder for Masm {
    fn id(&self) -> &'static str {
        "masm"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        masm::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        masm::collect_sources(ctx)
    }
}

impl Builder for Fasm {
    fn id(&self) -> &'static str {
        "fasm"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        fasm::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        fasm::collect_sources(ctx)
    }
}

impl Builder for Llc {
    fn id(&self) -> &'static str {
        "llvm_ir"
    }
    fn build(&self, ctx: &BuildContext) -> Result<f64, String> {
        llvm_ir::build(ctx)
    }
    fn collect_sources(&self, ctx: &BuildContext) -> Result<Vec<String>, String> {
        llvm_ir::collect_sources(ctx)
    }
}

pub fn builder_for(compiler: &str) -> &'static dyn Builder {
    let c = compiler.to_lowercase();
    if c.contains("clang-cl") {
        return &Msvc;
    }
    if c == "as" || c.contains("gas") {
        return &Gas;
    }
    if c.contains("llc") {
        return &Llc;
    }
    if c.contains("fasm") {
        return &Fasm;
    }
    if c.contains("nasm") {
        return &Nasm;
    }
    if c == "ml" || c == "ml64" || c.contains("masm") {
        return &Masm;
    }
    if c == "cl" || c.contains("msvc") {
        return &Msvc;
    }
    &Cc
}

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    if !check_compiler_exists(ctx.compiler) {
        return Err(format!(
            "Compiler not found: {}. Make sure it is installed and available in PATH.",
            ctx.compiler
        ));
    }
    builder_for(ctx.compiler).build(ctx)
}

fn check_compiler_exists(compiler: &str) -> bool {
    let name = if compiler.is_empty() {
        "cc"
    } else {
        match compiler.to_lowercase().as_str() {
            "gas" | "gnu-as" => "as",
            "nasm" => "nasm",
            "fasm" | "fasm64" => "fasm",
            "masm" | "ml" | "ml64" => "ml",
            _ => compiler,
        }
    };
    std::process::Command::new(name)
        .arg("--version")
        .output()
        .is_ok()
}

pub fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    builder_for(ctx.compiler).collect_sources(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_resolves_by_compiler_name() {
        assert_eq!(builder_for("gcc").id(), "cc");
        assert_eq!(builder_for("clang").id(), "cc");
        assert_eq!(builder_for("clang++").id(), "cc");
        assert_eq!(builder_for("").id(), "cc");
        assert_eq!(builder_for("nasm").id(), "nasm");
        assert_eq!(builder_for("as").id(), "gas");
        assert_eq!(builder_for("llc-18").id(), "llvm_ir");
        assert_eq!(builder_for("fasm").id(), "fasm");
        assert_eq!(builder_for("ml64").id(), "masm");
        assert_eq!(builder_for("cl").id(), "msvc");
        assert_eq!(builder_for("clang-cl").id(), "msvc");
    }
}
