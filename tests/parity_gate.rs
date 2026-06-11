// tests/parity_gate.rs — Parity test: Rust binary vs golden oracle pins for gate --all
//
// Invocation contract (MANIFEST.md §2): cwd = repo root (fixture root), args = ["gate", "--all"].
// Parity contract: stdout + stderr BYTE-EXACT vs golden pins gate--{dirty,clean}.
// Usage-error: ["gate", "--no-such-flag"] → exit 2.
// Pin is oracle: test RED → fix src/gate.rs or harness, NEVER fix pin/fixture/repin.sh.
//
// Harness mirrors repin.sh build_fixture_repo() + section gate of repin.sh (lines 134-162).
// Same union fixture as parity_runtime.rs (reuse build_fixture_repo / copy_dir_all_excluding).

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers (mirrors parity_runtime.rs — precedent P004)
// ─────────────────────────────────────────────────────────────────────────────

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

/// Build the fixture repo in `tmp_dir` per repin.sh build_fixture_repo().
/// Mirror of parity_runtime.rs build_fixture_repo() — repin.sh lines 34-88.
/// For gate parity: union fixture (same as other checks — fixture covers all checks).
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

// ─────────────────────────────────────────────────────────────────────────────
// Parity tests — gate--dirty / gate--clean
// ─────────────────────────────────────────────────────────────────────────────

fn run_parity_test(fixture_name: &str) {
    let pin_key = format!("gate--{}", fixture_name);

    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    build_fixture_repo(fixture_name, tmp.path());

    // Run binary with cwd = tmp (MANIFEST §2), env hermetic (Luật chơi 8)
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["gate", "--all"]);
    cmd.current_dir(tmp.path());
    cmd.env_remove("ALLOW_DATA_LOSS"); // hermetic — Luật chơi 8

    let output = cmd.output().expect("binary execution failed");

    // Compare exit code from pin (golden:206-209)
    let exp_code = expected_exit_code(&pin_key);
    let actual_code = output.status.code().unwrap_or(-1);
    assert_eq!(
        actual_code, exp_code,
        "exit code mismatch for {}: expected {} got {}",
        pin_key, exp_code, actual_code
    );

    // Compare stdout BYTE-EXACT with pin (Luật chơi 4)
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

    // Compare stderr BYTE-EXACT with pin (MANIFEST §4 rule 7 — port check emits WARN to stderr)
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
fn parity_gate_dirty() {
    run_parity_test("dirty");
}

#[test]
fn parity_gate_clean() {
    run_parity_test("clean");
}

// ─────────────────────────────────────────────────────────────────────────────
// Usage-error test — gate without --all / gate with unknown flag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gate_usage_error_unknown_flag() {
    // golden/security-gate.sh:14 — unknown flag → exit 2 (via clap, text deviation OK per anchor #9)
    let exp_code = expected_exit_code("gate--usage-error");
    assert_eq!(exp_code, 2, "gate--usage-error pin must be exit 2");

    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["gate", "--no-such-flag"]);
    let output = cmd.output().expect("binary execution failed");
    let actual_code = output.status.code().unwrap_or(-1);
    // anchor #9: exit-code-only assertion (text deviation OK — not asserting stdout/stderr text)
    assert_eq!(actual_code, 2, "gate --no-such-flag must exit 2");
}

#[test]
fn gate_bare_exits_2() {
    // `gate` without `--all` → clap usage error exit 2 (Task 1 Lưu ý 2)
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["gate"]);
    let output = cmd.output().expect("binary execution failed");
    let actual_code = output.status.code().unwrap_or(-1);
    assert_eq!(actual_code, 2, "gate (bare, no --all) must exit 2");
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit probes — Task 3 (a)-(e) — synthetic in-code, F07
// ─────────────────────────────────────────────────────────────────────────────

