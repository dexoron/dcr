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

use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, printc};

pub fn help() -> i32 {
    println!("DCR (Dexoron Cargo Realization)");
    println!("C project manager inspired by Cargo.");
    println!();
    printc("USAGE:", BOLD_GREEN);
    printc("    dcr <command> [options]", BOLD_CYAN);
    println!();
    printc("COMMANDS:", BOLD_GREEN);
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
    printc("FLAGS:", BOLD_GREEN);
    println!("    --help            Show command help");
    println!("    --update          Update dcr to the latest version");
    println!("    --version         Show dcr version");
    println!();
    printc("OPTIONS:", BOLD_GREEN);
    println!("    --debug           Use debug profile");
    println!("    --release         Use release profile");
    println!("    --force           Force rebuild (build/run)");
    println!("    --clean           Clean before build (build/run)");
    println!("    --all             Clean all workspace members (clean)");
    println!();
    printc("EXAMPLES:", BOLD_GREEN);
    printc("    dcr new hello", BOLD_CYAN);
    printc("    dcr build --release", BOLD_CYAN);
    printc("    dcr run --debug", BOLD_CYAN);
    println!();
    printc("TIP:", BOLD_GREEN);
    println!("    Run 'dcr <command> --help' for command-specific help.");
    0
}
