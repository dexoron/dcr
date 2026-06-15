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

fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_string());
    println!("cargo:rustc-env=DCR_TARGET={target}");

    // Let's try to retrieve the commit hash for the version
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // Check if there are any uncommitted changes in the repository
    let is_dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    if let Some(commit_hash) = commit {
        let dirty_suffix = if is_dirty { "-dirty" } else { "" };
        println!(
            "cargo:rustc-env=DCR_GIT_INFO= (git {}{})",
            commit_hash, dirty_suffix
        );
    } else {
        println!("cargo:rustc-env=DCR_GIT_INFO=");
    }
}