/// (a) Accumulator: one check fail → sections AFTER still run, gate exit 1.
/// Probe: build dirty fixture (port FAIL, secrets FAIL, runtime FAIL) — all 3 checks run.
/// Evidence: all 3 INV names in output + exit 1.
#[test]
fn gate_accumulator_runs_all_sections_on_fail() {
    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    build_fixture_repo("dirty", tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["gate", "--all"]);
    cmd.current_dir(tmp.path());
    cmd.env_remove("ALLOW_DATA_LOSS");

    let output = cmd.output().expect("binary execution failed");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // golden/security-gate.sh:206-209 — exit 1 if FAIL > 0
    assert_eq!(code, 1, "dirty fixture must exit 1 (accumulator FAIL>0)");

    // All 3 failing checks must appear in output (accumulator — not early-exit)
    assert!(stdout.contains("--- INV-001:"), "INV-001 section must be present");
    assert!(stdout.contains("--- INV-009:"), "INV-009 section must be present");
    assert!(stdout.contains("--- INV-010:"), "INV-010 section must be present");

    // INV-009 and INV-010 appear AFTER INV-001 — prove post-fail sections ran
    let pos_001 = stdout.find("--- INV-001:").unwrap();
    let pos_009 = stdout.find("--- INV-009:").unwrap();
    let pos_010 = stdout.find("--- INV-010:").unwrap();
    assert!(pos_001 < pos_009, "INV-009 must come after INV-001 (order matters)");
    assert!(pos_009 < pos_010, "INV-010 must come after INV-009 (order matters)");
}

/// (b) All clean → exit 0 + all section lines present + summary shows 0 failed.
#[test]
fn gate_all_clean_exit_0_full_sections() {
    let tmp = tempfile::tempdir().expect("tempdir creation failed");
    build_fixture_repo("clean", tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(["gate", "--all"]);
    cmd.current_dir(tmp.path());
    cmd.env_remove("ALLOW_DATA_LOSS");

    let output = cmd.output().expect("binary execution failed");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(code, 0, "clean fixture must exit 0");

    // All 9 section headers present (INV-007 skipped — mechanical-only)
    for inv in &["INV-001", "INV-002", "INV-003", "INV-004", "INV-005", "INV-006", "INV-008", "INV-009", "INV-010"] {
        assert!(stdout.contains(&format!("--- {}:", inv)), "section {} must be present", inv);
    }
    // INV-007 must NOT appear (skip in mechanical-only — golden:169-174)
    assert!(!stdout.contains("INV-007"), "INV-007 must be absent in mechanical-only mode");

    // Summary: 0 failed (golden:205)
    assert!(stdout.contains("0 failed"), "summary must show 0 failed");
}

/// (c) `gate` bare (missing --all) → exit 2 (already tested in gate_bare_exits_2)
/// Verified by gate_bare_exits_2 above.

/// (d) Inline checks — probe each inline check mechanism (INV-002..006 + INV-008)
/// Mechanism-based probes using synthetic tempdir setups.

/// Extract the content of a named section (from `--- INV-XXX:` to start of next section or end).
fn extract_section<'a>(stdout: &'a str, inv: &str) -> &'a str {
    let marker = format!("--- {}:", inv);
    let start = stdout.find(marker.as_str()).unwrap_or_else(|| panic!("section {} not found", inv));
    // Next section starts at the next occurrence of "--- INV-" after current
    let rest = &stdout[start + marker.len()..];
    let end = rest.find("\n--- INV-").map(|p| start + marker.len() + p + 1).unwrap_or(stdout.len());
    &stdout[start..end]
}

/// INV-002: docker-compose.yml with :latest tag → FAIL
#[test]
fn inv_002_latest_tag_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    // Write docker-compose.yml with :latest (except umami/portainer)
    fs::write(tmp.path().join("docker-compose.yml"), "services:\n  app:\n    image: myapp:latest\n").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-002");
    assert!(block.contains("  FAIL"), "INV-002 must FAIL for :latest tag");
}

/// INV-002: umami exception not flagged → PASS
#[test]
fn inv_002_umami_exception_pass() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    fs::write(tmp.path().join("docker-compose.yml"),
        "services:\n  umami:\n    image: ghcr.io/umami-software/umami:postgresql-latest\n").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-002");
    assert!(block.contains("  PASS"), "INV-002: umami exception must PASS");
}

/// INV-003: .env.example with real value → FAIL
#[test]
fn inv_003_real_value_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    fs::write(tmp.path().join("docker-compose.yml"), "services:\n").unwrap();
    fs::write(tmp.path().join(".env.example"), "DATABASE_URL=realsecretvalue\n").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-003");
    assert!(block.contains("  FAIL"), "INV-003 must FAIL for real value in .env.example");
}

/// INV-004: missing .env.staging in .gitignore → FAIL
#[test]
fn inv_004_missing_gitignore_entry_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    fs::write(tmp.path().join("docker-compose.yml"), "services:\n").unwrap();
    // Only 3 of 4 patterns — missing .env.staging
    fs::write(tmp.path().join(".gitignore"), ".env.production\n.env.backup\n.env.local\n").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-004");
    assert!(block.contains("  FAIL"), "INV-004 must FAIL when .env.staging missing from .gitignore");
    assert!(block.contains(".env.staging"), "output must name the missing file");
}

