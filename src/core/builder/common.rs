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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

static OUTPUT_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

pub fn get_output_lock() -> &'static Mutex<()> {
    OUTPUT_MUTEX.get_or_init(|| Mutex::new(()))
}

pub fn collect_sources(
    roots: &[PathBuf],
    extensions: &[&str],
    exclude_dirs: &[PathBuf],
    include_paths: &[String],
) -> Result<Vec<String>, String> {
    let mut sources = Vec::new();
    for root in roots {
        collect_sources_rec(root, extensions, &mut sources, exclude_dirs, include_paths)?;
    }
    sources.sort();
    Ok(sources)
}

fn collect_sources_rec(
    dir: &Path,
    extensions: &[&str],
    out: &mut Vec<String>,
    exclude_dirs: &[PathBuf],
    include_paths: &[String],
) -> Result<(), String> {
    let full_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| format!("src dir error: {err}"))?
            .join(dir)
    };
    if is_excluded(&full_dir, exclude_dirs, include_paths) && include_paths.is_empty() {
        return Ok(());
    }
    let entries = fs::read_dir(&full_dir).map_err(|err| format!("src dir error: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("src dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, exclude_dirs, include_paths) && include_paths.is_empty() {
                continue;
            }
            collect_sources_rec(&path, extensions, out, exclude_dirs, include_paths)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if is_excluded(&path, exclude_dirs, include_paths) {
            continue;
        }
        let ext_raw = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        let ext_lower = ext_raw.to_lowercase();
        let matched = extensions
            .iter()
            .any(|allowed| *allowed == ext_raw || *allowed == ext_lower);
        if matched {
            out.push(normalize_source_path(&path));
        }
    }
    Ok(())
}

pub fn is_excluded(path: &Path, exclude_dirs: &[PathBuf], include_paths: &[String]) -> bool {
    if matches_patterns(path, include_paths, true) {
        return false;
    }
    if matches_patterns(path, include_paths, false) {
        return true;
    }
    exclude_dirs.iter().any(|dir| path.starts_with(dir))
}

fn matches_patterns(path: &Path, patterns: &[String], positive: bool) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let cwd = std::env::current_dir().ok();
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = cwd.as_ref() {
        base.join(path)
    } else {
        path.to_path_buf()
    };
    let abs = abs_path.to_string_lossy().replace('\\', "/");
    let rel = cwd
        .as_ref()
        .and_then(|base| abs_path.strip_prefix(base).ok())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let rel_dot = if rel.is_empty() {
        String::new()
    } else {
        format!("./{rel}")
    };
    for raw in patterns {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let is_neg = trimmed.starts_with('!');
        if is_neg == positive {
            continue;
        }
        let normalized = trimmed.trim_start_matches('!').replace('\\', "/");
        if normalized.is_empty() {
            continue;
        }
        if has_glob_magic(&normalized) {
            let pat = match glob::Pattern::new(&normalized) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if pat.matches(&abs)
                || (!rel.is_empty() && (pat.matches(&rel) || pat.matches(&rel_dot)))
            {
                return true;
            }
            continue;
        }
        let norm = normalized.trim_end_matches('/');
        let candidate = Path::new(norm);
        let full = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else if let Some(base) = cwd.as_ref() {
            base.join(candidate)
        } else {
            candidate.to_path_buf()
        };
        if abs_path.starts_with(&full) {
            return true;
        }
    }
    false
}

pub fn has_glob_magic(value: &str) -> bool {
    value.chars().any(|c| matches!(c, '*' | '?' | '['))
}

pub fn parallel_build<F>(total_tasks: usize, task_fn: F) -> Result<(), String>
where
    F: Fn(usize) -> Result<(), String> + Sync + Send,
{
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let counter = std::sync::atomic::AtomicUsize::new(0);
    let err_msg = std::sync::Mutex::new(None);

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                loop {
                    if err_msg.lock().unwrap().is_some() {
                        break;
                    }

                    let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if i >= total_tasks {
                        break;
                    }

                    if let Err(e) = task_fn(i) {
                        let mut err = err_msg.lock().unwrap();
                        if err.is_none() {
                            *err = Some(e);
                        }
                        break;
                    }
                }
            });
        }
    });

    if let Some(err) = err_msg.into_inner().unwrap() {
        return Err(err);
    }

    Ok(())
}

pub fn run_command_sync_output(cmd: &mut Command) -> Result<(), String> {
    let output = cmd.output().map_err(|err| format!("Build failed: {err}"))?;
    let _guard = OUTPUT_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        Ok(())
    } else {
        Err("Build failed".to_string())
    }
}

