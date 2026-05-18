pub mod common;
pub mod git;
pub mod lock;
pub mod register;

use crate::core::config::Config;
use crate::core::deps::common::ResolvedDeps;
use std::path::Path;

pub fn resolve_deps(
    config: &Config,
    _profile: &str,
    _target: Option<&str>,
    project_root: &Path,
) -> Result<ResolvedDeps, String> {
    let mut resolved = ResolvedDeps::default();
    let project_name = config
        .get("package.name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let project_version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let deps_table = config.get("dependencies").and_then(|v| v.as_table());
    let lock_packages = Vec::new();

    if let Some(deps) = deps_table {
        for (name, value) in deps {
            if register::is_registry_dep(value) {
                let pkg_info = register::resolve_package_from_registry(name)?;
                let dep_root = register::package_root_from_registry_info(&pkg_info)?;

                resolved.include_dirs.push(
                    register::registry_include_dir(&dep_root)
                        .to_string_lossy()
                        .to_string(),
                );
                resolved.lib_dirs.push(
                    register::registry_lib_dir(&dep_root)
                        .to_string_lossy()
                        .to_string(),
                );
                resolved.libs.push(name.clone());
            } else if let Some(path) = path_dep_path(value) {
                let dep_root = project_root.join(path);
                if let Some(table) = value.as_table() {
                    if let Some(includes) = table.get("include").and_then(|v| v.as_array()) {
                        for inc in includes {
                            if let Some(inc_str) = inc.as_str() {
                                resolved
                                    .include_dirs
                                    .push(dep_root.join(inc_str).to_string_lossy().to_string());
                            }
                        }
                    } else {
                        push_if_exists(&mut resolved.include_dirs, &dep_root.join("include"));
                    }

                    if let Some(lib_dirs) = table.get("lib").and_then(|v| v.as_array()) {
                        for lib_dir in lib_dirs {
                            if let Some(lib_dir_str) = lib_dir.as_str() {
                                resolved
                                    .lib_dirs
                                    .push(dep_root.join(lib_dir_str).to_string_lossy().to_string());
                            }
                        }
                    } else {
                        push_default_lib_dirs(&mut resolved.lib_dirs, &dep_root);
                    }

                    if let Some(libs) = table.get("libs").and_then(|v| v.as_array()) {
                        for lib in libs {
                            if let Some(lib_str) = lib.as_str() {
                                resolved.libs.push(lib_str.to_string());
                            }
                        }
                    } else {
                        resolved.libs.push(name.clone());
                    }
                } else {
                    push_if_exists(&mut resolved.include_dirs, &dep_root.join("include"));
                    push_default_lib_dirs(&mut resolved.lib_dirs, &dep_root);
                    resolved.libs.push(name.clone());
                }
            }
        }
    }

    crate::core::deps::lock::write_lock(
        project_root,
        &project_name,
        &project_version,
        &lock_packages,
    )?;

    Ok(resolved)
}

fn path_dep_path(value: &toml::Value) -> Option<&str> {
    if let Some(table) = value.as_table() {
        return table.get("path").and_then(|v| v.as_str());
    }
    register::path_from_string_dep(value)
}

fn push_if_exists(paths: &mut Vec<String>, path: &Path) {
    if path.exists() {
        paths.push(path.to_string_lossy().to_string());
    }
}

fn push_default_lib_dirs(paths: &mut Vec<String>, dep_root: &Path) {
    for dir in ["lib", "lib64"] {
        push_if_exists(paths, &dep_root.join(dir));
    }
    push_if_exists(paths, &dep_root.join("target").join("lib"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn path_dep_path_supports_table_and_legacy_strings() {
        let table = Value::Table(
            [(
                "path".to_string(),
                Value::String("./libs/mylib".to_string()),
            )]
            .into_iter()
            .collect(),
        );
        assert_eq!(path_dep_path(&table), Some("./libs/mylib"));
        assert_eq!(
            path_dep_path(&Value::String("path:./libs/mylib".to_string())),
            Some("./libs/mylib")
        );
        assert_eq!(
            path_dep_path(&Value::String("./libs/mylib".to_string())),
            Some("./libs/mylib")
        );
        assert_eq!(path_dep_path(&Value::String("1.2.3".to_string())), None);
    }

    #[test]
    fn default_lib_dirs_include_packaged_library_output() {
        let root = std::env::temp_dir().join(format!(
            "dcr_default_lib_dirs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("target/lib")).unwrap();
        let mut paths = Vec::new();
        push_default_lib_dirs(&mut paths, &root);
        assert!(paths.iter().any(|p| p.ends_with("target/lib")));
        let _ = std::fs::remove_dir_all(root);
    }
}
