from ..utils.fs import check_dir
import shutil


def clean(args: list[str]) -> int:
    items: list[str] = check_dir()
    if len(args) >= 1:
        print("Ошибка: команда не поддежривает аргументы")
        return 1
    if "dcr.toml" not in items:
        print("Ошибка: вы находитесь не в корне проекта")
        return 1
    if "target" not in items:
        print("Вы уже очистили сборки")
        return 0

    shutil.rmtree("target")
    print("Сборки очишены")
    return 0
