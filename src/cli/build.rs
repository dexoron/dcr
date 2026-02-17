use crate::config::{PROFILE, PROJECT_COMPILER, flags};
use crate::utils::fs::check_dir;
use crate::utils::text::{BOLD, BRIGHT_GREEN, BRIGHT_RED, colored};
use std::fs;
use std::process::Command;
use std::time::Instant;

pub fn build(args: &[String]) -> i32 {
    let mut active_profile = PROFILE.to_string();
    let red_bold = BRIGHT_RED.to_owned() + BOLD;
    let green_bold = BRIGHT_GREEN.to_owned() + BOLD;
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

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

    println!(
        "    Сборка проекта `{}`\n    Профиль: {}\n    Компилятор: {}\n",
        colored(&project_name, &green_bold),
        colored(&active_profile, &green_bold),
        colored(PROJECT_COMPILER, &green_bold)
    );

    if !items.contains(&"target".to_string()) {
        let _ = fs::create_dir("./target");
    }
    let target_items = check_dir(Some("target")).unwrap_or_default();
    if !target_items.contains(&active_profile) {
        let _ = fs::create_dir(format!("./target/{active_profile}"));
    }

    let src_items = check_dir(Some("src")).unwrap_or_default();
    if src_items.contains(&"main.c".to_string()) {
        let compile_flags = flags(&active_profile).unwrap_or(&[]);
        let start_time = Instant::now();
        let mut cmd = Command::new(PROJECT_COMPILER);
        cmd.arg("./src/main.c");
        for flag in compile_flags {
            cmd.arg(flag);
        }
        cmd.arg("-o").arg(format!("./target/{active_profile}/main"));

        match cmd.status() {
            Ok(status) if status.success() => {
                let times = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
                println!(
                    "    {} Сборка завершена успешно, за {} секунд",
                    colored("✔", &green_bold),
                    colored(&times.to_string(), &green_bold)
                );
                return 0;
            }
            _ => {
                println!("{}: сборка не удалась", colored("error", &red_bold));
                return 1;
            }
        }
    }

    0
}
