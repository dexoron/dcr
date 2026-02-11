import os


def init(args) -> int:
    items = os.listdir(os.getcwd())
    project_name = os.path.basename(os.getcwd())

    if items:
        print("ERROR: directory is not empty")
        return 1

    os.mkdir("src")

    with open("./src/main.c", "w") as file:
        file.write(
            "#include <stdio.h>\n\n"
            "int main(void) {\n"
            '    printf("Hello World!\\n");\n'
            "    return 0;\n"
            "}\n"
        )

    with open("./dcr.toml", "w") as file:
        file.write(
            "[package]\n"
            f'name = "{project_name}"\n'
            'version = "0.1.0"\n'
            'edition = "2026"\n\n'
            "[dependencies]\n"
        )

    return 0
