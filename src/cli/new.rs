use crate::config::FILE_MAIN_C;
use crate::core::config::Config;
use crate::utils::fs::check_dir;
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_CYAN, BOLD_GREEN, colored, printc};
use std::fs;
use std::io::Write;
use toml::Value;

pub fn new(args: &[String]) -> i32 {
    let items = check_dir(None).unwrap_or_default();

    if args.is_empty() {
        error("Project name not specified");
        return 1;
    }
    if args.len() > 1 {
        warn("Command does not support additional arguments");
        return 1;
    }

    let project_name = &args[0];
    println!(
        "Creating a Project `{}`...",
        colored(project_name, BOLD_CYAN)
    );

    if items.contains(project_name) {
        error(&format!(
            "Directory `{}` already exists\n",
            colored(project_name, BOLD_CYAN)
        ));
        printc("Подсказка:", BOLD_CYAN);
        println!(
            "    Use `{}` to initialize an existing project\n    or specify a different project name",
            colored("dcr init", BOLD_CYAN)
        );
        return 1;
    }

    if fs::create_dir(project_name).is_err() {
        error("Failed to create directory");
        return 1;
    }
    println!(
        "    {} Directory created {}",
        colored("✔", BOLD_GREEN),
        project_name
    );

    let toml_path = format!("./{project_name}/dcr.toml");
    let mut config = match Config::new(&toml_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            error("Failed to create dcr.toml");
            return 1;
        }
    };
    if config
        .edit("package.name", Value::String(project_name.to_string()))
        .is_err()
    {
        error("Failed to write dcr.toml");
        return 1;
    }
    println!(
        "    {} Created file {}",
        colored("✔", BOLD_GREEN),
        colored("dcr.toml", BOLD_CYAN)
    );

    if fs::create_dir_all(format!("./{project_name}/src")).is_err() {
        error("Failed to create src/");
        return 1;
    }
    let main_c_path = format!("./{project_name}/src/main.c");
    let mut main_c = match fs::File::create(&main_c_path) {
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
        colored(project_name, BOLD_GREEN)
    );
    printc("Next step:", BOLD_GREEN);
    printc(&format!("    cd {}\n    dcr run", project_name), BOLD_CYAN);
    0
}
