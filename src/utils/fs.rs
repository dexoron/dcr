use std::io;
use std::path::{Path, PathBuf};

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
