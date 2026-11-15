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

/// Core dependency resolver for DCR projects.
pub mod common;
pub mod lock;
pub mod register;

use crate::core::build::builder::artifact::custom_artifact_file_name;
use crate::core::build::resolve::get_string_with_profile_and_target;
use crate::core::build_config::Config;
use crate::core::deps::common::ResolvedDeps;
use crate::core::deps::lock::{DepLock, write_lock};
use crate::utils::build::{resolve_pkg_config_flags, run_pkg_config};
use std::path::Path;

/// Returns the version string from a dependency's dcr.toml file, or an empty string if missing.
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

/// Resolves dependencies from config into include/lib dirs and lib names, and writes the lock file.
///
/// Registry, path, and git deps contribute to `ResolvedDeps`. Path and git sources support both
/// DCR packages (built from `dcr.toml`) and prebuilt libraries (described with include/lib/libs).
///
/// # Parameters
/// - `config`: Loaded `dcr.toml`.
/// - `profile` / `target`: Active build profile and target, used for per-profile/target
///   `filename` / `extension` overrides.
/// - `project_root`: Root used to resolve path deps and write `dcr.lock`.
///
/// # Returns
/// Aggregated include/lib/libs, or an error string on resolve/lock failure.
pub fn resolve_deps(
    config: &Config,
    profile: &str,
    target: Option<&str>,
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
        // Process each dependency declaration in the TOML table
        for (name, value) in deps {
            if let Some(packages) = pkg_config_dep(value, name)? {
                let (cflags, ldflags) = resolve_pkg_config_flags(&packages, &[], &[])?;
                resolved.cflags.extend(cflags);
                resolved.ldflags.extend(ldflags);
                let version = packages
                    .first()
                    .and_then(|pkg| run_pkg_config(pkg, "--modversion").ok())
                    .map(|v| v.trim().to_string())
                    .unwrap_or_default();
                lock_packages.push(DepLock {
                    name: name.clone(),
                    version,
                    checksum: String::new(),
                    source: format!("pkg-config+{}", packages.join(",")),
                });
            } else if register::is_registry_dep(value) {
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
                let fallback = package_name(&dep_root).unwrap_or_else(|| name.clone());
                resolved.libs.push(package_link_name(
                    &dep_root, &fallback, profile, target, None,
                ));
                resolved
                    .package_roots
                    .push(dep_root.to_string_lossy().to_string());

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
                let dcr_package_name = package_name(&dep_root);
                let use_dcr_package = use_dcr_package(value, dcr_package_name.is_some(), name)?;
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
                        push_package_output_dirs(&mut resolved, &dep_root);
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
                        let fallback = dcr_package_name.as_deref().unwrap_or(name);
                        resolved.libs.push(package_link_name(
                            &dep_root, fallback, profile, target, None,
                        ));
                    }
                } else {
                    push_if_exists(&mut resolved.include_dirs, &dep_root.join("include"));
                    push_package_output_dirs(&mut resolved, &dep_root);
                    push_default_lib_dirs(&mut resolved.lib_dirs, &dep_root);
                    let fallback = dcr_package_name.as_deref().unwrap_or(name);
                    resolved.libs.push(package_link_name(
                        &dep_root, fallback, profile, target, None,
                    ));
                }

                if use_dcr_package {
                    resolved
                        .package_roots
                        .push(dep_root.to_string_lossy().to_string());
                    push_package_output_dirs(&mut resolved, &dep_root);
                } else if !has_complete_prebuilt_layout(value) {
                    return Err(format!(
                        "Path dependency `{}` has no dcr.toml; specify include, lib, and libs for a prebuilt library",
                        name
                    ));
                }

                lock_packages.push(DepLock {
                    name: name.clone(),
                    version: dep_version(&dep_root),
                    checksum: String::new(),
                    source: format!("path+{}", dep_root.display()),
                });
            } else if let Some(git_info) = git_dep(value) {
                let dep_root = register::fetch_git_dependency(
                    git_info.url,
                    git_info.branch.as_deref(),
                    git_info.tag.as_deref(),
                    git_info.rev.as_deref(),
                )?;
                let dcr_package_name = package_name(&dep_root);
                let use_dcr_package = use_dcr_package(value, dcr_package_name.is_some(), name)?;
                let (has_include, has_lib, has_libs) = value
                    .as_table()
                    .map(|table| push_explicit_layout(&mut resolved, &dep_root, table))
                    .unwrap_or((false, false, false));
                if !has_include {
                    push_if_exists(&mut resolved.include_dirs, &dep_root.join("include"));
                    if use_dcr_package {
                        push_package_output_dirs(&mut resolved, &dep_root);
                    }
                }
                if !has_lib {
                    push_default_lib_dirs(&mut resolved.lib_dirs, &dep_root);
                }
                if !has_libs {
                    let fallback = dcr_package_name.as_deref().unwrap_or(name);
                    resolved.libs.push(package_link_name(
                        &dep_root, fallback, profile, target, None,
                    ));
                }
                if use_dcr_package {
                    resolved
                        .package_roots
                        .push(dep_root.to_string_lossy().to_string());
                } else if !(has_include && has_lib && has_libs) {
                    return Err(format!(
                        "Git dependency `{}` has no dcr.toml; specify include, lib, and libs for a prebuilt library",
                        name
                    ));
                }
                let commit = register::git_commit(&dep_root)?;
                lock_packages.push(DepLock {
                    name: name.clone(),
                    version: git_info.version.unwrap_or(commit.clone()),
                    checksum: commit.clone(),
                    source: format!("git+{}#{}", git_info.url, commit),
                });
            }
        }
    }

    // Write the lock file with all resolved dependencies
    write_lock(
        project_root,
        &project_name,
        &project_version,
        &lock_packages,
    )?;

    Ok(resolved)
}

