import os
import subprocess

from dcr.utils.fs import check_dir
from ..config import flags, profile
from .build import build


def run(args: list[str] | None = None) -> int:
    active_profile: str = profile
    if "dcr.toml" not in check_dir():
        print("Ошибка: не найден файл dcr.toml")
        return 1

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

    build(args)
    print("Запуск проекта")
    return subprocess.run([f"./target/{active_profile}/main"]).returncode
