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

    let release_dir = host_profile_dir(&root, "release");
    let debug_dir = host_profile_dir(&root, "debug");
    assert!(
        !release_dir.exists(),
        "root release profile dir should be removed: {}",
        release_dir.display()
    );
    assert!(
        debug_dir.exists(),
        "root debug profile dir should remain: {}",
        debug_dir.display()
    );

    let out = run_dcr_env(&["clean", "--all"], &root, &envs);
    assert!(out.status.success(), "workspace clean --all should succeed");
    assert!(!root.join("target").exists(), "root target/ should be gone");
}

/// A workspace member library with a non-conventional artifact name must be
/// linked from the workspace `target/lib`, where packaging copies it.
#[test]
fn workspace_library_with_custom_artifact_name_links() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping workspace library artifact test");
        return;
    };

    let root = unique_sandbox_dir("workspace_lib_name");
    let out = run_dcr(&["init"], &root);
    assert!(out.status.success(), "root init should succeed");

    let lib_dir = root.join("src").join("mylib");
    let app_dir = root.join("src").join("myapp");
    std::fs::create_dir_all(lib_dir.join("src")).expect("create lib src");
    std::fs::create_dir_all(app_dir.join("src")).expect("create app src");

    std::fs::write(
        lib_dir.join("dcr.toml"),
        "[package]\nname = \"mylib\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\nkind = \"staticlib\"\nfilename = \"sct\"\nextension = \"a\"\n",
    )
    .expect("write lib dcr.toml");
    std::fs::write(lib_dir.join("src/mathlib.h"), "int answer(void);\n").expect("write lib header");
    std::fs::write(
        lib_dir.join("src/mathlib.c"),
        "int answer(void) { return 42; }\n",
    )
    .expect("write lib source");

    std::fs::write(
        app_dir.join("dcr.toml"),
        "[package]\nname = \"myapp\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nkind = \"bin\"\n",
    )
    .expect("write app dcr.toml");
    std::fs::write(
        app_dir.join("src/main.c"),
        "#include \"mathlib.h\"\nint main(void) { return answer() == 42 ? 0 : 1; }\n",
    )
    .expect("write app source");

    let root_toml = std::fs::read_to_string(root.join("dcr.toml")).expect("read root dcr.toml");
    let workspace_toml = format!(
        "{root_toml}\n[workspace]\nmylib = {{ path = \"src/mylib\", deps = [] }}\nmyapp = {{ path = \"src/myapp\", deps = [\"mylib\"] }}\n"
    );
    std::fs::write(root.join("dcr.toml"), workspace_toml).expect("write workspace dcr.toml");

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &root, &envs);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "workspace build with a custom library artifact name failed:\n{combined}"
    );
    assert!(
        root.join("target").join("lib").join("sct.a").is_file(),
        "workspace library should be packaged into the workspace target/lib:\n{combined}"
    );
}
