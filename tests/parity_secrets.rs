// tests/parity_secrets.rs — Parity test: Rust binary vs golden oracle pins
//
// Invocation contract (MANIFEST.md §2): cwd = repo root (fixture root), no args.
// Binary args: ["check", "secrets"]
// Pin is oracle: test RED → fix src/checks/secrets.rs, NEVER fix pin/fixture.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Copy a directory tree recursively from src to dest.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

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

fn run_parity_test(fixture_name: &str) {
    let pin_key = format!("secrets--{}", fixture_name);
    let fixture_src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/golden/fixtures/{}", fixture_name));

    // Task 3 step 1: tempdir → copy fixture tree
    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    copy_dir_all(&fixture_src, tmp.path()).expect("fixture copy failed");

    // Task 3 step 2: run binary with cwd = temp dir
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["check", "secrets"]);
    cmd.current_dir(tmp.path());

    let output = cmd.output().expect("binary execution failed");

    // Task 3 step 3: compare exit code (from pin, not hardcoded)
    let exp_code = expected_exit_code(&pin_key);
    let actual_code = output.status.code().unwrap_or(-1);
    assert_eq!(
        actual_code,
        exp_code,
        "exit code mismatch for {}: expected {} got {}",
        pin_key, exp_code, actual_code
    );

    // Task 3 step 3: compare stdout BYTE-EXACT with pin
    let exp_stdout = expected_stdout(&pin_key);
    let actual_stdout = output.stdout.clone();
    assert_eq!(
        actual_stdout,
        exp_stdout,
        "stdout mismatch for {}:\nexpected (len={}):\n{}\nactual (len={}):\n{}",
        pin_key,
        exp_stdout.len(),
        String::from_utf8_lossy(&exp_stdout),
        actual_stdout.len(),
        String::from_utf8_lossy(&actual_stdout)
    );

    // Task 3 step 3: compare stderr (empty if no pin file)
    let exp_stderr = expected_stderr(&pin_key);
    assert_eq!(
        output.stderr,
        exp_stderr,
        "stderr mismatch for {}",
        pin_key
    );
}

#[test]
fn parity_secrets_dirty() {
    run_parity_test("dirty");
}

#[test]
fn parity_secrets_clean() {
    run_parity_test("clean");
}
