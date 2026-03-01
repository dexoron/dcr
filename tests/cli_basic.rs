use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static BUILD_ONCE: Once = Once::new();

fn bin_path() -> PathBuf {
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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let pid = std::process::id();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("dcr_{prefix}_{pid}_{n}_{now}"));
    std::fs::create_dir_all(&path).expect("failed to create temp dir");
    path
}

fn run_dcr(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(bin_path())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run dcr")
}

fn ensure_bin_built() {
    BUILD_ONCE.call_once(|| {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build failed");
    });
}

#[test]
fn help_and_version_work() {
    let dir = unique_temp_dir("help");
    let out = run_dcr(&["--help"], &dir);
    assert!(out.status.success(), "--help should succeed");

    let out = run_dcr(&["--version"], &dir);
    assert!(out.status.success(), "--version should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("dcr"), "version output should mention dcr");
}

#[test]
fn new_creates_project_layout() {
    let dir = unique_temp_dir("new");
    let out = run_dcr(&["new", "hello"], &dir);
    assert!(out.status.success(), "dcr new should succeed");

    let project_dir = dir.join("hello");
    assert!(project_dir.is_dir(), "project dir should exist");
    assert!(project_dir.join("dcr.toml").is_file(), "dcr.toml missing");
    assert!(
        project_dir.join("src").join("main.c").is_file(),
        "src/main.c missing"
    );
}

#[test]
fn init_and_clean_remove_target() {
    let dir = unique_temp_dir("init");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let target_debug = dir.join("target").join("debug");
    std::fs::create_dir_all(&target_debug).expect("failed to create target/debug");
    std::fs::write(target_debug.join("dummy.o"), "x").expect("failed to write dummy file");

    let out = run_dcr(&["clean"], &dir);
    assert!(out.status.success(), "dcr clean should succeed");
    assert!(!dir.join("target").exists(), "target should be removed");
}
