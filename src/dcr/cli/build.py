import os
import subprocess
from dcr.config import flags


def check_dir(dir: str | None = None) -> list[str]:
    if dir in (None, ".", "./"):
        return os.listdir(os.getcwd())
    else:
        return os.listdir(os.getcwd() + "/" + dir)


def build(args: list[str] | str) -> int:
    if "dcr.toml" not in check_dir():
        print("error: Not font dcr.toml")
        return 1
    if "target" not in check_dir():
        os.mkdir("./target")

    if len(args) == 1 and args[0] in flags:
        profile: str = args[0][2:]
    else:
        print("error: Unkown flags")
        return 1
    if len(args) == 0:
        profile: str = "debug"

    if profile not in check_dir("target"):
        os.mkdir(f"./target/{profile}")
    if "main.c" in check_dir("src"):
        print(f"Run build project as profiles {profile}")
        subprocess.run(["gcc", "./src/main.c", "-o", f"./target/{profile}/main"])
        print("Build complite")
    return 0
