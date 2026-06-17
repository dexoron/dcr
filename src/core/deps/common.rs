#[derive(Debug, Clone, Default)]
pub struct ResolvedDeps {
    pub include_dirs: Vec<String>,
    pub lib_dirs: Vec<String>,
    pub libs: Vec<String>,
}
