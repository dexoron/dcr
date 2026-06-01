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
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Running"), "dcr run should start");

    let out = run_dcr_env(&["clean", "--release"], &dir, &envs);
    assert!(out.status.success(), "dcr clean --release should succeed");
    let target_dir = "target/x86_64-unknown-linux-gnu".to_string();
    assert!(
        !dir.join(&target_dir).join("release").exists(),
        "target/x86_64-unknown-linux-gnu/release should be removed"
    );
    assert!(
        dir.join(&target_dir).join("debug").is_dir(),
        "target/x86_64-unknown-linux-gnu/debug should remain"
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

    let target_dir = "target/x86_64-unknown-linux-gnu";
    assert!(
        !root.join(target_dir).join("release").exists(),
        "root target/x86_64-unknown-linux-gnu/release should be removed"
    );
    assert!(
        root.join(target_dir).join("debug").exists(),
        "root target/x86_64-unknown-linux-gnu/debug should remain"
    );
}

#[test]
fn dcr_add_dependencies() {
    let dir = unique_sandbox_dir("add_dep");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    // Test path: prefix
    let out = run_dcr(&["add", "mylib", "path:./libs/mylib"], &dir);
    assert!(out.status.success(), "dcr add path should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("mylib = { path = \"./libs/mylib\" }"),
        "path dep not found in toml"
    );

    // Test github: prefix
    let out = run_dcr(&["add", "gh_lib", "github:user/repo"], &dir);
    assert!(out.status.success(), "dcr add github should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("gh_lib = { git = \"https://github.com/user/repo\" }"),
        "github dep not found in toml"
    );

    // Test git: prefix (generic)
    let out = run_dcr(&["add", "custom_git", "git:host.com/user/repo"], &dir);
    assert!(out.status.success(), "dcr add custom git should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("custom_git = { git = \"https://host.com/user/repo\" }"),
        "custom git dep not found in toml"
    );

    // Test git: prefix (github default)
    let out = run_dcr(&["add", "git_short", "git:user/repo"], &dir);
    assert!(out.status.success(), "dcr add git short should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("git_short = { git = \"https://github.com/user/repo\" }"),
        "git short dep not found in toml"
    );

    // Test flags (branch)
    let out = run_dcr(
        &["add", "branch_lib", "github:user/repo", "--branch", "dev"],
        &dir,
    );
    assert!(out.status.success(), "dcr add with branch should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("branch_lib = { git = \"https://github.com/user/repo\", branch = \"dev\" }"),
        "branch lib not found in toml"
    );

    // Test failure on no prefix
    let out = run_dcr(&["add", "fail_lib", "user/repo"], &dir);
    assert!(!out.status.success(), "dcr add without prefix should fail");
}

#[test]
fn dcr_builds_lib_package() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping lib package test");
        return;
    };

    let dir = unique_sandbox_dir("lib_package");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    let updated_toml = toml
        .replace("kind = \"bin\"", "kind = \"staticlib\"")
        .replace("type = \"none\"", "type = \"lib\"");
    std::fs::write(dir.join("dcr.toml"), updated_toml).expect("failed to write toml");

    std::fs::write(dir.join("src").join("my_lib.h"), "void hello();")
        .expect("failed to write header");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(out.status.success(), "dcr build should succeed");

    let target_dir = dir.join("target");
    assert!(
        target_dir.join("include").join("my_lib.h").is_file(),
        "include/my_lib.h missing"
    );
    assert!(target_dir.join("lib").exists(), "lib directory missing");
}