pub fn normalize_source_path(path: &Path) -> String {
    if !path.is_absolute() {
        return path.to_string_lossy().to_string();
    }
    if let Ok(base) = std::env::current_dir()
        && let Ok(rel) = path.strip_prefix(&base)
    {
        return format!("./{}", rel.to_string_lossy());
    }
    path.to_string_lossy().to_string()
}

pub fn object_path(obj_dir: &Path, source: &str, obj_ext: &str) -> String {
    let src_path = Path::new(source);
    let stripped = src_path.strip_prefix("./").unwrap_or(src_path);
    let rel = stripped
        .components()
        .skip(1)
        .collect::<std::path::PathBuf>();
    let rel = if rel.components().next().is_some() {
        rel
    } else {
        stripped.to_path_buf()
    };
    let mut out = obj_dir.join(rel);
    let ext = out
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    let obj_ext = obj_ext.trim_start_matches('.');
    if !ext.is_empty() {
        let stem = out
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        out.set_file_name(format!("{stem}.{ext}.{obj_ext}"));
    } else {
        out.set_extension(obj_ext);
    }
    out.to_string_lossy().to_string()
}

pub fn needs_rebuild(source: &str, object: &str) -> bool {
    let src_time = fs::metadata(source).and_then(|m| m.modified());
    let obj_time = fs::metadata(object).and_then(|m| m.modified());
    let o_time = match obj_time {
        Ok(t) => t,
        Err(_) => return true,
    };
    match src_time {
        Ok(s) if s > o_time => return true,
        Err(_) => return true,
        _ => {}
    }

    let d_file = PathBuf::from(object).with_extension("d");
    if let Ok(content) = fs::read_to_string(&d_file) {
        let deps = parse_d_file(&content);
        for dep in deps {
            let dep_path = Path::new(&dep);
            if dep_path == Path::new(object) || dep_path == Path::new(source) {
                continue;
            }
            if let Ok(dep_meta) = fs::metadata(dep_path) {
                if let Ok(dep_time) = dep_meta.modified()
                    && dep_time > o_time
                {
                    return true;
                }
            } else {
                return true; // Missing dependency triggers rebuild
            }
        }
    }
    false
}

pub fn needs_link(objects: &[String], output_path: &str) -> bool {
    let out_meta = match fs::metadata(output_path) {
        Ok(m) => m,
        Err(_) => return true,
    };
    let out_time = match out_meta.modified() {
        Ok(t) => t,
        Err(_) => return true,
    };
    for obj in objects {
        let obj_meta = match fs::metadata(obj) {
            Ok(m) => m,
            Err(_) => return true,
        };
        let obj_time = match obj_meta.modified() {
            Ok(t) => t,
            Err(_) => return true,
        };
        if obj_time > out_time {
            return true;
        }
    }
    false
}

