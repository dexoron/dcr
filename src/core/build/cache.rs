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

use crate::core::build::builder::BuildContext;
use crate::core::build::common;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Computes a build fingerprint by hashing context fields and file metadata (SHA-256).
///
/// Includes `dcr.toml` (required) and `dcr.lock` when present, plus path/size/mtime
/// for each source, header, and library file.
///
/// # Parameters
/// - `ctx`: Build context (profile, tools, flags, dirs, …).
/// - `sources` / `headers` / `lib_files`: Inputs whose metadata enters the hash.
///
/// # Returns
/// Hex-encoded fingerprint, or an error if `dcr.toml` or a listed file cannot be read.
pub(crate) fn compute_build_fingerprint(
    ctx: &BuildContext,
    sources: &[String],
    headers: &[PathBuf],
    lib_files: &[PathBuf],
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(ctx.profile.as_bytes());
    hasher.update(ctx.project_name.as_bytes());
    hasher.update(ctx.compiler.as_bytes());
    hasher.update(ctx.language.as_bytes());
    hasher.update(ctx.standard.as_bytes());
    hasher.update(ctx.kind.as_bytes());
    if let Some(v) = ctx.target_dir {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.platform {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.linker {
        hasher.update(v.as_bytes());
    }
    if let Some(v) = ctx.archiver {
        hasher.update(v.as_bytes());
    }
    for value in ctx.include_dirs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.lib_dirs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.libs {
        hasher.update(value.as_bytes());
    }
    for value in ctx.cflags {
        hasher.update(value.as_bytes());
    }
    for value in ctx.ldflags {
        hasher.update(value.as_bytes());
    }
    // Include project metadata from dcr.toml in the fingerprint
    let toml =
        fs::read_to_string("dcr.toml").map_err(|err| format!("Failed to read dcr.toml: {err}"))?;
    hasher.update(toml.as_bytes());
    if let Ok(lock) = fs::read_to_string("dcr.lock") {
        hasher.update(lock.as_bytes());
    }
    // Include source, header, and library files in the fingerprint
    for source in sources {
        let path = Path::new(source);
        update_hasher_with_file(&mut hasher, path)?;
    }
    for header in headers {
        update_hasher_with_file(&mut hasher, header)?;
    }
    for lib in lib_files {
        update_hasher_with_file(&mut hasher, lib)?;
    }
    Ok(to_hex(&hasher.finalize()))
}

/// Whether the build can be skipped: main artifact exists and cache hash matches.
///
/// # Parameters
/// - `ctx`: Used to locate the output artifact and `.dcr-build.hash`.
/// - `fingerprint`: Fresh hash from [`compute_build_fingerprint`].
///
/// # Returns
/// `true` only when both the artifact and matching cache file are present.
pub(crate) fn should_skip_build(ctx: &BuildContext, fingerprint: &str) -> bool {
    let output = build_output_path(ctx);
    if !Path::new(&output).is_file() {
        return false;
    }
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    let cached = fs::read_to_string(cache_path).unwrap_or_default();
    cached.trim() == fingerprint
}

/// Persists the build fingerprint to the cache file on disk.
pub(crate) fn write_build_fingerprint(ctx: &BuildContext, fingerprint: &str) -> Result<(), String> {
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Failed to create cache dir: {err}"))?;
    }
    fs::write(cache_path, format!("{fingerprint}\n"))
        .map_err(|err| format!("Failed to write cache: {err}"))
}

/// Returns the path to the build cache hash file based on profile and target directory.
fn build_cache_path(profile: &str, target_dir: Option<&str>) -> PathBuf {
    match target_dir {
        Some(dir) => Path::new(dir).join(".dcr-build.hash"),
        None => Path::new("./target").join(profile).join(".dcr-build.hash"),
    }
}

/// Resolves the output path for the build artifact using the provided context.
pub fn build_output_path(ctx: &BuildContext) -> String {
    crate::core::build::builder::artifact::resolve_artifact_path(
        ctx.kind,
        ctx.profile,
        ctx.project_name,
        ctx.target_dir,
        ctx.output_filename,
        ctx.output_extension,
    )
    .unwrap_or_else(|| {
        crate::platform::bin_path(
            ctx.profile,
            ctx.output_filename.unwrap_or(ctx.project_name),
            ctx.target_dir,
        )
    })
}

/// Collects header files from source roots and include directories, filtering out excluded paths.
pub(crate) fn collect_header_files(
    ctx: &BuildContext,
    project_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut roots = Vec::new();
    if ctx.source_roots.is_empty() {
        roots.push(project_root.join("src"));
    } else {
        roots.extend(ctx.source_roots.iter().cloned());
    }
    let abs_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    for dir in ctx.include_dirs {
        let p = Path::new(dir).to_path_buf();
        if let Ok(abs_p) = p.canonicalize() {
            if abs_p.starts_with(&abs_root) {
                roots.push(abs_p);
            }
        } else if p.starts_with(&abs_root) {
            roots.push(p);
        }
    }
    for root in roots {
        if !root.exists() {
            continue;
        }
        if root.is_file() {
            if is_header_file(&root) {
                out.push(root);
            }
        } else {
            collect_header_files_rec(&root, &mut out, ctx.exclude_dirs, ctx.include_paths)?;
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Recursively scans a directory to collect header files, respecting exclusion rules.
fn collect_header_files_rec(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    exclude_dirs: &[PathBuf],
    include_paths: &[String],
) -> Result<(), String> {
    if common::is_excluded(dir, exclude_dirs, include_paths) && include_paths.is_empty() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| format!("read_dir error: {err}"))? {
        let entry = entry.map_err(|err| format!("read_dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            if common::is_excluded(&path, exclude_dirs, include_paths) && include_paths.is_empty() {
                continue;
            }
            collect_header_files_rec(&path, out, exclude_dirs, include_paths)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if common::is_excluded(&path, exclude_dirs, include_paths) {
            continue;
        }
        if is_header_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// Checks if the given path has a header file extension.
fn is_header_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
    matches!(ext, "h" | "hpp" | "hh" | "hxx" | "inc")
}

/// Collects library files from lib directories using platform-specific candidates.
pub(crate) fn collect_lib_files(ctx: &BuildContext) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ctx.lib_dirs {
        let dir_path = Path::new(dir);
        for lib in ctx.libs {
            for candidate in lib_candidates(lib) {
                let path = dir_path.join(candidate);
                if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out
}

/// Candidate library filenames for the **host** OS (`cfg!(target_os = …)`).
fn lib_candidates(name: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        return vec![format!("{name}.lib")];
    }
    if cfg!(target_os = "macos") {
        return vec![
            format!("lib{name}.a"),
            format!("lib{name}.dylib"),
            format!("lib{name}.so"),
        ];
    }
    vec![
        format!("lib{name}.a"),
        format!("lib{name}.so"),
        format!("lib{name}.so.0"),
    ]
}

/// Updates the hasher with the file's path, size, and last modification time.
fn update_hasher_with_file(hasher: &mut Sha256, path: &Path) -> Result<(), String> {
    hasher.update(path.to_string_lossy().as_bytes());
    let meta = fs::metadata(path).map_err(|err| format!("source read error: {err}"))?;
    hasher.update(meta.len().to_le_bytes());
    if let Ok(modified) = meta.modified()
        && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
    {
        hasher.update(duration.as_nanos().to_le_bytes());
    }
    Ok(())
}

/// Converts a byte slice containing the SHA256 hash to a hexadecimal string.
fn to_hex(bytes: &[u8]) -> String {
    crate::utils::fs::to_hex(bytes)
}
