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

#[allow(dead_code)]
pub const RESET: &str = "\x1b[0m";

#[allow(dead_code)]
pub const BOLD: &str = "\x1b[1m";

#[allow(dead_code)]
pub const BRIGHT_RED: &str = "\x1b[91m";
#[allow(dead_code)]
pub const BRIGHT_GREEN: &str = "\x1b[92m";
#[allow(dead_code)]
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
#[allow(dead_code)]
pub const BRIGHT_CYAN: &str = "\x1b[96m";
#[allow(dead_code)]
pub const BOLD_RED: &str = "\x1b[1m\x1b[91m";
#[allow(dead_code)]
pub const BOLD_GREEN: &str = "\x1b[1m\x1b[92m";
#[allow(dead_code)]
pub const BOLD_YELLOW: &str = "\x1b[1m\x1b[93m";
#[allow(dead_code)]
pub const BOLD_CYAN: &str = "\x1b[1m\x1b[96m";

/// Applies ANSI escape codes to format the message with the given style.
///
/// This is a utility for colored output in terminal applications.
#[allow(dead_code)]
pub fn colored(msg: &str, style: &str) -> String {
    format!("{style}{msg}{RESET}")
}

/// Prints `msg` to stdout with the given ANSI `style`.
#[allow(dead_code)]
pub fn printc(msg: &str, style: &str) {
    println!("{style}{msg}{RESET}");
}