/// Git dependency specification (url + optional version/ref for the lockfile).
struct GitDep<'a> {
    url: &'a str,
    version: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
}

/// Parses a git dependency from a TOML value table.
fn git_dep(value: &toml::Value) -> Option<GitDep<'_>> {
    if let Some(table) = value.as_table()
        && let Some(url) = table.get("git").and_then(|v| v.as_str())
    {
        let version = table
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let branch = table
            .get("branch")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let tag = table
            .get("tag")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let rev = table
            .get("rev")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        return Some(GitDep {
            url,
            version,
            branch,
            tag,
            rev,
        });
    }
    None
}

/// Extracts one or more pkg-config package names from a dependency table.
/// Both `pkg-config` and the legacy-style `pkg_config` spelling are accepted.
fn pkg_config_dep(
    value: &toml::Value,
    dependency_name: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(table) = value.as_table() else {
        return Ok(None);
    };
    let raw = table.get("pkg-config").or_else(|| table.get("pkg_config"));
    let Some(raw) = raw else {
        return Ok(None);
    };
    let packages = if let Some(pkg) = raw.as_str() {
        vec![pkg.to_string()]
    } else {
        raw.as_array()
            .ok_or_else(|| {
                format!(
                    "Dependency `{dependency_name}` field `pkg-config` must be a string or array of strings"
                )
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    format!(
                        "Dependency `{dependency_name}` field `pkg-config` must contain only strings"
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect()
    };
    if packages.is_empty() || packages.iter().any(|pkg| pkg.trim().is_empty()) {
        return Err(format!(
            "Dependency `{dependency_name}` field `pkg-config` must not be empty"
        ));
    }
    Ok(Some(packages))
}

/// Extracts the path for a path-based dependency, supporting both table format and legacy string format.
fn path_dep_path(value: &toml::Value) -> Option<&str> {
    if let Some(table) = value.as_table() {
        return table.get("path").and_then(|v| v.as_str());
    }
    register::path_from_string_dep(value)
}

/// Adds a directory path to the list if it exists on the filesystem.
fn push_if_exists(paths: &mut Vec<String>, path: &Path) {
    if path.exists() {
        paths.push(path.to_string_lossy().to_string());
    }
}

/// Adds common library directory candidates to the list if they exist.
fn push_default_lib_dirs(paths: &mut Vec<String>, dep_root: &Path) {
    for dir in ["lib", "lib64"] {
        push_if_exists(paths, &dep_root.join(dir));
    }
    push_if_exists(paths, &dep_root.join("target").join("lib"));
}

/// Adds explicitly configured prebuilt include/lib directories and linker names.
fn push_explicit_layout(
    resolved: &mut ResolvedDeps,
    dep_root: &Path,
    table: &toml::map::Map<String, toml::Value>,
) -> (bool, bool, bool) {
    let mut has_include = false;
    let mut has_lib = false;
    let mut has_libs = false;
    if let Some(includes) = table.get("include").and_then(|v| v.as_array()) {
        has_include = true;
        for inc in includes {
            if let Some(path) = inc.as_str() {
                resolved
                    .include_dirs
                    .push(dep_root.join(path).to_string_lossy().to_string());
            }
        }
    }
    if let Some(lib_dirs) = table.get("lib").and_then(|v| v.as_array()) {
        has_lib = true;
        for path in lib_dirs {
            if let Some(path) = path.as_str() {
                resolved
                    .lib_dirs
                    .push(dep_root.join(path).to_string_lossy().to_string());
            }
        }
    }
    if let Some(libs) = table.get("libs").and_then(|v| v.as_array()) {
        has_libs = true;
        for lib in libs {
            if let Some(name) = lib.as_str() {
                resolved.libs.push(name.to_string());
            }
        }
    }
    (has_include, has_lib, has_libs)
}

/// Reports whether a source without a DCR manifest fully describes its prebuilt layout.
fn has_complete_prebuilt_layout(value: &toml::Value) -> bool {
    value.as_table().is_some_and(|table| {
        ["include", "lib", "libs"].iter().all(|field| {
            table
                .get(*field)
                .and_then(|value| value.as_array())
                .is_some_and(|values| !values.is_empty())
        })
    })
}

/// Selects automatic, forced-build, or forced-prebuilt dependency resolution.
fn use_dcr_package(
    value: &toml::Value,
    has_manifest: bool,
    dependency_name: &str,
) -> Result<bool, String> {
    let mode = value
        .as_table()
        .and_then(|table| table.get("mode"))
        .map(|value| {
            value.as_str().ok_or_else(|| {
                format!(
                    "Dependency `{dependency_name}` field `mode` must be \"build\" or \"prebuild\""
                )
            })
        })
        .transpose()?;
    match mode {
        None => Ok(has_manifest),
        Some("build") if has_manifest => Ok(true),
        Some("build") => Err(format!(
            "Dependency `{dependency_name}` uses mode = \"build\" but has no dcr.toml"
        )),
        Some("prebuild") => Ok(false),
        Some(_) => Err(format!(
            "Dependency `{dependency_name}` field `mode` must be \"build\" or \"prebuild\""
        )),
    }
}

/// Adds DCR's packaged output directories even before the dependency is built.
fn push_package_output_dirs(resolved: &mut ResolvedDeps, dep_root: &Path) {
    push_unique_path(
        &mut resolved.include_dirs,
        dep_root.join("target").join("include"),
    );
    push_unique_path(&mut resolved.lib_dirs, dep_root.join("target").join("lib"));
}

/// Adds a path once, preserving the linker and include-search order.
fn push_unique_path(paths: &mut Vec<String>, path: std::path::PathBuf) {
    let path = path.to_string_lossy().to_string();
    if !paths.contains(&path) {
        paths.push(path);
    }
}

/// Returns the linker reference for a DCR dependency.
///
/// A custom `build.filename` / `build.extension` changes the packaged artifact
/// name. When the result still follows the platform convention (`libfoo.a`,
/// `libfoo.so`, `foo.lib`) a plain `-l` name is returned; otherwise the exact
/// packaged artifact path is returned so the linker can consume it directly.
///
/// `lib_root` is the directory the dependency is packaged into: `None` means
/// `<package>/target/lib`, while workspace builds pass the workspace
/// `target/lib`, because `package_library` copies member libraries there.
pub(crate) fn package_link_name(
    path: &Path,
    fallback: &str,
    profile: &str,
    target: Option<&str>,
    lib_root: Option<&Path>,
) -> String {
    let Ok(config) = Config::open(&path.join("dcr.toml").to_string_lossy()) else {
        return fallback.to_string();
    };
    let kind = get_string_with_profile_and_target(&config, "kind", profile, target);
    if kind.as_str() != "staticlib" && kind.as_str() != "sharedlib" {
        return fallback.to_string();
    }
    let filename = get_string_with_profile_and_target(&config, "filename", profile, target);
    let extension = get_string_with_profile_and_target(&config, "extension", profile, target);
    let filename = filename.trim();
    let extension = extension.trim();
    if filename.is_empty() && extension.is_empty() {
        return fallback.to_string();
    }
    let base = if filename.is_empty() {
        // `extension` without `filename` uses the dependency's real package
        // name, which may differ from the key in `[dependencies]`.
        config
            .get("package.name")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(fallback)
    } else {
        filename
    };
    let file = custom_artifact_file_name(
        kind.as_str(),
        base,
        if extension.is_empty() {
            None
        } else {
            Some(extension)
        },
    );
    if let Some(link_name) = linkable_library_name(&file) {
        return link_name;
    }

    // Exact filename (e.g. an explicit `extension`): `-l` cannot find it, so
    // pass the packaged artifact by absolute path instead. `canonicalize_path`
    // also strips the Windows `\\?\` verbatim prefix, which MinGW gcc mangles.
    let file_name = Path::new(&file)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file.as_str());
    let lib_dir = match lib_root {
        Some(dir) => dir.to_path_buf(),
        None => path.join("target").join("lib"),
    };
    crate::utils::fs::canonicalize_path(&lib_dir)
        .join(file_name)
        .to_string_lossy()
        .to_string()
}

/// Returns the conventional `-l`/MSVC name for an artifact, or `None` when the
/// filename needs to be passed to the linker as an exact path.
fn linkable_library_name(file: &str) -> Option<String> {
    if file.contains('/') || file.contains('\\') {
        return None;
    }
    #[cfg(windows)]
    {
        let lower = file.to_ascii_lowercase();
        if lower.ends_with(".lib") {
            let stem = &file[..file.len() - 4];
            if stem.is_empty() {
                return None;
            }
            return Some(stem.to_string());
        }
        None
    }
    #[cfg(not(windows))]
    {
        let stem = file.strip_prefix("lib")?;
        let stem = if let Some(stem) = stem.strip_suffix(".a") {
            stem
        } else if cfg!(target_os = "macos") {
            stem.strip_suffix(".dylib")?
        } else {
            stem.strip_suffix(".so")?
        };
        if stem.is_empty() {
            None
        } else {
            Some(stem.to_string())
        }
    }
}

/// Reads a DCR package name when the dependency has a manifest.
fn package_name(path: &Path) -> Option<String> {
    Config::open(&path.join("dcr.toml").to_string_lossy())
        .ok()
        .and_then(|config| {
            config
                .get("package.name")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    /// Tests that path_dep_path correctly handles both TOML table and legacy string inputs for path dependencies.
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
    fn pkg_config_dep_accepts_string_and_array() {
        let single: Value = toml::from_str("pkg-config = \"fmt\"").unwrap();
        assert_eq!(
            pkg_config_dep(&single, "fmt"),
            Ok(Some(vec!["fmt".to_string()]))
        );
        let many: Value = toml::from_str("pkg_config = [\"fmt\", \"zlib\"]").unwrap();
        assert_eq!(
            pkg_config_dep(&many, "base"),
            Ok(Some(vec!["fmt".to_string(), "zlib".to_string()]))
        );
    }

    /// Verifies that push_default_lib_dirs includes the target/lib directory when it exists.
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
        std::fs::create_dir_all(root.join("target").join("lib")).unwrap();
        let mut paths = Vec::new();
        push_default_lib_dirs(&mut paths, &root);
        assert!(paths.iter().any(|p| {
            let n = p.replace('\\', "/");
            n.ends_with("target/lib")
        }));
        let _ = std::fs::remove_dir_all(root);
    }
}
