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

use super::path_util::{default_bin_rel, default_lib_rel, join_dir, with_exe_suffix};

pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = with_exe_suffix(name);
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_bin_rel(profile, &file, None),
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = if name.to_ascii_lowercase().ends_with(".lib") {
        name.to_string()
    } else {
        format!("{name}.lib")
    };
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = if name.to_ascii_lowercase().ends_with(".efi") {
        name.to_string()
    } else {
        format!("{name}.efi")
    };
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = if name.to_ascii_lowercase().ends_with(".dll") {
        name.to_string()
    } else {
        format!("{name}.dll")
    };
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}
