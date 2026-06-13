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

pub mod git;

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsKind {
    Git,
    None,
}

impl VcsKind {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "none" => Ok(Self::None),
            _ => Err(format!("Unknown VCS: {}", s)),
        }
    }
}

pub fn init_vcs(kind: VcsKind, path: &Path) -> Result<(), String> {
    match kind {
        VcsKind::Git => git::init(path),
        VcsKind::None => Ok(()),
    }
}

// look for git repo in parents to avoid nested repos
pub fn find_existing_vcs(path: &Path) -> Option<VcsKind> {
    let mut current = if path.is_file() {
        path.parent()
    } else {
        Some(path)
    };

    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(VcsKind::Git);
        }
        current = dir.parent();
    }

    None
}
