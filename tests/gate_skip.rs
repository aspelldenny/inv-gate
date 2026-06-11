// tests/gate_skip.rs — P007: probes for --skip-absent behavior (Task 4)
//
// Fixtures: synthetic in-code (F07 — no new files in tests/golden/).
// Hermetic: env_remove("ALLOW_DATA_LOSS"), LF, fixed dates.
// Probes: (a) SKIP + exit 0; (b) default no flag; (c) fail-closed per-INV; (c2) guard kép;
//         (d) flag no-op when files present; (e) INV-004 not skippable; (f) skip+fail together.
// Rule: test red → fix gate.rs/serve.rs/harness, NEVER fix pins (Luật chơi 2).

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use std::process;

// ─────────────────────────────────────────────────────────────────────────────
// Harness helpers (mirror parity_gate.rs — Tầng 2)
// ─────────────────────────────────────────────────────────────────────────────

fn git_in(dir: &Path, args: &[&str]) {
    let out = process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
        .output()
        .expect("git command failed");
    if !out.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn git_commit_all(path: &Path) {
    process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["commit", "-q", "-m", "probe commit"])
        .current_dir(path)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
        .output()
        .unwrap();
}

/// Build a minimal hermetic git repo:
/// - .gitignore with all 4 INV-004 entries (satisfied)
/// - NO sentry.ts, NO sentry.*.config.* (INV-005 absent prerequisites)
/// - NO docker-compose.yml (INV-008 absent prerequisite)
/// - clean source tree (no secrets in src/)
fn build_minimal_repo(tmp: &Path) {
    git_in(tmp, &["init", "-q", "."]);
    git_in(tmp, &["config", "user.name", "P007 Test"]);
    git_in(tmp, &["config", "user.email", "test@inv-gate.local"]);
    git_in(tmp, &["config", "commit.gpgsign", "false"]);

    // INV-004: all 4 entries present (INV-004 PASS)
    fs::write(
        tmp.join(".gitignore"),
        ".env.production\n.env.staging\n.env.backup\n.env.local\n",
    )
    .unwrap();

    // No sentry.ts, no sentry.*.config.*, no docker-compose.yml
    // Clean src/ for INV-009
    fs::create_dir_all(tmp.join("src")).unwrap();
    fs::write(tmp.join("src/main.rs"), "fn main() {}\n").unwrap();

    // Remote: clean (no token in URL — INV-010)
    git_in(tmp, &["add", "-A"]);
    git_in(tmp, &["commit", "-q", "-m", "minimal baseline"]);
    git_in(tmp, &["remote", "add", "origin", "https://github.com/example/minimal.git"]);
}

/// Build a full webapp-like repo where all INV prerequisites are present.
/// Used for probe (d): flag should be no-op (stdout + stderr byte-identical).
fn build_full_repo(tmp: &Path) {
    git_in(tmp, &["init", "-q", "."]);
    git_in(tmp, &["config", "user.name", "P007 Full Test"]);
    git_in(tmp, &["config", "user.email", "test@inv-gate.local"]);
    git_in(tmp, &["config", "commit.gpgsign", "false"]);

    // INV-004: all 4 entries
    fs::write(
        tmp.join(".gitignore"),
        ".env.production\n.env.staging\n.env.backup\n.env.local\n",
    )
    .unwrap();

    // INV-005: sentry.ts with beforeSend (PASS — prerequisite present)
    fs::create_dir_all(tmp.join("src/lib")).unwrap();
    fs::write(
        tmp.join("src/lib/sentry.ts"),
        "export const s = { beforeSend: () => null };\n",
    )
    .unwrap();

    // INV-008: docker-compose.yml with non-internal services (PASS — prerequisite present)
    fs::write(
        tmp.join("docker-compose.yml"),
        "services:\n  nginx:\n    image: nginx:1.25.0\n    ports:\n      - \"443:443\"\n",
    )
    .unwrap();

    // INV-002: no :latest
    // INV-003: no .env.example (absent = PASS)
    // INV-006: no astro-service/app.py (absent = PASS)

    // Clean src/ for INV-009
    fs::create_dir_all(tmp.join("src")).unwrap();
    fs::write(tmp.join("src/main.rs"), "fn main() {}\n").unwrap();

    // Remote: clean (no token)
    git_in(tmp, &["add", "-A"]);
    git_in(tmp, &["commit", "-q", "-m", "full baseline"]);
    git_in(tmp, &["remote", "add", "origin", "https://github.com/example/full.git"]);
}

