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

use log::{LevelFilter, Log, Metadata, Record};

struct CliLogger;

impl Log for CliLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level().to_level_filter() <= log::max_level()
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let kind = match record.level() {
            log::Level::Warn => "warning",
            log::Level::Error => "error",
            log::Level::Info => "info",
            log::Level::Debug => "debug",
            log::Level::Trace => "trace",
        };
        crate::utils::output::emit(kind, &record.args().to_string());
    }
    fn flush(&self) {}
}

// A function for initializing the logging lib with basic settings.
pub fn init() {
    static LOGGER: CliLogger = CliLogger;
    log::set_logger(&LOGGER).expect("logger already initialized");
    log::set_max_level(if crate::utils::output::verbose() {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });
}
