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

use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use glob::glob;
use std::process::Command;

pub fn fmt(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc("    dcr fmt", BOLD_CYAN);
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Formats all C/C++ source files using clang-format.");
        println!("    Scans src/ and tests/ directories recursively.");
        return 0;
    }

    let patterns = [
        "src/**/*.c",
        "src/**/*.cpp",
        "src/**/*.cxx",
        "src/**/*.cc",
        "src/**/*.h",
        "src/**/*.hpp",
        "src/**/*.hxx",
        "src/**/*.hh",
        "tests/**/*.c",
        "tests/**/*.cpp",
        "tests/**/*.cxx",
        "tests/**/*.cc",
        "tests/**/*.h",
        "tests/**/*.hpp",
        "tests/**/*.hxx",
        "tests/**/*.hh",
    ];

    let mut files = Vec::new();
    for pattern in patterns {
        if let Ok(paths) = glob(pattern) {
            for path in paths.flatten() {
                if let Some(s) = path.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }

    if files.is_empty() {
        println!("    {} No files to format", colored("Format", BOLD_GREEN));
        return 0;
    }

    println!(
        "    {} {} files",
        colored("Formatting", BOLD_GREEN),
        files.len()
    );

    let status = Command::new("clang-format").arg("-i").args(&files).status();

    match status {
        Ok(s) if s.success() => {
            println!("    {} successful", colored("Format", BOLD_GREEN));
            0
        }
        Ok(s) => {
            error(&format!("clang-format failed with status: {}", s));
            1
        }
        Err(e) => {
            error(&format!("Failed to execute clang-format: {}", e));
            1
        }
    }
}
