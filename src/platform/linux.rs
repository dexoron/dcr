pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}"),
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/lib{}.a", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/lib{name}.a"),
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/lib{}.so", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/lib{name}.so"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_path_default() {
        assert_eq!(bin_path("debug", "hello", None), "./target/debug/hello");
    }

    #[test]
    fn bin_path_release() {
        assert_eq!(
            bin_path("release", "hello", None),
            "./target/release/hello"
        );
    }

    #[test]
    fn bin_path_custom_target() {
        assert_eq!(bin_path("debug", "hello", Some("out")), "out/hello");
    }

    #[test]
    fn bin_path_custom_target_trailing_slash() {
        assert_eq!(bin_path("debug", "hello", Some("out/")), "out/hello");
    }

    #[test]
    fn lib_path_default() {
        assert_eq!(
            lib_path("debug", "mylib", None),
            "./target/debug/libmylib.a"
        );
    }

    #[test]
    fn lib_path_custom_target() {
        assert_eq!(lib_path("debug", "mylib", Some("out")), "out/libmylib.a");
    }

    #[test]
    fn shared_lib_path_default() {
        assert_eq!(
            shared_lib_path("debug", "mylib", None),
            "./target/debug/libmylib.so"
        );
    }

    #[test]
    fn shared_lib_path_custom_target() {
        assert_eq!(
            shared_lib_path("release", "mylib", Some("dist")),
            "dist/libmylib.so"
        );
    }
}

