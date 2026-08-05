/// POSIX platform-specific path utilities.
/// Provides functions to build paths for binaries, libraries, and EFI files based on profile and optional target directory.
use super::path_util::{default_bin_rel, default_lib_rel, join_dir};

/// Returns the path to a binary executable for the given profile, name, and optional target directory.
/// If a target directory is provided, it joins the directory with the name; otherwise, it uses the default binary relative path.
pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => join_dir(dir, name),
        None => default_bin_rel(profile, name, None),
    }
}

/// Returns the path to a static library for the given profile, name, and optional target directory.
/// The library file is named "lib{name}.a". If a target directory is provided, it joins the directory with the file name; otherwise, it uses the default library relative path.
pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.a");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

/// Returns the path to an ELF binary for the given profile, name, and optional target directory.
/// Delegates to bin_path since ELF binaries are standard binaries.
pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    bin_path(profile, name, target_dir)
}

/// Returns the path to an EFI executable for the given profile, name, and optional target directory.
/// The EFI file is named "{name}.efi". If a target directory is provided, it joins the directory with the file name; otherwise, it uses the default library relative path.
pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("{name}.efi");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}

/// Returns the path to a shared library for the given profile, name, and optional target directory.
/// The shared library file is named "lib{name}.so". If a target directory is provided, it joins the directory with the file name; otherwise, it uses the default library relative path.
pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    let file = format!("lib{name}.so");
    match target_dir {
        Some(dir) => join_dir(dir, &file),
        None => default_lib_rel(profile, &file, None),
    }
}
