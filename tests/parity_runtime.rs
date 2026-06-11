// tests/parity_runtime.rs — Parity test: Rust binary vs golden oracle pins for INV-010
//
// Invocation contract (MANIFEST.md §2): cwd = repo root (fixture root), no args.
// Binary args: ["check", "runtime"]
// Pin is oracle: test RED → fix src/checks/runtime.rs or env-setup, NEVER fix pin/fixture.
//
// Env reconstruction per tests/golden/repin.sh:81-87 + :53 (anchor #11 ✅).
// 3 files scanned in clean fixture: .git/config + scripts/check-schema-safety.sh + docker-compose.yml

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process;

/// Load exit_codes.json and return the expected exit code for the given key.
fn expected_exit_code(key: &str) -> i32 {
    let pins_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/pins/exit_codes.json");
    let content = fs::read_to_string(&pins_path)
        .expect("exit_codes.json must exist (P001 shipped)");
    let json: Value = serde_json::from_str(&content)
        .expect("exit_codes.json must be valid JSON");
    json[key]
        .as_i64()
        .expect(&format!("exit_codes.json must have key '{}'", key)) as i32
}

/// Load expected stdout from pin file.
fn expected_stdout(key: &str) -> Vec<u8> {
    let pin_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/golden/pins/{}.stdout.txt", key));
    fs::read(&pin_path).expect(&format!("pin file {}.stdout.txt must exist", key))
}

/// Load expected stderr from pin file if it exists, otherwise empty.
fn expected_stderr(key: &str) -> Vec<u8> {
    let pin_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/golden/pins/{}.stderr.txt", key));
    match fs::read(&pin_path) {
        Ok(content) => content,
        Err(_) => Vec::new(),
    }
}

/// Run a git command in a given directory, panicking on error.
fn git_in(dir: &Path, args: &[&str]) {
    let status = process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
        .output()
        .expect("git command failed");
    if !status.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&status.stdout),
            String::from_utf8_lossy(&status.stderr)
        );
    }
}

/// Build the fixture repo in `tmp_dir` per repin.sh build_fixture_repo().
/// - Copy fixture files (excl prisma schema files handled via 2-commit)
/// - Copy golden scripts to scripts/ layout (repin.sh:49-53)
/// - Git init + hermetic config (repin.sh:57-65)
/// - 2-commit setup (repin.sh:67-77)
/// - INV-010 remote inject per `branch` (repin.sh:81-87)
fn build_fixture_repo(branch: &str, tmp: &Path) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_src = manifest_dir.join(format!("tests/golden/fixtures/{}", branch));
    let golden_dir = manifest_dir.join("golden");

    // Copy fixture files excluding schema files (handled in 2-commit flow)
    copy_dir_all_excluding(&fixture_src, tmp, &["schema.before.prisma", "schema.after.prisma"])
        .expect("fixture copy failed");

    // Copy golden scripts into tmp/scripts/ layout (repin.sh:49-53)
    let tmp_scripts = tmp.join("scripts");
    fs::create_dir_all(&tmp_scripts).expect("scripts dir creation failed");
    for script in &[
        "check-hardcoded-secrets.py",
        "check-port-bind.py",
        "check-runtime-secrets.py",
        "check-schema-safety.sh",
    ] {
        fs::copy(golden_dir.join(script), tmp_scripts.join(script))
            .expect(&format!("copy golden script {} failed", script));
    }
    fs::copy(golden_dir.join("security-gate.sh"), tmp.join("security-gate.sh"))
        .expect("copy security-gate.sh failed");

    // Init hermetic git repo (repin.sh:57-60)
    git_in(tmp, &["init", "-q", "."]);
    git_in(tmp, &["config", "user.name", "P001 Pin Harness"]);
    git_in(tmp, &["config", "user.email", "pin@inv-gate.local"]);
    git_in(tmp, &["config", "commit.gpgsign", "false"]);

    // Commit 1: schema.before.prisma as baseline (repin.sh:67-71)
    let prisma_dir = tmp.join("prisma");
    fs::create_dir_all(&prisma_dir).expect("prisma dir creation");
    fs::copy(
        fixture_src.join("prisma/schema.before.prisma"),
        prisma_dir.join("schema.prisma"),
    ).expect("copy schema.before failed");
    git_in(tmp, &["add", "-A"]);
    git_in(tmp, &["commit", "-q", "-m", "P001 fixture baseline"]);

    // Commit 2: schema.after.prisma (repin.sh:73-77)
    fs::copy(
        fixture_src.join("prisma/schema.after.prisma"),
        prisma_dir.join("schema.prisma"),
    ).expect("copy schema.after failed");
    git_in(tmp, &["add", "prisma/schema.prisma"]);
    git_in(tmp, &["commit", "-q", "-m", "P001 fixture schema change"]);

    // INV-010 remote inject (repin.sh:81-87)
    if branch == "dirty" {
        // Token: ghp_ + 36 alphanum = 40 chars total (repin.sh:83, O2.1 verified)
        git_in(tmp, &[
            "remote", "add", "origin",
            "https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git",
        ]);
    } else {
        git_in(tmp, &[
            "remote", "add", "origin",
            "https://github.com/example/fixture.git",
        ]);
    }
}

/// Copy directory tree, skipping files whose names match any entry in `exclude_names`.
fn copy_dir_all_excluding(src: &Path, dst: &Path, exclude_names: &[&str]) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let fname = entry.file_name();
        let fname_str = fname.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == fname_str.as_ref()) {
            continue;
        }
        let dst_path = dst.join(&fname);
        if ty.is_dir() {
            copy_dir_all_excluding(&entry.path(), &dst_path, exclude_names)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn run_parity_test(fixture_name: &str) {
    let pin_key = format!("runtime--{}", fixture_name);

    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    build_fixture_repo(fixture_name, tmp.path());

    // Run binary with cwd = tmp (MANIFEST §2)
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["check", "runtime"]);
    cmd.current_dir(tmp.path());

    let output = cmd.output().expect("binary execution failed");

    // Compare exit code from pin
    let exp_code = expected_exit_code(&pin_key);
    let actual_code = output.status.code().unwrap_or(-1);
    assert_eq!(
        actual_code, exp_code,
        "exit code mismatch for {}: expected {} got {}",
        pin_key, exp_code, actual_code
    );

    // Compare stdout BYTE-EXACT with pin
    let exp_stdout = expected_stdout(&pin_key);
    let actual_stdout = output.stdout.clone();
    assert_eq!(
        actual_stdout, exp_stdout,
        "stdout mismatch for {}:\nexpected (len={}):\n{}\nactual (len={}):\n{}",
        pin_key,
        exp_stdout.len(),
        String::from_utf8_lossy(&exp_stdout),
        actual_stdout.len(),
        String::from_utf8_lossy(&actual_stdout)
    );

    // Compare stderr (must be empty — golden emits no stderr for runtime check)
    let exp_stderr = expected_stderr(&pin_key);
    assert_eq!(
        output.stderr, exp_stderr,
        "stderr mismatch for {}: expected empty, got: {}",
        pin_key,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn parity_runtime_dirty() {
    run_parity_test("dirty");
}

#[test]
fn parity_runtime_clean() {
    run_parity_test("clean");
}
