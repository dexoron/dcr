import time
import os
import subprocess
from ..config import flags, profile, c_comp
from dcr.utils.fs import check_dir


def build(args: list[str] | None = None) -> int:
    active_profile: str = profile
    if "dcr.toml" not in check_dir():
        print("Ошибка: не найден файл dcr.toml")
        return 1
    if "target" not in check_dir():
        os.mkdir("./target")

    if args:
        if len(args) >= 1 and args[0].startswith("--"):
            candidate: str = args[0][2:]
            if candidate in flags:
                active_profile: str = candidate
            else:
                print("Ошибка: неверный флаг сборки")
                return 1
        else:
            print("Ошибка: флаг сборки не найден")
            return 1

    if active_profile not in check_dir("target"):
        os.mkdir(f"./target/{active_profile}")
    if "main.c" in check_dir("src"):
        print(f"Запуск сборки с профилем {active_profile}")
        compile_flags = flags[active_profile]
        start_time: float = time.time()
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
        print(f"Сборка завершена, за {times} секунд")
    return 0
