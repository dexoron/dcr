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

/// C++ language support for the DCR build system.
pub mod qt;

use crate::core::build::language::Language;

pub struct Cxx;

/// C++ language implementation.
impl Language for Cxx {
    /// Returns the language ID "cxx".
    fn id(&self) -> &'static str {
        "cxx"
    }
    /// Returns the extensions for C++ files.
    fn extensions(&self) -> &'static [&'static str] {
        &["cpp", "cxx", "cc"]
    }
    /// Checks if the token matches C++ language names, ignoring case.
    fn matches_token(&self, token: &str) -> bool {
        // Case-insensitive comparison using to_lowercase
        matches!(token.to_lowercase().as_str(), "c++" | "cpp" | "cxx")
    }
}
