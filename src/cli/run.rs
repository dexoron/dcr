use crate::cli::build::build;
use crate::config::{PROFILE, flags};
use crate::utils::fs::check_dir;
use crate::utils::text::{BOLD, BRIGHT_RED, colored};
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let red_bold = BRIGHT_RED.to_owned() + BOLD;
    let mut active_profile = PROFILE.to_string();

    let items = check_dir(None).unwrap_or_default();
    if !items.contains(&"dcr.toml".to_string()) {
        println!("{}: не найден файл dcr.toml", colored("error", &red_bold));
        return 1;
    }

    if let Some(first_arg) = args.first() {
        if first_arg.starts_with("--") {
            let candidate = first_arg.trim_start_matches("--");
            if flags(candidate).is_some() {
                active_profile = candidate.to_string();
            } else {
                println!("{}: неизвестный флаг сборки", colored("error", &red_bold));
                return 1;
            }
        } else {
            println!("{}: неизвестный аргумент", colored("error", &red_bold));
            return 1;
        }
    }

    let build_status = build(args);
    if build_status == 0 {
        println!("\n    Запуск target/{active_profile}/main");
        println!("--------------------------------");
        return Command::new(format!("./target/{active_profile}/main"))
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    if check_dir(Some(&format!("target/{active_profile}")))
        .unwrap_or_default()
        .contains(&active_profile)
    {
        println!("Запуск последнего релиза");
        return Command::new(format!("./target/{active_profile}/main"))
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    println!("Исправьте ошибки в коде для запуска проекта");
    0
}
