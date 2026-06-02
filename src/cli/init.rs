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

use crate::config::FILE_MAIN_C;
use crate::core::config::{validate_package_name, Config};
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use std::fs;
use std::io::Write;
use toml::Value;

pub fn init(args: &[String]) -> i32 {
    if !args.is_empty() {
        warn("Command does not support additional arguments");
        return 1;
    }

    let items = check_dir(None).unwrap_or_default();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    if !items.is_empty() {
        error("Directory not empty");
        return 1;
    }

    if let Err(e) = validate_package_name(&project_name) {
        error(&format!(
            "Invalid project name `{}`: {}",
            colored(&project_name, BOLD_CYAN),
            e
        ));
        return 1;
    }

    let cwd = std::env::current_dir()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    println!("Initializing the project in {cwd}");

    let mut config = match Config::new("./dcr.toml") {
        Ok(cfg) => cfg,
        Err(_) => {
            error("Failed to create dcr.toml");
            return 1;
        }
    };
    if config
        .edit("package.name", Value::String(project_name.clone()))
        .is_err()
    {
        error("Failed to write dcr.toml");
        return 1;
    }
    println!(
        "    {} Created file {}",
        colored("✔", BOLD_GREEN),
        colored("dcr.toml", BOLD_CYAN)
    );

    if fs::create_dir("src").is_err() {
        error("Failed to create src/");
        return 1;
    }
    let mut main_c = match fs::File::create("./src/main.c") {
        Ok(file) => file,
        Err(_) => {
            error("Failed to create src/main.c");
            return 1;
        }
    };
    if main_c.write_all(FILE_MAIN_C.as_bytes()).is_err() {
        error("Failed to write src/main.c");
        return 1;
    }
    println!(
        "    {} Created file {}",
        colored("✔", BOLD_GREEN),
        colored("src/main.c", BOLD_CYAN)
    );

    println!(
        "Project `{}` successfully created\n",
        colored(project_name.as_str(), BOLD_GREEN)
    );
    printc("Next step:", BOLD_GREEN);
    printc("    dcr run", BOLD_CYAN);
    0
}
