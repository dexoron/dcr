import os
import shutil
from ..utils.fs import check_dir
from ..utils.text import colored, BRIGHT_RED, BOLD, BRIGHT_GREEN, BRIGHT_YELLOW
from ..config import flags


def clean(args: list[str] | None = None) -> int:
    project_name = os.path.basename(os.getcwd())
    items: list[str] = check_dir()

    if args and len(args) > 1:
        print(colored("error", BRIGHT_RED + BOLD) + ": неизвестный аргумент")
        return 1

    if "dcr.toml" not in items:
        print(colored("error", BRIGHT_RED + BOLD) + ": не найден файл dcr.toml")
        return 1

    print("    Очистка проекта `" + colored(project_name, BRIGHT_GREEN + BOLD) + "`")

    if "target" not in items:
        print(colored("warn", BRIGHT_YELLOW + BOLD) + ": директория target не найдена")
        return 1

    if args:
        profile = args[0]

        if profile.startswith("--"):
            profile = profile[2:]

        if profile not in flags:
            print(colored("error", BRIGHT_RED + BOLD) + ": неизвестный профиль")
            return 1

        if profile not in check_dir("target"):
            print(
                colored("warn", BRIGHT_YELLOW + BOLD)
                + f": директория target/{profile} не найдена"
            )
            return 1

        print("    Профиль: " + colored(profile, BRIGHT_GREEN + BOLD))
        shutil.rmtree(os.path.join("target", profile))
        print(
            colored("\n    ✔", BRIGHT_GREEN + BOLD)
            + f" Удалена директория target/{profile}"
        )
        return 0

    shutil.rmtree("target")
    print(colored("\n    ✔", BRIGHT_GREEN + BOLD) + " Удалена директория target")
    return 0
