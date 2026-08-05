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

/// Assembly language module for the DCR build system.
pub mod common;
pub mod fasm;
pub mod gas;
pub mod masm;
pub mod nasm;

use crate::core::build::language::Language;

/// Represents the assembly language support in the build system.
pub struct Asm;

impl Language for Asm {
    /// Returns the identifier string for this language.
    fn id(&self) -> &'static str {
        "asm"
    }

    /// Returns the list of supported file extensions for assembly files.
    fn extensions(&self) -> &'static [&'static str] {
        &["s", "S", "asm"]
    }

    /// Checks whether the given token matches the assembly language, using case-insensitive comparison.
    fn matches_token(&self, token: &str) -> bool {
        token.eq_ignore_ascii_case("asm")
    }
}
