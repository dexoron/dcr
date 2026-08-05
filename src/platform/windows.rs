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

/// Returns the path to the binary executable for the given profile, name, and optional target directory.
pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = with_exe_suffix(name);
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_bin_rel(profile, &file, None),
    }
}

/// Returns the path to the library file for the given profile, name, and optional target directory.
/// The library name is forced to end with ".lib".
pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    // Ensure the library file name ends with ".lib"
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

/// Returns the path to the ELF binary for the given profile, name, and optional target directory.
pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

/// Returns the path to the EFI binary for the given profile, name, and optional target directory.
/// The binary name is forced to end with ".efi".
pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    // Ensure the EFI file name ends with ".efi"
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

/// Returns the path to the shared library (DLL) for the given profile, name, and optional target directory.
/// The library name is forced to end with ".dll".
pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    // Ensure the shared library file name ends with ".dll"
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
