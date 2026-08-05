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

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

/// Represents a registry configuration entry with its URL and priority.
#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub url: String,
    pub priority: i32,
}

/// Configuration struct for DCR registries, mapping registry names to Registry details.
#[derive(Debug, Serialize, Deserialize)]
pub struct DcrConfig {
    pub registry: std::collections::HashMap<String, Registry>,
}

/// Helper function to get the user's home directory path.
fn home_dir() -> Option<PathBuf> {
    crate::utils::fs::home_dir()
}

/// Helper to get the DCR config directory path.
fn dcr_config_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".dcr"))
}

/// Loads DCR registry configuration from the TOML file in the user's home directory.
pub fn get_registry_config() -> Option<DcrConfig> {
    let home = home_dir()?;
    let config_path = home.join(".dcr/config.toml");
    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(config_path).ok()?;
    toml::from_str(&content).ok()
}

/// Returns the path to the registry index file, preferring the DCR_INDEX_PATH env var or falling back to ~/.dcr/index.json.
pub fn get_index_path() -> PathBuf {
    std::env::var("DCR_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dcr_config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("index.json")
        })
}

/// Returns the root directory for registry cache, based on the index path's parent, creating it if necessary.
pub fn get_registry_cache_root() -> PathBuf {
    let index_path = get_index_path();
    if let Some(parent) = index_path.parent() {
        if parent.exists() {
            return parent.to_path_buf();
        }
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Warning: failed to create registry cache directory: {e}");
        }
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    }
}

/// Extracts the package root path from registry info. If the path points to a dcr.toml manifest, returns its parent directory.
pub fn package_root_from_registry_info(pkg_info: &JsonValue) -> Result<PathBuf, String> {
    let raw_path = pkg_info
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Registry package is missing path")?;
    if raw_path.trim().is_empty() {
        return Err("Registry package path is empty".to_string());
    }

    let full_path = registry_path_to_pathbuf(raw_path.trim());

    if full_path.file_name().and_then(|v| v.to_str()) == Some("dcr.toml") {
        return full_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("Invalid registry package path: {}", full_path.display()));
    }

    Ok(full_path)
}

/// Converts a raw registry path string to a PathBuf, handling file:// prefixes, Windows extended paths, and relative paths by joining to cache root.
pub fn registry_path_to_pathbuf(raw: &str) -> PathBuf {
    let s = raw.trim();
    if let Some(rest) = s.strip_prefix("file://") {
        if rest.len() >= 3 && rest.as_bytes()[0] == b'/' && rest.as_bytes()[2] == b':' {
            return PathBuf::from(&rest[1..]);
        }
        if rest.len() >= 2 && rest.as_bytes()[1] == b':' {
            return PathBuf::from(rest);
        }
        if rest.starts_with('/') {
            return PathBuf::from(rest);
        }
        return PathBuf::from(format!("/{rest}"));
    }

    let mut s = s.replace('\\', "/");
    if let Some(rest) = s.strip_prefix("//?/") {
        s = rest.to_string();
    } else if let Some(rest) = s.strip_prefix("//./") {
        s = rest.to_string();
    }
    if s.len() >= 3 && s.as_bytes()[0] == b'/' && s.as_bytes()[2] == b':' {
        s = s[1..].to_string();
    }

    let path = PathBuf::from(&s);
    if path.is_absolute() || (s.len() >= 2 && s.as_bytes()[1] == b':') || s.starts_with('/') {
        path
    } else {
        get_registry_cache_root().join(path)
    }
}

/// Returns the include directory path for a dependency root, typically under target/include.
pub fn registry_include_dir(dep_root: &Path) -> PathBuf {
    dep_root.join("target").join("include")
}

/// Returns the library directory path for a dependency root, typically under target/lib.
pub fn registry_lib_dir(dep_root: &Path) -> PathBuf {
    dep_root.join("target").join("lib")
}

/// Looks up `name` in the local package index (`DCR_INDEX_PATH` / `~/.dcr/index.json`).
///
/// Configured registries are sorted by priority, but the lookup currently always reads the
/// same local index path (per-registry URLs are unused).
///
/// # Parameters
/// - `name`: Package name as in the index.
///
/// # Returns
/// Package JSON object, or an error if config/index is missing or the name is absent.
pub fn resolve_package_from_registry(name: &str) -> Result<JsonValue, String> {
    let config = get_registry_config().ok_or("No registry config found")?;
    let mut registries: Vec<(&String, &Registry)> = config.registry.iter().collect();
    registries.sort_by_key(|b| std::cmp::Reverse(b.1.priority));

    for (_name_reg, _reg) in registries {
        let index_path = get_index_path();
        if index_path.exists() {
            let index_content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
            let index: JsonValue =
                serde_json::from_str(&index_content).map_err(|e| e.to_string())?;
            if let Some(pkgs) = index.get("packages").and_then(|v| v.as_array()) {
                for pkg in pkgs {
                    if pkg.get("name").and_then(|v| v.as_str()) == Some(name) {
                        return Ok(pkg.clone());
                    }
                }
            }
        }
    }
    Err(format!(
        "Package {} not found in registry (checked: {:?})",
        name,
        get_index_path()
    ))
}

