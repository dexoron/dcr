pub const PROFILE: &str = "debug";
pub const FILE_MAIN_C: &str = r#"#include <stdio.h>

int main(void) {
    printf("Hello World!\n");
    return 0;
}
"#;

pub fn flags(profile: &str) -> Option<&'static [&'static str]> {
    match profile {
        "release" => Some(&["-O3", "-DNDEBUG", "-march=native"]),
        "debug" => Some(&[
            "-O0",
            "-g",
            "-Wall",
            "-Wextra",
            "-fno-omit-frame-pointer",
            "-DDEBUG",
        ]),
        _ => None,
    }
}
