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
use crate::core::deps::register;
use crate::utils::fs::find_project_root;
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use toml::Value;
use toml::map::Map;

pub struct AddArgs {
    pub name: String,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub version_from_registry: Option<String>,
}

pub fn add(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr add <name> <source> [--branch <b>] [--tag <t>] [--rev <r>]",
            BOLD_CYAN,
        );
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Adds a dependency to the project.");
        println!();
        printc("SOURCES:", BOLD_GREEN);
        println!("    github:user/repo          GitHub repository");
        println!("    gitlab:user/repo          GitLab repository");
        println!("    git:host.com/user/repo    Generic git repository");
        println!("    path:./path/to/lib        Local path dependency");
        println!("    <version>                 Version from registry");
        println!();
        printc("OPTIONS:", BOLD_GREEN);
        println!("    --branch <b>    Use a specific branch");
        println!("    --tag <t>       Use a specific tag");
        println!("    --rev <r>       Use a specific commit hash");
        return 0;
    }

    let add_args = match parse_add_args(args) {
        Ok(a) => a,
        Err(code) => return code,
    };

    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };

    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => dir,
        Ok(None) => {
            error("dcr.toml file not found");
            return 1;
        }
        Err(_) => {
            error("Failed to find project root");
            return 1;
        }
    };

    let mut config = match Config::open(&root.join("dcr.toml").to_string_lossy()) {
        Ok(cfg) => cfg,
        Err(err) => {
            error(&err.to_string());
            return 1;
        }
    };

    let dep_value = if let Some(git_url) = add_args.git {
        if add_args.branch.is_none() && add_args.tag.is_none() && add_args.rev.is_none() {
            let mut table = Map::new();
            table.insert("git".to_string(), Value::String(git_url));
            Value::Table(table)
        } else {
            let mut table = Map::new();
            table.insert("git".to_string(), Value::String(git_url));
            if let Some(b) = add_args.branch {
                table.insert("branch".to_string(), Value::String(b));
            }
            if let Some(t) = add_args.tag {
                table.insert("tag".to_string(), Value::String(t));
            }
            if let Some(r) = add_args.rev {
                table.insert("rev".to_string(), Value::String(r));
            }
            Value::Table(table)
        }
    } else if let Some(path) = add_args.path {
        let mut table = Map::new();
        table.insert("path".to_string(), Value::String(path));
        Value::Table(table)
    } else if let Some(version) = add_args.version_from_registry {
        Value::String(version)
    } else {
        error("Dependency source (path or git) must be provided");
        return 1;
    };

    let key = format!("dependencies.{}", add_args.name);
    if let Err(err) = config.edit(&key, dep_value) {
        error(&format!("Failed to update dcr.toml: {}", err));
        return 1;
    }

    println!(
        "    {} dependency `{}` to dcr.toml",
        colored("Added", BOLD_GREEN),
        add_args.name
    );
    0
}

fn parse_add_args(args: &[String]) -> Result<AddArgs, i32> {
    if args.is_empty() {
        error("Usage: dcr add <name> <source> [--branch <branch> | --tag <tag> | --rev <rev>]");
        error("Sources:");
        error("  github:user/repo          -> github.com/user/repo");
        error("  gitlab:user/repo          -> gitlab.com/user/repo");
        error("  git:host.com/user/repo    -> host.com/user/repo");
        error("  path:./path/to/lib        -> local path");
        return Err(1);
    }

    let mut name = String::new();
    let source_spec: String;
    let mut branch = None;
    let mut tag = None;
    let mut rev = None;

    let mut iter = args.iter();
    if let Some(n) = iter.next() {
        if n.starts_with("--") {
            error("First argument must be the dependency name");
            return Err(1);
        }
        name = n.clone();
    }

    if let Some(s) = iter.next() {
        if s.starts_with("--") {
            error("Second argument must be the dependency source");
            return Err(1);
        }
        source_spec = s.clone();
    } else {
        // No source provided — try registry auto-lookup
        match register::resolve_package_from_registry(&name) {
            Ok(info) => {
                let version = info
                    .get("latest_version")
                    .or_else(|| info.get("version"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        (
                            "Registry entry for `".to_string() + &name + "` has no version field",
                            1,
                        )
                    })
                    .map_err(|(msg, code)| {
                        error(&msg);
                        code
                    })?;
                return Ok(AddArgs {
                    name,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    version_from_registry: Some(version.to_string()),
                });
            }
            Err(e) => {
                error(&format!("Cannot add `{}` without source: {}", name, e));
                return Err(1);
            }
        }
    }

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--branch" => {
                branch = iter.next().cloned();
                if branch.is_none() {
                    error("--branch requires a value");
                    return Err(1);
                }
            }
            "--tag" => {
                tag = iter.next().cloned();
                if tag.is_none() {
                    error("--tag requires a value");
                    return Err(1);
                }
            }
            "--rev" => {
                rev = iter.next().cloned();
                if rev.is_none() {
                    error("--rev requires a value");
                    return Err(1);
                }
            }
            _ => {
                error(&format!("Unknown argument: {}", arg));
                return Err(1);
            }
        }
    }

    let mut path = None;
    let mut git = None;

    if let Some(p) = source_spec.strip_prefix("path:") {
        path = Some(p.to_string());
    } else if let Some(g) = source_spec.strip_prefix("github:") {
        git = Some(format!("https://github.com/{}", g));
    } else if let Some(g) = source_spec.strip_prefix("gitlab:") {
        git = Some(format!("https://gitlab.com/{}", g));
    } else if let Some(g) = source_spec.strip_prefix("git:") {
        if g.contains('/') && !g.contains('.') {
            // git:user/repo -> default to github
            git = Some(format!("https://github.com/{}", g));
        } else {
            // git:host.com/user/repo
            let url = if g.starts_with("http") || g.starts_with("git@") {
                g.to_string()
            } else {
                format!("https://{}", g)
            };
            git = Some(url);
        }
    } else if source_spec.starts_with("http://")
        || source_spec.starts_with("https://")
        || source_spec.starts_with("git@")
    {
        git = Some(source_spec);
    } else {
        error("Source must have a prefix (path:, git:, github:, gitlab:) or be a full URL");
        return Err(1);
    }

    Ok(AddArgs {
        name,
        path,
        git,
        branch,
        tag,
        rev,
        version_from_registry: None,
    })
}