fn run_gate(tmp: &Path, extra_args: &[&str]) -> (String, String, i32) {
    let mut cmd = Command::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    let mut args = vec!["gate", "--all"];
    args.extend_from_slice(extra_args);
    cmd.args(&args);
    cmd.current_dir(tmp);
    cmd.env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().expect("binary execution failed");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

// ─────────────────────────────────────────────────────────────────────────────
// (a) minimal repo + flag → SKIP lines + 2 warnings + exit 0
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_flag_skips_inv005_inv008() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    // SKIP lines present (LOUD — per Task 1.3)
    assert!(
        stdout.contains("  SKIP (no sentry.ts / sentry.*.config.* present)"),
        "INV-005 SKIP line must be present: got:\n{}", stdout
    );
    assert!(
        stdout.contains("  SKIP (file docker-compose.yml absent)"),
        "INV-008 SKIP line must be present: got:\n{}", stdout
    );

    // Summary: 2 warnings
    assert!(
        stdout.contains("2 warnings"),
        "summary must show 2 warnings: got:\n{}", stdout
    );

    // Skipped invariants line
    assert!(
        stdout.contains("Skipped invariants: INV-005 INV-008"),
        "must have 'Skipped invariants: INV-005 INV-008': got:\n{}", stdout
    );

    // Exit 0
    assert_eq!(code, 0, "minimal repo + --skip-absent must exit 0: got {}", code);
}

