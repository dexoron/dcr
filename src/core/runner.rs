use crate::platform;
use crate::utils::fs::check_dir;
use crate::utils::log::warn;
use std::path::Path;
use std::process::Command;

pub fn run_binary(project_name: &str, profile: &str, target_dir: Option<&str>) -> i32 {
    let bin_path = platform::bin_path(profile, project_name, target_dir);
    if Path::new(&bin_path).exists() {
        return Command::new(bin_path)
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    if target_dir.is_none()
        && check_dir(Some(&format!("target/{profile}")))
            .unwrap_or_default()
            .contains(&profile.to_string())
    {
        warn("Launch of the latest release");
        return Command::new(platform::bin_path(profile, project_name, target_dir))
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    if target_dir.is_some() && Path::new(&bin_path).exists() {
        warn("Launch of the latest release");
        return Command::new(bin_path)
            .status()
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);
    }

    1
}
