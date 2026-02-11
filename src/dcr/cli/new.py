from dcr.config import (
    file_dcr_toml,
    file_main_c,
    project_version,
    project_language,
    project_compiler,
)
import os
from dcr.utils.fs import check_dir


def new(args: list[str]) -> int:
    items: list[str] = check_dir()
    if len(args) == 0:
        print("Ошибка: не указано название проекта")
        return 1
    if len(args) > 1:
        print("Ошибка: команда не поддежривает аргументы")
        return 1
    project_name: str = args[0]
    if project_name in items:
        print(f"Ошибка: директория {project_name} занята")
        return 1

    os.makedirs(f"./{project_name}/src")

    with open(f"./{project_name}/src/main.c", "w") as file:
        file.write(file_main_c)

    with open(f"./{project_name}/dcr.toml", "w") as file:
        file.write(
            file_dcr_toml.format(
                project_name=project_name,
                project_version=project_version,
                project_language=project_language,
                project_compiler=project_compiler,
            )
        )

    return 0
