mod common;
use common::*;

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
fn add_preserves_run_and_untyped_keys() {
    // Regression: `dcr add` used to drop the [run] section and non-whitelisted
    // [build] keys (filename/extension/include) when re-serializing dcr.toml.
    let dir = unique_sandbox_dir("preserve_add");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    let updated = toml.replace(
        "[build]",
        "[build]\nfilename = \"KERNEL\"\nextension = \"ELF\"\ninclude = [\"src/include\"]",
    ) + "\n[run]\ncmd = \"qemu-system-aarch64 -kernel KERNEL\"\n";
    std::fs::write(&toml_path, updated).expect("failed to write dcr.toml");

    let out = run_dcr(&["add", "zlib", "path:../zlib"], &dir);
    assert!(out.status.success(), "dcr add should succeed");

    let saved = std::fs::read_to_string(&toml_path).expect("failed to read dcr.toml");
    assert!(
        saved.contains("filename = \"KERNEL\""),
        "build.filename lost:\n{saved}"
    );
    assert!(
        saved.contains("extension = \"ELF\""),
        "build.extension lost:\n{saved}"
    );
    assert!(
        saved.contains("include = [\"src/include\"]"),
        "build.include lost:\n{saved}"
    );
    assert!(
        saved.contains("[run]") && saved.contains("qemu-system-aarch64"),
        "[run] section lost:\n{saved}"
    );
    assert!(
        saved.contains("zlib = { path = \"../zlib\" }"),
        "dependency not added as inline table:\n{saved}"
    );
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
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("run") || combined.contains("Running"),
        "dcr run should start"
    );

    std::fs::write(
        dir.join("src/main.c"),
        r#"#include <stdio.h>
int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        printf("ARG:%s\n", argv[i]);
    }
    return 0;
}
"#,
    )
    .expect("rewrite main.c");
    let out = run_dcr_env(&["run", "--force", "--", "--test_help", "foo"], &dir, &envs);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "dcr run -- <args> should succeed:\n{combined}"
    );
    assert!(
        combined.contains("ARG:--test_help") && combined.contains("ARG:foo"),
        "binary should receive args after `--`:\n{combined}"
    );

    let out = run_dcr_env(&["clean", "--release"], &dir, &envs);
    assert!(out.status.success(), "dcr clean --release should succeed");
    let release_dir = host_profile_dir(&dir, "release");
    let debug_dir = host_profile_dir(&dir, "debug");
    assert!(
        !release_dir.exists(),
        "release profile dir should be removed: {}",
        release_dir.display()
    );
    assert!(
        debug_dir.is_dir(),
        "debug profile dir should remain: {}",
        debug_dir.display()
    );

    let out = run_dcr_env(&["clean"], &dir, &envs);
    assert!(out.status.success(), "dcr clean should succeed");
    assert!(!dir.join("target").exists(), "target should be removed");
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
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
        eprintln!("skipping linux-target path assert: build failed (no suitable toolchain)");
        return;
    }

    let out_dir = dir
        .join("target")
        .join("x86_64-unknown-linux-gnu")
        .join("debug");
    assert_artifact_in(
        &out_dir,
        &project_name,
        "build.target=linux should place artifact under target/x86_64-unknown-linux-gnu/debug",
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

    assert_artifact_in(
        &dir.join("_BUILD"),
        &project_name,
        "artifact should be under custom out_dir _BUILD",
    );

    let default_path = default_artifact_path(&dir, &project_name);
    assert!(
        !default_path.exists(),
        "artifact should NOT be at default path when out_dir is set"
    );
}

#[test]
fn flat_bin_nasm_build() {
    let nasm_ok = std::process::Command::new("nasm")
        .arg("-v")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !nasm_ok {
        eprintln!("nasm not found; skipping flat-bin test");
        return;
    }

    let dir = unique_sandbox_dir("flat_bin");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    std::fs::write(
        dir.join("dcr.toml"),
        r#"[package]
name = "flatdemo"
version = "0.1.0"

[build]
language = "asm"
compiler = "nasm"
kind = "flat-bin"
extension = "bin"
roots = ["src"]
"#,
    )
    .expect("write dcr.toml");

    std::fs::create_dir_all(dir.join("src")).expect("src");
    std::fs::write(
        dir.join("src/boot.asm"),
        "bits 16\norg 0x7c00\ncli\nhlt\ntimes 510-($-$$) db 0\ndw 0xaa55\n",
    )
    .expect("write boot.asm");

    let out = run_dcr(&["build"], &dir);
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(out.status.success(), "flat-bin nasm build should succeed");

    let boot = host_profile_dir(&dir, "debug").join("boot.bin");
    assert!(
        boot.is_file(),
        "expected boot.bin under host profile dir: {}",
        boot.display()
    );
}

#[test]
fn package_build_target_used_without_cli_flag() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping package target test");
        return;
    };

    let dir = unique_sandbox_dir("pkg_target");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success());

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).unwrap();
    let project_name = parse_project_name(&toml);
    let updated = toml.replace("[build]", "[build]\ntarget = \"x86_64-unknown-linux-gnu\"");
    std::fs::write(&toml_path, updated).unwrap();

    let envs = [("DCR_COMPILER", compiler)];
    let out = run_dcr_env(&["build"], &dir, &envs);
    if !out.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
        eprintln!("skipping package-target path assert: build failed (no suitable toolchain)");
        return;
    }

    let out_dir = dir
        .join("target")
        .join("x86_64-unknown-linux-gnu")
        .join("debug");
    assert_artifact_in(
        &out_dir,
        &project_name,
        "artifact should be under package build.target path",
    );
}

#[test]
fn host_target_section_ldflags_apply_without_cli_target() {
    let Some(compiler) = available_compiler() else {
        eprintln!("no compiler found; skipping host target ldflags test");
        return;
    };

    let dir = unique_sandbox_dir("host_ldflags");
    let out = run_dcr(&["init"], &dir);
    assert!(out.status.success(), "dcr init should succeed");

    let host = if cfg!(target_os = "linux") {
        let arch = std::env::consts::ARCH;
        let env = if cfg!(target_env = "musl") {
            "musl"
        } else {
            "gnu"
        };
        format!("{arch}-unknown-linux-{env}")
    } else if cfg!(target_os = "macos") {
        format!("{}-apple-darwin", std::env::consts::ARCH)
    } else if cfg!(target_os = "windows") {
        let env = if cfg!(target_env = "gnu") {
            "gnu"
        } else {
            "msvc"
        };
        format!("{}-pc-windows-{env}", std::env::consts::ARCH)
    } else {
        eprintln!("unsupported host for host_target_section test; skipping");
        return;
    };

    let toml_path = dir.join("dcr.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("read dcr.toml");
    let updated = format!(
        "{toml}\n[build.{host}]\nldflags = [\"-Wl,-rpath,/nonexistent-dcr-host-ldflags\"]\n"
    );
    std::fs::write(&toml_path, updated).expect("write dcr.toml");

    let envs = [("DCR_COMPILER", compiler), ("DCR_DEBUG", "1")];
    let out = run_dcr_env(&["build", "--force"], &dir, &envs);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("-Wl,-rpath,/nonexistent-dcr-host-ldflags")
            || combined.contains("rpath,/nonexistent-dcr-host-ldflags"),
        "host target section ldflags must be applied on native build:\n{combined}"
    );
}