/// True if the TOML value is a registry-style dependency.
///
/// Strings: not path-like and not git-like. Tables: have version/features/optional/registry
/// and no `git`/`path`/`url` keys.
pub fn is_registry_dep(value: &TomlValue) -> bool {
    if let Some(raw) = value.as_str() {
        let raw = raw.trim();
        return !is_path_like_string(raw) && !is_git_like_string(raw);
    }
    if let Some(table) = value.as_table() {
        if table.contains_key("git") || table.contains_key("path") || table.contains_key("url") {
            return false;
        }
        table.contains_key("version")
            || table.contains_key("features")
            || table.contains_key("optional")
            || table.contains_key("registry")
    } else {
        false
    }
}

/// Extracts the path string from a dependency value if it's a path-like dependency.
pub fn path_from_string_dep(value: &TomlValue) -> Option<&str> {
    let raw = value.as_str()?.trim();
    if let Some(path) = raw.strip_prefix("path:") {
        return Some(path);
    }
    if is_path_like_string(raw) {
        return Some(raw);
    }
    None
}

/// Helper to check if a string represents a local file path.
fn is_path_like_string(raw: &str) -> bool {
    raw.starts_with("path:")
        || raw.starts_with("./")
        || raw.starts_with("../")
        || raw.starts_with('/')
        || raw.starts_with("~/")
        || raw.contains('\\')
}

/// Helper to check if a string represents a git or remote dependency.
fn is_git_like_string(raw: &str) -> bool {
    raw.starts_with("git:")
        || raw.starts_with("github:")
        || raw.starts_with("gitlab:")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("git@")
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::map::Map;

    #[test]
    fn registry_dep_detection_excludes_local_and_git_strings() {
        assert!(is_registry_dep(&TomlValue::String("1.2.3".to_string())));
        assert!(!is_registry_dep(&TomlValue::String(
            "path:./libs/mylib".to_string()
        )));
        assert!(!is_registry_dep(&TomlValue::String(
            "./libs/mylib".to_string()
        )));
        assert!(!is_registry_dep(&TomlValue::String(
            "git:https://example.com/repo.git".to_string()
        )));
    }

    #[test]
    fn registry_dep_detection_excludes_source_tables() {
        let mut path_table = Map::new();
        path_table.insert(
            "path".to_string(),
            TomlValue::String("./libs/mylib".to_string()),
        );
        assert!(!is_registry_dep(&TomlValue::Table(path_table)));

        let mut registry_table = Map::new();
        registry_table.insert("features".to_string(), TomlValue::Array(Vec::new()));
        assert!(is_registry_dep(&TomlValue::Table(registry_table)));
    }

    #[test]
    fn registry_package_root_accepts_package_dir_or_manifest_path() {
        let root = std::env::temp_dir().join(format!("dcr_reg_pkg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let root_str = root.to_string_lossy().replace('\\', "/");

        let pkg = serde_json::json!({ "path": root_str });
        assert_eq!(
            package_root_from_registry_info(&pkg)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            root_str
        );

        let manifest = format!("{root_str}/dcr.toml");
        let pkg = serde_json::json!({ "path": manifest });
        assert_eq!(
            package_root_from_registry_info(&pkg)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            root_str
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn registry_path_strips_windows_extended_prefix() {
        let p = registry_path_to_pathbuf("//?/D:/cache/mylib");
        let s = p.to_string_lossy().replace('\\', "/");
        assert_eq!(s, "D:/cache/mylib");

        let p2 = registry_path_to_pathbuf("/D:/cache/mylib");
        let s2 = p2.to_string_lossy().replace('\\', "/");
        assert_eq!(s2, "D:/cache/mylib");

        let p3 = registry_path_to_pathbuf("file:///D:/cache/mylib");
        let s3 = p3.to_string_lossy().replace('\\', "/");
        assert_eq!(s3, "D:/cache/mylib");
    }
}
