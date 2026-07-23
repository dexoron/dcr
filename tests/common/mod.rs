use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static BUILD_ONCE: Once = Once::new();

pub fn bin_path() -> PathBuf {
    if let Ok(exe) = std::env::var("CARGO_BIN_EXE_dcr") {
        return PathBuf::from(exe);
    }
    ensure_bin_built();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(format!("dcr{}", std::env::consts::EXE_SUFFIX));
    path
}

pub fn unique_sandbox_dir(prefix: &str) -> PathBuf {
    let pid = std::process::id();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("sandbox");
    path.push("cli-tests");
    path.push(format!("dcr_{prefix}_{pid}_{n}_{now}"));
    std::fs::create_dir_all(&path).expect("failed to create temp dir");
    path
}

pub fn run_dcr(args: &[&str], cwd: &Path) -> std::process::Output {
    run_dcr_env(args, cwd, &[])
}

pub fn run_dcr_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(args).current_dir(cwd);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to run dcr")
}

pub fn ensure_bin_built() {
    BUILD_ONCE.call_once(|| {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build failed");
    });
}

#[allow(dead_code)]
pub fn available_compiler() -> Option<&'static str> {
    for candidate in ["gcc", "clang", "cc"] {
        let ok = Command::new(candidate)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return Some(candidate);
        }
    }
    None
}

#[allow(dead_code)]
pub fn is_git_in_path() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn parse_project_name(toml: &str) -> String {
    toml.lines()
        .find(|l| l.trim().starts_with("name ="))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .expect("could not parse project name from dcr.toml")
}

#[allow(dead_code)]
pub fn host_profile_dir(project_root: &Path, profile: &str) -> PathBuf {
    let target = project_root.join("target");
    if cfg!(target_os = "linux") {
        let arch = std::env::consts::ARCH;
        let env = if cfg!(target_env = "musl") {
            "musl"
        } else {
            "gnu"
        };
        target
            .join(format!("{arch}-unknown-linux-{env}"))
            .join(profile)
    } else if cfg!(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        target.join(format!("{arch}-unknown-{os}")).join(profile)
    } else {
        target.join(profile)
    }
}

#[allow(dead_code)]
pub fn bin_name(project_name: &str) -> String {
    if cfg!(windows) {
        format!("{project_name}.exe")
    } else {
        project_name.to_string()
    }
}

#[allow(dead_code)]
pub fn default_artifact_path(project_root: &Path, project_name: &str) -> PathBuf {
    host_profile_dir(project_root, "debug").join(bin_name(project_name))
}

#[allow(dead_code)]
pub fn artifact_candidates(dir: &Path, project_name: &str) -> Vec<PathBuf> {
    let base = bin_name(project_name);
    let bare = project_name.to_string();
    vec![
        dir.join(&base),
        dir.join(&bare),
        dir.join(format!("{project_name}.exe")),
    ]
}

#[allow(dead_code)]
pub fn assert_artifact_in(dir: &Path, project_name: &str, ctx: &str) {
    let candidates = artifact_candidates(dir, project_name);
    assert!(
        candidates.iter().any(|p| p.is_file()),
        "{ctx}: expected binary in {}, candidates: {candidates:?}",
        dir.display()
    );
}
