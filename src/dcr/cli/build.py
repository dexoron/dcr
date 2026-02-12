import time
import os
import subprocess
from ..config import flags, profile, c_comp
from ..utils.fs import check_dir
from ..utils.text import colored, BRIGHT_GREEN, BOLD, BRIGHT_RED


def build(args: list[str] | None = None) -> int:
    active_profile: str = profile
    project_name = os.path.basename(os.getcwd())
    if "dcr.toml" not in check_dir():
        print(colored("error", BRIGHT_RED + BOLD) + ": не найден файл dcr.toml")
        return 1
    if args:
        if len(args) >= 1 and args[0].startswith("--"):
            candidate: str = args[0][2:]
            if candidate in flags:
                active_profile: str = candidate
            else:
                print(colored("error", BRIGHT_RED + BOLD) + ": неизвестный флаг сборки")
                return 1
        else:
            print(colored("error", BRIGHT_RED + BOLD) + ": неизвестный аргумент")
            return 1

    print(
        "    Сборка проекта `"
        + colored(project_name, BRIGHT_GREEN + BOLD)
        + "`\n    Профиль: "
        + colored(active_profile, BRIGHT_GREEN + BOLD)
        + "\n    Компилятор: "
        + colored(c_comp, BRIGHT_GREEN + BOLD)
        + "\n"
    )

    if "target" not in check_dir():
        os.mkdir("./target")
    if active_profile not in check_dir("target"):
        os.mkdir(f"./target/{active_profile}")
    if "main.c" in check_dir("src"):
        compile_flags = flags[active_profile]
        start_time: float = time.time()
        try:
            subprocess.run(
                [
                    c_comp,
                    "./src/main.c",
                    *compile_flags,
                    "-o",
                    f"./target/{active_profile}/main",
                ],
                check=True,
            )
            end_time: float = time.time()
            times: float = end_time - start_time
            times: float = int(times * 100) / 100
            print(
                "    "
                + colored("✔", BRIGHT_GREEN + BOLD)
                + " Сборка завершена успешно, за "
                + colored(str(times), BRIGHT_GREEN + BOLD)
                + " секунд"
            )
            return 0
        except:
            print(colored("error", BRIGHT_RED + BOLD) + ": сборка не удалась")
            return 1

    return 0
