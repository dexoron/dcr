import os
import subprocess
from ..config import flags, profile, c_comp


def check_dir(dir: str | None = None) -> list[str]:
    if dir in (None, ".", "./"):
        return os.listdir(os.getcwd())
    else:
        return os.listdir(os.getcwd() + "/" + dir)


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
                active_profile = candidate
            else:
                print("Ошибка: неверный флаг сборки")
                return 1
        else:
            print("Ошибка: флаг сборки не найден")
            return 1

    if active_profile not in check_dir("target"):
        os.mkdir(f"./target/{active_profile}")
    if "main.c" in check_dir("src"):
        print(f"Run build project as profiles {active_profile}")
        compile_flags = flags[active_profile]
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
        print("Build complite")
    return 0
