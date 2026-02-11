import os


def check_dir(dir: str | None = None) -> list[str]:
    if dir in (None, ".", "./"):
        return os.listdir(os.getcwd())
    else:
        return os.listdir(os.getcwd() + "/" + dir)