// ─────────────────────────────────────────────────────────────────────────────
// (b) minimal repo WITHOUT flag → INV-005/INV-008 FAIL, exit 1 (default intact)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn no_flag_default_fail_on_minimal_repo() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &[]);

    // INV-005 must FAIL (no beforeSend — absent files = no sentry config at all)
    let inv005_sec = extract_section(&stdout, "INV-005");
    assert!(
        inv005_sec.contains("  FAIL"),
        "INV-005 must FAIL without --skip-absent: got:\n{}", inv005_sec
    );

    // INV-008 must FAIL (docker-compose.yml absent → golden stderr + exit 1)
    let inv008_sec = extract_section(&stdout, "INV-008");
    assert!(
        inv008_sec.contains("  FAIL"),
        "INV-008 must FAIL without --skip-absent: got:\n{}", inv008_sec
    );

    // Exit 1
    assert_eq!(code, 1, "minimal repo without flag must exit 1: got {}", code);

    // No SKIP lines
    assert!(
        !stdout.contains("  SKIP"),
        "no flag must not produce SKIP lines: got:\n{}", stdout
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (c) fail-closed: sentry.ts PRESENT but missing beforeSend + flag → FAIL (not SKIP)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_does_not_skip_inv005_when_sentry_ts_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    // Add sentry.ts WITHOUT beforeSend/beforeBreadcrumb
    fs::create_dir_all(tmp.path().join("src/lib")).unwrap();
    fs::write(tmp.path().join("src/lib/sentry.ts"), "export const s = {};\n").unwrap();
    git_commit_all(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    // INV-005 prerequisite (sentry.ts) is present → guard does NOT fire → check runs → FAIL
    let inv005_sec = extract_section(&stdout, "INV-005");
    assert!(
        inv005_sec.contains("  FAIL"),
        "INV-005 must FAIL when sentry.ts present but no scrubber (fail-closed): got:\n{}", inv005_sec
    );
    assert!(
        !inv005_sec.contains("  SKIP"),
        "INV-005 must NOT SKIP when sentry.ts present: got:\n{}", inv005_sec
    );

    // Exit 1 (INV-005 failure)
    assert_eq!(code, 1, "fail-closed: must exit 1 when sentry.ts present but no scrubber");
}

// ─────────────────────────────────────────────────────────────────────────────
// (c2) guard kép (O1.1): sentry.client.config.ts present but NO sentry.ts + flag → INV-005 RUNS
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_guard_kep_sentry_config_present_runs_check() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    // Add sentry.client.config.ts (sentry.*.config.* glob match) WITHOUT scrubber
    // — no sentry.ts present
    fs::write(
        tmp.path().join("sentry.client.config.ts"),
        "// sentry config without scrubber\nexport default {};\n",
    )
    .unwrap();
    git_commit_all(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    // Guard kép: sentry.client.config.ts present → second guard fires → INV-005 RUNS
    let inv005_sec = extract_section(&stdout, "INV-005");
    assert!(
        !inv005_sec.contains("  SKIP"),
        "INV-005 must NOT SKIP when sentry.*.config.* present (guard kép O1.1): got:\n{}", inv005_sec
    );
    assert!(
        inv005_sec.contains("  FAIL"),
        "INV-005 must FAIL when sentry.client.config.ts present but no scrubber: got:\n{}", inv005_sec
    );

    // Exit 1 (INV-005 failure prevents skip exit 0)
    assert_eq!(code, 1, "guard kép: must exit 1 when sentry.client.config.ts has no scrubber");
}

// ─────────────────────────────────────────────────────────────────────────────
// (c) fail-closed: docker-compose.yml PRESENT with internal ports: + flag → FAIL (not SKIP)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_does_not_skip_inv008_when_compose_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    // Add docker-compose.yml with internal service using ports:
    fs::write(
        tmp.path().join("docker-compose.yml"),
        "services:\n  nextjs:\n    image: nextjs:1.0\n    ports:\n      - \"3000:3000\"\n",
    )
    .unwrap();
    git_commit_all(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    // INV-008 prerequisite (docker-compose.yml) is present → guard does NOT fire → FAIL
    let inv008_sec = extract_section(&stdout, "INV-008");
    assert!(
        inv008_sec.contains("  FAIL"),
        "INV-008 must FAIL when docker-compose.yml present with internal ports: (fail-closed): got:\n{}", inv008_sec
    );
    assert!(
        !inv008_sec.contains("  SKIP"),
        "INV-008 must NOT SKIP when docker-compose.yml present: got:\n{}", inv008_sec
    );

    assert_eq!(code, 1, "fail-closed: must exit 1 when docker-compose.yml has internal ports:");
}

// ─────────────────────────────────────────────────────────────────────────────
// (d) flag no-op when all prerequisites present: stdout + stderr byte-identical with/without flag
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_flag_noop_when_all_files_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_full_repo(tmp.path());

    let (stdout_no_flag, stderr_no_flag, code_no_flag) = run_gate(tmp.path(), &[]);
    let (stdout_flag, stderr_flag, code_flag) = run_gate(tmp.path(), &["--skip-absent"]);

    assert_eq!(
        stdout_no_flag, stdout_flag,
        "stdout must be byte-identical with/without --skip-absent when all files present"
    );
    assert_eq!(
        stderr_no_flag, stderr_flag,
        "stderr must be byte-identical with/without --skip-absent when all files present"
    );
    assert_eq!(
        code_no_flag, code_flag,
        "exit code must be identical with/without --skip-absent when all files present"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (e) INV-004 not skippable: .gitignore missing entry + flag → INV-004 still FAIL
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_absent_does_not_skip_inv004() {
    let tmp = tempfile::tempdir().expect("tempdir");

    // Set up repo with .gitignore missing entries (INV-004 FAIL)
    git_in(tmp.path(), &["init", "-q", "."]);
    git_in(tmp.path(), &["config", "user.name", "test"]);
    git_in(tmp.path(), &["config", "user.email", "test@test.com"]);
    git_in(tmp.path(), &["config", "commit.gpgsign", "false"]);

    // Only 3 of 4 required entries (missing .env.staging)
    fs::write(
        tmp.path().join(".gitignore"),
        ".env.production\n.env.backup\n.env.local\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    git_commit_all(tmp.path());
    git_in(tmp.path(), &["remote", "add", "origin", "https://github.com/example/test.git"]);

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    let inv004_sec = extract_section(&stdout, "INV-004");
    assert!(
        inv004_sec.contains("  FAIL"),
        "INV-004 must FAIL even with --skip-absent (not in allowlist): got:\n{}", inv004_sec
    );
    assert!(
        !inv004_sec.contains("  SKIP"),
        "INV-004 must NOT SKIP (cố ý — allowlist only INV-005/INV-008): got:\n{}", inv004_sec
    );

    assert_eq!(code, 1, "INV-004 not skippable: must exit 1 even with flag");
}

// ─────────────────────────────────────────────────────────────────────────────
// (f) skipped > 0 but another INV FAIL → exit 1, both Failed and Skipped lines present
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn skip_and_fail_coexist_exit_1() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo(tmp.path());

    // Introduce a :latest tag (INV-002 FAIL) — minimal repo has no docker-compose.yml
    // so add one with :latest for a non-internal service (to avoid INV-008 fire)
    // and also skip the sentry.*.config.* guard
    fs::write(
        tmp.path().join("docker-compose.yml"),
        "services:\n  myapp:\n    image: myapp:latest\n",
    )
    .unwrap();
    git_commit_all(tmp.path());

    let (stdout, _stderr, code) = run_gate(tmp.path(), &["--skip-absent"]);

    // INV-002 FAIL (has :latest)
    let inv002_sec = extract_section(&stdout, "INV-002");
    assert!(
        inv002_sec.contains("  FAIL"),
        "INV-002 must FAIL with :latest tag: got:\n{}", inv002_sec
    );

    // INV-005 SKIP (sentry.ts absent + no sentry.*.config.*)
    assert!(
        stdout.contains("  SKIP (no sentry.ts / sentry.*.config.* present)"),
        "INV-005 must still SKIP: got:\n{}", stdout
    );

    // INV-008: docker-compose.yml IS present now (myapp is not internal) → PASS
    let inv008_sec = extract_section(&stdout, "INV-008");
    assert!(
        inv008_sec.contains("  PASS"),
        "INV-008 must PASS for non-internal service: got:\n{}", inv008_sec
    );

    // Summary must have both Failed and Skipped lines
    assert!(
        stdout.contains("Failed invariants:"),
        "must have 'Failed invariants:' when FAIL>0: got:\n{}", stdout
    );
    assert!(
        stdout.contains("Skipped invariants:"),
        "must have 'Skipped invariants:' when skip>0: got:\n{}", stdout
    );

    // Exit 1 (FAIL > 0)
    assert_eq!(code, 1, "skipped + failed must exit 1 (FAIL takes precedence)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: extract named section content from stdout
// ─────────────────────────────────────────────────────────────────────────────
fn extract_section<'a>(stdout: &'a str, inv: &str) -> &'a str {
    let marker = format!("--- {}:", inv);
    let start = stdout
        .find(marker.as_str())
        .unwrap_or_else(|| panic!("section {} not found in stdout:\n{}", inv, stdout));
    let rest = &stdout[start + marker.len()..];
    let end = rest
        .find("\n--- INV-")
        .map(|p| start + marker.len() + p + 1)
        .unwrap_or(stdout.len());
    &stdout[start..end]
}
