use crate::config::{FILE_MAIN_C, file_dcr_toml};
use crate::utils::fs::check_dir;
use crate::utils::text::{BOLD, BRIGHT_CYAN, BRIGHT_GREEN, BRIGHT_RED, colored, printc};
use std::fs;
use std::io::Write;

pub fn init(args: &[String]) -> i32 {
    let red_bold = BRIGHT_RED.to_owned() + BOLD;
    let green_bold = BRIGHT_GREEN.to_owned() + BOLD;
    let cyan_bold = BRIGHT_CYAN.to_owned() + BOLD;

    if !args.is_empty() {
        println!(
            "{}: команда не поддерживает доп. аргументы",
            colored("error", &red_bold)
        );
        return 1;
    }

    let items = check_dir(None).unwrap_or_default();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    if !items.is_empty() {
        println!("{}: директория не пуста", colored("error", &red_bold));
        return 1;
    }

    let cwd = std::env::current_dir()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    println!("Инициализация проекта в {cwd}");

    let mut dcr_toml = match fs::File::create("./dcr.toml") {
        Ok(file) => file,
        Err(_) => {
            println!(
                "{}: не удалось создать dcr.toml",
                colored("error", &red_bold)
            );
            return 1;
        }
    };
    if dcr_toml
        .write_all(file_dcr_toml(&project_name).as_bytes())
        .is_err()
    {
        println!(
            "{}: не удалось записать dcr.toml",
            colored("error", &red_bold)
        );
        return 1;
    }
    println!(
        "    {} Создан файл {}",
        colored("✔", &green_bold),
        colored("dcr.toml", &cyan_bold)
    );

    if fs::create_dir("src").is_err() {
        println!("{}: не удалось создать src", colored("error", &red_bold));
        return 1;
    }
    let mut main_c = match fs::File::create("./src/main.c") {
        Ok(file) => file,
        Err(_) => {
            println!(
                "{}: не удалось создать src/main.c",
                colored("error", &red_bold)
            );
            return 1;
        }
    };
    if main_c.write_all(FILE_MAIN_C.as_bytes()).is_err() {
        println!(
            "{}: не удалось записать src/main.c",
            colored("error", &red_bold)
        );
        return 1;
    }
    println!(
        "    {} Создан файл {}\n",
        colored("✔", &green_bold),
        colored("src/main.c", &cyan_bold)
    );

    println!(
        "Проект `{}` успешно создан\n",
        colored(&project_name, &green_bold)
    );
    printc("Следуюший шаг:", &green_bold);
    printc("    dcr run", &cyan_bold);
    0
}
