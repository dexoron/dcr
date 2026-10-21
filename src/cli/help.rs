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

/// Displays the help information for the DCR CLI tool.
///
/// Prints usage, commands, flags, options, examples, and a tip.
///
/// # Returns
/// Always `0`.
pub fn help() -> i32 {
    use crate::utils::help::HelpBuilder;
    let mut help = HelpBuilder::new("DCR (Dexoron Cargo Realization)");
    help.description("C/C++ project manager inspired by Cargo.");
    help.section("USAGE").example("dcr <command> [options]");
    help.section("COMMANDS")
        .item("new <name>", "Create a new project")
        .item("init", "Initialize the current directory as a project")
        .item("build", "Build the project (default: --debug)")
        .item("run", "Build and run the project")
        .item("add <name> ...", "Add a dependency to dcr.toml")
        .item("tree", "Display the dependency tree")
        .item("test", "Run project tests (alias: tests)")
        .item("fmt", "Format C/C++ sources using clang-format")
        .item("lint", "Run clang-tidy on C/C++ sources")
        .item("setup", "Show configured package registries")
        .item("clean", "Remove build artifacts")
        .item("gen <subcommand>", "Generate IDE integration files");
    help.section("FLAGS")
        .item("--help", "Show command help")
        .item("--update", "Update DCR")
        .item("--version", "Show DCR version");
    help.section("OPTIONS")
        .item("--output human|plain|json", "Select output format")
        .item("--color auto|always|never", "Select color policy")
        .item("--quiet", "Suppress ordinary diagnostics")
        .item("--verbose", "Enable detailed diagnostics")
        .item("--debug / --release", "Select build profile")
        .item("--force", "Force rebuild (build/run)")
        .item("--clean", "Clean before build (build/run)")
        .item("--all", "Clean all workspace members");
    help.example("dcr new hello")
        .example("dcr build --release")
        .example("dcr run --debug");
    help.tip("Run 'dcr <command> --help' for command-specific help.");
    help.render();
    0
}
