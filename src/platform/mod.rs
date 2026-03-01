#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::bin_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::bin_path(profile, name, target_dir);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::bin_path(profile, name, target_dir);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        match target_dir {
            Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/{name}"),
        }
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::lib_path(profile, name, target_dir);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::lib_path(profile, name, target_dir);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        match target_dir {
            Some(dir) => format!("{}/lib{}.a", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/lib{name}.a"),
        }
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::shared_lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::shared_lib_path(profile, name, target_dir);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::shared_lib_path(profile, name, target_dir);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        match target_dir {
            Some(dir) => format!("{}/lib{}.so", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/lib{name}.so"),
        }
    }
}
