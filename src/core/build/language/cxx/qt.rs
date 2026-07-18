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

use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use crate::utils::build::run_pkg_config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn process_qt(ctx: &BuildContext, build_dir: &Path) -> Result<Option<PathBuf>, String> {
    let qt_dir = build_dir.join("qt");
    fs::create_dir_all(&qt_dir).map_err(|e| format!("Failed to create qt dir: {e}"))?;

    let extensions = vec!["ui", "qrc", "h", "hpp", "hxx"];
    let files = common::collect_sources(
        ctx.source_roots,
        &extensions,
        ctx.exclude_dirs,
        ctx.include_paths,
    )?;

    let mut qt_moc_args = Vec::new();
    let qt_modules = vec!["Qt6Core", "Qt6Widgets", "Qt6Gui", "Qt6Svg"];

    for module in qt_modules {
        if let Ok(cflags) = run_pkg_config(module, "--cflags") {
            for flag in cflags.split_whitespace() {
                if flag.starts_with("-I") || flag.starts_with("-D") {
                    qt_moc_args.push(flag.to_string());
                }
            }
        }
    }

    let mut generated_any = false;

    for file in files {
        let path = Path::new(&file);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "ui" => {
                if let Some(uic) = ctx.uic {
                    let out_name = path.file_stem().unwrap().to_str().unwrap();
                    let out_file = qt_dir.join(format!("ui_{out_name}.h"));
                    run_command(
                        uic,
                        &[path.to_str().unwrap(), "-o", out_file.to_str().unwrap()],
                        ctx.verbose,
                    )?;
                    generated_any = true;
                }
            }
            "qrc" => {
                if let Some(rcc) = ctx.rcc {
                    let out_name = path.file_stem().unwrap().to_str().unwrap();
                    let out_file = qt_dir.join(format!("qrc_{out_name}.cpp"));
                    run_command(
                        rcc,
                        &[
                            path.to_str().unwrap(),
                            "-name",
                            out_name,
                            "-o",
                            out_file.to_str().unwrap(),
                        ],
                        ctx.verbose,
                    )?;
                    generated_any = true;
                }
            }
            "h" | "hpp" | "hxx" => {
                if let Some(moc) = ctx.moc {
                    let content = fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
                    if content.contains("Q_OBJECT") {
                        let out_name = path.file_stem().unwrap().to_str().unwrap();
                        let out_file = qt_dir.join(format!("moc_{out_name}.cpp"));

                        let mut moc_args = Vec::new();

                        for arg in &qt_moc_args {
                            moc_args.push(arg.as_str());
                        }

                        for inc in ctx.include_paths.iter().chain(ctx.include_dirs.iter()) {
                            moc_args.push("-I");
                            moc_args.push(inc);
                        }

                        moc_args.push(path.to_str().unwrap());
                        moc_args.push("-o");
                        moc_args.push(out_file.to_str().unwrap());

                        run_command(moc, &moc_args, ctx.verbose)?;
                        generated_any = true;
                    }
                }
            }
            _ => {}
        }
    }

    if generated_any {
        Ok(Some(qt_dir))
    } else {
        Ok(None)
    }
}

fn run_command(cmd_path: &str, args: &[&str], verbose: bool) -> Result<(), String> {
    let mut cmd = Command::new(cmd_path);
    cmd.args(args);
    if verbose || std::env::var("DCR_DEBUG").is_ok() {
        eprintln!("[dcr] {:?}", cmd);
    }
    match cmd.status() {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("Command {} failed with status {}", cmd_path, s)),
        Err(e) => Err(format!("Failed to execute {}: {}", cmd_path, e)),
    }
}
