from ..utils.text import (
    colored,
    printc,
    BRIGHT_GREEN,
    BOLD,
    BRIGHT_CYAN,
    BRIGHT_RED,
)
from ..config import (
    file_dcr_toml,
    file_main_c,
    project_version,
    project_language,
    project_compiler,
)
import os
from ..utils.fs import check_dir


def new(args: list[str]) -> int:
    items: list[str] = check_dir()
    if len(args) == 0:
        print(colored("error", BRIGHT_RED + BOLD) + ": не указанно название проекта")
        return 1
    if len(args) > 1:
        print(
            colored("error", BRIGHT_RED + BOLD)
            + ": команда не поддерживает доп. аргументы"
        )
        return 1
    project_name: str = args[0]
    print("Создание проекта `" + colored(project_name, BRIGHT_GREEN + BOLD) + "`...")

    if project_name in items:
        print(
            colored("error", BRIGHT_RED + BOLD)
            + ": директория `"
            + colored(project_name, BRIGHT_GREEN + BOLD)
            + "` уже существует\n"
        )
        printc("Подсказка:", BRIGHT_GREEN + BOLD)
        print(
            "    Используй `"
            + colored("dcr init", BRIGHT_CYAN + BOLD)
            + "` для инициализации существующего проекта\n    или задай другое имя проекта"
        )
        return 1

    os.mkdir(project_name)
    print(
        "    "
        + colored("✔", BRIGHT_GREEN + BOLD)
        + " Создана директория "
        + project_name
    )

    with open(f"./{project_name}/dcr.toml", "w") as file:
        file.write(
            file_dcr_toml.format(
                project_name=project_name,
                project_version=project_version,
                project_language=project_language,
                project_compiler=project_compiler,
            )
        )
    print(
        "    "
        + colored("✔", BRIGHT_GREEN + BOLD)
        + " Создан файл "
        + colored("dcr.toml", BRIGHT_CYAN + BOLD)
    )

    os.makedirs(f"./{project_name}/src")
    with open(f"./{project_name}/src/main.c", "w") as file:
        file.write(file_main_c)
    print(
        "    "
        + colored("✔", BRIGHT_GREEN + BOLD)
        + " Создан файл "
        + colored("src/main.c\n", BRIGHT_CYAN + BOLD)
    )

    print(
        "Проект `" + colored(project_name, BRIGHT_GREEN + BOLD) + "` успешно создан\n"
    )
    printc("Следуюший шаг:", BRIGHT_GREEN + BOLD)
    printc("    cd " + project_name + "\n    dcr run", BRIGHT_CYAN + BOLD)

    return 0
