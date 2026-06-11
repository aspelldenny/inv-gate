// tests/parity_port.rs — Parity test: Rust binary vs golden oracle pins for INV-001
//
// Invocation contract (MANIFEST.md §1): cwd = fixture dir (static copy), no git needed.
// Binary args: ["check", "port"]
// Pin is oracle: test RED → fix src/checks/port.rs, NEVER fix pin/fixture.
//
// Fixtures: copy docker-compose.yml only (anchor #5 ✅ — 2 missing files fire WARN on stderr).
// STDERR: byte-exact vs port--{dirty,clean}.stderr.txt pins (2-line WARN each — NOT empty).
// P004 contract: stderr is pinned (unlike runtime/secrets which have empty stderr pins).

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;

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

/// Load expected stderr from pin file if it exists, otherwise empty.
/// Port pins HAVE stderr files (anchor #5 ✅) — this pattern from parity_runtime.rs.
fn expected_stderr(key: &str) -> Vec<u8> {
    let pin_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/golden/pins/{}.stderr.txt", key));
    match fs::read(&pin_path) {
        Ok(content) => content,
        Err(_) => Vec::new(),
    }
}

/// Run parity test for a single fixture (dirty or clean).
/// Copies only the fixture's docker-compose.yml into a temp dir (no git needed — static scan).
/// The 2 missing compose files (docker-compose.dev.yml, astro-service/docker-compose.yml)
/// are intentionally absent to fire WARN on stderr — matching the pins.
fn run_parity_test(fixture_name: &str) {
    let pin_key = format!("port--{}", fixture_name);

    let tmp = tempfile::tempdir().expect("tempdir creation failed");

    // Copy ONLY docker-compose.yml from fixture dir (anchor #5 ✅ — only 1 of 3 files present)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join(format!("tests/golden/fixtures/{}", fixture_name));
    let src_compose = fixture_dir.join("docker-compose.yml");
    let dst_compose = tmp.path().join("docker-compose.yml");
    fs::copy(&src_compose, &dst_compose)
        .expect("docker-compose.yml fixture copy failed");

    // Run binary: check port, cwd = temp dir (MANIFEST §1)
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["check", "port"]);
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

    // Compare stderr BYTE-EXACT with pin (port pins HAVE stderr — 2-line WARN each)
    // This is different from runtime/secrets which expect empty stderr.
    let exp_stderr = expected_stderr(&pin_key);
    assert_eq!(
        output.stderr, exp_stderr,
        "stderr mismatch for {}:\nexpected (len={}):\n{}\nactual (len={}):\n{}",
        pin_key,
        exp_stderr.len(),
        String::from_utf8_lossy(&exp_stderr),
        output.stderr.len(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn parity_port_dirty() {
    run_parity_test("dirty");
}

#[test]
fn parity_port_clean() {
    run_parity_test("clean");
}
