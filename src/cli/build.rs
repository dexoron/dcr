use crate::config::{PROFILE, flags};
use crate::core::builder::{BuildContext, build as build_project, collect_sources};
use crate::core::config::Config;
use crate::core::deps::resolve_deps;
use crate::core::workspace::parse_workspace;
use crate::utils::fs::{check_dir, find_project_root};
use crate::utils::log::{error, warn};
use crate::utils::text::{BOLD_GREEN, colored};
use glob::glob;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
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
        let tc_uic = get_config_opt(&config, "toolchain.uic");
        let tc_moc = get_config_opt(&config, "toolchain.moc");
        let tc_rcc = get_config_opt(&config, "toolchain.rcc");
        let build_cflags = get_config_list(&config, "build.cflags")?;
        let build_ldflags = get_config_list(&config, "build.ldflags")?;
        let pkg_configs = get_config_list(&config, "build.pkg_config")?;
        let build_generated = get_config_list(&config, "build.generated")?;
        let build_steps = get_build_steps(&config)?;

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
        let (resolved_cflags, resolved_ldflags) =
            resolve_pkg_config_flags(&pkg_configs, &build_cflags, &build_ldflags)?;
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
            cflags: &resolved_cflags,
            ldflags: &resolved_ldflags,
        };
        if std::env::var("DCR_DEBUG").is_ok() {
            eprintln!("[dcr] debug: compiler={}", ctx.compiler);
            eprintln!("[dcr] debug: cflags={:?}", ctx.cflags);
            eprintln!("[dcr] debug: ldflags={:?}", ctx.ldflags);
            eprintln!("[dcr] debug: lib_dirs={:?}", ctx.lib_dirs);
            eprintln!("[dcr] debug: libs={:?}", ctx.libs);
        }
        let tool_execs = resolve_toolchain_execs(&tc_uic, &tc_moc, &tc_rcc, &pkg_configs);
        let step_flags =
            build_step_flags(&resolved_cflags, &resolved.include_dirs, &resolved_compiler);
        let steps_dirty = build_steps_need_run(&build_steps)?;
        if steps_dirty {
            clean_generated_files(&build_generated)?;
            run_build_steps(&build_steps, &tool_execs, &step_flags)?;
        }
        let sources = collect_sources(&ctx)?;
        let headers = collect_header_files(&ctx, project_root)?;
        let lib_files = collect_lib_files(&ctx);
        let fingerprint = compute_build_fingerprint(&ctx, &sources, &headers, &lib_files)?;
        let mut skip = should_skip_build(&ctx, &fingerprint);
        if steps_dirty {
            skip = false;
        }
        let debug_enabled = std::env::var("DCR_DEBUG").is_ok();
        if skip && !debug_enabled {
            return Ok(());
        }
        println!(
            "   {} {} v{}",
            colored("Compiling", BOLD_GREEN),
            project_name,
            project_version
        );
        if !skip {
            run_build(&ctx)?;
            write_build_fingerprint(&ctx, &fingerprint)?;
        }
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

fn resolve_pkg_config_flags(
    pkgs: &[String],
    base_cflags: &[String],
    base_ldflags: &[String],
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut cflags = base_cflags.to_vec();
    let mut ldflags = base_ldflags.to_vec();
    if pkgs.is_empty() {
        return Ok((cflags, ldflags));
    }
    for pkg in pkgs {
        let c_out = run_pkg_config(pkg, "--cflags")?;
        let l_out = run_pkg_config(pkg, "--libs")?;
        cflags.extend(split_flags(&c_out));
        ldflags.extend(split_flags(&l_out));
    }
    Ok((cflags, ldflags))
}

