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

use std::{path::Path, process::Command};

/// Runs the built binary for a project, forwarding `bin_args`.
///
/// # Parameters
/// - `bin_path`: Resolved path of the artifact to execute (honours
///   `build.filename` / `build.extension` and the target/profile).
/// - `bin_args`: Arguments passed to the process.
///
/// # Returns
/// Child exit code, or `1` if the binary is missing or cannot be spawned.
pub fn run_binary(bin_path: &str, bin_args: &[String]) -> i32 {
    // Check if the binary exists before attempting to run it
    if Path::new(bin_path).exists() {
        let status = Command::new(bin_path).args(bin_args).status();
        match status {
            Ok(s) => {
                return s.code().unwrap_or(0);
            }
            Err(_) => {
                return 1;
            }
        }
    }
    // Return 1 if binary not found or execution failed
    1
}
