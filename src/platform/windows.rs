pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.exe", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.exe"),
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.lib", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.lib"),
    }
}
