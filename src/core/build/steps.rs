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

use crate::core::build::engine::ToolchainExecs;
use crate::core::build_config::Config;
use crate::utils::build::{VersionInfo, profile_table, substitute_vars};
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct BuildStep {
    name: String,
    input: String,
    output: String,
    cmd: String,
}

pub(crate) struct StepVars<'a> {
    pub(crate) profile: &'a str,
    pub(crate) version: &'a str,
    pub(crate) version_major: &'a str,
    pub(crate) version_minor: &'a str,
    pub(crate) version_patch: &'a str,
    pub(crate) version_suffix: &'a str,
    pub(crate) version_suffix_dash: &'a str,
}

fn get_build_steps_from_value(value: &toml::Value, key: &str) -> Result<Vec<BuildStep>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array"))?;
    let mut out = Vec::new();
    for item in arr {
        let tbl = item
            .as_table()
            .ok_or_else(|| format!("{key} entries must be tables"))?;
        let name = tbl
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let input = tbl
            .get("in")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let output = tbl
            .get("out")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let cmd = tbl
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() || input.is_empty() || output.is_empty() || cmd.is_empty() {
            return Err(format!("{key} entries must include name, in, out, cmd"));
        }
        out.push(BuildStep {
            name,
            input,
            output,
            cmd,
        });
    }
    Ok(out)
}

pub(crate) fn get_build_steps_with_profile(
    config: &Config,
    field: &str,
    profile: &str,
) -> Result<Vec<BuildStep>, String> {
    if let Some(table) = profile_table(config, profile)
        && let Some(value) = table.get(field)
    {
        return get_build_steps_from_value(value, &format!("build.{profile}.{field}"));
    }
    get_build_steps(config, &format!("build.{field}"))
}

fn get_build_steps(config: &Config, key: &str) -> Result<Vec<BuildStep>, String> {
    let value = match config.get(key) {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    get_build_steps_from_value(value, key)
}

use crate::core::build::report::{BuildEvent, BuildReporter};
use std::sync::{Arc, atomic::AtomicBool};

pub(crate) fn run_build_steps(
    steps: &[BuildStep],
    tools: &ToolchainExecs,
    step_flags: &str,
    vars: &StepVars,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
) -> Result<(), String> {
    for step in steps {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Build interrupted".to_string());
        }
        run_build_step(step, tools, step_flags, vars, cancel, rep)?;
    }
    Ok(())
}

fn run_build_step(
    step: &BuildStep,
    tools: &ToolchainExecs,
    step_flags: &str,
    vars: &StepVars,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
) -> Result<(), String> {
    let input_pattern = expand_step_value(&step.input, "", vars);
    let inputs = expand_glob(&input_pattern)?;
    if inputs.is_empty() {
        return Ok(());
    }
    let needs_stem = step.output.contains("{stem}");
    if inputs.len() > 1 && !needs_stem {
        return Err(format!(
            "build.steps '{}' output must include {{stem}} for multiple inputs",
            step.name
        ));
    }
    for input in inputs {
        if !input.is_file() {
            continue;
        }
        let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
        let out_path = PathBuf::from(expand_step_value(&step.output, stem, vars));
        if !should_run_step(&input, &out_path) {
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create step output dir: {err}"))?;
        }
        let cmd = substitute_step_cmd(&step.cmd, &input, &out_path, tools, step_flags, stem, vars);
        let status = run_shell_command(&cmd, cancel, rep)
            .map_err(|err| format!("Failed to run step '{}': {err}", step.name))?;
        if !status.success() {
            return Err(format!("Step '{}' failed", step.name));
        }
    }
    Ok(())
}

