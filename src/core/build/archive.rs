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

use crate::core::build_config::ArchiveConfig;
use fatfs::{FileSystem, FormatVolumeOptions, format_volume};
use glob::glob;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

struct OffsetFile<T> {
    inner: T,
    offset: u64,
}

impl<T: Read> Read for OffsetFile<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for OffsetFile<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for OffsetFile<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(n) => {
                let actual = self.inner.seek(SeekFrom::Start(n + self.offset))?;
                if actual < self.offset {
                    Ok(0)
                } else {
                    Ok(actual - self.offset)
                }
            }
            SeekFrom::Current(n) => {
                let actual = self.inner.seek(SeekFrom::Current(n))?;
                if actual < self.offset {
                    Ok(0)
                } else {
                    Ok(actual - self.offset)
                }
            }
            SeekFrom::End(n) => {
                let actual = self.inner.seek(SeekFrom::End(n))?;
                if actual < self.offset {
                    Ok(0)
                } else {
                    Ok(actual - self.offset)
                }
            }
        }
    }
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    let mut num_str = String::new();
    let mut unit = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
        } else {
            unit.push(c);
        }
    }
    let val: f64 = num_str
        .parse()
        .map_err(|_| format!("invalid size number: {num_str}"))?;
    let unit = unit.trim().to_lowercase();
    let multiplier = match unit.as_str() {
        "k" | "kb" => 1024.0,
        "m" | "mb" => 1024.0 * 1024.0,
        "g" | "gb" => 1024.0 * 1024.0 * 1024.0,
        "" => 1.0,
        _ => return Err(format!("unknown unit: {unit}")),
    };
    Ok((val * multiplier) as u64)
}

pub fn pack_archive(
    project_root: &Path,
    config: &ArchiveConfig,
    profile: &str,
) -> Result<(), String> {
    let output_substituted = config.output.replace("{profile}", profile);
    let output_path = project_root.join(&output_substituted);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create output parent dir: {e}"))?;
    }

    let size = if let Some(sz_str) = &config.size {
        parse_size(sz_str)?
    } else {
        1474560
    };

    let offset = if let Some(off_str) = &config.offset {
        parse_size(off_str)?
    } else {
        0
    };

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&output_path)
        .map_err(|e| format!("failed to open output file: {e}"))?;

    let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if current_len < size {
        file.set_len(size)
            .map_err(|e| format!("failed to set image file size: {e}"))?;
    }

    if offset == 0
        && let Some(boot_rel) = &config.bootsector
    {
        let boot_rel_substituted = boot_rel.replace("{profile}", profile);
        let boot_path = project_root.join(&boot_rel_substituted);
        if boot_path.is_file() {
            let mut boot_file =
                File::open(&boot_path).map_err(|e| format!("failed to open bootsector: {e}"))?;
            let mut boot_bytes = vec![0; 512];
            let _ = boot_file
                .read(&mut boot_bytes)
                .map_err(|e| format!("failed to read bootsector: {e}"))?;
            file.seek(SeekFrom::Start(0))
                .map_err(|e| format!("failed to seek to start: {e}"))?;
            file.write_all(&boot_bytes)
                .map_err(|e| format!("failed to write bootsector: {e}"))?;
        }
    }

    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("failed to seek to start: {e}"))?;

    let mut wrapper = OffsetFile {
        inner: file,
        offset,
    };

    let label = config.label.as_deref().unwrap_or("VOLUME");
    let mut label_bytes = [b' '; 11];
    let bytes_to_copy = label.len().min(11);
    label_bytes[..bytes_to_copy].copy_from_slice(&label.as_bytes()[..bytes_to_copy]);

    let fat_type = match config.format.to_lowercase().as_str() {
        "fat12" => Some(fatfs::FatType::Fat12),
        "fat16" => Some(fatfs::FatType::Fat16),
        "fat32" => Some(fatfs::FatType::Fat32),
        _ => None,
    };

    let mut options = FormatVolumeOptions::new().volume_label(label_bytes);
    if let Some(t) = fat_type {
        options = options.fat_type(t);
    }

    format_volume(&mut wrapper, options)
        .map_err(|e| format!("failed to format FAT volume: {e}"))?;

    wrapper
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("failed to seek to start after format: {e}"))?;

    let fs = FileSystem::new(wrapper, fatfs::FsOptions::new())
        .map_err(|e| format!("failed to mount FAT fs: {e}"))?;
    let root_dir = fs.root_dir();

    for layout in &config.layout {
        let from_substituted = layout.from.replace("{profile}", profile);
        let from_pattern = project_root
            .join(&from_substituted)
            .to_string_lossy()
            .to_string();

        let paths = if from_pattern.contains('*') || from_pattern.contains('?') {
            let mut matched = Vec::new();
            if let Ok(entries) = glob(&from_pattern) {
                for path in entries.flatten() {
                    matched.push(path);
                }
            }
            matched
        } else {
            let p = project_root.join(&from_substituted);
            if p.exists() { vec![p] } else { Vec::new() }
        };

        for path in paths {
            if !path.is_file() {
                continue;
            }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename.is_empty() {
                continue;
            }

            let mut target_filename = filename.to_string();
            let mut target_dir_str = layout.to.trim_matches('/');

            let is_glob = from_pattern.contains('*') || from_pattern.contains('?');
            if !is_glob
                && !layout.to.ends_with('/')
                && let Some(last_part) = Path::new(&layout.to).file_name().and_then(|n| n.to_str())
                && last_part.contains('.')
            {
                target_filename = last_part.to_string();
                let parent_path = Path::new(&layout.to).parent().unwrap_or(Path::new(""));
                target_dir_str = parent_path.to_str().unwrap_or("").trim_matches('/');
            }

            let mut current_dir = root_dir.clone();
            if !target_dir_str.is_empty() {
                let parts = target_dir_str.split('/');
                for part in parts {
                    if part.is_empty() {
                        continue;
                    }
                    current_dir = match current_dir.open_dir(part) {
                        Ok(dir) => dir,
                        Err(_) => current_dir
                            .create_dir(part)
                            .map_err(|e| format!("failed to create dir {part}: {e}"))?,
                    };
                }
            }

            let mut out_file = current_dir
                .create_file(&target_filename)
                .map_err(|e| format!("failed to create file {target_filename}: {e}"))?;
            let mut in_file =
                File::open(&path).map_err(|e| format!("failed to open input file: {e}"))?;
            let mut buffer = Vec::new();
            in_file
                .read_to_end(&mut buffer)
                .map_err(|e| format!("failed to read input file: {e}"))?;
            out_file
                .write_all(&buffer)
                .map_err(|e| format!("failed to write to FAT file {target_filename}: {e}"))?;
        }
    }

    Ok(())
}
