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

use crate::core::build::language::Language;

/// Represents the C programming language support within the DCR build system.
pub struct C;

impl Language for C {
    /// Returns the unique identifier for the C language.
    fn id(&self) -> &'static str {
        "c"
    }

    /// Returns an array containing the file extension associated with C source code.
    fn extensions(&self) -> &'static [&'static str] {
        &["c"]
    }

    /// Checks whether the given token matches the C language identifier in a case-insensitive manner.
    fn matches_token(&self, token: &str) -> bool {
        token.eq_ignore_ascii_case("c")
    }
}
