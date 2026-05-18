use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub url: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DcrConfig {
    pub registry: std::collections::HashMap<String, Registry>,
}

pub fn get_registry_config() -> Option<DcrConfig> {
    let home = std::env::var("HOME").ok()?;
    let config_path = Path::new(&home).join(".dcr/config.toml");
    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(config_path).ok()?;
    toml::from_str(&content).ok()
}

pub fn get_index_path() -> PathBuf {
    std::env::var("DCR_INDEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            Path::new(&home).join(".dcr").join("index.json")
        })
}

pub fn get_registry_cache_root() -> PathBuf {
    get_index_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn package_root_from_registry_info(pkg_info: &JsonValue) -> Result<PathBuf, String> {
    let raw_path = pkg_info
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Registry package is missing path")?;
    if raw_path.trim().is_empty() {
        return Err("Registry package path is empty".to_string());
    }

    let path = Path::new(raw_path);
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        get_registry_cache_root().join(path)
    };

    if full_path.file_name().and_then(|v| v.to_str()) == Some("dcr.toml") {
        return full_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("Invalid registry package path: {}", full_path.display()));
    }

    Ok(full_path)
}

pub fn registry_include_dir(dep_root: &Path) -> PathBuf {
    dep_root.join("target").join("include")
}

pub fn registry_lib_dir(dep_root: &Path) -> PathBuf {
    dep_root.join("target").join("lib")
}

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

pub fn is_registry_dep(value: &TomlValue) -> bool {
    if let Some(raw) = value.as_str() {
        let raw = raw.trim();
        return !is_path_like_string(raw) && !is_git_like_string(raw);
    }
    if let Some(table) = value.as_table() {
        return !table.contains_key("git")
            && !table.contains_key("path")
            && !table.contains_key("url")
            && !table.contains_key("version");
    }
    false
}

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

fn is_path_like_string(raw: &str) -> bool {
    raw.starts_with("path:")
        || raw.starts_with("./")
        || raw.starts_with("../")
        || raw.starts_with('/')
        || raw.starts_with("~/")
        || raw.contains('\\')
}

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
        let pkg = serde_json::json!({ "path": "/tmp/pkg" });
        assert_eq!(
            package_root_from_registry_info(&pkg).unwrap(),
            PathBuf::from("/tmp/pkg")
        );

        let pkg = serde_json::json!({ "path": "/tmp/pkg/dcr.toml" });
        assert_eq!(
            package_root_from_registry_info(&pkg).unwrap(),
            PathBuf::from("/tmp/pkg")
        );
    }
}
