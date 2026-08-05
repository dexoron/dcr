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

use crate::core::build_config::Config;
use crate::utils::build::{get_config_str, normalize_target_os, profile_table};

/// Retrieves a raw TOML value from the config using a prioritized lookup order based on target, profile, and base sections.
///
/// The keys are checked in this order:
/// {section}.{target}.{profile}.{field}, {section}.{profile}.{target}.{field}, {section}.{target}.{field}, {section}.{profile}.{field}, {section}.{field}
///
/// Returns the value if found, otherwise None.
pub(crate) fn get_config_value_raw(
    config: &Config,
    section: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Option<toml::Value> {
    // Order: target.profile, profile.target, target, profile, base
    let keys = [
        target.map(|t| {
            format!(
                "{}.{}.{}.{}",
                section,
                normalize_target_os(t),
                profile,
                field
            )
        }),
        target.map(|t| {
            format!(
                "{}.{}.{}.{}",
                section,
                profile,
                normalize_target_os(t),
                field
            )
        }),
        target.map(|t| format!("{}.{}.{}", section, normalize_target_os(t), field)),
        Some(format!("{}.{}.{}", section, profile, field)),
        Some(format!("{}.{}", section, field)),
    ];
    for key in keys.into_iter().flatten() {
        if let Some(val) = config.get(&key) {
            return Some(val.clone());
        }
    }
    None
}

/// Checks if the 'inherit' flag is enabled for the given section, profile, and target.
///
/// Returns true by default if the flag is not set.
pub(crate) fn get_inherit(
    config: &Config,
    section: &str,
    profile: &str,
    target: Option<&str>,
) -> bool {
    get_config_value_raw(config, section, "inherit", profile, target)
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// Extracts a trimmed non-empty string value from the config.
///
/// Returns Some(value) if found, None otherwise.
pub(crate) fn get_config_value(
    config: &Config,
    section: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Option<String> {
    get_config_value_raw(config, section, field, profile, target)
        .and_then(|v| v.as_str().map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
}

/// Fetches a string value from the build section, inheriting from base if enabled.
///
/// Returns the value or default empty string if not inheriting.
pub(crate) fn get_string_with_profile_and_target(
    config: &Config,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> String {
    if get_inherit(config, "build", profile, target) {
        get_config_value(config, "build", field, profile, target)
            .unwrap_or_else(|| get_config_str(config, &format!("build.{field}")))
    } else {
        get_config_value(config, "build", field, profile, target).unwrap_or_default()
    }
}

/// Gets a build string value for the given profile without a target.
pub fn get_build_string_with_profile(config: &Config, field: &str, profile: &str) -> String {
    get_string_with_profile_and_target(config, field, profile, None)
}

/// Gets a string value from the language-specific build.{lang} section.
pub(crate) fn get_lang_string(
    config: &Config,
    lang: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Option<String> {
    get_config_value(config, &format!("build.{lang}"), field, profile, target)
}

/// Gets a list of strings from the language-specific build.{lang} section.
pub(crate) fn get_lang_list(
    config: &Config,
    lang: &str,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Result<Vec<String>, String> {
    match get_config_value_raw(config, &format!("build.{lang}"), field, profile, target) {
        Some(val) => parse_string_array(&val, field),
        None => Ok(Vec::new()),
    }
}

/// Parses a TOML array into a Vec<String>, erroring if not an array of strings.
pub(crate) fn parse_string_array(value: &toml::Value, key: &str) -> Result<Vec<String>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array of strings"))?;
    let mut out = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| format!("{key} must be an array of strings"))?;
        out.push(s.to_string());
    }
    Ok(out)
}

/// Collects a list of strings from config, supporting inheritance from base and profile/target overrides.
///
/// If inherit is true, starts with the base list and appends custom entries if defined.
pub(crate) fn get_list_with_profile_and_target(
    config: &Config,
    field: &str,
    profile: &str,
    target: Option<&str>,
) -> Result<Vec<String>, String> {
    let inherit = get_inherit(config, "build", profile, target);
    let mut out = if inherit {
        get_config_list(config, &format!("build.{field}"))?
    } else {
        Vec::new()
    };
    if let Some(val) = get_config_value_raw(config, "build", field, profile, target) {
        if let Some(_arr) = val.as_array() {
            let is_base = inherit
                && config
                    .get(&format!("build.{field}"))
                    .map(|v| v == &val)
                    .unwrap_or(false);
            // Skip re-adding the base list when this is the base definition to prevent duplication
            if !is_base {
                let custom = parse_string_array(&val, &format!("build.{field}"))?;
                if inherit {
                    out.extend(custom);
                } else {
                    out = custom;
                }
            }
        }
    } else if inherit {
        if let Some(table) = profile_table(config, profile)
            && let Some(value) = table.get(field)
        {
            let mut extra = parse_string_array(value, &format!("build.{profile}.{field}"))?;
            out.append(&mut extra);
        }
        if let Some(t) = target {
            let normalized_t = normalize_target_os(t);
            if let Some(table) = profile_table(config, normalized_t)
                && let Some(value) = table.get(field)
            {
                let mut extra =
                    parse_string_array(value, &format!("build.{normalized_t}.{field}"))?;
                out.append(&mut extra);
            }
        }
    }
    Ok(out)
}

/// Gets the list of strings for the given profile without a target.
pub(crate) fn get_list_with_profile(
    config: &Config,
    field: &str,
    profile: &str,
) -> Result<Vec<String>, String> {
    get_list_with_profile_and_target(config, field, profile, None)
}

/// Returns the list of targets for the given profile.
pub(crate) fn get_targets(config: &Config, profile: &str) -> Result<Vec<String>, String> {
    get_list_with_profile(config, "targets", profile)
}

/// Parses the config value as a list of strings.
///
/// # Parameters
/// - `config`: Project config.
/// - `key`: Dot-path key (e.g. `build.sources`).
///
/// # Returns
/// - `Ok([])` if the key is missing.
/// - `Ok(vec)` for an array of strings.
/// - `Err` if present but not an array of strings.
pub(crate) fn get_config_list(config: &Config, key: &str) -> Result<Vec<String>, String> {
    let value = match config.get(key) {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array of strings"))?;
    let mut out = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| format!("{key} must be an array of strings"))?;
        out.push(s.to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::utils::build::normalize_target_os;

    #[test]
    fn test_normalize_target_os() {
        assert_eq!(normalize_target_os("linux"), "x86_64-unknown-linux-gnu");
        assert_eq!(normalize_target_os("macos"), "x86_64-apple-darwin");
        assert_eq!(normalize_target_os("windows"), "x86_64-pc-windows-msvc");
        assert_eq!(
            normalize_target_os("x86_64-unknown-linux-gnu"),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(normalize_target_os("unknown"), "unknown");
    }
}
