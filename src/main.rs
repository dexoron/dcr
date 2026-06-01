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
use crate::utils::log::error;
mod cli;
mod config;
mod core;
mod platform;
mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        std::process::exit(cli::help::help());
    }

    let cmd = args[1].as_str();
    let rest = &args[2..];

    let code = match cmd {
        "new" => cli::r#new::new(rest),
        "init" => cli::init::init(rest),
        "setup" => cli::setup::setup(rest),
        "add" => cli::add::add(rest),
        "build" => cli::build::build(rest),
        "run" => cli::run::run(rest),
        "tree" => cli::tree::tree(rest),
        "test" | "tests" => cli::test::test(rest),
        "fmt" => cli::fmt::fmt(rest),
        "clean" => cli::clean::clean(rest),
        "gen" => cli::r#gen::r#gen(rest),
        "--version" => {
            println!("dcr {} ({})", env!("CARGO_PKG_VERSION"), env!("DCR_TARGET"));
            0
        }
        "--help" => cli::help::help(),
        "--update" => cli::flag_update::flag_update(rest),
        _ => {
            error("Unknown command or argument");
            0
        }
    };

    std::process::exit(code);
}
