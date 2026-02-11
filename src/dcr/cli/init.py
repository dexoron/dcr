from dcr.config import (
    file_dcr_toml,
    file_main_c,
    project_version,
    project_language,
    project_compiler,
)
import os
from dcr.utils.fs import check_dir

def init(args: list[str] | None = None) -> int:
    if args:
        print("Ошибка: команда init не поддерживает аргументы")
        return 1
    items = check_dir()
    project_name = os.path.basename(os.getcwd())

    if items:
        print("Ошибка: директория не пустая")
        return 1

    os.mkdir("src")

    with open("./src/main.c", "w") as file:
        file.write(file_main_c)

    with open("./dcr.toml", "w") as file:
        file.write(
            file_dcr_toml.format(
                project_name=project_name,
                project_version=project_version,
                project_language=project_language,
                project_compiler=project_compiler,
            )
        )

    return 0
