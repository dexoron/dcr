use crate::utils::log::error;
use crate::utils::text::{BOLD_GREEN, colored};
use glob::glob;
use std::process::Command;

pub fn fmt(_args: &[String]) -> i32 {
    let patterns = [
        "src/**/*.c",
        "src/**/*.cpp",
        "src/**/*.h",
        "src/**/*.hpp",
        "tests/**/*.c",
        "tests/**/*.cpp",
        "tests/**/*.h",
        "tests/**/*.hpp",
    ];

    let mut files = Vec::new();
    for pattern in patterns {
        if let Ok(paths) = glob(pattern) {
            for path in paths.flatten() {
                if let Some(s) = path.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }

    if files.is_empty() {
        println!("    {} No files to format", colored("Format", BOLD_GREEN));
        return 0;
    }

    println!(
        "    {} {} files",
        colored("Formatting", BOLD_GREEN),
        files.len()
    );

    let status = Command::new("clang-format").arg("-i").args(&files).status();

    match status {
        Ok(s) if s.success() => {
            println!("    {} successful", colored("Format", BOLD_GREEN));
            0
        }
        Ok(s) => {
            error(&format!("clang-format failed with status: {}", s));
            1
        }
        Err(e) => {
            error(&format!("Failed to execute clang-format: {}", e));
            1
        }
    }
}
