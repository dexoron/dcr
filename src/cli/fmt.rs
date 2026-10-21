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

use glob::glob;
use std::process::Command;

/// Formats all C/C++ source files using clang-format.
///
/// Scans the `src/` and `tests/` directories recursively for files with common
/// C/C++ extensions and formats them in place with clang-format.
///
/// If the first argument is `--help`, prints usage information and returns 0.
pub fn fmt(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        let mut help = crate::utils::help::HelpBuilder::new("dcr fmt");
        help.section("USAGE").example("dcr fmt");
        help.description("Formats all C/C++ source files using clang-format.")
            .note("Scans src/ and tests/ directories recursively.");
        help.render();
        return 0;
    }

    // Collect all matching files from src/ and tests/ using glob patterns
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
        crate::utils::output::status("Format", "no files to format");
        return 0;
    }

    // Format the collected files
    crate::utils::output::status("Formatting", format!("{} files", files.len()));

    let status = Command::new("clang-format").arg("-i").args(&files).status();

    match status {
        Ok(s) if s.success() => {
            crate::utils::output::success("Format successful");
            0
        }
        Ok(s) => {
            error!("clang-format failed with status: {s}");
            1
        }
        Err(e) => {
            error!("Failed to execute clang-format: {e}");
            1
        }
    }
}