#[test]
fn registry_dependency_is_built_from_cache() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping registry dependency build test");
        return;
    };

    let root = unique_sandbox_dir("registry_dep");
    let home = root.join("home");
    let dcr_home = home.join(".dcr");
    let dep = root.join("cache").join("mylib");
    let app = root.join("app");
    std::fs::create_dir_all(dcr_home.as_path()).expect("failed to create dcr home");
    std::fs::create_dir_all(dep.join("src")).expect("failed to create dep src");
    std::fs::create_dir_all(app.join("src")).expect("failed to create app src");

    std::fs::write(
        dcr_home.join("config.toml"),
        "[registry.local]\nurl = \"file://local\"\npriority = 1\n",
    )
    .expect("failed to write registry config");
    std::fs::write(
        dcr_home.join("index.json"),
        format!(
            "{{\"packages\":[{{\"name\":\"mylib\",\"latest_version\":\"0.1.0\",\"path\":\"{}\"}}]}}",
            dep.display()
        ),
    )
    .expect("failed to write registry index");

    std::fs::write(
        dep.join("dcr.toml"),
        "[package]\nname = \"mylib\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"staticlib\"\n\n[dependencies]\n",
    )
    .expect("failed to write dep dcr.toml");
    std::fs::write(dep.join("src").join("mylib.h"), "int answer(void);\n")
        .expect("failed to write header");
    std::fs::write(
        dep.join("src").join("mylib.c"),
        "int answer(void) { return 42; }\n",
    )
    .expect("failed to write dep source");

    std::fs::write(
        app.join("dcr.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\ntype = \"none\"\n\n[build]\nlanguage = \"c\"\nstandard = \"c11\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\nmylib = \"0.1.0\"\n",
    )
    .expect("failed to write app dcr.toml");
    std::fs::write(
        app.join("src").join("main.c"),
        "#include \"mylib.h\"\nint main(void) { return answer() == 42 ? 0 : 1; }\n",
    )
    .expect("failed to write app source");

    let index_path = dcr_home.join("index.json");
    let envs = [
        ("DCR_COMPILER", compiler),
        ("HOME", home.to_str().unwrap()),
        ("DCR_INDEX_PATH", index_path.to_str().unwrap()),
    ];
    let out = run_dcr_env(&["build"], &app, &envs);
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(
        out.status.success(),
        "registry dependency build should succeed"
    );
    assert!(
        dep.join("target").join("include").join("mylib.h").is_file(),
        "registry dependency headers were not packaged"
    );
    assert!(
        dep.join("target").join("lib").exists(),
        "registry dependency library directory missing"
    );
}

fn parse_project_name(toml: &str) -> String {
    toml.lines()
        .find(|l| l.trim().starts_with("name ="))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .expect("could not parse project name from dcr.toml")
}

fn default_artifact_path(project_root: &Path, project_name: &str) -> PathBuf {
    project_root
        .join("target")
        .join("x86_64-unknown-linux-gnu")
        .join("debug")
        .join(project_name)
}

#[test]
fn build_with_target_config() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping target config test");
        return;
    };

    let dir = unique_sandbox_dir("target_cfg");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    let project_name = parse_project_name(&toml);
    let updated = toml.replace("[build]", "[build]\ntarget = \"linux\"");
    std::fs::write(&toml_path, updated).expect("failed to write dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    assert!(
        out.status.success(),
        "dcr build with target = \"linux\" should succeed"
    );

    let artifact = default_artifact_path(&dir, &project_name);
    assert!(
        artifact.is_file(),
        "artifact should be at default path target/x86_64-unknown-linux-gnu/debug/{}",
        project_name
    );
}

#[test]
fn build_with_out_dir() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping out_dir test");
        return;
    };

    let dir = unique_sandbox_dir("out_dir");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    let project_name = parse_project_name(&toml);
    let updated = toml.replace("[build]", "[build]\nout_dir = \"./_BUILD\"");
    std::fs::write(&toml_path, updated).expect("failed to write dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(
        out.status.success(),
        "dcr build with out_dir should succeed"
    );

    let artifact = dir.join("_BUILD").join(&project_name);
    assert!(
        artifact.is_file(),
        "artifact should be at _BUILD/{} (custom out_dir)",
        project_name
    );

    let default_path = default_artifact_path(&dir, &project_name);
    assert!(
        !default_path.exists(),
        "artifact should NOT be at default path when out_dir is set"
    );
}

#[test]
fn dcr_test_runs_without_sandbox_dependency() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping dcr test integration");
        return;
    };

    let dir = unique_sandbox_dir("dcr_test_independent");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let envs = [("DCR_CC", compiler)];
    let out_init = run_dcr_env(&["test", "--init"], &dir, &envs);
    assert!(out_init.status.success(), "dcr test --init should succeed");

    let out = run_dcr_env(&["test"], &dir, &envs);
    assert!(out.status.success(), "dcr test should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("TOTAL: 1"), "TOTAL summary line missing");
    assert!(
        stdout.contains("PASS:  1"),
        "PASS summary line missing\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}
