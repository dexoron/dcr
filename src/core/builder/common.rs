use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_sources(
    extensions: &[&str],
    exclude_dirs: &[PathBuf],
) -> Result<Vec<String>, String> {
    let mut sources = Vec::new();
    collect_sources_rec("./src", extensions, &mut sources, exclude_dirs)?;
    sources.sort();
    if sources.is_empty() {
        return Err("No source files found in ./src".to_string());
    }
    Ok(sources)
}

fn collect_sources_rec(
    dir: &str,
    extensions: &[&str],
    out: &mut Vec<String>,
    exclude_dirs: &[PathBuf],
) -> Result<(), String> {
    let dir_path = Path::new(dir);
    let full_dir = if dir_path.is_absolute() {
        dir_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| format!("src dir error: {err}"))?
            .join(dir_path)
    };
    if is_excluded(&full_dir, exclude_dirs) {
        return Ok(());
    }
    let entries = fs::read_dir(&full_dir).map_err(|err| format!("src dir error: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("src dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, exclude_dirs) {
                continue;
            }
            collect_sources_rec(&path.to_string_lossy(), extensions, out, exclude_dirs)?;
            continue;
        }
        if !path.is_file() {
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

pub fn is_excluded(path: &Path, exclude_dirs: &[PathBuf]) -> bool {
    exclude_dirs.iter().any(|dir| path.starts_with(dir))
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
    let rel = src_path
        .strip_prefix("./src")
        .or_else(|_| src_path.strip_prefix("src"))
        .unwrap_or(src_path);
    let mut out = obj_dir.join(rel);
    out.set_extension(obj_ext.trim_start_matches('.'));
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

fn parse_d_file(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let text = content.replace("\\\n", " ").replace("\\\r\n", " ");
    let mut target_end = 0;
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == ':' {
            if i == 1 && chars[0].is_ascii_alphabetic() && i + 1 < chars.len() && (chars[i+1] == '\\' || chars[i+1] == '/') {
                continue; // Windows drive letter
            }
            target_end = i + 1;
            break;
        }
    }
    
    let deps_str = if target_end > 0 { &text[target_end..] } else { &text };
    
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("dcr_test_{prefix}_{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn object_path_basic() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/main.c", "o");
        assert_eq!(result, "target/debug/obj/main.o");
    }

    #[test]
    fn object_path_nested() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/core/utils.c", "o");
        assert_eq!(result, "target/debug/obj/core/utils.o");
    }

    #[test]
    fn object_path_no_prefix() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "src/main.c", "o");
        assert_eq!(result, "target/debug/obj/main.o");
    }

    #[test]
    fn object_path_msvc_ext() {
        let obj_dir = Path::new("target/debug/obj");
        let result = object_path(obj_dir, "./src/main.c", "obj");
        assert_eq!(result, "target/debug/obj/main.obj");
    }

    #[test]
    fn needs_rebuild_no_object() {
        let dir = temp_dir("rebuild_no_obj");
        let src = dir.join("test.c");
        fs::write(&src, "int main() {}").unwrap();
        let obj = dir.join("test.o");
        assert!(needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()));
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
        assert!(!needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()));
    }

    #[test]
    fn needs_rebuild_stale() {
        let dir = temp_dir("rebuild_stale");
        let src = dir.join("test.c");
        let obj = dir.join("test.o");
        fs::write(&obj, "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&src, "int main() { return 1; }").unwrap();
        assert!(needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()));
    }

    #[test]
    fn is_excluded_match() {
        let excluded = vec![PathBuf::from("/project/src/vendor")];
        assert!(is_excluded(Path::new("/project/src/vendor"), &excluded));
        assert!(is_excluded(
            Path::new("/project/src/vendor/lib.c"),
            &excluded
        ));
    }

    #[test]
    fn is_excluded_no_match() {
        let excluded = vec![PathBuf::from("/project/src/vendor")];
        assert!(!is_excluded(Path::new("/project/src/main.c"), &excluded));
        assert!(!is_excluded(Path::new("/other/vendor"), &excluded));
    }

    #[test]
    fn collect_sources_c_files() {
        let dir = temp_dir("collect_c");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.c"), "").unwrap();
        fs::write(src.join("utils.c"), "").unwrap();
        fs::write(src.join("README.md"), "").unwrap(); // should be ignored

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = collect_sources(&["c"], &[]);
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

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = collect_sources(&["cpp", "cxx", "cc"], &[]);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn collect_sources_empty_dir() {
        let dir = temp_dir("collect_empty");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = collect_sources(&["c"], &[]);
        std::env::set_current_dir(prev).unwrap();

        assert!(result.is_err(), "empty src should return error");
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
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = collect_sources(&["c"], &exclude);
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

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = collect_sources(&["c"], &[]);
        std::env::set_current_dir(prev).unwrap();

        let sources = result.expect("should find sources");
        assert_eq!(sources.len(), 2);
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
        
        let d_content = format!("{}: {} {}", obj.to_string_lossy(), src.to_string_lossy(), header.to_string_lossy());
        fs::write(&d_file, d_content).unwrap();

        assert!(!needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()), "should be fresh");

        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&header, "#define A 2").unwrap(); // modify header

        assert!(needs_rebuild(&src.to_string_lossy(), &obj.to_string_lossy()), "changed header should trigger rebuild");
    }
}

