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

pub struct BuildRunFlags {
    pub profile: String,
    pub target: Option<String>,
    pub workspace: Option<String>,
    pub force: bool,
    pub clean: bool,
    pub verbose: bool,
    pub print_artifact_path: bool,
    pub bin_args: Vec<String>,
}

pub fn split_double_dash(args: &[String]) -> (&[String], &[String]) {
    match args.iter().position(|a| a == "--") {
        Some(i) => (&args[..i], &args[i + 1..]),
        None => (args, &[]),
    }
}

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

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn split_double_dash_empty_tail() {
        let args = s(&["--release", "--force"]);
        let (left, right) = split_double_dash(&args);
        assert_eq!(left, &s(&["--release", "--force"]));
        assert!(right.is_empty());
    }

    #[test]
    fn split_double_dash_with_bin_args() {
        let args = s(&["--release", "--", "--test_help", "x"]);
        let (left, right) = split_double_dash(&args);
        assert_eq!(left, &s(&["--release"]));
        assert_eq!(right, &s(&["--test_help", "x"]));
    }

    #[test]
    fn parse_run_flags_forwards_after_double_dash() {
        let flags = parse_build_run_flags(&s(&["--release", "--", "--test_help"])).unwrap();
        assert_eq!(flags.profile, "release");
        assert_eq!(flags.bin_args, s(&["--test_help"]));
    }
}
