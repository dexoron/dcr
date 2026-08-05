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

use super::path_util::{default_bin_rel, default_lib_rel, join_dir};
use crate::utils::build::default_target_triple;

/// Returns the host target triple used for default artifact paths on BSD.
fn host_triple() -> String {
    default_target_triple()
}

/// Returns the path to a binary for the given profile, name, and optional target directory.
pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => join_dir(dir, name),
        None => default_bin_rel(profile, name, Some(&host_triple())),
    }
}

/// Returns the path to a static library for the given profile, name, and optional target directory.
pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.a");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, Some(&host_triple())),
    }
}

/// Returns the path to an ELF binary for the given profile, name, and optional target directory.
pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

/// Returns the path to an EFI binary for the given profile, name, and optional target directory.
pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("{name}.efi");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, Some(&host_triple())),
    }
}

/// Returns the path to a shared library for the given profile, name, and optional target directory.
pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.so");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, Some(&host_triple())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_path_default() {
        let expected = default_bin_rel("debug", "hello", Some(&host_triple()));
        assert_eq!(bin_path("debug", "hello", None), expected);
    }

    #[test]
    fn lib_path_default() {
        let expected = default_lib_rel("debug", "libmylib.a", Some(&host_triple()));
        assert_eq!(lib_path("debug", "mylib", None), expected);
    }

    #[test]
    fn shared_lib_path_default() {
        let expected = default_lib_rel("debug", "libmylib.so", Some(&host_triple()));
        assert_eq!(shared_lib_path("debug", "mylib", None), expected);
    }
}
