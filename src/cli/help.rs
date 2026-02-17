use crate::utils::text::{BOLD, BRIGHT_CYAN, BRIGHT_GREEN, colored};

pub fn help() -> i32 {
    let green_bold = BRIGHT_GREEN.to_owned() + BOLD;
    let cyan_bold = BRIGHT_CYAN.to_owned() + BOLD;

    println!("DCR (Dexoron Cargo Realization)");
    println!("Менеджер C-проектов, вдохновленный Cargo.");
    println!();
    println!("{}", colored("ИСПОЛЬЗОВАНИЕ:", &green_bold));
    println!("{}", colored("    dcr <команда> [опции]", &cyan_bold));
    println!();
    println!("{}", colored("КОМАНДЫ:", &green_bold));
    println!("    new <name>        Создать новый проект");
    println!("    init              Инициализировать текущую директорию как проект");
    println!("    build [--profile] Собрать проект (по умолчанию: --debug)");
    println!("    run [--profile]   Собрать и запустить (по умолчанию: --debug)");
    println!("    clean             Удалить директорию target");
    println!("{}", colored("ФЛАГИ:", &green_bold));
    println!("    --help            Показать справку по команде");
    println!("    --update          Обновить dcr до актуальной версии");
    println!("    --version         Показать версию dcr");
    println!();
    println!("{}", colored("ОПЦИИ:", &green_bold));
    println!("    --debug           Сборка с профилем debug");
    println!("    --release         Сборка с профилем release");
    println!();
    println!("{}", colored("ПРИМЕРЫ:", &green_bold));
    println!("{}", colored("    dcr new hello", &cyan_bold));
    println!("{}", colored("    dcr build --release", &cyan_bold));
    println!("{}", colored("    dcr run --debug", &cyan_bold));
    println!();
    println!("{}", colored("ПОДСКАЗКА:", &green_bold));
    println!("    Запусти 'dcr <команда> --help' для справки по команде.");
    0
}
