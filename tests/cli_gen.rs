mod common;
use common::*;
use std::fs;
use std::path::Path;

#[test]
fn gen_project_info_has_artifact_path() {
    let dir = unique_sandbox_dir("gen_pi");
    let out = run_dcr(&["new", "app", "--vcs", "none"], &dir);
    assert!(out.status.success());
    let project = dir.join("app");

    let out = run_dcr(&["gen", "project-info", "-q"], &project);
    assert!(
        out.status.success(),
        "project-info failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"artifact_path\""),
        "missing artifact_path: {stdout}"
    );
    assert!(stdout.contains("\"target_dir\""), "missing target_dir");
    assert!(
        stdout.contains("\"artifact_kind\""),
        "missing artifact_kind"
    );
    assert!(stdout.contains("\"source_roots\""), "missing source_roots");

    let root_canon = fs::canonicalize(&project).unwrap();
    assert!(
        stdout.contains(&root_canon.to_string_lossy().to_string()) || stdout.contains("app"),
        "root should be absolute-ish: {stdout}"
    );

    assert!(project.join(".dcr").join("build-info.json").is_file());
    assert!(project.join(".dcr").join("toolchain.json").is_file());
    assert!(project.join(".clangd").is_file());
    let clangd = fs::read_to_string(project.join(".clangd")).unwrap();
    assert!(clangd.contains("CompilationDatabase: .dcr"));
}

#[test]
fn gen_compile_commands_in_dcr_dir() {
    let dir = unique_sandbox_dir("gen_cc");
    let out = run_dcr(&["new", "app", "--vcs", "none"], &dir);
    assert!(out.status.success());
    let project = dir.join("app");

    let out = run_dcr(&["gen", "compile-commands", "-q"], &project);
    assert!(
        out.status.success(),
        "compile-commands failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !project.join("compile_commands.json").exists(),
        "compile_commands.json must not be in project root"
    );
    let cc = project.join(".dcr").join("compile_commands.json");
    assert!(cc.is_file(), "expected .dcr/compile_commands.json");
    let content = fs::read_to_string(&cc).unwrap();
    assert!(content.contains("\"directory\""), "missing directory field");
    assert!(content.contains("\"file\""), "missing file field");
    assert!(content.starts_with('['));

    assert!(project.join(".clangd").is_file());
    assert!(project.join(".dcr").join("ide").is_dir());
}

#[test]
fn gen_clangd_not_overwritten_if_custom() {
    let dir = unique_sandbox_dir("gen_clangd");
    let out = run_dcr(&["new", "app", "--vcs", "none"], &dir);
    assert!(out.status.success());
    let project = dir.join("app");

    let custom = "CompileFlags:\n  Add: [-Wall]\n";
    fs::write(project.join(".clangd"), custom).unwrap();

    let out = run_dcr(&["gen", "compile-commands", "-q"], &project);
    assert!(out.status.success());
    let after = fs::read_to_string(project.join(".clangd")).unwrap();
    assert_eq!(after, custom, "custom .clangd must not be overwritten");
}

#[test]
fn gen_vscode_launch_uses_resolver() {
    let dir = unique_sandbox_dir("gen_vscode");
    let out = run_dcr(&["new", "app", "--vcs", "none"], &dir);
    assert!(out.status.success());
    let project = dir.join("app");

    let out = run_dcr(&["gen", "vscode"], &project);
    assert!(
        out.status.success(),
        "gen vscode failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let launch = fs::read_to_string(project.join(".vscode").join("launch.json")).unwrap();
    assert!(launch.contains("\"program\""), "launch missing program");
    let normalized = launch.replace("\\\\", "/").replace('\\', "/");
    let expected_rel = host_profile_dir(&project, "debug")
        .join(bin_name("app"))
        .strip_prefix(&project)
        .unwrap_or_else(|_| Path::new("target"))
        .to_string_lossy()
        .replace('\\', "/");
    assert!(
        normalized.contains(&expected_rel),
        "program should use host profile layout ({expected_rel}): {launch}"
    );
    let settings = fs::read_to_string(project.join(".vscode").join("settings.json")).unwrap();
    assert!(
        settings.contains(".dcr") || settings.contains("compile-commands-dir"),
        "settings should point at .dcr: {settings}"
    );
}

#[test]
fn build_print_artifact_path() {
    if available_compiler().is_none() {
        return;
    }
    let dir = unique_sandbox_dir("build_print");
    let out = run_dcr(&["new", "app", "--vcs", "none"], &dir);
    assert!(out.status.success());
    let project = dir.join("app");

    let out = run_dcr(&["build", "--print-artifact-path"], &project);
    assert!(
        out.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let path_line = stdout
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .unwrap_or("");
    assert!(
        Path::new(path_line).is_absolute() || path_line.contains("app"),
        "expected artifact path on stdout, got: {stdout}"
    );
    assert!(
        project.join(".dcr").join("build-info.json").is_file(),
        "build-info.json should be written after build"
    );
}
