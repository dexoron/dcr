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

use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Resolves the path to the user's home directory across different operating systems.
///
/// Checks the `HOME` environment variable (Unix-like systems) first,
/// then falls back to `USERPROFILE` (Windows).
pub fn home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(profile));
    }
    None
}

/// Converts a byte slice into a lowercase hexadecimal string.
pub fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Lists all entry names (files and directories) inside a target directory.
#[allow(dead_code)]
pub fn check_dir(dir: Option<&str>) -> io::Result<Vec<String>> {
    let path: PathBuf = match dir {
        None | Some(".") | Some("./") => std::env::current_dir()?,
        Some(value) => std::env::current_dir()?.join(value),
    };

    let mut items = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        items.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(items)
}

/// Searches upwards from a starting path to locate the root directory containing `dcr.toml`.
pub fn find_project_root(start: &Path) -> io::Result<Option<PathBuf>> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("dcr.toml").is_file() {
            return Ok(Some(current));
        }
        if !current.pop() {
            break;
        }
    }
    Ok(None)
}

/// Executes a closure inside a specified directory and restores the original cwd afterwards.
///
/// # Parameters
/// - `dir`: Directory to `cd` into for the duration of `f`.
/// - `f`: Work to run with that cwd (its `Result` is returned as-is).
///
/// # Returns
/// The closure's result, or an error if getting/changing directory fails.
/// Cwd is restored even when `f` returns `Err`.
pub fn with_dir<F, T>(dir: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let prev = std::env::current_dir().map_err(|_| "Failed to get current dir".to_string())?;
    std::env::set_current_dir(dir).map_err(|_| "Failed to change directory".to_string())?;
    let result = f();
    // Restore previous working directory regardless of closure result.
    let _ = std::env::set_current_dir(prev);
    result
}

/// Strips Windows verbatim/extended-length prefixes (e.g. `\\?\`, `\\.\`) and standardizes slashes.
///
/// Normalize path separators for cross-platform safety and removes platform-specific extended prefixes.
pub fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    let mut cleaned = s.replace('\\', "/");
    if let Some(rest) = cleaned.strip_prefix("//?/") {
        cleaned = rest.to_string();
    } else if let Some(rest) = cleaned.strip_prefix("//./") {
        cleaned = rest.to_string();
    } else if let Some(rest) = cleaned.strip_prefix(r"\\?\") {
        cleaned = rest.replace('\\', "/");
    } else if let Some(rest) = cleaned.strip_prefix(r"\\.\") {
        cleaned = rest.replace('\\', "/");
    }
    if cleaned.len() >= 3 && cleaned.as_bytes()[0] == b'/' && cleaned.as_bytes()[2] == b':' {
        cleaned = cleaned[1..].to_string();
    }
    if cfg!(windows) {
        PathBuf::from(cleaned.replace('/', "\\"))
    } else {
        PathBuf::from(cleaned)
    }
}

/// Canonicalizes a path to its absolute form and cleans up Windows extended-length prefixes.
///
/// Falls back to joining relative paths with the current working directory if canonicalization fails.
pub fn canonicalize_path(path: &Path) -> PathBuf {
    let raw = std::fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    strip_verbatim_prefix(raw)
}

/// Resolves a path relative to a root directory and returns a canonicalized absolute path.
pub fn absolute_join(root: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    canonicalize_path(&joined)
}

/// Ensures the `.dcr` internal metadata directory and its `ide` subfolder exist within a project root.
pub fn ensure_dcr_dir(root: &Path) -> io::Result<PathBuf> {
    let dcr = root.join(".dcr");
    std::fs::create_dir_all(dcr.join("ide"))?;
    Ok(dcr)
}

/// Writes `contents` via a `.tmp.<pid>` file then renames into place (best-effort replace).
///
/// # Parameters
/// - `path`: Final destination path (parent dirs are created).
/// - `contents`: Bytes to write.
///
/// # Returns
/// `Ok(())` after a successful rename (with fallbacks if the target already exists).
pub fn atomic_write(path: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tmp".to_string());
    let tmp = path.with_file_name(format!("{file_name}.tmp.{pid}"));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(contents.as_ref())?;
        let _ = f.sync_all();
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Handle cross-device moves or existing file replacement fallbacks.
            if path.exists() {
                let _ = std::fs::remove_file(path);
                match std::fs::rename(&tmp, path) {
                    Ok(()) => Ok(()),
                    Err(e2) => {
                        let _ = std::fs::remove_file(&tmp);
                        Err(e2)
                    }
                }
            } else {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }
}

/// Ensures that `.gitignore` exists in the project directory and includes the `.dcr/` output rule.
///
/// If `.gitignore` does not exist, creates one with standard `/target` and `.dcr/` entries.
pub fn ensure_gitignore_has_dcr(project_dir: &Path) -> io::Result<()> {
    let path = project_dir.join(".gitignore");
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let has = content.lines().any(|l| {
            let t = l.trim();
            t == ".dcr/" || t == ".dcr" || t == "/.dcr/" || t == "/.dcr"
        });
        if !has {
            let mut new_content = content;
            if !new_content.ends_with('\n') && !new_content.is_empty() {
                new_content.push('\n');
            }
            new_content.push_str(".dcr/\n");
            std::fs::write(&path, new_content)?;
        }
    } else {
        std::fs::write(&path, "/target\n.dcr/\n")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn canonicalize_path_absolute() {
        let cwd = std::env::current_dir().unwrap();
        let got = canonicalize_path(&cwd);
        assert!(got.is_absolute());
    }

    #[test]
    fn canonicalize_path_relative_fallback() {
        let p = Path::new("does-not-exist-xyz-12345");
        let got = canonicalize_path(p);
        assert!(got.is_absolute());
    }

    #[test]
    fn atomic_write_roundtrip() {
        let dir = std::env::temp_dir().join(format!("dcr_atomic_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.json");
        atomic_write(&path, b"hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
        atomic_write(&path, b"world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "world");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_dcr_dir_creates_ide() {
        let dir = std::env::temp_dir().join(format!("dcr_dir_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dcr = ensure_dcr_dir(&dir).unwrap();
        assert!(dcr.is_dir());
        assert!(dcr.join("ide").is_dir());
        let _ = fs::remove_dir_all(&dir);
    }
}
