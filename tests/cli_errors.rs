use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
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

fn unique_sandbox_dir(prefix: &str) -> PathBuf {
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

fn run_dcr(args: &[&str], cwd: &Path) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(args).current_dir(cwd);
    cmd.output().expect("failed to run dcr")
}

fn run_dcr_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(args).current_dir(cwd);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to run dcr")
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

fn available_compiler() -> Option<&'static str> {
    for candidate in ["clang", "gcc", "cc"] {
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

fn host_profile_dir(project_root: &Path, profile: &str) -> PathBuf {
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

// --- Tests ---

#[test]
fn no_args_shows_help() {
    let dir = unique_sandbox_dir("noargs");
    let out = run_dcr(&[], &dir);
    // Should succeed (help returns 0) and produce some output
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("DCR") || stdout.contains("dcr") || stdout.contains("USAGE"),
        "no args should show help text"
    );
}

#[test]
fn unknown_command_fails() {
    let dir = unique_sandbox_dir("unknown_cmd");
    let out = run_dcr(&["foobar"], &dir);
    assert!(
        !out.status.success(),
        "unknown command should exit non-zero"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("error") || combined.contains("Unknown"),
        "unknown command should print error"
    );
}

#[test]
fn build_without_toml_fails() {
    let dir = unique_sandbox_dir("build_no_toml");
    let out = run_dcr(&["build"], &dir);
    assert!(!out.status.success(), "build without dcr.toml should fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("dcr.toml") || combined.contains("not found"),
        "should mention missing dcr.toml"
    );
}

#[test]
fn run_without_toml_fails() {
    let dir = unique_sandbox_dir("run_no_toml");
    let out = run_dcr(&["run"], &dir);
    assert!(!out.status.success(), "run without dcr.toml should fail");
}

#[test]
fn clean_without_toml_fails() {
    let dir = unique_sandbox_dir("clean_no_toml");
    let out = run_dcr(&["clean"], &dir);
    assert!(!out.status.success(), "clean without dcr.toml should fail");
}

#[test]
fn new_existing_dir_fails() {
    let dir = unique_sandbox_dir("new_exist");
    // Create the project directory first
    std::fs::create_dir_all(dir.join("hello")).expect("failed to create dir");
    let out = run_dcr(&["new", "hello"], &dir);
    assert!(!out.status.success(), "dcr new on existing dir should fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("already exists") || combined.contains("error"),
        "should report dir already exists"
    );
}

#[test]
fn init_nonempty_dir_fails() {
    let dir = unique_sandbox_dir("init_nonempty");
    // Put a file in the dir so it's not empty
    std::fs::write(dir.join("dummy.txt"), "x").expect("failed to write");
    let out = run_dcr(&["init"], &dir);
    assert!(
        !out.status.success(),
        "dcr init in non-empty dir should fail"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("not empty") || combined.contains("error"),
        "should report dir not empty"
    );
}

#[test]
fn build_unknown_flag_fails() {
    let dir = unique_sandbox_dir("build_badflag");
    // Init a valid project first
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "init should succeed");
    let out = run_dcr(&["build", "--foobar"], &dir);
    assert!(!out.status.success(), "build with unknown flag should fail");
}

#[test]
fn run_library_project_fails() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping run_library test");
        return;
    };

    let dir = unique_sandbox_dir("run_lib");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "init should succeed");

    // Edit dcr.toml to set kind = "staticlib"
    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    let updated = toml.replace("kind = \"bin\"", "kind = \"staticlib\"");
    std::fs::write(&toml_path, updated).expect("failed to write dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["run"], &dir, &envs);
    assert!(!out.status.success(), "dcr run on staticlib should fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("library") || combined.contains("Cannot run"),
        "should report cannot run library"
    );
}

#[test]
fn clean_specific_profile() {
    let dir = unique_sandbox_dir("clean_profile");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "init should succeed");

    let debug_dir = host_profile_dir(&dir, "debug");
    let release_dir = host_profile_dir(&dir, "release");
    std::fs::create_dir_all(&debug_dir).expect("create debug");
    std::fs::write(debug_dir.join("dummy.o"), "x").expect("write");
    std::fs::create_dir_all(&release_dir).expect("create release");
    std::fs::write(release_dir.join("dummy.o"), "x").expect("write");

    let out = run_dcr(&["clean", "--release"], &dir);
    assert!(out.status.success(), "clean --release should succeed");
    assert!(
        !release_dir.exists(),
        "release profile dir should be removed: {}",
        release_dir.display()
    );
    assert!(
        debug_dir.is_dir(),
        "debug profile dir should remain: {}",
        debug_dir.display()
    );
}

#[test]
fn new_no_name_fails() {
    let dir = unique_sandbox_dir("new_noname");
    let out = run_dcr(&["new"], &dir);
    assert!(!out.status.success(), "dcr new without name should fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("not specified") || combined.contains("error"),
        "should report name not specified"
    );
}

#[test]
fn staticlib_build() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping staticlib test");
        return;
    };
    let dir = unique_sandbox_dir("staticlib");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "init should succeed");

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("read dcr.toml");
    let updated = toml.replace("kind = \"bin\"", "kind = \"staticlib\"");
    std::fs::write(&toml_path, updated).expect("write dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    assert!(out.status.success(), "staticlib build should succeed");
    // Check that a .a file exists
    let lib_path = dir
        .join("target")
        .join("x86_64-unknown-linux-gnu")
        .join("debug");
    let has_lib = std::fs::read_dir(&lib_path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| e.file_name().to_string_lossy().ends_with(".a"))
        })
        .unwrap_or(false);
    assert!(has_lib, "staticlib should produce a .a file");
}

#[test]
fn build_release_profile() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping release build test");
        return;
    };
    let dir = unique_sandbox_dir("release_build");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "init should succeed");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build", "--release"], &dir, &envs);
    assert!(out.status.success(), "release build should succeed");
    assert!(
        dir.join("target")
            .join("x86_64-unknown-linux-gnu")
            .join("release")
            .is_dir(),
        "target/x86_64-unknown-linux-gnu/release should exist"
    );
}

#[test]
fn version_contains_version_string() {
    let dir = unique_sandbox_dir("version");
    let out = run_dcr(&["--version"], &dir);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should contain version like "dcr 0.2.8 (target)"
    assert!(
        stdout.contains("dcr "),
        "version output should start with 'dcr '"
    );
    assert!(
        stdout.contains('.'),
        "version output should contain a version number"
    );
}
