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
mod path_util;
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
pub mod posix;
#[cfg(target_os = "windows")]
pub mod windows;

/// Host OS path for a binary artifact.
///
/// Backend is chosen at compile time via `cfg!(target_os = …)`, not by the build target triple.
///
/// # Parameters
/// - `profile`: Build profile directory segment.
/// - `name`: Binary basename (may already include extension on some hosts).
/// - `target_dir`: Optional absolute/relative artifact directory override.
///
/// # Returns
/// Path string suitable for the current host layout.
pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::bin_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        macos::bin_path(profile, name, target_dir)
    }
    #[cfg(target_os = "windows")]
    {
        windows::bin_path(profile, name, target_dir)
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        bsd::bin_path(profile, name, target_dir)
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
        posix::bin_path(profile, name, target_dir)
    }
}

/// Host OS path for an ELF-style artifact (`profile`, `name`, optional `target_dir`).
///
/// Backend is chosen at compile time via `cfg!(target_os = …)`, not by the build target triple.
pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::elf_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        macos::elf_path(profile, name, target_dir)
    }
    #[cfg(target_os = "windows")]
    {
        windows::elf_path(profile, name, target_dir)
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        bsd::elf_path(profile, name, target_dir)
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
        posix::elf_path(profile, name, target_dir)
    }
}

/// Host OS path for an EFI (`.efi`) artifact (`profile`, `name`, optional `target_dir`).
///
/// Backend is chosen at compile time via `cfg!(target_os = …)`, not by the build target triple.
pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::efi_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        macos::efi_path(profile, name, target_dir)
    }
    #[cfg(target_os = "windows")]
    {
        windows::efi_path(profile, name, target_dir)
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        bsd::efi_path(profile, name, target_dir)
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
        posix::efi_path(profile, name, target_dir)
    }
}

/// Host OS path for a static library (`profile`, `name`, optional `target_dir`).
///
/// Backend is chosen at compile time via `cfg!(target_os = …)`, not by the build target triple.
pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        macos::lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "windows")]
    {
        windows::lib_path(profile, name, target_dir)
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        bsd::lib_path(profile, name, target_dir)
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
        posix::lib_path(profile, name, target_dir)
    }
}

/// Host OS path for a shared library (`profile`, `name`, optional `target_dir`).
///
/// Backend is chosen at compile time via `cfg!(target_os = …)`, not by the build target triple.
pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::shared_lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "macos")]
    {
        macos::shared_lib_path(profile, name, target_dir)
    }
    #[cfg(target_os = "windows")]
    {
        windows::shared_lib_path(profile, name, target_dir)
    }
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        bsd::shared_lib_path(profile, name, target_dir)
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
        posix::shared_lib_path(profile, name, target_dir)
    }
}
