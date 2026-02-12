from ..utils.text import colored, BRIGHT_RED, BOLD, printc, BRIGHT_GREEN, BRIGHT_CYAN
from ..config import (
    file_dcr_toml,
    file_main_c,
    project_version,
    project_language,
    project_compiler,
)
import os
from ..utils.fs import check_dir


def init(args: list[str] | None = None) -> int:
    if args:
        print(
            colored("error", BRIGHT_RED + BOLD)
            + ": команда не поддерживает доп. аргументы"
        )
        return 1
    items: list[str] = check_dir()
    project_name = os.path.basename(os.getcwd())

    if items:
        print(colored("error", BRIGHT_RED + BOLD) + ": директория не пуста")
        return 1

    print("Инициализация проекта в " + os.getcwd())

    with open("./dcr.toml", "w") as file:
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

    os.mkdir("src")
    with open("./src/main.c", "w") as file:
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
    printc("    dcr run", BRIGHT_CYAN + BOLD)

    return 0
