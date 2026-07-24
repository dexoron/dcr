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
use crate::core::build::common;
use crate::core::build::{BuildEvent, BuildReporter, BuildRequest, run_build};
use crate::core::build_config::Config;
use crate::utils::fs::find_project_root;
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, BOLD_YELLOW, colored, printc};
use std::io::IsTerminal;
use std::sync::{Arc, atomic::AtomicBool};

pub use crate::core::build::get_build_string_with_profile;

fn status(verb: &str, style: &str) -> String {
    colored(&format!("{verb:<9}"), style)
}

struct CliReporter;

impl CliReporter {
    fn line(&self, verb: &str, style: &str, rest: &str) {
        eprintln!("  {} {}", status(verb, style), rest);
    }
}

impl BuildReporter for CliReporter {
    fn on_event(&mut self, event: BuildEvent<'_>) {
        match event {
            BuildEvent::TargetStart {
                index,
                total,
                target,
            } => {
                common::finish_progress_line();
                self.line("target", BOLD_CYAN, &format!("{index}/{total}  {target}"));
            }
            BuildEvent::ProjectStart {
                name,
                profile,
                target,
            } => {
                common::finish_progress_line();
                self.line(
                    "project",
                    BOLD_CYAN,
                    &format!("{name}  ({profile}, {target})"),
                );
            }
            BuildEvent::DepBuilding { name, version } => {
                common::finish_progress_line();
                self.line("dep", BOLD_YELLOW, &format!("{name} v{version}"));
            }
            BuildEvent::DepReady {
                name,
                version,
                rebuilt,
            } => {
                common::finish_progress_line();
                if rebuilt {
                    self.line("ready", BOLD_GREEN, &format!("{name} v{version}"));
                }
            }
            BuildEvent::Compiling { name, version } => {
                common::finish_progress_line();
                let label = format!("  {} {} v{}", status("compile", BOLD_GREEN), name, version);
                common::set_progress_label(Some(label.clone()));
                if !std::io::stderr().is_terminal() {
                    eprintln!("{label}");
                }
            }
            BuildEvent::Packing { path } => {
                common::finish_progress_line();
                self.line("pack", BOLD_CYAN, path);
            }
            BuildEvent::Finished { secs } => {
                common::finish_progress_line();
                self.line("done", BOLD_GREEN, &format!("in {secs}s"));
            }
            BuildEvent::CompilerOutput { stream, text } => {
                common::interrupt_progress_for_output();
                if stream == "stderr" {
                    eprint!("{text}");
                } else {
                    print!("{text}");
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
            let bt = get_build_string_with_profile(&config, "target", &flags.profile);
            if !bt.is_empty() {
                flags.target = Some(bt);
            }
        }
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
            common::finish_progress_line();
            error(&err.message);
            1
        }
    }
}
