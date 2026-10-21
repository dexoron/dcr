mod common;
use common::*;

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
    assert!(
        stderr.contains("test result: 1 passed; 0 failed; 0 skipped; 1 total"),
        "test result summary line missing\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("PASS") || stderr.contains("Passed"),
        "test pass output missing\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}
