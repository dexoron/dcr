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
    let toml =
        fs::read_to_string("dcr.toml").map_err(|err| format!("Failed to read dcr.toml: {err}"))?;
    hasher.update(toml.as_bytes());
    if let Ok(lock) = fs::read_to_string("dcr.lock") {
        hasher.update(lock.as_bytes());
    }
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

pub(crate) fn should_skip_build(ctx: &BuildContext, fingerprint: &str) -> bool {
    let output = build_output_path(ctx);
    if !Path::new(&output).is_file() {
        return false;
    }
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    let cached = fs::read_to_string(cache_path).unwrap_or_default();
    cached.trim() == fingerprint
}

pub(crate) fn write_build_fingerprint(ctx: &BuildContext, fingerprint: &str) -> Result<(), String> {
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Failed to create cache dir: {err}"))?;
    }
    fs::write(cache_path, format!("{fingerprint}\n"))
        .map_err(|err| format!("Failed to write cache: {err}"))
}

fn build_cache_path(profile: &str, target_dir: Option<&str>) -> PathBuf {
    match target_dir {
        Some(dir) => Path::new(dir).join(".dcr-build.hash"),
        None => Path::new("./target").join(profile).join(".dcr-build.hash"),
    }
}

fn build_output_path(ctx: &BuildContext) -> String {
    if crate::utils::build::is_flat_bin(ctx.kind) {
        return crate::core::build::builder::artifact::flat_output_path(ctx);
    }

    let name = ctx.output_filename.unwrap_or(ctx.project_name);
    let ext = ctx.output_extension.unwrap_or("");

    let final_name = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    };

    if ctx.kind == "staticlib" {
        return crate::platform::lib_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "sharedlib" {
        return crate::platform::shared_lib_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "efi" {
        return crate::platform::efi_path(ctx.profile, &final_name, ctx.target_dir);
    }
    if ctx.kind == "elf" {
        return crate::platform::elf_path(ctx.profile, &final_name, ctx.target_dir);
    }
    crate::platform::bin_path(ctx.profile, &final_name, ctx.target_dir)
}

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

fn is_header_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
    matches!(ext, "h" | "hpp" | "hh" | "hxx" | "inc")
}

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

fn to_hex(bytes: &[u8]) -> String {
    crate::utils::fs::to_hex(bytes)
}
