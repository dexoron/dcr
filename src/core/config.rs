use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;
use toml::map::Map;

const DEFAULT_VERSION: &str = "0.1.0";
const DEFAULT_LANGUAGE: &str = "c";
const DEFAULT_STANDARD: &str = "c11";
const DEFAULT_COMPILER: &str = "clang";
const DEFAULT_KIND: &str = "bin";

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    Invalid(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "I/O error: {err}"),
            ConfigError::TomlDe(err) => write!(f, "TOML parse error: {err}"),
            ConfigError::TomlSer(err) => write!(f, "TOML serialize error: {err}"),
            ConfigError::Invalid(msg) => write!(f, "Invalid config: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::TomlDe(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::TomlSer(err)
    }
}

pub struct Config {
    path: PathBuf,
    data: Value,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let path = PathBuf::from(path);
        let data = if path.exists() {
            read_toml(&path)?
        } else {
            let default_value = default_toml()?;
            write_toml(&path, &default_value)?;
            default_value
        };
        let cfg = Self { path, data };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn open(path: &str) -> Result<Self, ConfigError> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(ConfigError::Invalid("dcr.toml not found".into()));
        }
        let data = read_toml(&path)?;
        let cfg = Self { path, data };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        let parts: Vec<&str> = key.split('.').collect();
        get_path(&self.data, &parts)
    }

    #[allow(dead_code)]
    pub fn add(&mut self, key: &str, value: Value) -> Result<(), ConfigError> {
        self.set(key, value)
    }

    pub fn edit(&mut self, key: &str, value: Value) -> Result<(), ConfigError> {
        self.set(key, value)
    }
    #[allow(dead_code)]
    pub fn check(&self) -> bool {
        self.validate().is_ok()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let package = self
            .get("package")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::Invalid("missing [package]".into()))?;

        for key in ["name", "version"] {
            let value = package.get(key).and_then(|v| v.as_str()).unwrap_or("");
            if value.trim().is_empty() {
                return Err(ConfigError::Invalid(format!("package.{key} is empty")));
            }
        }

        let build = self
            .get("build")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::Invalid("missing [build]".into()))?;

        for key in ["language", "standard", "compiler"] {
            let value = build.get(key).and_then(|v| v.as_str()).unwrap_or("");
            if value.trim().is_empty() {
                return Err(ConfigError::Invalid(format!("build.{key} is empty")));
            }
        }
        if let Some(platform) = build.get("platform")
            && !platform
                .as_str()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        {
            return Err(ConfigError::Invalid("build.platform is empty".into()));
        }
        if let Some(toolchain) = self.get("toolchain").and_then(|v| v.as_table()) {
            for key in ["cc", "cxx", "as", "ar", "ld"] {
                if let Some(v) = toolchain.get(key)
                    && !v.as_str().map(|s| !s.trim().is_empty()).unwrap_or(false)
                {
                    return Err(ConfigError::Invalid(format!("toolchain.{key} is invalid")));
                }
            }
        }
        if let Some(kind) = build.get("kind").and_then(|v| v.as_str()) {
            let kind = kind.trim();
            if !kind.is_empty() && kind != "bin" && kind != "staticlib" && kind != "sharedlib" {
                return Err(ConfigError::Invalid("build.kind is invalid".into()));
            }
        }
        self.validate_workspace()?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        write_toml(&self.path, &self.data)
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), ConfigError> {
        let parts: Vec<&str> = key.split('.').collect();
        set_path(&mut self.data, &parts, value)?;
        self.save()?;
        Ok(())
    }
}

