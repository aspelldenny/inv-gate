// tests/parity_schema.rs — Parity test: Rust binary vs golden oracle pins for schema-safety
//
// Invocation contract (MANIFEST.md §2): cwd = repo root (2-commit git fixture), no args.
// Binary args: ["check", "schema"]
// Pin is oracle: test RED → fix src/checks/schema.rs or env-setup, NEVER fix pin/fixture.
//
// Env reconstruction per tests/golden/repin.sh:34-88 (anchor #14 ✅):
//   - git init + hermetic config (user.name, email, gpgsign false)
//   - fixed dates GIT_AUTHOR_DATE + GIT_COMMITTER_DATE = "2026-01-01T00:00:00 +0000"
//   - commit 1: schema.before.prisma as baseline
//   - commit 2: schema.after.prisma as change
//   - SKIP: remote-inject (O1.4 — schema check does not read remote)
// env_remove("ALLOW_DATA_LOSS") on every Command (anchor #16 ✅, Task 5 Lưu ý 1).
//
// Stderr: expected empty — git stderr suppressed in schema.rs (golden: 2>/dev/null).
// Note: stdout byte-exact vs pin; all 4 raw diff lines in dirty pin (anchor #13 ✅).

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

/// Load expected stdout from pin file (byte-exact).
fn expected_stdout(key: &str) -> Vec<u8> {
    let pin_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/golden/pins/{}.stdout.txt", key));
    fs::read(&pin_path).expect(&format!("pin file {}.stdout.txt must exist", key))
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

/// Build the fixture repo for schema parity — repin.sh:34-88 pattern, skip remote-inject (O1.4).
/// - Copy fixture files (excl schema files)
/// - Git init + hermetic config
/// - 2-commit setup (schema.before → schema.after)
fn build_fixture_repo_schema(branch: &str, tmp: &Path) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_src = manifest_dir.join(format!("tests/golden/fixtures/{}", branch));

    // Copy fixture files (exclude prisma schema files — handled via 2-commit)
    // For schema check, we only need the 2-commit git structure
    // (repin.sh:42-44 copies all except schema.before/after.prisma)
    copy_dir_excluding(&fixture_src, tmp, &["schema.before.prisma", "schema.after.prisma"])
        .expect("fixture copy failed");

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

    // NOTE: skip remote-inject (O1.4 Tầng 2 — schema check does not read .git/config remote)
}

/// Copy directory tree, skipping files whose basenames appear in any subdirectory matching exclude_names.
fn copy_dir_excluding(src: &Path, dst: &Path, exclude_names: &[&str]) -> std::io::Result<()> {
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
            copy_dir_excluding(&entry.path(), &dst_path, exclude_names)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn run_parity_test(fixture_name: &str) {
    let pin_key = format!("schema--{}", fixture_name);

    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    build_fixture_repo_schema(fixture_name, tmp.path());

    // Run binary with cwd = tmp (MANIFEST §2), env_remove("ALLOW_DATA_LOSS") — anchor #16 ✅
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["check", "schema"]);
    cmd.current_dir(tmp.path());
    cmd.env_remove("ALLOW_DATA_LOSS"); // hermetic — Task 5 Lưu ý 1

    let output = cmd.output().expect("binary execution failed");

    // Compare exit code from pin
    let exp_code = expected_exit_code(&pin_key);
    let actual_code = output.status.code().unwrap_or(-1);
    assert_eq!(
        actual_code, exp_code,
        "exit code mismatch for {}: expected {} got {}",
        pin_key, exp_code, actual_code
    );

    // Compare stdout BYTE-EXACT with pin (anchor #13 ✅ — banner + 4 raw lines + instruction block)
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

    // Compare stderr — must be empty (git stderr suppressed in schema.rs via Stdio::null())
    assert_eq!(
        output.stderr, b"",
        "stderr must be empty for {}, got: {}",
        pin_key,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn parity_schema_dirty() {
    run_parity_test("dirty");
}

#[test]
fn parity_schema_clean() {
    run_parity_test("clean");
}
