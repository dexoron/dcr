/// Represents resolved dependencies for a build target, including include directories, library directories, and libraries.
#[derive(Debug, Clone, Default)]
pub struct ResolvedDeps {
    pub include_dirs: Vec<String>,
    pub lib_dirs: Vec<String>,
    pub libs: Vec<String>,
    pub package_roots: Vec<String>,
}
