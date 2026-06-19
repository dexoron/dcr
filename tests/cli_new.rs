mod common;
use common::*;

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
fn new_vcs_options_work() {
    // 1. Test --vcs none
    let dir = unique_sandbox_dir("new_vcs_none");
    let out = run_dcr(&["new", "hello_none", "--vcs", "none"], &dir);
    assert!(out.status.success(), "dcr new --vcs none should succeed");
    let project_dir = dir.join("hello_none");
    assert!(project_dir.is_dir());
    assert!(!project_dir.join(".git").exists());
    assert!(!project_dir.join(".gitignore").exists());

    // 2. Test --vcs git
    if is_git_in_path() {
        let dir = unique_sandbox_dir("new_vcs_git");
        let out = run_dcr(&["new", "hello_git", "--vcs", "git"], &dir);
        assert!(out.status.success(), "dcr new --vcs git should succeed");
        let project_dir = dir.join("hello_git");
        assert!(project_dir.is_dir());
        assert!(project_dir.join(".git").is_dir());
        assert!(project_dir.join(".gitignore").is_file());
        let gitignore = std::fs::read_to_string(project_dir.join(".gitignore")).unwrap();
        assert!(gitignore.contains("/target"));
    }
}

#[test]
fn init_vcs_options_work() {
    // 1. Test --vcs none
    let dir = unique_sandbox_dir("init_vcs_none");
    let out = run_dcr(&["init", "--vcs", "none"], &dir);
    assert!(out.status.success(), "dcr init --vcs none should succeed");
    assert!(!dir.join(".git").exists());
    assert!(!dir.join(".gitignore").exists());

    // 2. Test --vcs git
    if is_git_in_path() {
        let dir = unique_sandbox_dir("init_vcs_git");
        let out = run_dcr(&["init", "--vcs", "git"], &dir);
        assert!(out.status.success(), "dcr init --vcs git should succeed");
        assert!(dir.join(".git").is_dir());
        assert!(dir.join(".gitignore").is_file());
        let gitignore = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
        assert!(gitignore.contains("/target"));
    }
}
