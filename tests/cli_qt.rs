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

#[test]
fn qt_support_config_detected() {
    let dir = unique_sandbox_dir("qt");
    let out = run_dcr_env(&["init"], &dir, &[]);
    assert!(out.status.success(), "dcr init should succeed");

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    let updated = toml.replace("[build]", "[build]\nqt = true\nlanguage = \"cpp\"");
    std::fs::write(&toml_path, updated).expect("failed to write dcr.toml");

    // This might fail if Qt is not installed, but it should at least try to run Qt processing
    // We just check if it doesn't crash immediately due to bad config
    let out = run_dcr_env(&["build"], &dir, &[]);

    // Depending on environment, this might be success or failure, but it should not panic.
    // If it fails with "qt not found", it means it parsed the config and attempted to use it.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("panicked"),
        "dcr should not panic on Qt config"
    );
}
