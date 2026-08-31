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

    // A bare filesystem path is equivalent to the explicit path: form.
    let out = run_dcr(&["add", "otherlib", "../otherlib"], &dir);
    assert!(out.status.success(), "dcr add bare path should succeed");
    let toml = std::fs::read_to_string(dir.join("dcr.toml")).unwrap();
    assert!(
        toml.contains("otherlib = { path = \"../otherlib\" }"),
        "bare path dep not found in toml"
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
fn path_prebuilt_dependency_without_manifest_links() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping prebuilt path dependency test");
        return;
    };
    if !is_ar_in_path() {
        eprintln!("ar unavailable; skipping prebuilt path dependency test");
        return;
    }

    let root = unique_sandbox_dir("path_prebuilt_dep");
    let dep = root.join("dep");
    let app = root.join("app");
    write_prebuilt_library(
        &dep,
        compiler,
        "prebuilt",
        "int prebuilt_answer(void);\n",
        "int prebuilt_answer(void) { return 42; }\n",
    );
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(
        app.join("dcr.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\nprebuilt = { path = \"../dep\", include = [\"include\"], lib = [\"lib\"], libs = [\"prebuilt\"] }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.c"),
        "#include \"prebuilt.h\"\nint main(void) { return prebuilt_answer() == 42 ? 0 : 1; }\n",
    )
    .unwrap();

    let out = run_dcr_env(&["build"], &app, &[("DCR_COMPILER", compiler)]);
    assert!(
        out.status.success(),
        "prebuilt path dependency build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn path_dcr_dependency_is_built_before_consumer() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping path dependency build test");
        return;
    };
    let root = unique_sandbox_dir("path_dcr_dep");
    let dep = root.join("dep");
    let app = root.join("app");
    std::fs::create_dir_all(dep.join("src")).unwrap();
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(
        dep.join("dcr.toml"),
        "[package]\nname = \"mathlib\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"staticlib\"\n",
    )
    .unwrap();
    std::fs::write(dep.join("src/mathlib.h"), "int answer(void);\n").unwrap();
    std::fs::write(
        dep.join("src/mathlib.c"),
        "int answer(void) { return 42; }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("dcr.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\nrenamed = { path = \"../dep\" }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.c"),
        "#include \"mathlib.h\"\nint main(void) { return answer() == 42 ? 0 : 1; }\n",
    )
    .unwrap();

    let out = run_dcr_env(&["build"], &app, &[("DCR_COMPILER", compiler)]);
    assert!(
        out.status.success(),
        "path dependency build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dep.join("target/include/mathlib.h").is_file());
    assert!(dep.join("target/lib").is_dir());
}

#[test]
fn path_dcr_dependencies_build_transitively() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping transitive path dependency test");
        return;
    };
    let root = unique_sandbox_dir("path_dcr_transitive");
    let leaf = root.join("leaf");
    let middle = root.join("middle");
    let app = root.join("app");
    for project in [&leaf, &middle, &app] {
        std::fs::create_dir_all(project.join("src")).unwrap();
    }
    std::fs::write(
        leaf.join("dcr.toml"),
        "[package]\nname = \"leaf\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"staticlib\"\n",
    )
    .unwrap();
    std::fs::write(leaf.join("src/leaf.h"), "int leaf_value(void);\n").unwrap();
    std::fs::write(
        leaf.join("src/leaf.c"),
        "int leaf_value(void) { return 40; }\n",
    )
    .unwrap();
    std::fs::write(
        middle.join("dcr.toml"),
        "[package]\nname = \"middle\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"staticlib\"\n\n[dependencies]\nleaf = { path = \"../leaf\" }\n",
    )
    .unwrap();
    std::fs::write(middle.join("src/middle.h"), "int middle_value(void);\n").unwrap();
    std::fs::write(
        middle.join("src/middle.c"),
        "#include \"leaf.h\"\nint middle_value(void) { return leaf_value() + 2; }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("dcr.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\nmiddle = { path = \"../middle\" }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.c"),
        "#include \"middle.h\"\nint main(void) { return middle_value() == 42 ? 0 : 1; }\n",
    )
    .unwrap();

    let out = run_dcr_env(&["build"], &app, &[("DCR_COMPILER", compiler)]);
    assert!(
        out.status.success(),
        "transitive path dependency build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(leaf.join("target/lib").is_dir());
    assert!(middle.join("target/lib").is_dir());
}

#[test]
fn path_dependency_cycle_reports_the_full_chain() {
    let root = unique_sandbox_dir("path_dcr_cycle");
    let first = root.join("first");
    let second = root.join("second");
    for project in [&first, &second] {
        std::fs::create_dir_all(project.join("src")).unwrap();
    }
    std::fs::write(
        first.join("dcr.toml"),
        "[package]\nname = \"first\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nkind = \"staticlib\"\n\n[dependencies]\nsecond = { path = \"../second\" }\n",
    )
    .unwrap();
    std::fs::write(
        second.join("dcr.toml"),
        "[package]\nname = \"second\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\nkind = \"staticlib\"\n\n[dependencies]\nfirst = { path = \"../first\" }\n",
    )
    .unwrap();

    let out = run_dcr(&["build"], &first);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Dependency cycle detected"), "{stderr}");
    assert!(
        stderr.contains("first") && stderr.contains("second"),
        "{stderr}"
    );
}

