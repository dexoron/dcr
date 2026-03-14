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
    pub target_dir: Option<&'a str>,
    pub kind: &'a str,
    pub platform: Option<&'a str>,
    pub linker: Option<&'a str>,
    pub archiver: Option<&'a str>,
    pub exclude_dirs: &'a [std::path::PathBuf],
    pub include_dirs: &'a [String],
    pub lib_dirs: &'a [String],
    pub libs: &'a [String],
    pub cflags: &'a [String],
    pub ldflags: &'a [String],
}

pub fn build(ctx: &BuildContext) -> Result<f64, String> {
    let compiler = ctx.compiler.to_lowercase();
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
