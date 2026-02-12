from src.dcr.config import VERSION
import dcr.cli as cli
import sys


def main() -> int:
    if len(sys.argv) < 2:
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
        case "clean":
            cli.clean(args)
        case "--version":
            print(f"dcr {VERSION} (GNU/Linux)")
        case "--help":
            cli.help()
        case "--update":
            cli.flag_update(args)
        case _:
            print("Неизвестная команда или аргумент")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
