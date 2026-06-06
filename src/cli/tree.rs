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
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, printc};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use toml::Value;

pub fn tree(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc("    dcr tree", BOLD_CYAN);
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Displays the dependency tree of the current project.");
        return 0;
    }

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config = match Config::open("dcr.toml") {
        Ok(c) => c,
        Err(_) => {
            error("dcr.toml not found in current directory");
            return 1;
        }
    };

    let name = config
        .get("package.name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");

    printc(&format!("{} v{}", name, version), BOLD_CYAN);

    let mut seen = HashSet::new();
    seen.insert(name.to_string());

    if let Some(deps) = config.get("dependencies").and_then(|v| v.as_table()) {
        print_deps(deps, &current_dir, "", &mut seen);
    }

    0
}

fn print_deps(
    deps: &toml::value::Table,
    base_path: &Path,
    prefix: &str,
    seen: &mut HashSet<String>,
) {
    let mut dep_list: Vec<_> = deps.iter().collect();
    dep_list.sort_by_key(|(name, _)| *name);

    for (i, (name, value)) in dep_list.iter().enumerate() {
        let is_last = i == dep_list.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let dep_path = resolve_dep_path(value, base_path);
        let mut version = String::new();
        let mut sub_deps = None;
        let mut resolved_path = None;

        if let Some(path) = &dep_path {
            let dcr_toml = path.join("dcr.toml");
            if dcr_toml.exists()
                && let Ok(config) = Config::open(&dcr_toml.to_string_lossy())
            {
                if let Some(v) = config.get("package.version").and_then(|v| v.as_str()) {
                    version = format!(" v{}", v);
                }
                sub_deps = config
                    .get("dependencies")
                    .and_then(|v| v.as_table())
                    .cloned();
                resolved_path = Some(path.clone());
            }
        }

        if version.is_empty() {
            version = match value {
                Value::String(s) if s.starts_with("git:") || s.starts_with("github:") => {
                    format!(" ({})", s)
                }
                Value::Table(t) => {
                    if let Some(v) = t.get("version").and_then(|v| v.as_str()) {
                        format!(" v{}", v)
                    } else if let Some(git) = t.get("git").and_then(|v| v.as_str()) {
                        format!(" ({})", git)
                    } else {
                        "".to_string()
                    }
                }
                _ => "".to_string(),
            };
        }

        println!("{}{}{}{}", prefix, connector, name, version);

        if let Some(s_deps) = sub_deps
            && !seen.contains(*name)
        {
            seen.insert(name.to_string());
            let new_prefix = format!("{}{}", prefix, child_prefix);
            if let Some(path) = resolved_path {
                print_deps(&s_deps, &path, &new_prefix, seen);
            }
            seen.remove(*name);
        }
    }
}

fn resolve_dep_path(value: &Value, base_path: &Path) -> Option<PathBuf> {
    match value {
        Value::String(s) => s
            .strip_prefix("path:")
            .map(|stripped| base_path.join(stripped)),
        Value::Table(t) => t
            .get("path")
            .and_then(|v| v.as_str())
            .map(|path| base_path.join(path)),
        _ => None,
    }
}
