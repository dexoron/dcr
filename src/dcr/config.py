profile: str = "debug"
c_comp: str = "clang"
flags: dict[str, list[str]] = {
    "release": ["-O3", "-DNDEBUG", "-march=native"],
    "debug": ["-O0", "-g", "-Wall", "-Wextra", "-fno-omit-frame-pointer", "-DDEBUG"],
}
