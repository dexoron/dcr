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

use crate::utils::cli_styles::{HELP_EXAMPLES_ST, HELP_SECTION_TITLE_ST};
use crate::utils::log::sprintln;

/// Displays the help information for the DCR CLI tool.
///
/// Prints usage, commands, flags, options, examples, and a tip.
///
/// # Returns
/// Always `0`.
pub fn help() -> i32 {
    println!("DCR (Dexoron Cargo Realization)");
    println!("C project manager inspired by Cargo.");
    println!();
    sprintln!(HELP_SECTION_TITLE_ST, "USAGE:");
    sprintln!(HELP_EXAMPLES_ST, "    dcr <command> [options]");
    println!();
    sprintln!(HELP_SECTION_TITLE_ST, "COMMANDS:");
    println!("    new <name>        Create a new project");
    println!("    init              Initialize the current directory as a project");
    println!("    build [--profile] Build the project (default: --debug)");
    println!("    run [--profile]   Build and run the project (default: --debug)");
    println!("    add <name> ...    Add a dependency to dcr.toml");
    println!("    tree              Display the dependency tree");
    println!("    test              Run project tests (alias: tests)");
    println!("    fmt               Format all C/C++ source files using clang-format");
    println!("    lint              Run clang-tidy on C/C++ sources");
    println!("    setup             Show configured package registries");
    println!("    clean             Remove the target directory");
    println!("    gen <subcommand>  Generate IDE integration files");
    sprintln!(HELP_SECTION_TITLE_ST, "FLAGS:");
    println!("    --help            Show command help");
    println!("    --update          Update dcr to the latest version");
    println!("    --version         Show dcr version");
    println!();
    sprintln!(HELP_SECTION_TITLE_ST, "OPTIONS:");
    println!("    --debug           Use debug profile");
    println!("    --release         Use release profile");
    println!("    --force           Force rebuild (build/run)");
    println!("    --clean           Clean before build (build/run)");
    println!("    --all             Clean all workspace members (clean)");
    println!();
    sprintln!(HELP_SECTION_TITLE_ST, "EXAMPLES:");
    sprintln!(HELP_EXAMPLES_ST, "    dcr new hello");
    sprintln!(HELP_EXAMPLES_ST, "    dcr build --release");
    sprintln!(HELP_EXAMPLES_ST, "    dcr run --debug");
    println!();
    sprintln!(HELP_SECTION_TITLE_ST, "TIP:");
    println!("    Run 'dcr <command> --help' for command-specific help.");
    0
}
