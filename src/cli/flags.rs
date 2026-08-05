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

use crate::config::{PROFILE, flags};
use crate::utils::log::warn;

/// Flags shared by the `build` and `run` CLI commands.
pub struct BuildRunFlags {
    /// Build profile name (for example `debug` or `release`).
    pub profile: String,
    /// Optional target triple or platform name.
    pub target: Option<String>,
    /// Optional workspace member name (`--workspace <name>`).
    pub workspace: Option<String>,
    /// Force a rebuild even when artifacts appear up to date.
    pub force: bool,
    /// Clean build outputs before building.
    pub clean: bool,
    /// Enable verbose logging during the build or run.
    pub verbose: bool,
    /// Print the final artifact path to the user.
    pub print_artifact_path: bool,
    /// Arguments forwarded to the binary after a `--` separator.
    pub bin_args: Vec<String>,
}

/// Splits CLI arguments at the first `--` separator.
///
/// Everything before `--` is treated as DCR flags; everything after is
/// forwarded to the built binary as `bin_args`.
///
/// # Parameters
/// - `args`: Full argument list for the command (without the program name).
///
/// # Returns
/// `(dcr_args, bin_args)`. If `--` is absent, `bin_args` is empty.
pub fn split_double_dash(args: &[String]) -> (&[String], &[String]) {
    match args.iter().position(|a| a == "--") {
        Some(i) => (&args[..i], &args[i + 1..]),
        None => (args, &[]),
    }
}

/// Parses build/run flags from a list of CLI arguments.
///
/// Recognizes boolean flags (`--force`, `--clean`, `--verbose`,
/// `--print-artifact-path`), value flags (`--workspace`, `--target`), and
/// profile names registered via [`flags`]. Arguments after `--` become
/// [`BuildRunFlags::bin_args`].
///
/// # Parameters
/// - `args`: CLI arguments for build/run (may include `--`).
///
/// # Returns
/// - `Ok(BuildRunFlags)` on success.
/// - `Err(1)` for unknown flags, missing values, or a duplicate profile.
pub fn parse_build_run_flags(args: &[String]) -> Result<BuildRunFlags, i32> {
    let (dcr_args, bin_args_slice) = split_double_dash(args);
    let mut profile = PROFILE.to_string();
    let mut target = None;
    let mut workspace = None;
    let mut force = false;
    let mut clean = false;
    let mut verbose = false;
    let mut print_artifact_path = false;
    let mut iter = dcr_args.iter();

    while let Some(arg) = iter.next() {
        // DCR build/run options are long flags only (`--name`).
        if !arg.starts_with("--") {
            warn("Unknown argument");
            return Err(1);
        }
        let candidate = arg.trim_start_matches("--");
        if candidate == "force" {
            force = true;
            continue;
        }
        if candidate == "clean" {
            clean = true;
            continue;
        }
        if candidate == "verbose" {
            verbose = true;
            continue;
        }
        if candidate == "print-artifact-path" {
            print_artifact_path = true;
            continue;
        }
        if candidate == "workspace" {
            if let Some(w) = iter.next() {
                workspace = Some(w.clone());
            } else {
                warn("--workspace requires a value");
                return Err(1);
            }
            continue;
        }
        if candidate == "target" {
            if let Some(t) = iter.next() {
                target = Some(t.clone());
            } else {
                warn("--target requires a value");
                return Err(1);
            }
            continue;
        }
        // Profile names come from config::flags; only one profile may be set.
        if flags(candidate).is_some() {
            if profile != PROFILE {
                warn("Duplicate profile flag");
                return Err(1);
            }
            profile = candidate.to_string();
            continue;
        }
        warn("Unknown build flag");
        return Err(1);
    }

    Ok(BuildRunFlags {
        profile,
        target,
        workspace,
        force,
        clean,
        verbose,
        print_artifact_path,
        bin_args: bin_args_slice.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts string literals into owned `String` CLI arguments for tests.
    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    /// Checks that without `--`, the full list stays on the DCR side.
    #[test]
    fn split_double_dash_empty_tail() {
        let args = s(&["--release", "--force"]);
        let (left, right) = split_double_dash(&args);
        assert_eq!(left, &s(&["--release", "--force"]));
        assert!(right.is_empty());
    }

    /// Checks that arguments after `--` are split into the bin-args slice.
    #[test]
    fn split_double_dash_with_bin_args() {
        let args = s(&["--release", "--", "--test_help", "x"]);
        let (left, right) = split_double_dash(&args);
        assert_eq!(left, &s(&["--release"]));
        assert_eq!(right, &s(&["--test_help", "x"]));
    }

    /// Checks profile parsing and that bin args after `--` are forwarded.
    #[test]
    fn parse_run_flags_forwards_after_double_dash() {
        let flags = parse_build_run_flags(&s(&["--release", "--", "--test_help"])).unwrap();
        assert_eq!(flags.profile, "release");
        assert_eq!(flags.bin_args, s(&["--test_help"]));
    }
}
