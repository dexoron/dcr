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

pub fn run_binary(project_name: &str, profile: &str, target_dir: Option<&str>) -> i32 {
    let bin_path = platform::bin_path(profile, project_name, target_dir);
    if Path::new(&bin_path).exists() {
        let status = Command::new(&bin_path).status();
        match status {
            Ok(s) => {
                return s.code().unwrap_or(0);
            }
            Err(_) => {
                return 1;
            }
        }
    }
    1
}
