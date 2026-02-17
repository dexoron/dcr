use crate::config::{FILE_MAIN_C, file_dcr_toml};
use crate::utils::fs::check_dir;
use crate::utils::text::{BOLD, BRIGHT_CYAN, BRIGHT_GREEN, BRIGHT_RED, colored, printc};
use std::fs;
use std::io::Write;

pub fn new(args: &[String]) -> i32 {
    let items = check_dir(None).unwrap_or_default();
    let red_bold = BRIGHT_RED.to_owned() + BOLD;
    let green_bold = BRIGHT_GREEN.to_owned() + BOLD;
    let cyan_bold = BRIGHT_CYAN.to_owned() + BOLD;

    if args.is_empty() {
        println!(
            "{}: не указанно название проекта",
            colored("error", &red_bold)
        );
        return 1;
    }
    if args.len() > 1 {
        println!(
            "{}: команда не поддерживает доп. аргументы",
            colored("error", &red_bold)
        );
        return 1;
    }

    let project_name = &args[0];
    println!(
        "Создание проекта `{}`...",
        colored(project_name, &green_bold)
    );

    if items.contains(project_name) {
        println!(
            "{}: директория `{}` уже существует\n",
            colored("error", &red_bold),
            colored(project_name, &green_bold)
        );
        printc("Подсказка:", &green_bold);
        println!(
            "    Используй `{}` для инициализации существующего проекта\n    или задай другое имя проекта",
            colored("dcr init", &cyan_bold)
        );
        return 1;
    }

    if fs::create_dir(project_name).is_err() {
        println!(
            "{}: не удалось создать директорию",
            colored("error", &red_bold)
        );
        return 1;
    }
    println!(
        "    {} Создана директория {}",
        colored("✔", &green_bold),
        project_name
    );

    let toml_path = format!("./{project_name}/dcr.toml");
    let mut dcr_toml = match fs::File::create(&toml_path) {
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
        .write_all(file_dcr_toml(project_name).as_bytes())
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

    if fs::create_dir_all(format!("./{project_name}/src")).is_err() {
        println!("{}: не удалось создать src", colored("error", &red_bold));
        return 1;
    }
    let main_c_path = format!("./{project_name}/src/main.c");
    let mut main_c = match fs::File::create(&main_c_path) {
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
        colored(project_name, &green_bold)
    );
    printc("Следуюший шаг:", &green_bold);
    printc(&format!("    cd {project_name}\n    dcr run"), &cyan_bold);
    0
}
