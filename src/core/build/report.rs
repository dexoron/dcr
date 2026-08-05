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

// The reporter decouples the build engine from any particular frontend: the CLI
// renders events as coloured lines, a TUI/GUI would push them into a widget and
// MCP would serialise them to JSON. The engine only ever calls `on_event`.

use std::sync::{Arc, atomic::AtomicBool};

/// Represents a build request with options for the build process, including profile selection, target specification, force flag, verbosity, workspace, and a cancellation signal.
pub struct BuildRequest {
    pub profile: String,
    pub target: Option<String>,
    pub force: bool,
    pub verbose: bool,
    pub workspace: Option<String>,
    pub cancel: Arc<AtomicBool>,
}

/// Contains the result of a build operation, specifically the duration in seconds.
#[allow(dead_code)]
pub struct BuildOutcome {
    pub secs: f64,
}

/// Wraps a string message as a build error.
pub struct BuildError {
    pub message: String,
}

/// Creates a BuildError from a String message.
impl From<String> for BuildError {
    fn from(message: String) -> Self {
        BuildError { message }
    }
}

/// Represents events that the build engine can emit during compilation and packaging.
pub enum BuildEvent<'a> {
    TargetStart {
        index: usize,
        total: usize,
        target: &'a str,
    },
    ProjectStart {
        name: &'a str,
        profile: &'a str,
        target: &'a str,
    },
    DepBuilding {
        name: &'a str,
        version: &'a str,
    },
    DepReady {
        name: &'a str,
        version: &'a str,
        rebuilt: bool,
    },
    Compiling {
        name: &'a str,
        version: &'a str,
    },
    Packing {
        path: &'a str,
    },
    CompilerOutput {
        stream: &'a str,
        text: &'a str,
    },
    Finished {
        secs: f64,
    },
}

/// Trait for handling build events in different reporting backends.
pub trait BuildReporter {
    /// Callback invoked when a build event is generated.
    fn on_event(&mut self, event: BuildEvent<'_>);
}
