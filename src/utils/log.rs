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

use env_logger::{Builder, WriteStyle};
use log::{Level, LevelFilter};
use owo_colors::OwoColorize;
use std::io::Write;

// A function for initializing the logging lib with basic settings.
pub fn init() {
    Builder::new()
        .filter_level(LevelFilter::Warn)
        .write_style(WriteStyle::Auto)
        .format(|buf, record| {
            let colored_lv = match record.level() {
                Level::Error => record.level().red().to_string(),
                Level::Warn => record.level().yellow().to_string(),
                Level::Info => record.level().default_color().to_string(),
                _ => record.level().purple().italic().to_string(),
            };

            writeln!(
                buf,
                "{}: {}",
                colored_lv.to_lowercase().bold(),
                record.args(),
            )
        })
        .init();
}

/// Prints message to stdout with the given `Style`. Copy of println! macro
#[allow(unused_macros)]
macro_rules! sprintln {
    ($style:expr, $($arg:tt)*) => {
        {
            use ::owo_colors::OwoColorize as _;
            println!("{}", format!($($arg)*).style($style));
        }
    };
}

/// Prints message to stdout with the given `Style`. Copy of print! macro
#[allow(unused_macros)]
macro_rules! sprint {
    ($style:expr, $($arg:tt)*) => {
        {
            use ::owo_colors::OwoColorize as _;
            print!("{}", format!($($arg)*).style($style));
        }
    };
}

#[allow(unused_imports)]
pub(crate) use {sprint, sprintln};
