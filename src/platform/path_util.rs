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

use std::path::{Path, PathBuf};

pub fn join_dir(dir: &str, file: &str) -> String {
    let trimmed = dir.trim_end_matches(['/', '\\']);
    Path::new(trimmed).join(file).to_string_lossy().into_owned()
}

#[cfg_attr(not(windows), allow(dead_code))]
pub fn with_exe_suffix(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".exe")
        || lower.ends_with(".dll")
        || lower.ends_with(".sys")
        || lower.ends_with(".com")
        || lower.ends_with(".efi")
        || Path::new(name).extension().is_some()
    {
        name.to_string()
    } else {
        format!("{name}.exe")
    }
}

pub fn default_bin_rel(profile: &str, name: &str, triple_segment: Option<&str>) -> String {
    let mut p = PathBuf::from("./target");
    if let Some(t) = triple_segment {
        p = p.join(t);
    }
    p.join(profile).join(name).to_string_lossy().into_owned()
}

pub fn default_lib_rel(profile: &str, file: &str, triple_segment: Option<&str>) -> String {
    let mut p = PathBuf::from("./target");
    if let Some(t) = triple_segment {
        p = p.join(t);
    }
    p.join(profile).join(file).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_dir_trims_both_slashes() {
        assert_eq!(
            join_dir("out/", "a"),
            Path::new("out").join("a").to_string_lossy()
        );
        assert_eq!(
            join_dir(r"out\", "a"),
            Path::new("out").join("a").to_string_lossy()
        );
    }

    #[test]
    fn with_exe_no_double() {
        assert_eq!(with_exe_suffix("hello"), "hello.exe");
        assert_eq!(with_exe_suffix("hello.exe"), "hello.exe");
        assert_eq!(with_exe_suffix("KERNEL.ELF"), "KERNEL.ELF");
        assert_eq!(with_exe_suffix("x.dll"), "x.dll");
    }
}
