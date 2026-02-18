use crate::config::{FILE_MAIN_C, file_dcr_toml};
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use std::fs;
use std::io::Write;

pub fn init(args: &[String]) -> i32 {
    if !args.is_empty() {
        warn("Command does not support additional arguments");
        return 1;
    }

    let items = check_dir(None).unwrap_or_default();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    if !items.is_empty() {
        error("Directory not empty");
        return 1;
    }

    let cwd = std::env::current_dir()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    println!("Initializing the project in {cwd}");

    let mut dcr_toml = match fs::File::create("./dcr.toml") {
        Ok(file) => file,
        Err(_) => {
            error("Failed to create dcr.toml");
            return 1;
        }
    };
    if dcr_toml
        .write_all(file_dcr_toml(&project_name).as_bytes())
        .is_err()
    {
        error("Failed to write dcr.toml");
        return 1;
    }
    println!(
        "    {} Created file {}",
        colored("✔", BOLD_CYAN),
        colored("dcr.toml", BOLD_CYAN)
    );

    if fs::create_dir("src").is_err() {
        error("Failed to create src/");
        return 1;
    }
    let mut main_c = match fs::File::create("./src/main.c") {
        Ok(file) => file,
        Err(_) => {
            error("Failed to create src/main.c");
            return 1;
        }
    };
    if main_c.write_all(FILE_MAIN_C.as_bytes()).is_err() {
        error("Failed to write src/main.c");
        return 1;
    }
    println!(
        "    {} Created file {}\n",
        colored("✔", BOLD_GREEN),
        colored("src/main.c", BOLD_CYAN)
    );

    println!(
        "Project `{}` successfully created\n",
        colored(project_name.as_str(), BOLD_GREEN)
    );
    printc("Next step:", BOLD_GREEN);
    printc("    cd {}\n    dcr run", project_name.as_str());
    0
}