/// INV-005: no beforeSend/beforeBreadcrumb → FAIL
#[test]
fn inv_005_no_sentry_scrubber_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    fs::write(tmp.path().join("docker-compose.yml"), "services:\n").unwrap();
    // Override sentry.ts without beforeSend (setup_minimal_git_repo wrote one with beforeSend)
    fs::write(tmp.path().join("src/lib/sentry.ts"), "export const s = {};").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-005");
    assert!(block.contains("  FAIL"), "INV-005 must FAIL without beforeSend/beforeBreadcrumb");
}

/// INV-006: astro-service/app.py with CORS wildcard → FAIL
#[test]
fn inv_006_cors_wildcard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    fs::write(tmp.path().join("docker-compose.yml"), "services:\n").unwrap();
    fs::create_dir_all(tmp.path().join("astro-service")).unwrap();
    fs::write(tmp.path().join("astro-service/app.py"), "app = CORS(app, origins='*')\n").unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-006");
    assert!(block.contains("  FAIL"), "INV-006 must FAIL for CORS wildcard");
}

/// INV-008: internal service uses ports: → FAIL
#[test]
fn inv_008_internal_service_ports_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    // nextjs is in internal list — and has ports: directive
    let compose = "services:\n  nextjs:\n    image: nextjs:1.0\n    ports:\n      - \"3000:3000\"\n";
    fs::write(tmp.path().join("docker-compose.yml"), compose).unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-008");
    assert!(block.contains("  FAIL"), "INV-008 must FAIL when nextjs has ports:");
    assert!(block.contains("nextjs"), "output must name the violating service");
}

/// INV-008: non-internal service uses ports: → PASS (not in watch list)
#[test]
fn inv_008_external_service_ports_pass() {
    let tmp = tempfile::tempdir().expect("tempdir");
    setup_minimal_git_repo(tmp.path());
    let compose = "services:\n  nginx:\n    image: nginx:1.25.0\n    ports:\n      - \"443:443\"\n";
    fs::write(tmp.path().join("docker-compose.yml"), compose).unwrap();
    git_commit_all(tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let block = extract_section(&stdout, "INV-008");
    assert!(block.contains("  PASS"), "INV-008 must PASS for non-internal service nginx");
}

/// (e) Summary conditional: FAIL>0 → has "Failed invariants:" line; FAIL=0 → does NOT.
/// golden/security-gate.sh:206-208
#[test]
fn summary_conditional_failed_invs_present_when_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("dirty", tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Failed invariants:"), "summary must have 'Failed invariants:' when FAIL>0");
    // All 3 failing INVs in the list
    assert!(stdout.contains("INV-001"), "INV-001 in failed list");
    assert!(stdout.contains("INV-009"), "INV-009 in failed list");
    assert!(stdout.contains("INV-010"), "INV-010 in failed list");
}

#[test]
fn summary_conditional_no_failed_invs_when_clean() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("clean", tmp.path());

    let mut cmd = Command::cargo_bin("inv-gate").unwrap();
    cmd.args(["gate", "--all"]).current_dir(tmp.path()).env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // golden:206 — "Failed invariants:" line ONLY printed when FAIL>0
    assert!(!stdout.contains("Failed invariants:"), "clean run must NOT have 'Failed invariants:' line");
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers for unit probes
// ─────────────────────────────────────────────────────────────────────────────

/// Set up a minimal git repo with required .gitignore and sentry.ts for base probes.
fn setup_minimal_git_repo(path: &Path) {
    let git_cmds: &[&[&str]] = &[
        &["init", "-q", "."],
        &["config", "user.name", "test"],
        &["config", "user.email", "test@test.com"],
        &["config", "commit.gpgsign", "false"],
    ];
    for args in git_cmds {
        process::Command::new("git").args(*args).current_dir(path).output().unwrap();
    }
    // Default .gitignore satisfies INV-004
    fs::write(path.join(".gitignore"), ".env.production\n.env.staging\n.env.backup\n.env.local\n").unwrap();
    // Default sentry.ts satisfies INV-005
    fs::create_dir_all(path.join("src/lib")).unwrap();
    fs::write(path.join("src/lib/sentry.ts"), "export const s = { beforeSend: () => {} };").unwrap();
}

/// Commit all files in a git repo (for unit probes that need a committed state).
fn git_commit_all(path: &Path) {
    process::Command::new("git").args(["add", "-A"]).current_dir(path).output().unwrap();
    process::Command::new("git").args(["commit", "-q", "-m", "probe commit"])
        .current_dir(path)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
        .output().unwrap();
}
