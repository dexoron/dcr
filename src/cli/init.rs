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

use crate::prelude::*;

use crate::config::FILE_MAIN_C;
use crate::core::build_config::{Config, validate_package_name};
use crate::core::vcs::VcsKind;
use crate::utils::cli_styles::{
    BOLD_CYAN, BOLD_GREEN, Colorize, HELP_EXAMPLES_ST, HELP_SECTION_TITLE_ST, SUCCESS_ST,
};
use crate::utils::fs::check_dir;
use crate::utils::log::sprintln;
use std::{fs, io::Write};

/// Initializes the current working directory as a new DCR project.
///
/// Verifies that the current directory is empty, sets up the project structure by creating
/// `dcr.toml` and `src/main.c`, and configures version control (Git) integration if applicable.
pub fn init(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        sprintln!(HELP_SECTION_TITLE_ST, "USAGE:");
        sprintln!(HELP_EXAMPLES_ST, "    dcr init [--vcs <git|none>]");
        println!();
        sprintln!(HELP_SECTION_TITLE_ST, "DESCRIPTION:");
        println!("    Initializes the current directory as a DCR project.");
        println!("    The directory must be empty.");
        return 0;
    }

    // Parse options (--vcs) and collect positional arguments.
    let mut vcs_str = None;
    let mut clean_args = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--vcs" {
            if let Some(val) = iter.next() {
                vcs_str = Some(val.clone());
            } else {
                error!("--vcs requires a value");
                return 1;
            }
        } else if let Some(stripped) = arg.strip_prefix("--vcs=") {
            vcs_str = Some(stripped.to_string());
        } else {
            clean_args.push(arg.clone());
        }
    }

    if !clean_args.is_empty() {
        warn!("Command does not support additional arguments");
        return 1;
    }

    // Resolve target project name from current directory name and enforce empty directory check.
    let items = check_dir(None).unwrap_or_default();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    if !items.is_empty() {
        error!("Directory not empty");
        return 1;
    }

    if let Err(e) = validate_package_name(&project_name) {
        error!(
            "Invalid project name `{}`: {e}",
            project_name.style(BOLD_CYAN),
        );
        return 1;
    }

    let cwd = std::env::current_dir()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    println!("Initializing the project in {cwd}");

    // Create and initialize default manifest (dcr.toml).
    if Config::create("./dcr.toml", Some(&project_name)).is_err() {
        error!("Failed to create dcr.toml");
        return 1;
    }
    println!(
        "    {} Created file {}",
        "✔".style(SUCCESS_ST),
        "dcr.toml".style(BOLD_CYAN)
    );

    // Create src directory and write template main.c file.
    if fs::create_dir("src").is_err() {
        error!("Failed to create src/");
        return 1;
    }
    let mut main_c = match fs::File::create("./src/main.c") {
        Ok(file) => file,
        Err(_) => {
            error!("Failed to create src/main.c");
            return 1;
        }
    };
    if main_c.write_all(FILE_MAIN_C.as_bytes()).is_err() {
        error!("Failed to write src/main.c");
        return 1;
    }
    println!(
        "    {} Created file {}",
        "✔".style(SUCCESS_ST),
        "src/main.c".style(BOLD_CYAN)
    );

    // Resolve target VCS provider or detect existing repositories.
    let mut vcs_kind = VcsKind::Git;
    if let Some(ref vcs_val) = vcs_str {
        match VcsKind::parse(vcs_val) {
            Ok(kind) => vcs_kind = kind,
            Err(e) => {
                error!("{e}");
                return 1;
            }
        }
    } else if let Ok(cwd_path) = std::env::current_dir()
        && crate::core::vcs::find_existing_vcs(&cwd_path).is_some()
    {
        vcs_kind = VcsKind::None;
    }

    if vcs_kind == VcsKind::Git {
        if crate::utils::git::is_git_available() {
            let project_path = std::path::Path::new(".");
            if let Err(e) = crate::core::vcs::init_vcs(vcs_kind, project_path) {
                warn!("Failed to initialize git repository: {e}");
            } else {
                println!("    {} Initialized git repository", "✔".style(SUCCESS_ST));
            }
        } else {
            warn!(
                "Git is not installed or not found in PATH. Skipping Git repository initialization.",
            );
        }
    }

    if let Err(e) = crate::utils::fs::ensure_gitignore_has_dcr(std::path::Path::new(".")) {
        warn!("Failed to update .gitignore: {e}");
    }

    println!(
        "Project `{}` successfully created\n",
        project_name.style(SUCCESS_ST)
    );
    sprintln!(BOLD_GREEN, "Next step:");
    sprintln!(BOLD_CYAN, "    dcr run");
    0
}
