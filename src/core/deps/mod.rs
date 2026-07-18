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
pub mod lock;
pub mod register;

use crate::core::build_config::Config;
use crate::core::deps::common::ResolvedDeps;
use crate::core::deps::lock::{DepLock, write_lock};
use std::path::Path;

fn dep_version(path: &Path) -> String {
    Config::open(&path.join("dcr.toml").to_string_lossy())
        .ok()
        .and_then(|c| {
            c.get("package.version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

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
    let mut lock_packages: Vec<DepLock> = Vec::new();

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

                lock_packages.push(DepLock {
                    name: name.clone(),
                    version: pkg_info
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    checksum: String::new(),
                    source: format!(
                        "registry+{}",
                        pkg_info
                            .get("registry_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("https://dcr-registry.pages.dev")
                    ),
                });
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
                        push_if_exists(
                            &mut resolved.include_dirs,
                            &dep_root.join("target").join("include"),
                        );
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
                    push_if_exists(
                        &mut resolved.include_dirs,
                        &dep_root.join("target").join("include"),
                    );
                    push_default_lib_dirs(&mut resolved.lib_dirs, &dep_root);
                    resolved.libs.push(name.clone());
                }

                lock_packages.push(DepLock {
                    name: name.clone(),
                    version: dep_version(&dep_root),
                    checksum: String::new(),
                    source: format!("path+{}", dep_root.display()),
                });
            } else if let Some(git_info) = git_dep(value) {
                lock_packages.push(DepLock {
                    name: name.clone(),
                    version: git_info.version.unwrap_or_default(),
                    checksum: String::new(),
                    source: format!("git+{}", git_info.url),
                });
            }
        }
    }

    write_lock(
        project_root,
        &project_name,
        &project_version,
        &lock_packages,
    )?;

    Ok(resolved)
}

struct GitDep<'a> {
    url: &'a str,
    version: Option<String>,
}

fn git_dep(value: &toml::Value) -> Option<GitDep<'_>> {
    if let Some(table) = value.as_table()
        && let Some(url) = table.get("git").and_then(|v| v.as_str())
    {
        let version = table
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        return Some(GitDep { url, version });
    }
    None
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
