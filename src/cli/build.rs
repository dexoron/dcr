use crate::config::{PROFILE, flags};
use crate::core::builder::{BuildContext, build as build_project, collect_sources};
use crate::core::config::Config;
use crate::core::deps::resolve_deps;
use crate::core::workspace::parse_workspace;
use crate::utils::fs::{check_dir, find_project_root};
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn build(args: &[String]) -> i32 {
    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };
    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => dir,
        Ok(None) => {
            error("dcr.toml file not found");
            return 1;
        }
        Err(_) => {
            error("Failed to find project root");
            return 1;
        }
    };
    let active_profile = match parse_profile(args) {
        Ok(profile) => profile,
        Err(code) => return code,
    };
    match with_dir(&root, || build_from_root(&root, &active_profile)) {
        Ok(()) => 0,
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

fn parse_profile(args: &[String]) -> Result<String, i32> {
    if let Some(first_arg) = args.first() {
        if first_arg.starts_with("--") {
            let candidate = first_arg.trim_start_matches("--");
            if flags(candidate).is_some() {
                return Ok(candidate.to_string());
            }
            warn("Unknown build flag");
            return Err(1);
        }
        warn("Unknown argument");
        return Err(1);
    }
    Ok(PROFILE.to_string())
}

fn get_config_str(config: &Config, key: &str) -> String {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_config_opt(config: &Config, key: &str) -> Option<String> {
    let value = config.get(key)?.as_str()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn get_config_list(config: &Config, key: &str) -> Result<Vec<String>, String> {
    let value = match config.get(key) {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{key} must be an array of strings"))?;
    let mut out = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| format!("{key} must be an array of strings"))?;
        out.push(s.to_string());
    }
    Ok(out)
}

fn ensure_target_dirs(items: &[String], profile: &str, target_dir: Option<&str>) {
    if let Some(dir) = target_dir {
        let _ = fs::create_dir_all(dir);
        return;
    }
    if !items.contains(&"target".to_string()) {
        let _ = fs::create_dir("./target");
    }
    let target_items = check_dir(Some("target")).unwrap_or_default();
    if !target_items.contains(&profile.to_string()) {
        let _ = fs::create_dir(format!("./target/{profile}"));
    }
}

fn run_build(ctx: &BuildContext) -> Result<f64, String> {
    let start_time = Instant::now();
    match build_project(ctx) {
        Ok(times) => {
            let times = if times == 0.0 {
                ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0
            } else {
                times
            };
            Ok(times)
        }
        Err(_) => Err("Build failed".to_string()),
    }
}

fn build_from_root(root: &Path, profile: &str) -> Result<(), String> {
    let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;
    let project_name = get_config_str(&config, "package.name");
    let start_time = Instant::now();
    println!(
        "    Building project `{}`\n    Profile: {}",
        colored(&project_name, BOLD_GREEN),
        colored(profile, BOLD_GREEN)
    );
    if let Some(workspace) = parse_workspace(&config, root)? {
        build_workspace(&workspace, profile)?;
        let excludes: Vec<std::path::PathBuf> =
            workspace.members.iter().map(|m| m.path.clone()).collect();
        build_project_at(root, profile, &excludes)?;
        let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
        println!(
            "    {} Build completed successfully in {} seconds",
            colored("✔", BOLD_GREEN),
            colored(&elapsed.to_string(), BOLD_GREEN)
        );
        return Ok(());
    }
    build_project_at(root, profile, &[])?;
    let elapsed = ((start_time.elapsed().as_secs_f64() * 100.0).trunc()) / 100.0;
    println!(
        "    {} Build completed successfully in {} seconds",
        colored("✔", BOLD_GREEN),
        colored(&elapsed.to_string(), BOLD_GREEN)
    );
    Ok(())
}

fn build_workspace(
    workspace: &crate::core::workspace::Workspace,
    profile: &str,
) -> Result<(), String> {
    for member in &workspace.members {
        build_project_at(&member.path, profile, &[])?;
    }
    Ok(())
}

fn build_project_at(
    project_root: &Path,
    profile: &str,
    exclude_dirs: &[std::path::PathBuf],
) -> Result<(), String> {
    with_dir(project_root, || {
        let items = check_dir(None).map_err(|_| "Failed to read project directory".to_string())?;
        if !items.contains(&"dcr.toml".to_string()) {
            return Err("dcr.toml file not found".to_string());
        }
        let config = Config::open("./dcr.toml").map_err(|err| err.to_string())?;
        let project_name = get_config_str(&config, "package.name");
        let project_version = get_config_str(&config, "package.version");
        let project_compiler = get_config_str(&config, "build.compiler");
        let build_language = get_config_str(&config, "build.language");
        let build_standard = get_config_str(&config, "build.standard");
        let build_target = get_config_str(&config, "build.target");
        let build_kind = get_config_str(&config, "build.kind");
        let build_platform = get_config_str(&config, "build.platform");
        let tc_cc = get_config_opt(&config, "toolchain.cc");
        let tc_cxx = get_config_opt(&config, "toolchain.cxx");
        let tc_as = get_config_opt(&config, "toolchain.as");
        let tc_ar = get_config_opt(&config, "toolchain.ar");
        let tc_ld = get_config_opt(&config, "toolchain.ld");
        let build_cflags = get_config_list(&config, "build.cflags")?;
        let build_ldflags = get_config_list(&config, "build.ldflags")?;

        let resolved_compiler = resolve_compiler(
            &build_language,
            &project_compiler,
            tc_cc.as_deref(),
            tc_cxx.as_deref(),
            tc_as.as_deref(),
        );
        let resolved_linker = resolve_tool("DCR_LD", tc_ld.as_deref());
        let resolved_archiver = resolve_tool("DCR_AR", tc_ar.as_deref());

        ensure_target_dirs(&items, profile, normalize_target(&build_target));

        let resolved = resolve_deps(&config, profile, project_root)?;
        let ctx = BuildContext {
            profile,
            project_name: &project_name,
            compiler: &resolved_compiler,
            language: &build_language,
            standard: &build_standard,
            target_dir: normalize_target(&build_target),
            kind: normalize_kind(&build_kind),
            platform: normalize_platform(&build_platform),
            linker: resolved_linker.as_deref(),
            archiver: resolved_archiver.as_deref(),
            exclude_dirs,
            include_dirs: &resolved.include_dirs,
            lib_dirs: &resolved.lib_dirs,
            libs: &resolved.libs,
            cflags: &build_cflags,
            ldflags: &build_ldflags,
        };
        let sources = collect_sources(&ctx)?;
        let headers = collect_header_files(&ctx, project_root)?;
        let lib_files = collect_lib_files(&ctx);
        let fingerprint = compute_build_fingerprint(&ctx, &sources, &headers, &lib_files)?;
        if should_skip_build(&ctx, &fingerprint) {
            return Ok(());
        }
        println!(
            "   {} {} v{}",
            colored("Compiling", BOLD_GREEN),
            project_name,
            project_version
        );
        run_build(&ctx)?;
        write_build_fingerprint(&ctx, &fingerprint)?;
        Ok(())
    })
}

fn with_dir<F, T>(dir: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let prev = std::env::current_dir().map_err(|_| "Failed to get current dir".to_string())?;
    std::env::set_current_dir(dir).map_err(|_| "Failed to change directory".to_string())?;
    let result = f();
    let _ = std::env::set_current_dir(prev);
    result
}

fn normalize_target(target: &str) -> Option<&str> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn normalize_kind(kind: &str) -> &str {
    let trimmed = kind.trim();
    if trimmed.is_empty() { "bin" } else { trimmed }
}

fn normalize_platform(platform: &str) -> Option<&str> {
    let trimmed = platform.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_compiler(
    language: &str,
    compiler: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> String {
    let lang = language.to_lowercase();
    env_override_compiler(&lang)
        .or_else(|| toolchain_override_compiler(&lang, tc_cc, tc_cxx, tc_as))
        .unwrap_or_else(|| compiler.to_string())
}

fn env_override_compiler(lang: &str) -> Option<String> {
    if let Ok(value) = std::env::var("DCR_COMPILER") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if lang == "asm" {
        if let Ok(value) = std::env::var("DCR_AS") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        return None;
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Ok(value) = std::env::var("DCR_CXX")
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(value) = std::env::var("DCR_CC") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn toolchain_override_compiler(
    lang: &str,
    tc_cc: Option<&str>,
    tc_cxx: Option<&str>,
    tc_as: Option<&str>,
) -> Option<String> {
    if lang == "asm" {
        return tc_as.map(|v| v.to_string());
    }
    if (lang == "c++" || lang == "cpp" || lang == "cxx")
        && let Some(v) = tc_cxx
    {
        return Some(v.to_string());
    }
    tc_cc.map(|v| v.to_string())
}

fn resolve_tool(env_key: &str, fallback: Option<&str>) -> Option<String> {
    if let Ok(value) = std::env::var(env_key) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    fallback.map(|v| v.to_string())
}

fn compute_build_fingerprint(
    ctx: &BuildContext,
    sources: &[String],
    headers: &[std::path::PathBuf],
    lib_files: &[std::path::PathBuf],
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

fn should_skip_build(ctx: &BuildContext, fingerprint: &str) -> bool {
    let output = build_output_path(ctx);
    if !Path::new(&output).is_file() {
        return false;
    }
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    let cached = fs::read_to_string(cache_path).unwrap_or_default();
    cached.trim() == fingerprint
}

fn write_build_fingerprint(ctx: &BuildContext, fingerprint: &str) -> Result<(), String> {
    let cache_path = build_cache_path(ctx.profile, ctx.target_dir);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Failed to create cache dir: {err}"))?;
    }
    fs::write(cache_path, format!("{fingerprint}\n"))
        .map_err(|err| format!("Failed to write cache: {err}"))
}

fn build_cache_path(profile: &str, target_dir: Option<&str>) -> std::path::PathBuf {
    match target_dir {
        Some(dir) => Path::new(dir).join(format!(".dcr-build.{profile}.hash")),
        None => Path::new("./target").join(profile).join(".dcr-build.hash"),
    }
}

fn build_output_path(ctx: &BuildContext) -> String {
    if ctx.kind == "staticlib" {
        return crate::platform::lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
    }
    if ctx.kind == "sharedlib" {
        return crate::platform::shared_lib_path(ctx.profile, ctx.project_name, ctx.target_dir);
    }
    crate::platform::bin_path(ctx.profile, ctx.project_name, ctx.target_dir)
}

fn collect_header_files(
    ctx: &BuildContext,
    project_root: &Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut out = Vec::new();
    let mut roots = Vec::new();
    roots.push(project_root.join("src"));
    for dir in ctx.include_dirs {
        roots.push(Path::new(dir).to_path_buf());
    }
    for root in roots {
        if !root.exists() {
            continue;
        }
        collect_header_files_rec(&root, &mut out, ctx.exclude_dirs)?;
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn collect_header_files_rec(
    dir: &Path,
    out: &mut Vec<std::path::PathBuf>,
    exclude_dirs: &[std::path::PathBuf],
) -> Result<(), String> {
    if is_excluded(dir, exclude_dirs) {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| format!("read_dir error: {err}"))? {
        let entry = entry.map_err(|err| format!("read_dir error: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, exclude_dirs) {
                continue;
            }
            collect_header_files_rec(&path, out, exclude_dirs)?;
            continue;
        }
        if !path.is_file() {
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

fn collect_lib_files(ctx: &BuildContext) -> Vec<std::path::PathBuf> {
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

fn is_excluded(path: &Path, exclude_dirs: &[std::path::PathBuf]) -> bool {
    exclude_dirs.iter().any(|dir| path.starts_with(dir))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}
