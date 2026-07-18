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

pub mod common;
pub mod fasm;
pub mod gas;
pub mod masm;
pub mod nasm;

use crate::core::build::language::Language;

pub struct Asm;

impl Language for Asm {
    fn id(&self) -> &'static str {
        "asm"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["s", "S", "asm"]
    }
    fn matches_token(&self, token: &str) -> bool {
        token.eq_ignore_ascii_case("asm")
    }
}
