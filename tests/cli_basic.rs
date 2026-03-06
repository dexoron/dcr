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
    run_dcr_env(args, cwd, &[])
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

#[test]
fn help_and_version_work() {
    let dir = unique_sandbox_dir("help");
    let out = run_dcr(&["--help"], &dir);
    assert!(out.status.success(), "--help should succeed");

    let out = run_dcr(&["--version"], &dir);
    assert!(out.status.success(), "--version should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("dcr"), "version output should mention dcr");
}

#[test]
fn new_creates_project_layout() {
    let dir = unique_sandbox_dir("new");
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
    let dir = unique_sandbox_dir("init");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let target_debug = dir.join("target").join("debug");
    std::fs::create_dir_all(&target_debug).expect("failed to create target/debug");
    std::fs::write(target_debug.join("dummy.o"), "x").expect("failed to write dummy file");

    let out = run_dcr(&["clean"], &dir);
    assert!(out.status.success(), "dcr clean should succeed");
    assert!(!dir.join("target").exists(), "target should be removed");
}

#[test]
fn build_run_clean_flags_normal_project() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping build/run test");
        return;
    };

    let dir = unique_sandbox_dir("normal");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    assert!(out.status.success(), "dcr build should succeed");

    let out = run_dcr_env(&["build", "--release"], &dir, &envs);
    assert!(out.status.success(), "dcr build --release should succeed");

    let out = run_dcr_env(&["run"], &dir, &envs);
    assert!(out.status.success(), "dcr run should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Hello World!"), "run output missing");

    let out = run_dcr_env(&["clean", "--release"], &dir, &envs);
    assert!(out.status.success(), "dcr clean --release should succeed");
    assert!(
        !dir.join("target").join("release").exists(),
        "target/release should be removed"
    );
    assert!(
        dir.join("target").join("debug").is_dir(),
        "target/debug should remain"
    );

    let out = run_dcr_env(&["clean"], &dir, &envs);
    assert!(out.status.success(), "dcr clean should succeed");
    assert!(!dir.join("target").exists(), "target should be removed");
}

#[test]
fn workspace_build_and_clean_all() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping workspace test");
        return;
    };

    let root = unique_sandbox_dir("workspace");
    let out = run_dcr(&["init"], &root);
    assert!(out.status.success(), "root init should succeed");

    let members = [
        ("userspace", &[][..]),
        ("core", &["userspace"][..]),
        ("kernel", &["core"][..]),
    ];
    for (name, _) in &members {
        let member_dir = root.join("src").join(name);
        std::fs::create_dir_all(&member_dir).expect("failed to create member dir");
        let out = run_dcr(&["init"], &member_dir);
        assert!(out.status.success(), "member init should succeed");
    }

    let workspace_toml = "[package]\nname = \"ws-root\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[workspace]\nuserspace = { path = \"src/userspace\", deps = [] }\ncore = { path = \"src/core\", deps = [\"userspace\"] }\nkernel = { path = \"src/kernel\", deps = [\"core\"] }\n\n[dependencies]\n";
    std::fs::write(root.join("dcr.toml"), workspace_toml).expect("failed to write root dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &root, &envs);
    assert!(out.status.success(), "workspace build should succeed");

    let out = run_dcr_env(&["build"], &root, &envs);
    assert!(out.status.success(), "workspace build should succeed");

    let out = run_dcr_env(&["build", "--release"], &root, &envs);
    assert!(
        out.status.success(),
        "workspace build --release should succeed"
    );

    let out = run_dcr_env(&["clean", "--release", "--all"], &root, &envs);
    assert!(
        out.status.success(),
        "workspace clean --all --release should succeed"
    );

    for (name, _) in &members {
        let member = root.join("src").join(name);
        assert!(
            !member.join("target").join("release").exists(),
            "member target/release should be removed"
        );
        assert!(
            member.join("target").join("debug").exists(),
            "member target/debug should remain"
        );
    }
    assert!(
        !root.join("target").join("release").exists(),
        "root target/release should be removed"
    );
    assert!(
        root.join("target").join("debug").exists(),
        "root target/debug should remain"
    );
}
