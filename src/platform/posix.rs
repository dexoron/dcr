use super::path_util::{default_bin_rel, default_lib_rel, join_dir};

pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => join_dir(dir, name),
        None => default_bin_rel(profile, name, None),
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.a");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("{name}.efi");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.so");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}
