import dcr.cli as cli
import sys


def main() -> int:
    if len(sys.argv) < 2 or sys.argv[1] == "--help":
        cli.help()
        return 0

    cmd: str = sys.argv[1]
    args: str | list[str] = sys.argv[2:]

    match cmd:
        case "new":
            cli.new(args)
        case "init":
            cli.init(args)
        case "build":
            cli.build(args)
        case "run":
            cli.run(args)
        case _:
            print("Unknown command")

    return 0
