use crate::core::config::Config;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone)]
pub struct DepSpec {
    pub name: String,
    pub path_raw: String,
    pub include_raw: Option<Vec<String>>,
    pub lib_raw: Option<Vec<String>>,
    pub libs_raw: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDeps {
    pub include_dirs: Vec<String>,
    pub lib_dirs: Vec<String>,
    pub libs: Vec<String>,
}

pub fn resolve_deps(
    config: &Config,
    profile: &str,
    project_root: &Path,
) -> Result<ResolvedDeps, String> {
    let project_name = config
        .get("package.name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let project_version = config
        .get("package.version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let deps = parse_dependencies(config)?;
    if deps.is_empty() {
        return Ok(ResolvedDeps {
            include_dirs: Vec::new(),
            lib_dirs: Vec::new(),
            libs: Vec::new(),
        });
    }

    let mut include_dirs = Vec::new();
    let mut lib_dirs = Vec::new();
    let mut libs = Vec::new();
    let mut lock_packages = Vec::new();

    for dep in deps {
        let dep_path = resolve_path(project_root, &dep.path_raw)?;
        if !dep_path.is_dir() {
            return Err(format!(
                "Dependency '{}' path is not a directory: {}",
                dep.name,
                dep_path.display()
            ));
        }

        let include = resolve_paths(&dep_path, dep.include_raw.as_deref(), &["include"])?;
        let lib = resolve_paths(&dep_path, dep.lib_raw.as_deref(), &["lib", "lib64"])?;
        let libs_list = dep
            .libs_raw
            .clone()
            .unwrap_or_else(|| vec![dep.name.clone()]);

        if include.is_empty() || lib.is_empty() {
            return Err(format!(
                "Dependency '{}' missing include/lib dirs. Add include/lib in dcr.toml.",
                dep.name
            ));
        }

        include_dirs.extend(include.iter().map(|p| p.to_string_lossy().to_string()));
        lib_dirs.extend(lib.iter().map(|p| p.to_string_lossy().to_string()));
        libs.extend(libs_list.iter().cloned());

        let dep_cache = project_root
            .join("target")
            .join(profile)
            .join("deps")
            .join(&dep.name);
        sync_dep_dir(&dep_path, &dep_cache)
            .map_err(|err| format!("Failed to sync dep {}: {err}", dep.name))?;

        let dep_version = read_dep_version(&dep_path).unwrap_or_else(|| "0.0.0".to_string());
        let dep_checksum = compute_checksum(&dep_path)
            .map_err(|err| format!("Failed to hash dep {}: {err}", dep.name))?;
        lock_packages.push(DepLock {
            name: dep.name.clone(),
            version: dep_version,
            checksum: dep_checksum,
            source: format!("path+{}", dep.path_raw),
        });
    }

    write_lock(
        project_root,
        &project_name,
        &project_version,
        &lock_packages,
    )?;

    Ok(ResolvedDeps {
        include_dirs,
        lib_dirs,
        libs,
    })
}

#[derive(Debug, Clone)]
struct DepLock {
    name: String,
    version: String,
    checksum: String,
    source: String,
}

fn parse_dependencies(config: &Config) -> Result<Vec<DepSpec>, String> {
    let deps_val = match config.get("dependencies") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    let deps_table = deps_val
        .as_table()
        .ok_or_else(|| "dependencies is not a table".to_string())?;
    let mut deps = Vec::new();
    for (name, value) in deps_table {
        match value {
            Value::Table(tbl) => {
                if tbl.get("system").and_then(|v| v.as_bool()).unwrap_or(false) {
                    return Err(format!(
                        "Dependency '{}' uses system=true, which is not supported yet",
                        name
                    ));
                }
                let path = tbl
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!("Dependency '{}' missing path", name))?
                    .to_string();
                let include = parse_string_list(tbl.get("include"), name, "include")?;
                let lib = parse_string_list(tbl.get("lib"), name, "lib")?;
                let libs = parse_string_list(tbl.get("libs"), name, "libs")?;
                deps.push(DepSpec {
                    name: name.to_string(),
                    path_raw: path,
                    include_raw: include,
                    lib_raw: lib,
                    libs_raw: libs,
                });
            }
            _ => {
                return Err(format!(
                    "Dependency '{}' must be a table with path/include/lib",
                    name
                ));
            }
        }
    }
    deps.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(deps)
}

fn parse_string_list(
    value: Option<&Value>,
    dep_name: &str,
    key: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(value) = value else { return Ok(None) };
    let list = value.as_array().ok_or_else(|| {
        format!(
            "Dependency '{}' field '{}' must be an array of strings",
            dep_name, key
        )
    })?;
    let mut out = Vec::new();
    for item in list {
        let s = item
            .as_str()
            .ok_or_else(|| format!("Dependency '{}' field '{}' must be strings", dep_name, key))?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}

fn resolve_path(project_root: &Path, raw: &str) -> Result<PathBuf, String> {
    let p = Path::new(raw);
    let full = if p.is_absolute() {
        p.to_path_buf()
    } else {
        project_root.join(p)
    };
    Ok(full)
}

fn resolve_paths(
    base: &Path,
    raw: Option<&[String]>,
    defaults: &[&str],
) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if let Some(raw) = raw {
        for r in raw {
            let p = Path::new(r);
            let full = if p.is_absolute() {
                p.to_path_buf()
            } else {
                base.join(p)
            };
            if !full.exists() {
                return Err(format!("Path does not exist: {}", full.display()));
            }
            out.push(full);
        }
        return Ok(out);
    }

    for d in defaults {
        let candidate = base.join(d);
        if candidate.exists() {
            out.push(candidate);
        }
    }
    Ok(out)
}

fn sync_dep_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    copy_dir_all(src, dst)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn write_lock(
    project_root: &Path,
    project_name: &str,
    project_version: &str,
    packages: &[DepLock],
) -> Result<(), String> {
    let mut out = String::new();
    if !packages.is_empty() {
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", escape_value(project_name)));
        out.push_str(&format!(
            "version = \"{}\"\n",
            escape_value(project_version)
        ));
        out.push_str(&format!(
            "dependencies = [{}]\n\n",
            quote_list(&packages.iter().map(|p| p.name.clone()).collect::<Vec<_>>())
        ));
    }
    for pkg in packages {
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", escape_value(&pkg.name)));
        out.push_str(&format!("version = \"{}\"\n", escape_value(&pkg.version)));
        out.push_str(&format!("source = \"{}\"\n", escape_value(&pkg.source)));
        out.push_str(&format!("checksum = \"{}\"\n", escape_value(&pkg.checksum)));
        out.push('\n');
    }
    fs::write(project_root.join("dcr.lock"), out)
        .map_err(|err| format!("Failed to write dcr.lock: {err}"))?;
    Ok(())
}

fn quote_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("\"{}\"", escape_value(s)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_value(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn read_dep_version(dep_path: &Path) -> Option<String> {
    let path = dep_path.join("dcr.toml");
    let content = fs::read_to_string(path).ok()?;
    let value: Value = content.parse().ok()?;
    value
        .get("package")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn compute_checksum(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort();
    let mut hasher = Sha256::new();
    for rel in files {
        hasher.update(rel.as_bytes());
        let data =
            fs::read(root.join(&rel)).map_err(|err| format!("failed to read {}: {err}", rel))?;
        hasher.update(&data);
    }
    let hash = hasher.finalize();
    Ok(to_hex(&hash))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|err| format!("read_dir failed: {err}"))? {
        let entry = entry.map_err(|err| format!("read_dir failed: {err}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "target" {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            out.push(rel);
        }
    }
    Ok(())
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}
