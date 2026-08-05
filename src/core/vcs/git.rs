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

use std::fs;
use std::path::Path;

/// Initializes a Git repository in the target directory and configures its `.gitignore` file.
///
/// Executes `git init` via the VCS utility module, creates a default `.gitignore` if none exists,
/// or updates an existing `.gitignore` to ensure `.dcr/` and build outputs are ignored.
pub fn init(path: &Path) -> Result<(), String> {
    crate::utils::git::git_init(path)?;

    // Populate default .gitignore entries or append DCR metadata rules.
    let gitignore_path = path.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, "/target\n.dcr/\n")
            .map_err(|e| format!("failed to create .gitignore: {}", e))?;
    } else {
        crate::utils::fs::ensure_gitignore_has_dcr(path)
            .map_err(|e| format!("failed to update .gitignore: {}", e))?;
    }

    Ok(())
}
