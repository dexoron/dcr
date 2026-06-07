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

use crate::core::config::Config;
use crate::utils::log::warn;

pub struct VersionInfo {
    pub full: String,
    pub major: String,
    pub minor: String,
    pub patch: String,
    pub suffix: String,
    pub suffix_dash: String,
}

pub fn parse_version_info(version: &str) -> VersionInfo {
    let mut full = version.trim().to_string();
    if full.is_empty() {
        full = "0.0.0".to_string();
    }
    let full_clone = full.clone();
    let (base, suffix) = match full_clone.split_once('-') {
        Some((head, tail)) => (head, tail),
        None => (full_clone.as_str(), ""),
    };
    let mut parts = base.split('.');
    let major = parts.next().unwrap_or("0").to_string();
    let minor = parts.next().unwrap_or("0").to_string();
    let patch = parts.next().unwrap_or("0").to_string();
    let suffix_dash = if suffix.is_empty() {
        "".to_string()
    } else {
        format!("-{suffix}")
    };
    VersionInfo {
        full,
        major,
        minor,
        patch,
        suffix: suffix.to_string(),
        suffix_dash,
    }
}

pub fn normalize_target_os(target: &str) -> &str {
    match target {
        "linux" => "x86_64-unknown-linux-gnu",
        "macos" => "x86_64-apple-darwin",
        "windows" => "x86_64-pc-windows-msvc",
        _ if target.contains('-') => target,
        _ => {
            warn(&format!(
                "Unknown target '{}', using as-is. Supported short names: linux, macos, windows",
                target
            ));
            target
        }
    }
}

pub fn default_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    if cfg!(target_os = "linux") {
        let env = if cfg!(target_env = "musl") {
            "musl"
        } else {
            "gnu"
        };
        format!("{arch}-unknown-linux-{env}")
    } else if cfg!(target_os = "macos") {
        format!("{arch}-apple-darwin")
    } else if cfg!(target_os = "windows") {
        let env = if cfg!(target_env = "gnu") {
            "gnu"
        } else {
            "msvc"
        };
        format!("{arch}-pc-windows-{env}")
    } else if cfg!(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        format!("{arch}-unknown-{}", std::env::consts::OS)
    } else {
        "unknown".to_string()
    }
}

pub fn prepend_clang_target_flag(flags: &mut Vec<String>, target: Option<&str>, tool: &str) {
    let Some(target) = target.map(str::trim).filter(|t| !t.is_empty()) else {
        return;
    };
    if !tool.to_lowercase().contains("clang") {
        return;
    }

    let target_flag = format!("--target={target}");
    if !flags
        .iter()
        .any(|f| f == &target_flag || f.starts_with("--target="))
    {
        flags.insert(0, target_flag);
    }
}

pub fn get_config_str(config: &Config, key: &str) -> String {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn profile_table<'a>(config: &'a Config, profile: &str) -> Option<&'a toml::value::Table> {
    config
        .get("build")
        .and_then(|v| v.as_table())
        .and_then(|b| b.get(profile))
        .and_then(|v| v.as_table())
}

pub fn get_config_opt(config: &Config, key: &str) -> Option<String> {
    let value = config.get(key)?.as_str()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn get_string_with_profile(config: &Config, field: &str, profile: &str) -> String {
    let base = get_config_str(config, &format!("build.{field}"));
    let Some(table) = profile_table(config, profile) else {
        return base;
    };
    let value = table.get(field).and_then(|v| v.as_str()).unwrap_or("");
    let trimmed = value.trim();
    if trimmed.is_empty() {
        base
    } else {
        trimmed.to_string()
    }
}

pub fn get_config_list(config: &Config, key: &str) -> Vec<String> {
    config
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_list_with_profile(config: &Config, field: &str, profile: &str) -> Vec<String> {
    let mut out = get_config_list(config, &format!("build.{field}"));
    if let Some(table) = profile_table(config, profile)
        && let Some(extra) = table.get(field).and_then(|v| v.as_array())
    {
        out.extend(
            extra
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string()),
        );
    }
    out
}

pub fn get_bool_with_profile(config: &Config, field: &str, profile: &str, default: bool) -> bool {
    let base = config
        .get(&format!("build.{field}"))
        .and_then(|v| v.as_bool());
    let profile_val =
        profile_table(config, profile).and_then(|t| t.get(field).and_then(|v| v.as_bool()));
    profile_val.or(base).unwrap_or(default)
}

pub fn get_language_with_profile(config: &Config, profile: &str) -> Result<String, String> {
    if let Some(table) = profile_table(config, profile)
        && let Some(value) = table.get("language")
    {
        return parse_language_value(value, "build.language");
    }
    match config.get("build.language") {
        Some(v) => parse_language_value(v, "build.language"),
        None => Err("build.language is missing".to_string()),
    }
}

pub fn get_language_with_profile_or_default(config: &Config, profile: &str) -> String {
    get_language_with_profile(config, profile).unwrap_or_else(|_| "c".to_string())
}

pub fn parse_language_value(value: &toml::Value, key: &str) -> Result<String, String> {
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(format!("{key} is empty"));
        }
        return Ok(trimmed.to_string());
    }
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be string or array of strings"))?;
    let mut parts = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| format!("{key} must be string or array of strings"))?;
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(format!("{key} contains empty value"));
        }
        parts.push(trimmed.to_string());
    }
    if parts.is_empty() {
        return Err(format!("{key} is empty"));
    }
    Ok(parts.join(","))
}

