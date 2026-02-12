import platform
import shutil
import subprocess


def flag_update(args: list[str] | None = None) -> int:
    if args and len(args) > 0:
        print("Ошибка: команда не поддежривает аргументы")
        return 1
    if platform.system() != "Linux":
        print("Ошибка: обновление поддерживается только на Linux")
        return 1
    if shutil.which("curl") is None:
        print("Ошибка: не найден curl")
        return 1
    try:
        subprocess.run(
            ["bash", "-c", "curl -fsSL dcr.zov.tatar | bash"], check=True
        )
        return 0
    except subprocess.CalledProcessError:
        print("Ошибка: не удалось выполнить обновление")
        return 1