fn run_pkg_config(pkg: &str, arg: &str) -> Result<String, String> {
    let output = std::process::Command::new("pkg-config")
        .arg(arg)
        .arg(pkg)
        .output()
        .map_err(|err| format!("Failed to run pkg-config: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pkg-config failed for {pkg}: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn split_flags(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(ch) = chars.next() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
                continue;
            }
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            current.push(ch);
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                out.push(current.clone());
                current.clear();
            }
            continue;
        }
        if ch == '\\' {
            if let Some(next) = chars.next() {
                current.push(next);
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

struct ToolchainExecs {
    uic: String,
    moc: String,
    rcc: String,
}

fn resolve_toolchain_execs(
    uic: &Option<String>,
    moc: &Option<String>,
    rcc: &Option<String>,
    pkg_configs: &[String],
) -> ToolchainExecs {
    let qt_bins = resolve_qt_host_bins(pkg_configs);
    ToolchainExecs {
        uic: resolve_qt_tool(uic, qt_bins.as_deref(), "uic"),
        moc: resolve_qt_tool(moc, qt_bins.as_deref(), "moc"),
        rcc: resolve_qt_tool(rcc, qt_bins.as_deref(), "rcc"),
    }
}

fn resolve_qt_tool(configured: &Option<String>, qt_bins: Option<&Path>, tool: &str) -> String {
    if let Some(value) = configured {
        return value.clone();
    }
    if let Some(dir) = qt_bins {
        let candidate = dir.join(tool);
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
        if cfg!(target_os = "windows") {
            let candidate = dir.join(format!("{tool}.exe"));
            if candidate.is_file() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    if let Some(candidate) = detect_qt6_tool_variant(tool) {
        return candidate;
    }
    tool.to_string()
}

fn resolve_qt_host_bins(pkgs: &[String]) -> Option<PathBuf> {
    let qt_pkgs: Vec<&String> = pkgs.iter().filter(|p| p.starts_with("Qt6")).collect();
    if qt_pkgs.is_empty() {
        return None;
    }
    let preferred = ["Qt6Core", "Qt6Widgets", "Qt6Gui"];
    for name in preferred {
        if let Some(dir) = query_pkg_config_var(name, "host_bins") {
            return Some(dir);
        }
        if let Some(dir) = query_pkg_config_var(name, "libexecdir")
            && let Some(bin) = qt_bins_from_libexec(&dir)
        {
            return Some(bin);
        }
        if let Some(dir) = query_pkg_config_var(name, "bindir") {
            return Some(dir);
        }
    }
    for pkg in qt_pkgs {
        if let Some(dir) = query_pkg_config_var(pkg, "host_bins") {
            return Some(dir);
        }
        if let Some(dir) = query_pkg_config_var(pkg, "libexecdir")
            && let Some(bin) = qt_bins_from_libexec(&dir)
        {
            return Some(bin);
        }
        if let Some(dir) = query_pkg_config_var(pkg, "bindir") {
            return Some(dir);
        }
    }
    None
}

fn query_pkg_config_var(pkg: &str, var: &str) -> Option<PathBuf> {
    let output = std::process::Command::new("pkg-config")
        .arg(format!("--variable={var}"))
        .arg(pkg)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    if path.is_dir() { Some(path) } else { None }
}

fn qt_bins_from_libexec(libexec: &Path) -> Option<PathBuf> {
    for tool in ["moc", "uic", "rcc"] {
        if libexec.join(tool).is_file() {
            return Some(libexec.to_path_buf());
        }
        if cfg!(target_os = "windows") && libexec.join(format!("{tool}.exe")).is_file() {
            return Some(libexec.to_path_buf());
        }
    }
    let bin = libexec.join("bin");
    if bin.is_dir() { Some(bin) } else { None }
}

fn detect_qt6_tool_variant(tool: &str) -> Option<String> {
    [format!("{tool}6"), format!("{tool}-qt6")]
        .into_iter()
        .find(|candidate| is_on_path(candidate))
}

fn is_on_path(cmd: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[derive(Clone)]
struct BuildStep {
    name: String,
    input: String,
    output: String,
    cmd: String,
}

fn get_build_steps(config: &Config) -> Result<Vec<BuildStep>, String> {
    let value = match config.get("build.steps") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };
    let arr = value
        .as_array()
        .ok_or_else(|| "build.steps must be an array".to_string())?;
    let mut out = Vec::new();
    for item in arr {
        let tbl = item
            .as_table()
            .ok_or_else(|| "build.steps entries must be tables".to_string())?;
        let name = tbl
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let input = tbl
            .get("in")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let output = tbl
            .get("out")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let cmd = tbl
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() || input.is_empty() || output.is_empty() || cmd.is_empty() {
            return Err("build.steps entries must include name, in, out, cmd".to_string());
        }
        out.push(BuildStep {
            name,
            input,
            output,
            cmd,
        });
    }
    Ok(out)
}

fn run_build_steps(
    steps: &[BuildStep],
    tools: &ToolchainExecs,
    step_flags: &str,
) -> Result<(), String> {
    for step in steps {
        run_build_step(step, tools, step_flags)?;
    }
    Ok(())
}

fn run_build_step(
    step: &BuildStep,
    tools: &ToolchainExecs,
    step_flags: &str,
) -> Result<(), String> {
    let inputs = expand_glob(&step.input)?;
    if inputs.is_empty() {
        return Ok(());
    }
    let needs_stem = step.output.contains("{stem}");
    if inputs.len() > 1 && !needs_stem {
        return Err(format!(
            "build.steps '{}' output must include {{stem}} for multiple inputs",
            step.name
        ));
    }
    for input in inputs {
        let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
        let out_path = PathBuf::from(step.output.replace("{stem}", stem));
        if !should_run_step(&input, &out_path) {
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create step output dir: {err}"))?;
        }
        let cmd = substitute_step_cmd(&step.cmd, &input, &out_path, tools, step_flags, stem);
        let status = run_shell_command(&cmd)
            .map_err(|err| format!("Failed to run step '{}': {err}", step.name))?;
        if !status.success() {
            return Err(format!("Step '{}' failed", step.name));
        }
    }
    Ok(())
}

fn clean_generated_files(patterns: &[String]) -> Result<(), String> {
    for pattern in patterns {
        for path in expand_glob(pattern)? {
            if path.is_file() {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

fn expand_glob(pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let entries = glob(pattern).map_err(|err| format!("glob error: {err}"))?;
    for entry in entries {
        let path = entry.map_err(|err| format!("glob error: {err}"))?;
        out.push(path);
    }
    Ok(out)
}

fn should_run_step(input: &Path, output: &Path) -> bool {
    let in_time = fs::metadata(input).and_then(|m| m.modified());
    let out_time = fs::metadata(output).and_then(|m| m.modified());
    match (in_time, out_time) {
        (Ok(i), Ok(o)) => i > o,
        (Ok(_), Err(_)) => true,
        _ => true,
    }
}

fn substitute_step_cmd(
    template: &str,
    input: &Path,
    output: &Path,
    tools: &ToolchainExecs,
    step_flags: &str,
    stem: &str,
) -> String {
    template
        .replace("{in}", &input.to_string_lossy())
        .replace("{out}", &output.to_string_lossy())
        .replace("{uic}", &tools.uic)
        .replace("{moc}", &tools.moc)
        .replace("{rcc}", &tools.rcc)
        .replace("{cflags}", step_flags)
        .replace("{stem}", stem)
}

fn build_step_flags(cflags: &[String], include_dirs: &[String], compiler: &str) -> String {
    let mut out = Vec::new();
    let msvc_style = is_msvc_compiler(compiler) || cflags.iter().any(|f| f.starts_with('/'));
    for flag in cflags {
        if flag.starts_with("-I") || flag.starts_with("-D") {
            out.push(flag.clone());
        }
        if flag.starts_with("/I") || flag.starts_with("/D") {
            out.push(flag.clone());
        }
        if msvc_style && flag.starts_with("-D") {
            out.push(format!("/D{}", flag.trim_start_matches("-D")));
        }
    }
    for dir in include_dirs {
        out.push(format!("-I{dir}"));
        if msvc_style {
            out.push(format!("/I{dir}"));
        }
    }
    let mut dedup = Vec::new();
    for item in out {
        if !dedup.contains(&item) {
            dedup.push(item);
        }
    }
    dedup
        .into_iter()
        .map(quote_step_arg)
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_step_arg(arg: String) -> String {
    if !arg.chars().any(|c| c.is_whitespace() || c == '"') {
        return arg;
    }
    let escaped = arg.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn is_msvc_compiler(compiler: &str) -> bool {
    let lower = compiler.to_lowercase();
    lower.contains("cl.exe")
        || lower == "cl"
        || lower.contains("clang-cl")
        || lower.contains("msvc")
}

fn run_shell_command(cmd: &str) -> Result<std::process::ExitStatus, std::io::Error> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .status()
    } else {
        std::process::Command::new("sh").arg("-c").arg(cmd).status()
    }
}

fn build_steps_need_run(steps: &[BuildStep]) -> Result<bool, String> {
    for step in steps {
        let inputs = expand_glob(&step.input)?;
        if inputs.is_empty() {
            continue;
        }
        let needs_stem = step.output.contains("{stem}");
        if inputs.len() > 1 && !needs_stem {
            return Err(format!(
                "build.steps '{}' output must include {{stem}} for multiple inputs",
                step.name
            ));
        }
        for input in inputs {
            let stem = input.file_stem().and_then(|v| v.to_str()).unwrap_or("");
            let out_path = PathBuf::from(step.output.replace("{stem}", stem));
            if should_run_step(&input, &out_path) {
                return Ok(true);
            }
        }
    }
    Ok(false)
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