pub fn resolve_compiler(
    language: &str,
    compiler: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> String {
    let lang = primary_language(language);
    env_override_compiler(&lang)
        .or_else(|| toolchain_override_compiler(&lang, tc_cc, tc_cxx, tc_as))
        .unwrap_or_else(|| compiler.to_string())
}

fn env_override_compiler(lang: &str) -> Option<String> {
    if let Ok(value) = std::env::var("DCR_COMPILER") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if lang == "asm" {
        if let Ok(value) = std::env::var("DCR_AS") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        return None;
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Ok(value) = std::env::var("DCR_CXX")
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(value) = std::env::var("DCR_CC") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn toolchain_override_compiler(
    lang: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> Option<String> {
    if lang == "asm" {
        return tc_as.map(|v| v.to_string());
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Some(v) = tc_cxx
    {
        return Some(v.to_string());
    }
    tc_cc.map(|v| v.to_string())
}

pub fn primary_language(language: &str) -> String {
    let parts: Vec<String> = language
        .split(',')
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    for p in &parts {
        if p == "c++" || p == "cpp" || p == "cxx" {
            return p.clone();
        }
    }
    if parts.iter().any(|p| p == "c") {
        return "c".to_string();
    }
    if parts.iter().any(|p| p == "asm") {
        return "asm".to_string();
    }
    language.to_lowercase()
}

pub fn resolve_tool(env_key: &str, fallback: Option<&str>) -> Option<String> {
    if let Ok(value) = std::env::var(env_key) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    fallback.map(|v| v.to_string())
}

pub fn resolve_pkg_config_flags(
    pkgs: &[String],
    base_cflags: &[String],
    base_ldflags: &[String],
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut cflags = base_cflags.to_vec();
    let mut ldflags = base_ldflags.to_vec();
    for pkg in pkgs {
        let c_out = run_pkg_config(pkg, "--cflags")?;
        let l_out = run_pkg_config(pkg, "--libs")?;
        cflags.extend(split_flags(&c_out));
        ldflags.extend(split_flags(&l_out));
    }
    Ok((cflags, ldflags))
}

pub fn resolve_pkg_config_flags_lossy(
    pkgs: &[String],
    base_cflags: &[String],
    base_ldflags: &[String],
) -> (Vec<String>, Vec<String>) {
    match resolve_pkg_config_flags(pkgs, base_cflags, base_ldflags) {
        Ok(flags) => flags,
        Err(err) => {
            eprintln!("Warning: {err}");
            (base_cflags.to_vec(), base_ldflags.to_vec())
        }
    }
}

fn run_pkg_config(pkg: &str, arg: &str) -> Result<String, String> {
    let output = std::process::Command::new("pkg-config")
        .arg(arg)
        .arg(pkg)
        .output()
        .map_err(|err| format!("Failed to run pkg-config: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pkg-config failed for {pkg}: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn split_flags(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(ch) = chars.next() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_target_short_names() {
        assert_eq!(normalize_target_os("linux"), "x86_64-unknown-linux-gnu");
        assert_eq!(normalize_target_os("macos"), "x86_64-apple-darwin");
        assert_eq!(normalize_target_os("windows"), "x86_64-pc-windows-msvc");
        assert_eq!(
            normalize_target_os("x86_64-unknown-linux-gnu"),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(normalize_target_os("unknown"), "unknown");
    }

    #[test]
    fn default_target_triple_uses_host_arch() {
        let target = default_target_triple();
        if cfg!(target_os = "linux") {
            assert_eq!(
                target,
                format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
            );
        } else if cfg!(target_os = "macos") {
            assert_eq!(target, format!("{}-apple-darwin", std::env::consts::ARCH));
        } else if cfg!(target_os = "windows") {
            assert_eq!(
                target,
                format!("{}-pc-windows-msvc", std::env::consts::ARCH)
            );
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        )) {
            assert_eq!(
                target,
                format!(
                    "{}-unknown-{}",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                )
            );
        } else {
            assert_eq!(target, "unknown");
        }
    }

    #[test]
    fn prepend_clang_target_flag_adds_once_for_clang() {
        let mut flags = vec!["-O2".to_string()];
        prepend_clang_target_flag(&mut flags, Some("aarch64-apple-darwin"), "clang");
        prepend_clang_target_flag(&mut flags, Some("aarch64-apple-darwin"), "clang");
        assert_eq!(
            flags,
            vec![
                "--target=aarch64-apple-darwin".to_string(),
                "-O2".to_string()
            ]
        );
    }

    #[test]
    fn prepend_clang_target_flag_ignores_non_clang_tools() {
        let mut flags = vec!["-O2".to_string()];
        prepend_clang_target_flag(&mut flags, Some("aarch64-apple-darwin"), "gcc");
        assert_eq!(flags, vec!["-O2".to_string()]);
    }

    #[test]
    fn parse_version_parts() {
        let info = parse_version_info("1.2.3-beta");
        assert_eq!(info.full, "1.2.3-beta");
        assert_eq!(info.major, "1");
        assert_eq!(info.minor, "2");
        assert_eq!(info.patch, "3");
        assert_eq!(info.suffix, "beta");
        assert_eq!(info.suffix_dash, "-beta");
    }
}
