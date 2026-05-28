// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub mod bsd;
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
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        return bsd::bin_path(profile, name, target_dir);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        match target_dir {
            Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/{name}"),
        }
    }
}

pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::elf_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::elf_path(profile, name, target_dir);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::elf_path(profile, name, target_dir);
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        return bsd::elf_path(profile, name, target_dir);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        match target_dir {
            Some(dir) => format!("{}/{}", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/{name}"),
        }
    }
}

pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::efi_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        return macos::efi_path(profile, name, target_dir);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::efi_path(profile, name, target_dir);
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        return bsd::efi_path(profile, name, target_dir);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        match target_dir {
            Some(dir) => format!("{}/{}.efi", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/{name}.efi"),
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
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        return bsd::lib_path(profile, name, target_dir);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
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
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        return bsd::shared_lib_path(profile, name, target_dir);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        match target_dir {
            Some(dir) => format!("{}/lib{}.so", dir.trim_end_matches('/'), name),
            None => format!("./target/{profile}/lib{name}.so"),
        }
    }
}