impl Config {
    fn validate_workspace(&self) -> Result<(), ConfigError> {
        let Some(workspace) = self.get("workspace").and_then(|v| v.as_table()) else {
            return Ok(());
        };
        for (name, value) in workspace {
            let tbl = value
                .as_table()
                .ok_or_else(|| ConfigError::Invalid(format!("workspace.{name} must be a table")))?;
            let path = tbl.get("path").and_then(|v| v.as_str()).unwrap_or("");
            if path.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "workspace.{name}.path is empty"
                )));
            }
            if let Some(deps) = tbl.get("deps") {
                let arr = deps.as_array().ok_or_else(|| {
                    ConfigError::Invalid(format!("workspace.{name}.deps must be array"))
                })?;
                for item in arr {
                    let s = item.as_str().unwrap_or("");
                    if s.trim().is_empty() {
                        return Err(ConfigError::Invalid(format!(
                            "workspace.{name}.deps contains empty value"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

fn read_toml(path: &Path) -> Result<Value, ConfigError> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn write_toml(path: &Path, value: &Value) -> Result<(), ConfigError> {
    let content = format_toml(value)?;
    fs::write(path, content)?;
    Ok(())
}

fn default_toml() -> Result<Value, ConfigError> {
    let name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    let mut package = Map::new();
    package.insert("name".to_string(), Value::String(name));
    package.insert(
        "version".to_string(),
        Value::String(DEFAULT_VERSION.to_string()),
    );
    let mut build = Map::new();
    build.insert(
        "language".to_string(),
        Value::String(DEFAULT_LANGUAGE.to_string()),
    );
    build.insert(
        "standard".to_string(),
        Value::String(DEFAULT_STANDARD.to_string()),
    );
    build.insert(
        "compiler".to_string(),
        Value::String(DEFAULT_COMPILER.to_string()),
    );
    build.insert("kind".to_string(), Value::String(DEFAULT_KIND.to_string()));

    let mut root = Map::new();
    root.insert("package".to_string(), Value::Table(package));
    root.insert("build".to_string(), Value::Table(build));
    root.insert("dependencies".to_string(), Value::Table(Map::new()));

    Ok(Value::Table(root))
}

fn format_toml(value: &Value) -> Result<String, ConfigError> {
    let root = value
        .as_table()
        .ok_or_else(|| ConfigError::Invalid("root is not a table".into()))?;

    let package = root
        .get("package")
        .and_then(|v| v.as_table())
        .ok_or_else(|| ConfigError::Invalid("missing [package]".into()))?;
    let build = root
        .get("build")
        .and_then(|v| v.as_table())
        .ok_or_else(|| ConfigError::Invalid("missing [build]".into()))?;
    let deps = root
        .get("dependencies")
        .and_then(|v| v.as_table())
        .ok_or_else(|| ConfigError::Invalid("missing [dependencies]".into()))?;
    let toolchain = root.get("toolchain").and_then(|v| v.as_table());

    let name = package.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let version = package
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let language = build.get("language").and_then(|v| v.as_str()).unwrap_or("");
    let standard = build.get("standard").and_then(|v| v.as_str()).unwrap_or("");
    let compiler = build.get("compiler").and_then(|v| v.as_str()).unwrap_or("");

    let mut out = String::new();
    out.push_str("[package]\n");
    out.push_str(&format!("name = \"{name}\"\n"));
    out.push_str(&format!("version = \"{version}\"\n\n"));

    out.push_str("[build]\n");
    out.push_str(&format!("language = \"{language}\"\n"));
    out.push_str(&format!("standard = \"{standard}\"\n"));
    out.push_str(&format!("compiler = \"{compiler}\"\n"));
    let kind = build
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_KIND);
    out.push_str(&format!("kind = \"{kind}\"\n"));
    if let Some(target) = build.get("target").and_then(|v| v.as_str())
        && !target.trim().is_empty()
    {
        out.push_str(&format!("target = \"{target}\"\n"));
    }
    if let Some(cflags) = build.get("cflags") {
        out.push_str(&format!("cflags = {}\n", format_string_array(cflags)));
    }
    if let Some(ldflags) = build.get("ldflags") {
        out.push_str(&format!("ldflags = {}\n", format_string_array(ldflags)));
    }
    out.push('\n');

    if let Some(toolchain) = toolchain {
        let mut lines = Vec::new();
        for key in ["cc", "cxx", "as", "ar", "ld"] {
            if let Some(value) = toolchain.get(key).and_then(|v| v.as_str())
                && !value.trim().is_empty()
            {
                lines.push(format!("{key} = \"{value}\""));
            }
        }
        if !lines.is_empty() {
            out.push_str("[toolchain]\n");
            for line in lines {
                out.push_str(&format!("{line}\n"));
            }
            out.push('\n');
        }
    }

    out.push_str("[dependencies]\n");
    if !deps.is_empty() {
        let mut keys: Vec<&String> = deps.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(val) = deps.get(key) {
                out.push_str(&format!("{key} = {}\n", format_dep_value(val)));
            }
        }
    }
    Ok(out)
}

fn format_dep_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Table(tbl) => {
            let mut parts = Vec::new();
            if let Some(v) = tbl.get("path").and_then(|v| v.as_str()) {
                parts.push(format!("path = \"{v}\""));
            }
            if let Some(v) = tbl.get("system").and_then(|v| v.as_bool()) {
                parts.push(format!("system = {}", if v { "true" } else { "false" }));
            }
            if let Some(v) = tbl.get("include") {
                parts.push(format!("include = {}", format_string_array(v)));
            }
            if let Some(v) = tbl.get("lib") {
                parts.push(format!("lib = {}", format_string_array(v)));
            }
            if let Some(v) = tbl.get("libs") {
                parts.push(format!("libs = {}", format_string_array(v)));
            }
            format!("{{ {} }}", parts.join(", "))
        }
        _ => "\"\"".to_string(),
    }
}

fn format_string_array(value: &Value) -> String {
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| format!("\"{s}\"")))
            .collect();
        return format!("[{}]", items.join(", "));
    }
    "[]".to_string()
}

fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.as_table()?.get(*key)?;
    }
    Some(current)
}

