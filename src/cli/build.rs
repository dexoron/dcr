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

use crate::cli::clean::clean;
use crate::cli::flags::parse_build_run_flags;
use crate::core::build::{BuildEvent, BuildReporter, BuildRequest, run_build};
use crate::core::build_config::Config;
use crate::utils::build::default_target_triple;
use crate::utils::fs::find_project_root;
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use std::sync::{Arc, atomic::AtomicBool};

pub use crate::core::build::get_build_string_with_profile;

struct CliReporter;

impl BuildReporter for CliReporter {
    fn on_event(&mut self, event: BuildEvent<'_>) {
        match event {
            BuildEvent::TargetStart {
                index,
                total,
                target,
            } => {
                println!("    Building target {} of {}: {}", index, total, target);
            }
            BuildEvent::ProjectStart {
                name,
                profile,
                target,
            } => {
                println!(
                    "    Building project `{}`\n    Profile: {}\n      Target: {}",
                    colored(name, BOLD_GREEN),
                    colored(profile, BOLD_GREEN),
                    colored(target, BOLD_GREEN)
                );
            }
            BuildEvent::DepBuilding { name, version } => {
                print!(
                    "\r{:100}\r      {} {} v{}",
                    "",
                    colored("Building", BOLD_GREEN),
                    name,
                    version
                );
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            BuildEvent::DepReady {
                name,
                version,
                rebuilt,
            } => {
                if rebuilt {
                    print!(
                        "\r{:100}\r       {} {} v{}",
                        "",
                        colored("Ready", BOLD_GREEN),
                        name,
                        version
                    );
                    println!();
                } else {
                    println!(
                        "      {} {} v{}",
                        colored("Ready", BOLD_GREEN),
                        name,
                        version
                    );
                }
            }
            BuildEvent::Compiling { name, version } => {
                println!(
                    "   {} {} v{}",
                    colored("Compiling", BOLD_GREEN),
                    name,
                    version
                );
            }
            BuildEvent::Finished { secs } => {
                println!(
                    "    {} Build completed successfully in {} seconds",
                    colored("✔", BOLD_GREEN),
                    colored(&secs.to_string(), BOLD_GREEN)
                );
            }
            BuildEvent::CompilerOutput { stream, text } => {
                if stream == "stderr" {
                    eprintln!("{}", text);
                } else {
                    println!("{}", text);
                }
            }
        }
    }
}

pub fn build(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        printc("USAGE:", BOLD_GREEN);
        printc(
            "    dcr build [--debug | --release] [--target <triple>] [--force] [--clean] [--verbose]",
            BOLD_CYAN,
        );
        println!();
        printc("DESCRIPTION:", BOLD_GREEN);
        println!("    Compiles the project. Default profile is --debug.");
        println!();
        printc("OPTIONS:", BOLD_GREEN);
        println!("    --debug              Build with debug profile (default)");
        println!("    --release            Build with release profile");
        println!("    --target <triple>    Cross-compile for the given target");
        println!("    --force              Force a full rebuild");
        println!("    --clean              Clean before building");
        println!("    --verbose            Print detailed build output");
        println!("    --workspace <name>   Build a specific workspace member");
        return 0;
    }

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();
    ctrlc::set_handler(move || {
        cancel_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .ok();
    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };
    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => dir,
        Ok(None) => {
            error("dcr.toml file not found");
            return 1;
        }
        Err(_) => {
            error("Failed to find project root");
            return 1;
        }
    };
    let mut flags = match parse_build_run_flags(args) {
        Ok(v) => v,
        Err(code) => return code,
    };

    if flags.target.is_none() {
        let config_path = root.join("dcr.toml");
        if let Ok(config) = Config::open(config_path.to_str().unwrap())
            && !config.is_workspace_only()
        {
            let bt = get_build_string_with_profile(&config, "target", "debug");
            if !bt.is_empty() {
                flags.target = Some(bt);
            }
        }
    }
    if flags.target.is_none() {
        flags.target = Some(default_target_triple());
    }
    if flags.clean {
        let mut clean_args = Vec::new();
        clean_args.push(format!("--{}", flags.profile));
        let _ = clean(&clean_args);
    }

    let req = BuildRequest {
        profile: flags.profile,
        target: flags.target,
        force: flags.force,
        verbose: flags.verbose,
        workspace: flags.workspace,
        cancel,
    };
    let mut reporter = CliReporter;
    match run_build(&root, &req, &mut reporter) {
        Ok(_) => 0,
        Err(err) => {
            error(&err.message);
            1
        }
    }
}
