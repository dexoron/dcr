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