#[test]
fn git_dcr_dependency_is_cloned_built_and_locked() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping git dependency build test");
        return;
    };
    if !is_git_in_path() {
        eprintln!("git unavailable; skipping git dependency build test");
        return;
    }
    let root = unique_sandbox_dir("git_dcr_dep");
    let home = root.join("home");
    let origin = root.join("origin");
    let app = root.join("app");
    std::fs::create_dir_all(origin.join("src")).unwrap();
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(
        origin.join("dcr.toml"),
        "[package]\nname = \"gitmath\"\nversion = \"0.1.0\"\ntype = \"lib\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"staticlib\"\n",
    )
    .unwrap();
    std::fs::write(origin.join("src/gitmath.h"), "int git_answer(void);\n").unwrap();
    std::fs::write(
        origin.join("src/gitmath.c"),
        "int git_answer(void) { return 7; }\n",
    )
    .unwrap();
    for args in [
        vec!["init"],
        vec!["add", "."],
        vec![
            "-c",
            "user.email=dcr@example.test",
            "-c",
            "user.name=DCR",
            "commit",
            "-m",
            "init",
        ],
        vec!["tag", "v0.1.0"],
    ] {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(&origin)
            .status()
            .unwrap();
        assert!(status.success());
    }
    let origin_url = origin.to_string_lossy().to_string();
    std::fs::write(
        app.join("dcr.toml"),
        format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\ngitmath = {{ git = \"{origin_url}\", tag = \"v0.1.0\" }}\n"),
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.c"),
        "#include \"gitmath.h\"\nint main(void) { return git_answer() == 7 ? 0 : 1; }\n",
    )
    .unwrap();
    let home_s = home.to_string_lossy().to_string();
    let out = run_dcr_env(
        &["build"],
        &app,
        &[("DCR_COMPILER", compiler), ("HOME", home_s.as_str())],
    );
    assert!(
        out.status.success(),
        "git dependency build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lock = std::fs::read_to_string(app.join("dcr.lock")).unwrap();
    assert!(lock.contains("git+"));
    assert!(!lock.contains("checksum = \"\""));
}

#[test]
fn git_prebuilt_dependency_without_manifest_links() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping prebuilt git dependency test");
        return;
    };
    if !is_git_in_path() || !is_ar_in_path() {
        eprintln!("git or ar unavailable; skipping prebuilt git dependency test");
        return;
    }

    let root = unique_sandbox_dir("git_prebuilt_dep");
    let home = root.join("home");
    let origin = root.join("origin");
    let app = root.join("app");
    write_prebuilt_library(
        &origin,
        compiler,
        "gitprebuilt",
        "int git_prebuilt_answer(void);\n",
        "int git_prebuilt_answer(void) { return 9; }\n",
    );
    std::fs::create_dir_all(app.join("src")).unwrap();
    for args in [
        vec!["init"],
        vec!["add", "."],
        vec![
            "-c",
            "user.email=dcr@example.test",
            "-c",
            "user.name=DCR",
            "commit",
            "-m",
            "init",
        ],
        vec!["tag", "v0.1.0"],
    ] {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(&origin)
            .status()
            .unwrap();
        assert!(status.success());
    }
    let origin_url = origin.to_string_lossy().to_string();
    std::fs::write(
        app.join("dcr.toml"),
        format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[build]\nlanguage = \"c\"\ncompiler = \"clang\"\nkind = \"bin\"\n\n[dependencies]\ngitprebuilt = {{ git = \"{origin_url}\", tag = \"v0.1.0\", include = [\"include\"], lib = [\"lib\"], libs = [\"gitprebuilt\"] }}\n"),
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.c"),
        "#include \"gitprebuilt.h\"\nint main(void) { return git_prebuilt_answer() == 9 ? 0 : 1; }\n",
    )
    .unwrap();

    let home_s = home.to_string_lossy().to_string();
    let out = run_dcr_env(
        &["build"],
        &app,
        &[("DCR_COMPILER", compiler), ("HOME", home_s.as_str())],
    );
    assert!(
        out.status.success(),
        "prebuilt git dependency build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lock = std::fs::read_to_string(app.join("dcr.lock")).unwrap();
    assert!(lock.contains("git+"));
    assert!(!lock.contains("checksum = \"\""));
}

fn write_prebuilt_library(
    root: &std::path::Path,
    compiler: &str,
    name: &str,
    header: &str,
    source: &str,
) {
    let include_dir = root.join("include");
    let lib_dir = root.join("lib");
    let src_dir = root.join("src");
    std::fs::create_dir_all(&include_dir).unwrap();
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(include_dir.join(format!("{name}.h")), header).unwrap();
    let source_path = src_dir.join(format!("{name}.c"));
    let object_path = lib_dir.join(format!("{name}.o"));
    let archive_path = lib_dir.join(format!("lib{name}.a"));
    std::fs::write(&source_path, source).unwrap();
    let status = std::process::Command::new(compiler)
        .args(["-c"])
        .arg(&source_path)
        .args(["-o"])
        .arg(&object_path)
        .status()
        .unwrap();
    assert!(status.success(), "failed to compile prebuilt library");
    let status = std::process::Command::new("ar")
        .args(["rcs"])
        .arg(&archive_path)
        .arg(&object_path)
        .status()
        .unwrap();
    assert!(status.success(), "failed to archive prebuilt library");
}

fn is_ar_in_path() -> bool {
    std::process::Command::new("ar")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
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
