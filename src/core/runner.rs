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

use crate::platform;
use std::path::Path;
use std::process::Command;

/// Runs the built binary for a project, forwarding `bin_args`.
///
/// # Parameters
/// - `project_name`: Package/binary name used to resolve the artifact path.
/// - `profile`: Build profile segment under `target/`.
/// - `target_dir`: Optional artifact directory override.
/// - `bin_args`: Arguments passed to the process.
///
/// # Returns
/// Child exit code, or `1` if the binary is missing or cannot be spawned.
pub fn run_binary(
    project_name: &str,
    profile: &str,
    target_dir: Option<&str>,
    bin_args: &[String],
) -> i32 {
    let bin_path = platform::bin_path(profile, project_name, target_dir);
    // Check if the binary exists before attempting to run it
    if Path::new(&bin_path).exists() {
        let status = Command::new(&bin_path).args(bin_args).status();
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
