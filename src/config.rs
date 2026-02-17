pub const PROJECT_VERSION: &str = "0.1.0";
pub const PROJECT_LANGUAGE: &str = "c";
pub const PROJECT_COMPILER: &str = "clang";
pub const PROFILE: &str = "debug";

pub fn file_dcr_toml(project_name: &str) -> String {
    format!(
        r#"[package]
name = "{project_name}"
version = "{project_version}"
language = "{project_language}"
compiler = "{project_compiler}"

[dependencies]
"#,
        project_version = PROJECT_VERSION,
        project_language = PROJECT_LANGUAGE,
        project_compiler = PROJECT_COMPILER
    )
}

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
