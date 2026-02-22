pub mod clang;
pub mod gcc;
pub mod msvc;

pub struct BuildContext<'a> {
    pub profile: &'a str,
    pub project_name: &'a str,
    pub compiler: &'a str,
    pub language: &'a str,
    pub standard: &'a str,
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
    if compiler.contains("gcc") || compiler.contains("g++") {
        return gcc::build(ctx);
    }
    if compiler.contains("clang") || compiler.contains("clang++") {
        return clang::build(ctx);
    }
    if compiler == "cl" || compiler.contains("msvc") {
        return msvc::build(ctx);
    }
    clang::build(ctx)
}
