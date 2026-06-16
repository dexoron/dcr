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

pub mod common;
pub mod gas;
pub mod msvc;
pub mod nasm;
pub mod unix_cc;

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
}

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = ctx.compiler.to_lowercase();
    if !check_compiler_exists(ctx.compiler) {
        return Err(format!(
            "Compiler not found: {}. Make sure it is installed and available in PATH.",
            ctx.compiler
        ));
    }
    if compiler.contains("clang-cl") {
        return msvc::build(ctx);
    }
    if compiler == "as" || compiler.contains("gas") {
        return gas::build(ctx);
    }
    if compiler.contains("nasm") {
        return nasm::build(ctx);
    }
    if compiler == "cl" || compiler.contains("msvc") {
        return msvc::build(ctx);
    }
    unix_cc::build(ctx)
}

fn check_compiler_exists(compiler: &str) -> bool {
    let name = if compiler.is_empty() { "cc" } else { compiler };
    std::process::Command::new(name)
        .arg("--version")
        .output()
        .is_ok()
}

pub fn collect_sources(ctx: &BuildContext) -> Result<Vec<String>, String> {
    let compiler = ctx.compiler.to_lowercase();
    if compiler.contains("clang-cl") || compiler == "cl" || compiler.contains("msvc") {
        return msvc::collect_sources(ctx);
    }
    if compiler == "as" || compiler.contains("gas") {
        return gas::collect_sources(ctx);
    }
    if compiler.contains("nasm") {
        return nasm::collect_sources(ctx);
    }
    unix_cc::collect_sources(ctx)
}
