mod common;
use common::*;

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
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let spam = combined.matches("Directory target not found").count();
    assert!(
        spam <= 1,
        "clean --all should not spam missing member target/: {combined}"
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

    let out = run_dcr_env(&["clean", "--all"], &root, &envs);
    assert!(out.status.success(), "workspace clean --all should succeed");
    assert!(!root.join("target").exists(), "root target/ should be gone");
}
