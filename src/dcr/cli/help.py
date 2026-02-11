def help() -> int:
    print(
        "\n".join(
            [
                "DCR (Dexoron Cargo Realization)",
                "Менеджер C-проектов, вдохновленный Cargo.",
                "",
                "ИСПОЛЬЗОВАНИЕ:",
                "    dcr <команда> [опции]",
                "",
                "КОМАНДЫ:",
                "    new <name>        Создать новый проект",
                "    init              Инициализировать текущую директорию как проект",
                "    build [--profile] Собрать проект (по умолчанию: --debug)",
                "    run [--profile]   Собрать и запустить (по умолчанию: --debug)",
                "    clean             Удалить директорию target",
                "    help              Показать эту справку",
                "",
                "ОПЦИИ:",
                "    --debug           Сборка с профилем debug",
                "    --release         Сборка с профилем release",
                "",
                "ПРИМЕРЫ:",
                "    dcr new hello",
                "    dcr build --release",
                "    dcr run --debug",
                "",
                "ПОДСКАЗКА:",
                "    Запусти 'dcr <команда> --help' для справки по команде.",
            ]
        )
    )
    return 0