fn set_path(value: &mut Value, path: &[&str], new_value: Value) -> Result<(), ConfigError> {
    let mut current = value
        .as_table_mut()
        .ok_or_else(|| ConfigError::Invalid("root is not a table".into()))?;

    for key in &path[..path.len().saturating_sub(1)] {
        if !current.contains_key(*key) {
            current.insert((*key).to_string(), Value::Table(Map::new()));
        }
        current = current
            .get_mut(*key)
            .and_then(|v| v.as_table_mut())
            .ok_or_else(|| ConfigError::Invalid(format!("'{key}' is not a table")))?;
    }

    if let Some(last) = path.last() {
        current.insert((*last).to_string(), new_value);
        Ok(())
    } else {
        Err(ConfigError::Invalid("empty key".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("dcr_cfg_test_{prefix}_{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_toml_file(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("dcr.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_valid_toml() -> &'static str {
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\n"
    }

    #[test]
    fn open_valid_toml() {
        let dir = temp_dir("open_valid");
        let path = write_toml_file(&dir, minimal_valid_toml());
        let config = Config::open(&path.to_string_lossy());
        assert!(config.is_ok(), "should open valid toml");
    }

    #[test]
    fn open_invalid_toml_syntax() {
        let dir = temp_dir("open_invalid");
        let path = write_toml_file(&dir, "this is not [valid toml !!!");
        let config = Config::open(&path.to_string_lossy());
        assert!(config.is_err(), "should fail on invalid TOML syntax");
    }

    #[test]
    fn open_nonexistent_fails() {
        let result = Config::open("/tmp/dcr_nonexistent_file_12345.toml");
        assert!(result.is_err(), "should fail on nonexistent file");
    }

    #[test]
    fn validate_missing_package_fails() {
        let dir = temp_dir("no_package");
        let path = write_toml_file(
            &dir,
            "[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_err(), "missing [package] should fail validation");
    }

    #[test]
    fn validate_missing_build_fails() {
        let dir = temp_dir("no_build");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_err(), "missing [build] should fail validation");
    }

    #[test]
    fn validate_empty_language_fails() {
        let dir = temp_dir("empty_lang");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"\"\nstandard = \"c11\"\ncompiler = \"clang\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_err(), "empty language should fail validation");
    }

    #[test]
    fn validate_unknown_kind_fails() {
        let dir = temp_dir("bad_kind");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"exe\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_err(), "unknown kind 'exe' should fail validation");
    }

    #[test]
    fn validate_valid_kinds() {
        for kind in ["bin", "staticlib", "sharedlib"] {
            let dir = temp_dir(&format!("kind_{kind}"));
            let content = format!(
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"{kind}\"\n\n[dependencies]\n"
            );
            let path = write_toml_file(&dir, &content);
            let result = Config::open(&path.to_string_lossy());
            assert!(result.is_ok(), "kind '{kind}' should be valid");
        }
    }

    #[test]
    fn get_values() {
        let dir = temp_dir("get_values");
        let path = write_toml_file(&dir, minimal_valid_toml());
        let config = Config::open(&path.to_string_lossy()).unwrap();

        assert_eq!(
            config.get("package.name").and_then(|v| v.as_str()),
            Some("test")
        );
        assert_eq!(
            config.get("build.language").and_then(|v| v.as_str()),
            Some("c")
        );
        assert_eq!(
            config.get("build.kind").and_then(|v| v.as_str()),
            Some("bin")
        );
        assert!(config.get("nonexistent.key").is_none());
    }

    #[test]
    fn set_and_read_back() {
        let dir = temp_dir("set_value");
        let path = write_toml_file(&dir, minimal_valid_toml());
        let mut config = Config::open(&path.to_string_lossy()).unwrap();

        config
            .set("package.name", Value::String("newname".to_string()))
            .unwrap();
        assert_eq!(
            config.get("package.name").and_then(|v| v.as_str()),
            Some("newname")
        );

        // Verify persisted to disk
        let config2 = Config::open(&path.to_string_lossy()).unwrap();
        assert_eq!(
            config2.get("package.name").and_then(|v| v.as_str()),
            Some("newname")
        );
    }

    #[test]
    fn new_creates_default_config() {
        let dir = temp_dir("new_default");
        let path = dir.join("dcr.toml");
        let config = Config::new(&path.to_string_lossy()).unwrap();

        assert!(path.exists(), "dcr.toml should be created");
        assert_eq!(
            config.get("build.language").and_then(|v| v.as_str()),
            Some("c")
        );
        assert_eq!(
            config.get("build.standard").and_then(|v| v.as_str()),
            Some("c11")
        );
        assert_eq!(
            config.get("build.compiler").and_then(|v| v.as_str()),
            Some("clang")
        );
        assert_eq!(
            config.get("build.kind").and_then(|v| v.as_str()),
            Some("bin")
        );
    }

    #[test]
    fn validate_workspace_empty_path_fails() {
        let dir = temp_dir("ws_empty_path");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\n\n[workspace]\n[workspace.member1]\npath = \"\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(
            result.is_err(),
            "workspace member with empty path should fail"
        );
    }

    #[test]
    fn check_returns_bool() {
        let dir = temp_dir("check_bool");
        let path = write_toml_file(&dir, minimal_valid_toml());
        let config = Config::open(&path.to_string_lossy()).unwrap();
        assert!(config.check(), "valid config should return true");
    }
}
