mod common;
use common::*;

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
    let dep_abs = dep.canonicalize().unwrap_or(dep.clone());
    let mut dep_path = dep_abs.to_string_lossy().replace('\\', "/");
    if let Some(rest) = dep_path.strip_prefix("//?/") {
        dep_path = rest.to_string();
    }
    if dep_path.len() >= 3 && dep_path.as_bytes()[0] == b'/' && dep_path.as_bytes()[2] == b':' {
        dep_path = dep_path[1..].to_string();
    }
    std::fs::write(
        dcr_home.join("index.json"),
        serde_json::json!({
            "packages": [{
                "name": "mylib",
                "latest_version": "0.1.0",
                "path": dep_path
            }]
        })
        .to_string(),
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
    let home_s = home.to_string_lossy().to_string();
    let index_s = index_path.to_string_lossy().to_string();
    let mut envs = vec![
        ("DCR_COMPILER", compiler),
        ("HOME", home_s.as_str()),
        ("DCR_INDEX_PATH", index_s.as_str()),
    ];
    if cfg!(windows) {
        envs.push(("USERPROFILE", home_s.as_str()));
    }
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
