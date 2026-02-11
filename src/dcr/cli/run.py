import os
import subprocess

from ..config import flags, profile
from .build import build


def check_dir(dir: str | None = None) -> list[str]:
    if dir in (None, ".", "./"):
        return os.listdir(os.getcwd())
    else:
        return os.listdir(os.getcwd() + "/" + dir)


def run(args: list[str] | None = None) -> int:
    active_profile: str = profile
    if "dcr.toml" not in os.listdir(os.getcwd()):
        print("Ошибка: не найден файл dcr.toml")
        return 1

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

    build(args)
    print("Run project")
    return subprocess.run([f"./target/{active_profile}/main"]).returncode
