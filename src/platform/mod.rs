#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub fn bin_path(profile: &str, name: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::bin_path(profile, name)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::bin_path(profile, name);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::bin_path(profile, name);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        format!("./target/{profile}/{name}")
    }
}
