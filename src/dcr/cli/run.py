import os
import subprocess

from dcr.config import flags
from .build import build


def check_dir(dir: str | None = None) -> list[str]:
    if dir in (None, ".", "./"):
        return os.listdir(os.getcwd())
    else:
        return os.listdir(os.getcwd() + "/" + dir)


def run(args: list[str]) -> int:
    if "dcr.toml" not in os.listdir(os.getcwd()):
        print("error: Not fond dcr.toml")
        return 1

    if len(args) == 0:
        profile: str = "debug"
    elif len(args) == 1 and args[0] in flags:
        profile = args[0][2:]
    else:
        print("error: Unkown flags")
        return 1

    build(args)
    print("Run project")
    return subprocess.run([f"./target/{profile}/main"]).returncode
