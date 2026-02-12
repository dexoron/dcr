from ..utils.text import colored, BRIGHT_GREEN, BRIGHT_CYAN, BOLD


def help() -> int:
    print(
        "\n".join(
            [
                "DCR (Dexoron Cargo Realization)",
                "Менеджер C-проектов, вдохновленный Cargo.",
                "",
                colored("ИСПОЛЬЗОВАНИЕ:", BRIGHT_GREEN + BOLD),
                colored("    dcr <команда> [опции]", BRIGHT_CYAN + BOLD),
                "",
                colored("КОМАНДЫ:", BRIGHT_GREEN + BOLD),
                "    new <name>        Создать новый проект",
                "    init              Инициализировать текущую директорию как проект",
                "    build [--profile] Собрать проект (по умолчанию: --debug)",
                "    run [--profile]   Собрать и запустить (по умолчанию: --debug)",
                "    clean             Удалить директорию target",
                colored("ФЛАГИ:", BRIGHT_GREEN + BOLD),
                "    --help            Показать справку по команде",
                "    --update          Обновить dcr до актуальной версии",
                "    --version         Показать версию dcr",
                "",
                colored("ОПЦИИ:", BRIGHT_GREEN + BOLD),
                "    --debug           Сборка с профилем debug",
                "    --release         Сборка с профилем release",
                "",
                colored("ПРИМЕРЫ:", BRIGHT_GREEN + BOLD),
                colored("    dcr new hello", BRIGHT_CYAN + BOLD),
                colored("    dcr build --release", BRIGHT_CYAN + BOLD),
                colored("    dcr run --debug", BRIGHT_CYAN + BOLD),
                "",
                colored("ПОДСКАЗКА:", BRIGHT_GREEN + BOLD),
                "    Запусти 'dcr <команда> --help' для справки по команде.",
            ]
        )
    )
    return 0
