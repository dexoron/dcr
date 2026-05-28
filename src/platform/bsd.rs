pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
        None => {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let target = format!("{arch}-unknown-{os}");
            format!("./target/{target}/{profile}/{name}")
        }
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/lib{}.a", dir.trim_end_matches('/'), name),
        None => {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let target = format!("{arch}-unknown-{os}");
            format!("./target/{target}/{profile}/lib{name}.a")
        }
    }
}

pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
        None => {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let target = format!("{arch}-unknown-{os}");
            format!("./target/{target}/{profile}/{name}")
        }
    }
}

pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.efi", dir.trim_end_matches('/'), name),
        None => {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let target = format!("{arch}-unknown-{os}");
            format!("./target/{target}/{profile}/{name}.efi")
        }
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/lib{}.so", dir.trim_end_matches('/'), name),
        None => {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let target = format!("{arch}-unknown-{os}");
            format!("./target/{target}/{profile}/lib{name}.so")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_path_default() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let expected = format!("./target/{arch}-unknown-{os}/debug/hello");
        assert_eq!(bin_path("debug", "hello", None), expected);
    }

    #[test]
    fn lib_path_default() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let expected = format!("./target/{arch}-unknown-{os}/debug/libmylib.a");
        assert_eq!(lib_path("debug", "mylib", None), expected);
    }

    #[test]
    fn shared_lib_path_default() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let expected = format!("./target/{arch}-unknown-{os}/debug/libmylib.so");
        assert_eq!(shared_lib_path("debug", "mylib", None), expected);
    }
}
