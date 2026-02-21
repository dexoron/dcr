pub fn bin_path(profile: &str, name: &str) -> String {
    format!("./target/{profile}/{name}")
}
