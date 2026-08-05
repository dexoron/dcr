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

/// Core module for language support in the DCR build system.
/// Defines the Language trait and helper functions to register and lookup languages.
pub mod asm;
pub mod c;
pub mod cxx;
pub mod llvm_ir;

/// Trait representing a supported programming language.
/// Each implementation must specify its unique identifier, list of supported file extensions,
/// and a method to determine if a given token matches the language.
pub trait Language {
    fn id(&self) -> &'static str;
    fn extensions(&self) -> &'static [&'static str];
    fn matches_token(&self, token: &str) -> bool;
}

/// Returns a fixed-size array containing static references to all supported language types.
pub fn languages() -> [&'static dyn Language; 4] {
    [&c::C, &cxx::Cxx, &asm::Asm, &llvm_ir::LlvmIr]
}

/// Looks up the language implementation that matches the given token by checking each language's
/// matches_token method. Returns the first match or None if no language matches.
pub fn language_for_token(token: &str) -> Option<&'static dyn Language> {
    languages().into_iter().find(|l| l.matches_token(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures that input tokens are resolved to the correct language IDs.
    #[test]
    fn tokens_resolve_to_languages() {
        assert_eq!(language_for_token("c").unwrap().id(), "c");
        assert_eq!(language_for_token("c++").unwrap().id(), "cxx");
        assert_eq!(language_for_token("cpp").unwrap().id(), "cxx");
        assert_eq!(language_for_token("cxx").unwrap().id(), "cxx");
        assert_eq!(language_for_token("asm").unwrap().id(), "asm");
        assert!(language_for_token("rust").is_none());
    }

    /// Ensures that each language reports the expected set of file extensions.
    #[test]
    fn languages_expose_expected_extensions() {
        assert_eq!(c::C.extensions(), &["c"]);
        assert_eq!(cxx::Cxx.extensions(), &["cpp", "cxx", "cc"]);
        assert_eq!(asm::Asm.extensions(), &["s", "S", "asm"]);
    }
}
