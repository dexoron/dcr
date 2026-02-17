use crate::config::flags;
use crate::utils::fs::check_dir;
use crate::utils::text::{BOLD, BRIGHT_GREEN, BRIGHT_RED, BRIGHT_YELLOW, colored};
use std::fs;

pub fn clean(args: &[String]) -> i32 {
    let red_bold = BRIGHT_RED.to_owned() + BOLD;
    let green_bold = BRIGHT_GREEN.to_owned() + BOLD;
    let yellow_bold = BRIGHT_YELLOW.to_owned() + BOLD;

    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());
    let items = check_dir(None).unwrap_or_default();

    if args.len() > 1 {
        println!("{}: неизвестный аргумент", colored("error", &red_bold));
        return 1;
    }
    if !items.contains(&"dcr.toml".to_string()) {
        println!("{}: не найден файл dcr.toml", colored("error", &red_bold));
        return 1;
    }

    println!(
        "    Очистка проекта `{}`",
        colored(&project_name, &green_bold)
    );
    if !items.contains(&"target".to_string()) {
        println!(
            "{}: директория target не найдена",
            colored("warn", &yellow_bold)
        );
        return 1;
    }

    if let Some(arg) = args.first() {
        let mut profile = arg.clone();
        if profile.starts_with("--") {
            profile = profile.trim_start_matches("--").to_string();
        }
        if flags(&profile).is_none() {
            println!("{}: неизвестный профиль", colored("error", &red_bold));
            return 1;
        }

        let target_items = check_dir(Some("target")).unwrap_or_default();
        if !target_items.contains(&profile) {
            println!(
                "{}: директория target/{profile} не найдена",
                colored("warn", &yellow_bold)
            );
            return 1;
        }

        println!("    Профиль: {}", colored(&profile, &green_bold));
        let _ = fs::remove_dir_all(format!("target/{profile}"));
        println!(
            "{} Удалена директория target/{profile}",
            colored("\n    ✔", &green_bold)
        );
        return 0;
    }

    let _ = fs::remove_dir_all("target");
    println!(
        "{} Удалена директория target",
        colored("\n    ✔", &green_bold)
    );
    0
}
