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

/// macOS-specific utilities for constructing paths to build artifacts.
///
/// This module defines functions to generate paths for binaries, static
/// libraries, shared libraries, ELF files, and EFI executables on macOS.
use super::path_util::{default_bin_rel, default_lib_rel, join_dir};

/// Constructs the path to a binary for the given profile, name, and optional target directory.
pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => join_dir(dir, name),
        None => default_bin_rel(profile, name, None),
    }
}

/// Constructs the path to a static library archive for the given profile, name, and optional target directory.
pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.a");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

/// Constructs the path to an ELF binary for the given profile, name, and optional target directory.
pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

/// Constructs the path to an EFI executable for the given profile, name, and optional target directory.
pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("{name}.efi");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

/// Constructs the path to a shared library for the given profile, name, and optional target directory.
pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.dylib");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}
