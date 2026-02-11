import os


def new(args: list[str]) -> int:
    items: list[str] = os.listdir(os.getcwd())
    if len(args) == 0 or len(args) > 1:
        print("ERROR")
        return 1
    project_name: str = args[0]
    if project_name in items:
        print("ERROR")
        return 1

    os.makedirs(f"./{project_name}/src")

    with open(f"./{project_name}/src/main.c", "w") as file:
        file.write(
            "#include <stdio.h>\n\n"
            "int main(void) {\n"
            '    printf("Hello World!\\n");\n'
            "    return 0;\n"
            "}\n"
        )

    with open(f"./{project_name}/dcr.toml", "w") as file:
        file.write(
            "[package]\n"
            f'name = "{project_name}"\n'
            'version = "0.1.0"\n'
            'edition = "2026"\n\n'
            "[dependencies]\n"
        )

    return 0
