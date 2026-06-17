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

use crate::cli::build;
use crate::config::{FILE_DCR_TEST_H, FILE_TEST_C, flags};
use crate::core::build_config::Config;
use crate::utils::build::{
    get_config_opt, get_config_str, get_language_with_profile, get_list_with_profile,
    get_string_with_profile, resolve_compiler, resolve_pkg_config_flags,
};
use crate::utils::fs::{find_project_root, with_dir};
use crate::utils::log::error;
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, BOLD_RED, RESET, colored, printc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BOLD_BLUE: &str = "\x1b[1m\x1b[94m";

pub fn test(args: &[String]) -> i32 {
    let mut init_header = false;
    let mut profile = "debug";
    for arg in args {
        if arg == "--help" {
            printc("USAGE:", BOLD_GREEN);
            printc("    dcr test [--init] [--debug | --release]", BOLD_CYAN);
            println!();
            printc("ALIASES:", BOLD_GREEN);
            println!("    dcr tests");
            println!();
            printc("DESCRIPTION:", BOLD_GREEN);
            println!("    Runs project tests and prints a unified testsuite report.");
            println!();
            printc("OPTIONS:", BOLD_GREEN);
            println!("    --init            Create tests/dcr_test.h in current project");
            println!("    --debug           Build and run tests with debug profile (default)");
            println!("    --release         Build and run tests with release profile");
            return 0;
        }
        if arg == "--init" {
            init_header = true;
            continue;
        }
        if arg == "--debug" || arg == "--release" {
            profile = arg.trim_start_matches("--");
            continue;
        }
        error("Unknown argument");
        return 1;
    }

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

    if init_header {
        match with_dir(&root, ensure_test_header) {
            Ok(()) => {}
            Err(msg) => {
                error(&msg);
                return 1;
            }
        }
    }

    match with_dir(&root, || run_testsuite(profile)) {
        Ok(code) => code,
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

fn run_testsuite(profile: &str) -> Result<i32, String> {
    if build::build(&[format!("--{profile}")]) != 0 {
        return Ok(1);
    }

    let config = Config::open("./dcr.toml").map_err(|_| "Failed to read dcr.toml".to_string())?;
    let name = get_config_str(&config, "package.name");
    let language = get_language_with_profile(&config, profile)?;
    let compiler_cfg = get_string_with_profile(&config, "compiler", profile);
    let standard = get_string_with_profile(&config, "standard", profile);
    let package_type = get_config_str(&config, "package.type");
    let build_kind = get_string_with_profile(&config, "kind", profile);
    let tc_cc = get_config_opt(&config, "toolchain.cc");
    let tc_cxx = get_config_opt(&config, "toolchain.cxx");
    let tc_as = get_config_opt(&config, "toolchain.as");
    let compiler = resolve_compiler(
        &language,
        &compiler_cfg,
        tc_cc.as_deref(),
        tc_cxx.as_deref(),
        tc_as.as_deref(),
    );

    let mut cflags: Vec<String> = flags(profile)
        .unwrap_or(&[])
        .iter()
        .map(|v| v.to_string())
        .collect();
    cflags.extend(get_list_with_profile(&config, "cflags", profile));
    let ldflags = get_list_with_profile(&config, "ldflags", profile);
    let pkg_configs = get_list_with_profile(&config, "pkg_config", profile);
    let (cflags, ldflags) = resolve_pkg_config_flags(&pkg_configs, &cflags, &ldflags)?;
    let includes = test_include_dirs(&config, profile);
    let test_sources = collect_test_sources()?;
    if test_sources.is_empty() {
        return Err("tests/test.c not found; run 'dcr test --init' first".to_string());
    }

    let link_project_lib =
        package_type == "lib" || build_kind == "staticlib" || build_kind == "sharedlib";
    let mut stdout = String::new();
    let mut declared = Vec::new();
    let mut suite_success = true;

    fs::create_dir_all("./tests/target")
        .map_err(|e| format!("Failed to create tests/target: {e}"))?;
    for source in &test_sources {
        declared.extend(extract_test_names_path(source));
        let stem = source
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("test");
        let obj_path = Path::new("./tests/target").join(format!("{stem}.o"));
        let bin_path =
            Path::new("./tests/target").join(format!("{stem}{}", std::env::consts::EXE_SUFFIX));

        compile_test_source(&compiler, &standard, &cflags, &includes, source, &obj_path)?;
        link_test_binary(
            &compiler,
            &ldflags,
            link_project_lib.then_some(name.as_str()),
            &obj_path,
            &bin_path,
        )?;

        let out = Command::new(&bin_path)
            .output()
            .map_err(|_| format!("Failed to run `{}`", bin_path.display()))?;
        stdout.push_str(&String::from_utf8_lossy(&out.stdout));
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !stderr.trim().is_empty() {
            eprint!("{}", stderr);
        }
        if !out.status.success() {
            suite_success = false;
        }
    }

    let mut pass = 0;
    let mut skip = 0;
    let mut fail = 0;
    let mut parsed_any = false;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("[PASS] ") {
            println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
            pass += 1;
            parsed_any = true;
            continue;
        }
        if let Some(name) = line.strip_prefix("[SKIP] ") {
            println!("{} {}", colored("[SKIP]", BOLD_BLUE), name);
            skip += 1;
            parsed_any = true;
            continue;
        }
        if let Some(name) = line.strip_prefix("[FAIL] ") {
            println!("{} {}", colored("[FAIL]", BOLD_RED), name);
            fail += 1;
            parsed_any = true;
            continue;
        }
        if let Some((status, name)) = line.split_once('\t') {
            match status {
                "PASS" => {
                    println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
                    pass += 1;
                    parsed_any = true;
                }
                "SKIP" => {
                    println!("{} {}", colored("[SKIP]", BOLD_BLUE), name);
                    skip += 1;
                    parsed_any = true;
                }
                "FAIL" => {
                    println!("{} {}", colored("[FAIL]", BOLD_RED), name);
                    fail += 1;
                    parsed_any = true;
                }
                _ => {}
            }
        }
    }

    if !parsed_any {
        if suite_success {
            for name in &declared {
                println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
            }
            pass = declared.len() as i32;
        } else {
            println!("{} testsuite", colored("[FAIL]", BOLD_RED));
            fail = 1;
        }
    }

    let total = if parsed_any || declared.is_empty() {
        pass + skip + fail
    } else {
        declared.len() as i32
    };

    println!();
    println!("{}", colored("=====================", BOLD_GREEN));
    println!("{}", colored("  Testsuite summary  ", BOLD_GREEN));
    println!("{}", colored("=====================", BOLD_GREEN));
    println!("TOTAL: {}", total);
    print_field("PASS", pass, BOLD_GREEN);
    print_field("SKIP", skip, BOLD_BLUE);
    print_field("FAIL", fail, BOLD_RED);
    println!("{}", colored("=====================", BOLD_GREEN));

    if fail > 0 || !suite_success {
        return Ok(1);
    }
    Ok(0)
}

fn collect_test_sources() -> Result<Vec<PathBuf>, String> {
    let tests_dir = Path::new("./tests");
    if !tests_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut sources = Vec::new();
    for entry in fs::read_dir(tests_dir).map_err(|e| format!("Failed to read tests/: {e}"))? {
        let path = entry
            .map_err(|e| format!("Failed to read tests/: {e}"))?
            .path();
        if path.extension().and_then(|v| v.to_str()) == Some("c") {
            sources.push(path);
        }
    }
    sources.sort();
    Ok(sources)
}

fn test_include_dirs(config: &Config, profile: &str) -> Vec<String> {
    let mut dirs = vec![
        "./tests".to_string(),
        "./src".to_string(),
        "./target/include".to_string(),
    ];
    dirs.extend(get_list_with_profile(config, "include", profile));
    dirs
}

fn compile_test_source(
    compiler: &str,
    standard: &str,
    cflags: &[String],
    includes: &[String],
    source: &Path,
    obj_path: &Path,
) -> Result<(), String> {
    let mut cmd = Command::new(compiler);
    cmd.arg("-c").arg(source).arg("-o").arg(obj_path);
    if !standard.trim().is_empty() {
        cmd.arg(format!("-std={}", standard.trim()));
    }
    for flag in cflags {
        cmd.arg(flag);
    }
    for dir in includes {
        cmd.arg(format!("-I{dir}"));
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to compile {}: {e}", source.display()))?;
    print_command_output(&output);
    if !output.status.success() {
        return Err(format!("Compilation of {} failed", source.display()));
    }
    Ok(())
}

fn link_test_binary(
    compiler: &str,
    ldflags: &[String],
    project_lib: Option<&str>,
    obj_path: &Path,
    bin_path: &Path,
) -> Result<(), String> {
    let mut cmd = Command::new(compiler);
    cmd.arg(obj_path);
    if let Some(name) = project_lib {
        cmd.arg("-Ltarget/lib").arg(format!("-l{name}"));
    }
    for flag in ldflags {
        cmd.arg(flag);
    }
    cmd.arg("-o").arg(bin_path);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to link {}: {e}", bin_path.display()))?;
    print_command_output(&output);
    if !output.status.success() {
        return Err(format!("Linking of {} failed", bin_path.display()));
    }
    Ok(())
}

fn print_command_output(output: &std::process::Output) {
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn ensure_test_header() -> Result<(), String> {
    fs::create_dir_all("./tests").map_err(|_| "Failed to create tests/".to_string())?;
    fs::write("./tests/dcr_test.h", FILE_DCR_TEST_H)
        .map_err(|_| "Failed to write tests/dcr_test.h".to_string())?;
    println!("Created tests/dcr_test.h");
    fs::write("./tests/test.c", FILE_TEST_C)
        .map_err(|_| "Failed to write tests/test.c".to_string())?;
    println!("Created tests/test.c");
    Ok(())
}

fn extract_test_names_path(path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("TEST_CASE(")
            && let Some(name) = rest.strip_suffix("),")
        {
            names.push(name.to_string());
        }
    }
    names
}

fn print_field(label: &str, value: i32, color: &str) {
    if value == 0 {
        println!("{}:  {}", label, value);
    } else {
        println!("{}{}:  {}{}", color, label, value, RESET);
    }
}