fn parse_d_file(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let text = content.replace("\\\n", " ").replace("\\\r\n", " ");
    let mut target_end = 0;
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == ':' {
            if i == 1
                && chars[0].is_ascii_alphabetic()
                && i + 1 < chars.len()
                && (chars[i + 1] == '\\' || chars[i + 1] == '/')
            {
                continue; // Windows drive letter
            }
            target_end = i + 1;
            break;
        }
    }

    let deps_str = if target_end > 0 {
        &text[target_end..]
    } else {
        &text
    };

    let mut current_path = String::new();
    let mut in_escape = false;

    for c in deps_str.chars() {
        if in_escape {
            if c != '\n' && c != '\r' {
                current_path.push(c);
            }
            in_escape = false;
        } else if c == '\\' {
            in_escape = true;
        } else if c.is_whitespace() {
            if !current_path.is_empty() {
                deps.push(current_path.clone());
                current_path.clear();
            }
        } else {
            current_path.push(c);
        }
    }
    if !current_path.is_empty() {
        deps.push(current_path);
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(prefix: &str) -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("dcr_test_{prefix}_{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn default_roots() -> Vec<PathBuf> {
        vec![PathBuf::from("src")]
    }

    #[test]
    fn object_path_basic() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/main.c", "o");
        assert_eq!(result, "target/debug/obj/main.c.o");
    }

    #[test]
    fn object_path_nested() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/core/utils.c", "o");
        assert_eq!(result, "target/debug/obj/core/utils.c.o");
    }

    #[test]
    fn object_path_no_prefix() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "src/main.c", "o");
        assert_eq!(result, "target/debug/obj/main.c.o");
    }

    #[test]
    fn object_path_msvc_ext() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/main.c", "obj");
        assert_eq!(result, "target/debug/obj/main.c.obj");
    }

    #[test]
    fn needs_link_no_output() {
        let dir = temp_dir("link_no_out");
        let obj1 = dir.join("test1.o");
        let obj2 = dir.join("test2.o");
        let out = dir.join("app");
        fs::write(&obj1, "").unwrap();
        fs::write(&obj2, "").unwrap();
        let objs = vec![
            obj1.to_string_lossy().to_string(),
            obj2.to_string_lossy().to_string(),
        ];
        assert!(needs_link(&objs, &out.to_string_lossy()));
    }

    #[test]
    fn needs_link_fresh() {
        let dir = temp_dir("link_fresh");
        let obj1 = dir.join("test1.o");
        let obj2 = dir.join("test2.o");
        let out = dir.join("app");
        fs::write(&obj1, "").unwrap();
        fs::write(&obj2, "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&out, "").unwrap();
        let objs = vec![
            obj1.to_string_lossy().to_string(),
            obj2.to_string_lossy().to_string(),
        ];
        assert!(!needs_link(&objs, &out.to_string_lossy()));
    }

    #[test]
    fn needs_link_stale() {
        let dir = temp_dir("link_stale");
        let obj1 = dir.join("test1.o");
        let obj2 = dir.join("test2.o");
        let out = dir.join("app");
        fs::write(&obj1, "").unwrap();
        fs::write(&out, "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&obj2, "").unwrap();
        let objs = vec![
            obj1.to_string_lossy().to_string(),
            obj2.to_string_lossy().to_string(),
        ];
        assert!(needs_link(&objs, &out.to_string_lossy()));
    }

    #[test]
    fn needs_rebuild_no_object() {
        let dir = temp_dir("rebuild_no_obj");
        let src = dir.join("test.c");
        fs::write(&src, "int main() {}").unwrap();
        let obj = dir.join("test.o");
        assert!(needs_rebuild(
            &src.to_string_lossy(),
            &obj.to_string_lossy()
        ));
    }

    #[test]
    fn needs_rebuild_fresh() {
        let dir = temp_dir("rebuild_fresh");
        let src = dir.join("test.c");
        let obj = dir.join("test.o");
        fs::write(&src, "int main() {}").unwrap();
        // Sleep briefly to ensure mtime difference
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&obj, "").unwrap();
        assert!(!needs_rebuild(
            &src.to_string_lossy(),
            &obj.to_string_lossy()
        ));
    }

    #[test]
    fn needs_rebuild_stale() {
        let dir = temp_dir("rebuild_stale");
        let src = dir.join("test.c");
        let obj = dir.join("test.o");
        fs::write(&obj, "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&src, "int main() { return 1; }").unwrap();
        assert!(needs_rebuild(
            &src.to_string_lossy(),
            &obj.to_string_lossy()
        ));
    }

    #[test]
    fn is_excluded_match() {
        let include: Vec<String> = Vec::new();
        let excluded = vec![PathBuf::from("/project/src/vendor")];
        assert!(is_excluded(
            Path::new("/project/src/vendor"),
            &excluded,
            &include
        ));
        assert!(is_excluded(
            Path::new("/project/src/vendor/lib.c"),
            &excluded,
            &include
        ));
    }

    #[test]
    fn is_excluded_no_match() {
        let include: Vec<String> = Vec::new();
        let excluded = vec![PathBuf::from("/project/src/vendor")];
        assert!(!is_excluded(
            Path::new("/project/src/main.c"),
            &excluded,
            &include
        ));
        assert!(!is_excluded(
            Path::new("/other/vendor"),
            &excluded,
            &include
        ));
    }

    #[test]
    fn collect_sources_c_files() {
        let dir = temp_dir("collect_c");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.c"), "").unwrap();
        fs::write(src.join("utils.c"), "").unwrap();
        fs::write(src.join("README.md"), "").unwrap(); // should be ignored

        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let include: Vec<String> = Vec::new();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &[], &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 2);
        assert!(sources.iter().any(|s| s.ends_with("main.c")));
        assert!(sources.iter().any(|s| s.ends_with("utils.c")));
    }

    #[test]
    fn collect_sources_cpp_files() {
        let dir = temp_dir("collect_cpp");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.cpp"), "").unwrap();
        fs::write(src.join("helper.cxx"), "").unwrap();
        fs::write(src.join("other.cc"), "").unwrap();
        fs::write(src.join("skip.c"), "").unwrap(); // should be ignored

        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let include: Vec<String> = Vec::new();
        let roots = default_roots();
        let result = collect_sources(&roots, &["cpp", "cxx", "cc"], &[], &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn collect_sources_empty_dir() {
        let dir = temp_dir("collect_empty");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();

        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let include: Vec<String> = Vec::new();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &[], &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("empty src should return Ok");
        assert!(sources.is_empty(), "empty src should return empty vec");
    }

    #[test]
    fn collect_sources_respects_excludes() {
        let dir = temp_dir("collect_exclude");
        let src = dir.join("src");
        let vendor = src.join("vendor");
        fs::create_dir_all(&vendor).unwrap();
        fs::write(src.join("main.c"), "").unwrap();
        fs::write(vendor.join("lib.c"), "").unwrap(); // should be excluded

        let exclude = vec![vendor.clone()];
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let include: Vec<String> = Vec::new();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &exclude, &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 1);
        assert!(sources[0].ends_with("main.c"));
    }

    #[test]
    fn collect_sources_nested() {
        let dir = temp_dir("collect_nested");
        let src = dir.join("src");
        let sub = src.join("core").join("deep");
        fs::create_dir_all(&sub).unwrap();
        fs::write(src.join("main.c"), "").unwrap();
        fs::write(sub.join("nested.c"), "").unwrap();

        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let include: Vec<String> = Vec::new();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &[], &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn include_overrides_exclude() {
        let dir = temp_dir("include_override");
        let src = dir.join("src");
        let boot = src.join("boot");
        let arch = boot.join("arch");
        fs::create_dir_all(&arch).unwrap();
        fs::write(arch.join("start.s"), "").unwrap();
        fs::write(boot.join("skip.c"), "").unwrap();

        let exclude = vec![boot.clone()];
        let include = vec!["src/boot/arch/**".to_string()];
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let roots = default_roots();
        let result = collect_sources(&roots, &["s", "c"], &exclude, &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 1);
        assert!(sources[0].ends_with("start.s"));
    }

    #[test]
    fn exclude_glob_with_include_override() {
        let dir = temp_dir("exclude_glob");
        let src = dir.join("src");
        let legacy = src.join("legacy");
        let allow = legacy.join("allow");
        fs::create_dir_all(&allow).unwrap();
        fs::write(legacy.join("skip.c"), "").unwrap();
        fs::write(allow.join("keep.c"), "").unwrap();

        let exclude = vec![dir.join("src"), dir.join(".")];
        let include = vec![
            "!src/legacy/**".to_string(),
            "src/legacy/allow/**".to_string(),
        ];
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &exclude, &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 1);
        assert!(sources[0].ends_with("keep.c"));
    }

    #[test]
    fn exclude_glob_without_include_keeps_other_files() {
        let dir = temp_dir("exclude_glob_only");
        let src = dir.join("src");
        let gen_dir = src.join("gen");
        fs::create_dir_all(&gen_dir).unwrap();
        fs::write(src.join("main.c"), "").unwrap();
        fs::write(gen_dir.join("skip.c"), "").unwrap();

        let exclude: Vec<PathBuf> = Vec::new();
        let include = vec!["!src/gen/**".to_string()];
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let roots = default_roots();
        let result = collect_sources(&roots, &["c"], &exclude, &include);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 1);
        assert!(sources[0].ends_with("main.c"));
    }

    #[test]
    fn normalize_relative_path() {
        let result = normalize_source_path(Path::new("./src/main.c"));
        assert_eq!(result, "./src/main.c");
    }

    #[test]
    fn parse_d_file_gcc_format() {
        let content = "target/obj/main.o: src/main.c src/utils.h \\\n src/core/types.h";
        let deps = parse_d_file(content);
        assert_eq!(deps, vec!["src/main.c", "src/utils.h", "src/core/types.h"]);
    }

    #[test]
    fn parse_d_file_msvc_format() {
        let content = "target/obj/main.obj: \\\n  C:/sdk/include/windows.h \\\n  src/main.c";
        let deps = parse_d_file(content);
        assert_eq!(deps, vec!["C:/sdk/include/windows.h", "src/main.c"]);
    }

    #[test]
    fn needs_rebuild_header_modified() {
        let dir = temp_dir("rebuild_header");
        let src = dir.join("test.c");
        let header = dir.join("test.h");
        let obj = dir.join("test.o");
        let d_file = dir.join("test.d");

        fs::write(&src, "int main() {}").unwrap();
        fs::write(&header, "#define A 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&obj, "").unwrap();

        let d_content = format!(
            "{}: {} {}",
            obj.to_string_lossy(),
            src.to_string_lossy(),
            header.to_string_lossy()
        );
        fs::write(&d_file, d_content).unwrap();

        assert!(
            !needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()),
            "should be fresh"
        );

        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&header, "#define A 2").unwrap(); // modify header

        assert!(
            needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()),
            "changed header should trigger rebuild"
        );
    }
}
