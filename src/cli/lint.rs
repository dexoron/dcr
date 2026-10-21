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

use crate::prelude::*;

use crate::core::build::common;
use crate::core::build_config::Config;
use crate::utils::build::{get_bool_with_profile, get_list_with_profile};
use crate::utils::fs::find_project_root;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs clang-tidy on all C/C++ source files.
///
/// Scans source roots (configurable via build.roots) and tests/ directory.
pub fn lint(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        let mut help = crate::utils::help::HelpBuilder::new("dcr lint");
        help.section("USAGE").example("dcr lint [--fix]");
        help.description("Runs clang-tidy on all C/C++ source files.")
            .note("Scans source roots (configurable via build.roots) and tests/ directory.");
        help.section("OPTIONS")
            .item("--fix", "Apply clang-tidy suggestions automatically");
        help.render();
        return 0;
    }

    let do_fix = args.iter().any(|a| a == "--fix");

    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error!("Failed to determine current directory");
            return 1;
        }
    };

    let root = match find_project_root(&start_dir) {
        Ok(Some(r)) => r,
        Ok(None) => {
            error!("dcr.toml not found");
            return 1;
        }
        Err(_) => {
            error!("Failed to find project root");
            return 1;
        }
    };

    let config = match Config::open(&root.join("dcr.toml").to_string_lossy()) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load dcr.toml: {e}");
            return 1;
        }
    };

    let profile = "debug";
    let build_roots = get_list_with_profile(&config, "roots", profile);
    let src_disable = get_bool_with_profile(&config, "src_disable", profile, false);

    // Construct source roots from config
    let mut source_roots: Vec<String> = Vec::new();
    for raw in &build_roots {
        let p = Path::new(raw);
        if p.is_absolute() {
            source_roots.push(raw.clone());
        } else {
            source_roots.push(root.join(p).to_string_lossy().to_string());
        }
    }
    if !src_disable && source_roots.is_empty() {
        source_roots.push(root.join("src").to_string_lossy().to_string());
    }

    let cxx_extensions = ["c", "cpp", "cxx", "cc"];
    let exclude_dirs: Vec<PathBuf> = Vec::new();
    let include_paths: Vec<String> = Vec::new();

    let roots: Vec<PathBuf> = source_roots.iter().map(PathBuf::from).collect();
    let mut files =
        match common::collect_sources(&roots, &cxx_extensions, &exclude_dirs, &include_paths) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to collect source files: {e}");
                return 1;
            }
        };

    let tests_dir = root.join("tests");
    if tests_dir.is_dir() {
        let test_roots = vec![tests_dir];
        if let Ok(mut test_files) =
            common::collect_sources(&test_roots, &cxx_extensions, &exclude_dirs, &include_paths)
        {
            files.append(&mut test_files);
        }
    }

    files.sort();

    if files.is_empty() {
        crate::utils::output::status("Lint", "no files to lint");
        return 0;
    }

    crate::utils::output::status(
        "Linting",
        format!(
            "{} files {}",
            files.len(),
            if do_fix { "(with --fix)" } else { "" }
        ),
    );

    let mut total_issues = 0u32;
    let mut failed = 0u32;

    // Execute clang-tidy for each file and count issues
    for file in &files {
        let mut cmd = Command::new("clang-tidy");
        if do_fix {
            cmd.arg("--fix");
        }
        cmd.arg("--quiet");
        cmd.arg(file);

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to execute clang-tidy: {e}");
                return 1;
            }
        };

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        let issues: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("warning:") || l.contains("error:"))
            .collect();
        let issue_count = issues.len();

        if issue_count > 0 {
            total_issues += issue_count as u32;
            failed += 1;
            crate::utils::output::warning(format!("{file} ({issue_count} issues)"));
            for issue in &issues {
                crate::utils::output::compiler_output("stderr", issue);
            }
        } else {
            crate::utils::output::status("Checked", file);
        }
    }

    let total = files.len();
    let passed = total - failed as usize;

    if failed == 0 {
        crate::utils::output::success(format!("Finished checking {total} files; no issues"));
    } else {
        crate::utils::output::warning(format!(
            "Finished checking {total} files; {passed} passed, {total_issues} issues found"
        ));
    }

    0
}
