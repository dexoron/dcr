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

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;
use toml::map::Map;

const DEFAULT_VERSION: &str = "0.1.0";
const DEFAULT_LANGUAGE: &str = "c";
const DEFAULT_STANDARD: &str = "c11";
const DEFAULT_COMPILER: &str = "clang";
const DEFAULT_KIND: &str = "bin";
const VALID_KINDS: &[&str] = &[
    "bin",
    "staticlib",
    "sharedlib",
    "efi",
    "elf",
    "none",
    "custom",
];

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
    typed: DcrConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DcrConfig {
    #[serde(default)]
    pub package: Option<PackageConfig>,
    #[serde(default)]
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyConfig>,
    #[serde(default)]
    pub toolchain: Option<ToolchainConfig>,
    #[serde(default)]
    pub workspace: BTreeMap<String, WorkspaceMemberConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    #[serde(default, rename = "type")]
    pub pkg_type: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    pub language: LanguageConfig,
    #[serde(default)]
    pub standard: Option<String>,
    #[serde(default)]
    pub cxx_standard: Option<String>,
    pub compiler: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub out_dir: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub cflags: Vec<String>,
    #[serde(default)]
    pub ldflags: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub roots: Vec<String>,
    #[serde(default)]
    pub clean: Vec<String>,
    #[serde(default)]
    pub src_disable: Option<bool>,
    #[serde(default)]
    pub workspace_only: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LanguageConfig {
    One(String),
    Many(Vec<String>),
}

impl LanguageConfig {
    fn values(&self) -> Vec<&str> {
        match self {
            LanguageConfig::One(value) => vec![value.as_str()],
            LanguageConfig::Many(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencyConfig {
    Version(String),
    Table(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ToolchainConfig {
    pub cc: Option<String>,
    pub cxx: Option<String>,
    #[serde(rename = "as")]
    pub assembler: Option<String>,
    pub ar: Option<String>,
    pub ld: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceMemberConfig {
    pub path: String,
    #[serde(default)]
    pub deps: Vec<String>,
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
        let typed = parse_typed_config(&data)?;
        let cfg = Self { path, data, typed };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn open(path: &str) -> Result<Self, ConfigError> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(ConfigError::Invalid("dcr.toml not found".into()));
        }
        let data = read_toml(&path)?;
        let typed = parse_typed_config(&data)?;
        let cfg = Self { path, data, typed };
        cfg.validate()?;
        Ok(cfg)
    }

    #[allow(dead_code)]
    pub fn typed(&self) -> &DcrConfig {
        &self.typed
    }

    #[allow(dead_code)]
    pub fn package(&self) -> Option<&PackageConfig> {
        self.typed.package.as_ref()
    }

    #[allow(dead_code)]
    pub fn build_config(&self) -> Option<&BuildConfig> {
        self.typed.build.as_ref()
    }

    pub fn is_workspace_only(&self) -> bool {
        self.typed
            .build
            .as_ref()
            .map(|b| b.workspace_only)
            .unwrap_or(false)
            && !self.typed.workspace.is_empty()
    }

    /// Merge build fields from parent config into self where self has empty/default values.
    /// Used for workspace member inheritance (build.inherit = true).
    pub fn merge_parent(&mut self, parent: &Config) {
        let fields: &[&str] = &[
            "language",
            "standard",
            "cxx_standard",
            "compiler",
            "kind",
            "target",
            "platform",
            "cflags",
            "ldflags",
            "exclude",
            "include",
            "roots",
            "clean",
            "src_disable",
        ];
        let parent_build = match parent.data.get("build").and_then(|v| v.as_table()) {
            Some(t) => t.clone(),
            None => return,
        };

        let self_build = self
            .data
            .get("build")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();

        let mut changes: Vec<(String, toml::Value)> = Vec::new();

        for field in fields {
            let val = match parent_build.get(*field) {
                Some(v) => v,
                None => continue,
            };

            let self_val = self_build.get(*field);
            let should_inherit = match val {
                toml::Value::Array(_) => self_val.is_none(),
                toml::Value::String(_) => self_val
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true),
                _ => self_val.is_none(),
            };
            if should_inherit {
                changes.push((field.to_string(), val.clone()));
            }
        }

        if changes.is_empty() {
            return;
        }

        if !self.data.is_table() {
            let mut m = Map::new();
            m.insert("build".to_string(), toml::Value::Table(Map::new()));
            self.data = toml::Value::Table(m);
        }

        let tbl = self.data.as_table_mut().unwrap();
        let build_tbl = tbl
            .entry("build".to_string())
            .or_insert_with(|| toml::Value::Table(Map::new()));
        if let Some(build_tbl) = build_tbl.as_table_mut() {
            for (key, val) in changes {
                build_tbl.entry(key).or_insert(val);
            }
        }

        if let Ok(typed) = parse_typed_config(&self.data) {
            self.typed = typed;
        }
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
        if self.is_workspace_only() {
            return self.validate_workspace();
        }

        let pkg =
            self.typed.package.as_ref().ok_or_else(|| {
                ConfigError::Invalid("missing [package] section (required)".into())
            })?;
        let build = self
            .typed
            .build
            .as_ref()
            .ok_or_else(|| ConfigError::Invalid("missing [build] section (required)".into()))?;

        if pkg.name.trim().is_empty() {
            return Err(ConfigError::Invalid("package.name is empty".into()));
        }
        if pkg.version.trim().is_empty() {
            return Err(ConfigError::Invalid("package.version is empty".into()));
        }
        validate_package_name(&pkg.name)?;

        if let Some(pkg_type) = &pkg.pkg_type {
            let pkg_type = pkg_type.trim();
            if !pkg_type.is_empty() && pkg_type != "lib" && pkg_type != "app" && pkg_type != "none"
            {
                return Err(ConfigError::Invalid(
                    "package.type must be 'lib', 'app', or 'none'".into(),
                ));
            }
        }

        validate_language_config(&build.language, "build.language")?;
        let lang_is_asm = match &build.language {
            LanguageConfig::One(s) => s.trim().eq_ignore_ascii_case("asm"),
            LanguageConfig::Many(v) => v.len() == 1 && v[0].trim().eq_ignore_ascii_case("asm"),
        };
        if let Some(ref std) = build.standard
            && std.trim().is_empty()
            && !lang_is_asm
        {
            return Err(ConfigError::Invalid("build.standard is empty".into()));
        }
        if build.compiler.trim().is_empty() {
            return Err(ConfigError::Invalid("build.compiler is empty".into()));
        }
        if let Some(platform) = &build.platform
            && platform.trim().is_empty()
        {
            return Err(ConfigError::Invalid("build.platform is empty".into()));
        }
        validate_toolchain(self.typed.toolchain.as_ref())?;
        if let Some(kind) = &build.kind {
            let kind = kind.trim();
            if !kind.is_empty() && !VALID_KINDS.contains(&kind) {
                return Err(ConfigError::Invalid("build.kind is invalid".into()));
            }
        }
        validate_string_list(&build.exclude, "build.exclude")?;
        validate_string_list(&build.include, "build.include")?;
        validate_string_list(&build.roots, "build.roots")?;
        validate_string_list(&build.clean, "build.clean")?;
        validate_string_list(&build.cflags, "build.cflags")?;
        validate_string_list(&build.ldflags, "build.ldflags")?;
        if let Some(target) = &build.target {
            validate_non_empty_string(target, "build.target")?;
        }
        if let Some(out_dir) = &build.out_dir {
            validate_non_empty_string(out_dir, "build.out_dir")?;
        }
        for profile in ["release", "debug"] {
            if let Some(section) = self
                .data
                .get("build")
                .and_then(|v| v.as_table())
                .and_then(|build| build.get(profile))
            {
                let table = section.as_table().ok_or_else(|| {
                    ConfigError::Invalid(format!("build.{profile} must be a table"))
                })?;
                self.validate_profile_section(profile, table)?;
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
        let previous = self.data.clone();
        set_path(&mut self.data, &parts, value)?;
        self.typed = match parse_typed_config(&self.data) {
            Ok(typed) => typed,
            Err(err) => {
                self.data = previous;
                return Err(err);
            }
        };
        if let Err(err) = self.validate() {
            self.data = previous;
            self.typed = parse_typed_config(&self.data)?;
            return Err(err);
        }
        self.save()?;
        Ok(())
    }
}

fn parse_typed_config(value: &Value) -> Result<DcrConfig, ConfigError> {
    value.clone().try_into().map_err(ConfigError::TomlDe)
}

fn validate_package_name(name: &str) -> Result<(), ConfigError> {
    let trimmed = name.trim();
    if trimmed != name {
        return Err(ConfigError::Invalid(
            "package.name must not contain leading or trailing whitespace".into(),
        ));
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains("..") {
        return Err(ConfigError::Invalid("package.name is invalid".into()));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ConfigError::Invalid(
            "package.name must contain only ASCII letters, digits, '_' or '-'".into(),
        ));
    }
    Ok(())
}

fn validate_language_config(language: &LanguageConfig, key: &str) -> Result<(), ConfigError> {
    let values = language.values();
    if values.is_empty() {
        return Err(ConfigError::Invalid(format!("{key} is empty")));
    }
    for value in values {
        validate_non_empty_string(value, key)?;
    }
    Ok(())
}

fn validate_toolchain(toolchain: Option<&ToolchainConfig>) -> Result<(), ConfigError> {
    let Some(toolchain) = toolchain else {
        return Ok(());
    };
    for (key, value) in [
        ("cc", toolchain.cc.as_deref()),
        ("cxx", toolchain.cxx.as_deref()),
        ("as", toolchain.assembler.as_deref()),
        ("ar", toolchain.ar.as_deref()),
        ("ld", toolchain.ld.as_deref()),
    ] {
        if let Some(value) = value {
            validate_non_empty_string(value, &format!("toolchain.{key}"))?;
        }
    }
    Ok(())
}

fn validate_string_list(values: &[String], key: &str) -> Result<(), ConfigError> {
    for value in values {
        validate_non_empty_string(value, key)?;
    }
    Ok(())
}

fn validate_non_empty_string(value: &str, key: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Invalid(format!("{key} contains empty value")));
    }
    Ok(())
}

impl Config {
    fn validate_language(&self, value: &Value) -> Result<(), ConfigError> {
        if let Some(s) = value.as_str() {
            if s.trim().is_empty() {
                return Err(ConfigError::Invalid("build.language is empty".into()));
            }
            return Ok(());
        }
        let arr = value
            .as_array()
            .ok_or_else(|| ConfigError::Invalid("build.language must be string or array".into()))?;
        if arr.is_empty() {
            return Err(ConfigError::Invalid("build.language is empty".into()));
        }
        for item in arr {
            let s = item.as_str().unwrap_or("");
            if s.trim().is_empty() {
                return Err(ConfigError::Invalid(
                    "build.language contains empty value".into(),
                ));
            }
        }
        Ok(())
    }

    fn validate_profile_section(
        &self,
        profile: &str,
        table: &toml::value::Table,
    ) -> Result<(), ConfigError> {
        if let Some(lang) = table.get("language") {
            self.validate_language(lang)?;
        }
        for key in [
            "standard", "compiler", "kind", "target", "out_dir", "platform",
        ] {
            if let Some(value) = table.get(key) {
                let s = value.as_str().unwrap_or("");
                if s.trim().is_empty() {
                    return Err(ConfigError::Invalid(format!(
                        "build.{profile}.{key} is empty"
                    )));
                }
            }
        }
        if let Some(kind) = table.get("kind").and_then(|v| v.as_str()) {
            let kind = kind.trim();
            if !kind.is_empty() && !VALID_KINDS.contains(&kind) {
                return Err(ConfigError::Invalid(format!(
                    "build.{profile}.kind is invalid"
                )));
            }
        }
        if let Some(src_disable) = table.get("src_disable")
            && !src_disable.is_bool()
        {
            return Err(ConfigError::Invalid(format!(
                "build.{profile}.src_disable must be boolean"
            )));
        }
        for key in [
            "cflags",
            "ldflags",
            "exclude",
            "include",
            "roots",
            "pkg_config",
            "generated",
            "expect",
            "clean",
            "targets",
        ] {
            if let Some(val) = table.get(key) {
                let arr = val.as_array().ok_or_else(|| {
                    ConfigError::Invalid(format!(
                        "build.{profile}.{key} must be an array of strings"
                    ))
                })?;
                for item in arr {
                    let s = item.as_str().unwrap_or("");
                    if s.trim().is_empty() {
                        return Err(ConfigError::Invalid(format!(
                            "build.{profile}.{key} contains empty value"
                        )));
                    }
                }
            }
        }
        for key in ["steps", "post_steps"] {
            if let Some(val) = table.get(key) {
                let arr = val.as_array().ok_or_else(|| {
                    ConfigError::Invalid(format!("build.{profile}.{key} must be an array"))
                })?;
                for item in arr {
                    let tbl = item.as_table().ok_or_else(|| {
                        ConfigError::Invalid(format!(
                            "build.{profile}.{key} entries must be tables"
                        ))
                    })?;
                    for req in ["name", "in", "out", "cmd"] {
                        let s = tbl.get(req).and_then(|v| v.as_str()).unwrap_or("");
                        if s.trim().is_empty() {
                            return Err(ConfigError::Invalid(format!(
                                "build.{profile}.{key} missing {req}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

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
    package.insert("type".to_string(), Value::String("none".to_string()));
    package.insert("description".to_string(), Value::String("".to_string()));
    package.insert("author".to_string(), Value::String("".to_string()));
    package.insert(
        "license".to_string(),
        Value::String("GPL-3.0-or-later".to_string()),
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

    let language_value = build.get("language");
    let language = match language_value {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Array(arr)) => format_string_array(&Value::Array(arr.clone())),
        _ => "".to_string(),
    };
    let standard = build.get("standard").and_then(|v| v.as_str()).unwrap_or("");
    let compiler = build.get("compiler").and_then(|v| v.as_str()).unwrap_or("");

    let mut out = String::new();
    out.push_str("[package]\n");
    out.push_str(&format!("name = \"{name}\"\n"));
    out.push_str(&format!("version = \"{version}\"\n"));

    for key in [
        "type",
        "description",
        "author",
        "authors",
        "homepage",
        "license",
        "repository",
        "readme",
        "keywords",
        "categories",
    ] {
        if let Some(val) = package.get(key) {
            match val {
                Value::String(s) => out.push_str(&format!("{key} = \"{s}\"\n")),
                Value::Array(_) => out.push_str(&format!("{key} = {}\n", format_string_array(val))),
                _ => {}
            }
        }
    }
    out.push('\n');

    out.push_str("[build]\n");
    if language.starts_with('[') {
        out.push_str(&format!("language = {language}\n"));
    } else {
        out.push_str(&format!("language = \"{language}\"\n"));
    }
    if !standard.is_empty() {
        out.push_str(&format!("standard = \"{standard}\"\n"));
    }
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
    if let Some(out_dir) = build.get("out_dir").and_then(|v| v.as_str())
        && !out_dir.trim().is_empty()
    {
        out.push_str(&format!("out_dir = \"{out_dir}\"\n"));
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
            if let Some(v) = tbl.get("version").and_then(|v| v.as_str()) {
                parts.push(format!("version = \"{v}\""));
            }
            if let Some(v) = tbl.get("path").and_then(|v| v.as_str()) {
                parts.push(format!("path = \"{v}\""));
            }
            if let Some(v) = tbl.get("git").and_then(|v| v.as_str()) {
                parts.push(format!("git = \"{v}\""));
            }
            if let Some(v) = tbl.get("branch").and_then(|v| v.as_str()) {
                parts.push(format!("branch = \"{v}\""));
            }
            if let Some(v) = tbl.get("tag").and_then(|v| v.as_str()) {
                parts.push(format!("tag = \"{v}\""));
            }
            if let Some(v) = tbl.get("rev").and_then(|v| v.as_str()) {
                parts.push(format!("rev = \"{v}\""));
            }
            if let Some(v) = tbl.get("default-features").and_then(|v| v.as_bool()) {
                parts.push(format!(
                    "default-features = {}",
                    if v { "true" } else { "false" }
                ));
            }
            if let Some(v) = tbl.get("features") {
                parts.push(format!("features = {}", format_string_array(v)));
            }
            if let Some(v) = tbl.get("system").and_then(|v| v.as_bool()) {
                parts.push(format!("system = {}", if v { "true" } else { "false" }));
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
    fn exposes_typed_config() {
        let dir = temp_dir("typed_config");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"typed\"\nversion = \"1.2.3\"\ntype = \"lib\"\n\n[build]\nlanguage = [\"c\", \"c++\"]\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"staticlib\"\ncflags = [\"-Wall\"]\n\n[dependencies]\nfoo = \"1.0.0\"\n",
        );
        let config = Config::open(&path.to_string_lossy()).unwrap();
        assert_eq!(config.package().unwrap().name, "typed");
        assert_eq!(config.typed().package.as_ref().unwrap().version, "1.2.3");
        assert_eq!(config.build_config().unwrap().compiler, "clang");
        assert_eq!(config.build_config().unwrap().cflags, ["-Wall"]);
        assert!(config.typed().dependencies.contains_key("foo"));
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
    fn validate_wrong_field_type_fails() {
        let dir = temp_dir("wrong_type");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = [\"clang\"]\nkind = \"bin\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(
            result.is_err(),
            "typed config should reject wrong field types"
        );
    }

    #[test]
    fn validate_invalid_package_names_fail() {
        for name in [
            "../evil", "bad/name", "bad name", "", " name", "name ", "a.b",
        ] {
            let dir = temp_dir("bad_name");
            let content = format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\n"
            );
            let path = write_toml_file(&dir, &content);
            let result = Config::open(&path.to_string_lossy());
            assert!(result.is_err(), "package name `{name}` should fail");
        }
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
    fn validate_language_array() {
        let dir = temp_dir("lang_array");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = [\"c\", \"c++\", \"asm\"]\nstandard = \"c11\"\ncompiler = \"clang\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_ok(), "language array should be valid");
    }

    #[test]
    fn validate_language_array_empty_fails() {
        let dir = temp_dir("lang_array_empty");
        let path = write_toml_file(
            &dir,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = []\nstandard = \"c11\"\ncompiler = \"clang\"\n\n[dependencies]\n",
        );
        let result = Config::open(&path.to_string_lossy());
        assert!(result.is_err(), "empty language array should fail");
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
        for &kind in VALID_KINDS {
            let dir = temp_dir("valid_kind");
            let toml = format!(
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"{kind}\"\n\n[dependencies]\n"
            );
            let path = write_toml_file(&dir, &toml);
            assert!(
                Config::open(&path.to_string_lossy()).is_ok(),
                "kind '{kind}' should be valid"
            );
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