pub(crate) fn clean_generated_files(patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        for path in expand_glob(pattern)? {
            if path.is_file() {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

pub(crate) fn expand_glob(pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let entries = glob(pattern).map_err(|err| format!("glob error: {err}"))?;
    for entry in entries {
        let path = entry.map_err(|err| format!("glob error: {err}"))?;
        out.push(path);
    }
    Ok(out)
}

pub(crate) fn verify_expectations(patterns: &[String], vars: &StepVars) -> Result<(), String> {
    for pattern in patterns {
        let expanded = expand_step_value(pattern, "", vars);
        let matches = expand_glob(&expanded)?;
        if matches.is_empty() {
            return Err(format!("Expected artifact not found: {expanded}"));
        }
    }
    Ok(())
}

fn should_run_step(input: &Path, output: &Path) -> bool {
    let in_time = fs::metadata(input).and_then(|m| m.modified());
    let out_time = fs::metadata(output).and_then(|m| m.modified());
    match (in_time, out_time) {
        (Ok(i), Ok(o)) => i > o,
        (Ok(_), Err(_)) => true,
        _ => true,
    }
}

fn substitute_step_cmd(
    template: &str,
    input: &Path,
    output: &Path,
    tools: &ToolchainExecs,
    step_flags: &str,
    stem: &str,
    vars: &StepVars,
) -> String {
    let info = make_version_info(vars);
    let s = substitute_vars(template, &info, vars.profile, "");
    s.replace("{in}", &input.to_string_lossy())
        .replace("{out}", &output.to_string_lossy())
        .replace("{uic}", &tools.uic)
        .replace("{moc}", &tools.moc)
        .replace("{rcc}", &tools.rcc)
        .replace("{cflags}", step_flags)
        .replace("{stem}", stem)
}

pub(crate) fn build_step_flags(
    cflags: &[String],
    include_dirs: &[String],
    compiler: &str,
) -> String {
    let mut out = Vec::new();
    let msvc_style = is_msvc_compiler(compiler) || cflags.iter().any(|f| f.starts_with('/'));
    for flag in cflags {
        if flag.starts_with("-I") || flag.starts_with("-D") {
            out.push(flag.clone());
        }
        if flag.starts_with("/I") || flag.starts_with("/D") {
            out.push(flag.clone());
        }
        if msvc_style && flag.starts_with("-D") {
            out.push(format!("/D{}", flag.trim_start_matches("-D")));
        }
    }
    for dir in include_dirs {
        out.push(format!("-I{dir}"));
        if msvc_style {
            out.push(format!("/I{dir}"));
        }
    }
    out.sort();
    out.dedup();
    out.into_iter()
        .map(quote_step_arg)
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_step_arg(arg: String) -> String {
    if !arg.chars().any(|c| c.is_whitespace() || c == '"') {
        return arg;
    }
    let escaped = arg.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn is_msvc_compiler(compiler: &str) -> bool {
    let lower = compiler.to_lowercase();
    lower.contains("cl.exe")
        || lower == "cl"
        || lower.contains("clang-cl")
        || lower.contains("msvc")
}

fn run_shell_command(
    cmd: &str,
    cancel: &Arc<AtomicBool>,
    rep: &mut dyn BuildReporter,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut child = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    }?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    use std::io::{BufRead, BufReader};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    let cancel_clone = cancel.clone();
    let tx_stdout = tx.clone();
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if cancel_clone.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            if let Ok(line) = line {
                let _ = tx_stdout.send(("stdout", line));
            }
        }
    });

    let cancel_clone = cancel.clone();
    let tx_stderr = tx.clone();
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if cancel_clone.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            if let Ok(line) = line {
                let _ = tx_stderr.send(("stderr", line));
            }
        }
    });

    let status = child.wait()?;
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    drop(tx);
    while let Ok((stream, text)) = rx.recv() {
        rep.on_event(BuildEvent::CompilerOutput {
            stream,
            text: &text,
        });
    }

    Ok(status)
}

pub(crate) fn build_steps_need_run(steps: &[BuildStep], vars: &StepVars) -> Result<bool, String> {
    for step in steps {
        let input_pattern = expand_step_value(&step.input, "", vars);
        let inputs = expand_glob(&input_pattern)?;
        if inputs.is_empty() {
            continue;
        }
        let needs_stem = step.output.contains("{stem}");
        if inputs.len() > 1 && !needs_stem {
            return Err(format!(
                "build.steps '{}' output must include {{stem}} for multiple inputs",
                step.name
            ));
        }
        for input in inputs {
            if !input.is_file() {
                continue;
            }
            let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
            let out_path = PathBuf::from(expand_step_value(&step.output, stem, vars));
            if should_run_step(&input, &out_path) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn make_version_info(vars: &StepVars) -> VersionInfo {
    VersionInfo {
        full: vars.version.to_string(),
        major: vars.version_major.to_string(),
        minor: vars.version_minor.to_string(),
        patch: vars.version_patch.to_string(),
        suffix: vars.version_suffix.to_string(),
        suffix_dash: vars.version_suffix_dash.to_string(),
    }
}

fn expand_step_value(template: &str, stem: &str, vars: &StepVars) -> String {
    let info = make_version_info(vars);
    let s = substitute_vars(template, &info, vars.profile, "");
    s.replace("{stem}", stem)
}
