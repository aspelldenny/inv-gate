// src/checks/schema.rs — Port of golden/check-schema-safety.sh (Prisma schema-safety)
//
// PARITY CONTRACT: every surface cites golden/check-schema-safety.sh:<line>.
// DO NOT change bypass semantics, fallback chain, destructive patterns, output wording,
// or branch table without a separate phiếu + Tầng 1 + Giám sát review (security surface — CLAUDE.md).
//
// Script is bash `set -u` (no `set -e`) — bash first to be ported.
// Git called via std::process::Command (git2 crate banned, anchor #17 ✅).
// Stdout byte-exact vs pins; stderr suppressed (git: Stdio::null()).
//
// O1.2 (P004 Debate Log): golden/check-schema-safety.sh:33 uses SHA `4b825dc8669f8c0`
// (15 chars) — a golden bug (NOT the standard empty-tree SHA).
// P010 FIX: fallback now uses correct 40-char empty-tree SHA `4b825dc642cb6eb9a060e54bf8d69288fbee4904`.
// Deviation from golden is intentional and documented (CLAUDE.md §Deviations, CHANGELOG P010).
// See EMPTY_TREE_SHA constant + p010_probe_e_oracle_guard_empty_tree_sha for oracle guard.

use std::io::{self, Write};
use std::process::{Command, Stdio};

// golden/check-schema-safety.sh:23
const SCHEMA_FILE: &str = "prisma/schema.prisma";

// P010: empty-tree SHA-1 (git constant — hash of an empty tree object).
// oracle: git hash-object -t tree /dev/null
// Deviates from golden/check-schema-safety.sh:33 (truncated 15-char `4b825dc8669f8c0` = golden bug).
// sha256-repos have a different empty-tree hash — out of scope (golden never supported sha256-repos).
// See CLAUDE.md §Deviations and CHANGELOG P010.
const EMPTY_TREE_SHA: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

// golden/check-schema-safety.sh:18-20 — ALLOW_DATA_LOSS bypass
// Exact string "true" only — case-sensitive; default `false` when unset (bash :-false).
// Bypass echo VERBATIM (contains em dash UTF-8 U+2014):
//   `ALLOW_DATA_LOSS=true — bypass schema safety check (Sếp explicit ack).`
const BYPASS_ECHO: &str =
    "ALLOW_DATA_LOSS=true \u{2014} bypass schema safety check (S\u{1EBF}p explicit ack).";

// golden/check-schema-safety.sh:47 — header-skip pattern
// grep -vE '^---|^-\+\+\+'
const HEADER_SKIP_RE: &str = r"^---|^-\+\+\+";

// golden/check-schema-safety.sh:48 — destructive pattern
// grep -E '^-\s*(model|enum)\s+\w+|^-\s+\w+\s+\S+'
const DESTRUCTIVE_RE: &str = r"^-\s*(model|enum)\s+\w+|^-\s+\w+\s+\S+";

// Run git diff and return (stdout_string, success_bool).
// stderr suppressed (golden: 2>/dev/null — anchor #11 ✅).
fn git_diff(args: &[&str]) -> (String, bool) {
    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // golden/check-schema-safety.sh:32-34 — 2>/dev/null
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            (text, true)
        }
        _ => (String::new(), false),
    }
}

// golden/check-schema-safety.sh:32-34 — fallback chain 3 steps
// Step 1: git diff HEAD~1..HEAD -- prisma/schema.prisma
// Step 2: git diff EMPTY_TREE_SHA..HEAD -- prisma/schema.prisma  (P010 fix: 40-char SHA, not golden's 15-char)
// Step 3: "" (echo "" fallback)
// "fail" semantics = bash || (non-zero exit from git) ↔ Rust: !success_bool
fn get_diff() -> String {
    // Step 1
    let (diff, ok) = git_diff(&["diff", "HEAD~1..HEAD", "--", SCHEMA_FILE]);
    if ok {
        return diff;
    }

    // Step 2 — P010 fix: use correct empty-tree SHA-1 (40 chars) instead of golden's
    // truncated 15-char `4b825dc8669f8c0` (golden bug). See CLAUDE.md §Deviations + CHANGELOG P010.
    // oracle: git hash-object -t tree /dev/null = 4b825dc642cb6eb9a060e54bf8d69288fbee4904
    // sha256-repos out of scope. Deviation guarded by probe p010_probe_e_oracle_guard_empty_tree_sha.
    let (diff2, ok2) = git_diff(&["diff", &format!("{EMPTY_TREE_SHA}..HEAD"), "--", SCHEMA_FILE]);
    if ok2 {
        return diff2;
    }

    // Step 3: echo "" → empty string
    String::new()
}

// golden/check-schema-safety.sh:46-49 — filter DIFF for destructive patterns.
// Pipeline: skip header lines → match destructive lines.
// `|| true` on grep: no match is NOT an error → return empty Vec.
fn find_destructive(diff: &str) -> Vec<String> {
    use regex::Regex;

    let skip_re = Regex::new(HEADER_SKIP_RE).unwrap();
    let dest_re = Regex::new(DESTRUCTIVE_RE).unwrap();

    diff.lines()
        .filter(|line| !skip_re.is_match(line)) // grep -vE header-skip
        .filter(|line| dest_re.is_match(line))   // grep -E destructive
        .map(|s| s.to_string())
        .collect()
}

// golden/check-schema-safety.sh main logic — 6 branch table (anchor #12 ✅)
/// Buffered core — no stdout/stderr side effects; returns CheckOutput.
/// Called by both CLI run() and MCP serve tools.
pub fn run_core() -> crate::checks::CheckOutput {
    // Branch A — golden/check-schema-safety.sh:18-20
    let allow_data_loss = std::env::var("ALLOW_DATA_LOSS").unwrap_or_else(|_| "false".to_string());
    if allow_data_loss == "true" {
        return crate::checks::CheckOutput {
            stdout: format!("{}\n", BYPASS_ECHO),
            stderr: String::new(),
            code: 0,
        };
    }

    // Branch B — golden/check-schema-safety.sh:25-28
    if !std::path::Path::new(SCHEMA_FILE).exists() {
        return crate::checks::CheckOutput {
            stdout: format!("\u{274C} {} not found \u{2014} cannot check schema safety.\n", SCHEMA_FILE),
            stderr: String::new(),
            code: 1,
        };
    }

    // Fallback chain — golden/check-schema-safety.sh:32-34 (Branch C/D/E/F)
    let diff = get_diff();

    // Branch C + F — golden/check-schema-safety.sh:36-39
    // DIFF empty: no parent commit / both git calls fail / no change in schema.
    if diff.trim().is_empty() {
        return crate::checks::CheckOutput {
            stdout: "No schema diff vs HEAD~1 \u{2014} safe.\n".to_string(),
            stderr: String::new(),
            code: 0,
        };
    }

    // golden/check-schema-safety.sh:46-49 — find destructive lines
    let destructive = find_destructive(&diff);

    if !destructive.is_empty() {
        // Branch E — golden/check-schema-safety.sh:51-60
        let mut out = String::new();
        out.push_str(&format!("\u{274C} DESTRUCTIVE SCHEMA CHANGE DETECTED in {}:\n", SCHEMA_FILE));
        out.push_str(&format!("{}\n", destructive.join("\n")));
        out.push('\n');
        out.push_str("Field/model removed \u{2192} may cause DROP COLUMN/TABLE on db push.\n");
        out.push('\n');
        out.push_str("To proceed:\n");
        out.push_str("  CI:    re-run workflow_dispatch with input data_loss_ack=true.\n");
        out.push_str("  Local: verify backup < 24h, then ALLOW_DATA_LOSS=true bash scripts/check-schema-safety.sh\n");
        crate::checks::CheckOutput { stdout: out, stderr: String::new(), code: 1 }
    } else {
        // Branch D — golden/check-schema-safety.sh:63
        crate::checks::CheckOutput {
            stdout: "Schema diff present but no destructive pattern (field/model removed) detected \u{2014} safe.\n".to_string(),
            stderr: String::new(),
            code: 0,
        }
    }
}

/// CLI wrapper — prints buffered output to real stdout/stderr, returns exit code.
pub fn run() -> i32 {
    let out = run_core();
    print!("{}", out.stdout);
    eprint!("{}", out.stderr);
    // Flush stdout before exit (preserved for Branch E)
    let _ = io::stdout().flush();
    out.code
}

// ── Unit tests (Task 5 — 7 probes BẮT BUỘC per phiếu V2) ────────────────────
// F07: synthetic in-code, KHÔNG invent fixture files in tests/golden/fixtures/
#[cfg(test)]
mod tests {
    use super::*;

    // Helper: run `check schema` in a given temp dir with optional env var.
    fn run_schema_in(dir: &std::path::Path, env_allow_data_loss: Option<&str>) -> (i32, String) {
        let mut cmd = assert_cmd::Command::cargo_bin("inv-gate").unwrap();
        cmd.args(["check", "schema"]);
        cmd.current_dir(dir);
        // Always remove ALLOW_DATA_LOSS first (anchor #16 ✅ — hermetic env)
        cmd.env_remove("ALLOW_DATA_LOSS");
        if let Some(val) = env_allow_data_loss {
            cmd.env("ALLOW_DATA_LOSS", val);
        }
        let output = cmd.output().unwrap();
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        (code, stdout)
    }

    // Helper: git command in dir, panics on failure
    fn git_in(dir: &std::path::Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .unwrap();
        if !out.status.success() {
            panic!("git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr));
        }
    }

    // Build a 1-commit repo (for 1-commit + not-a-repo probes)
    fn build_1commit_repo(dir: &std::path::Path, schema_content: &str) {
        git_in(dir, &["init", "-q", "."]);
        git_in(dir, &["config", "user.name", "P004 Test"]);
        git_in(dir, &["config", "user.email", "test@inv-gate.local"]);
        git_in(dir, &["config", "commit.gpgsign", "false"]);
        let prisma = dir.join("prisma");
        std::fs::create_dir_all(&prisma).unwrap();
        std::fs::write(prisma.join("schema.prisma"), schema_content).unwrap();
        git_in(dir, &["add", "-A"]);
        git_in(dir, &["commit", "-q", "-m", "initial"]);
    }

    // Build a 2-commit repo (dirty or clean schema change)
    fn build_2commit_repo(dir: &std::path::Path, schema_before: &str, schema_after: &str) {
        git_in(dir, &["init", "-q", "."]);
        git_in(dir, &["config", "user.name", "P004 Test"]);
        git_in(dir, &["config", "user.email", "test@inv-gate.local"]);
        git_in(dir, &["config", "commit.gpgsign", "false"]);
        let prisma = dir.join("prisma");
        std::fs::create_dir_all(&prisma).unwrap();
        std::fs::write(prisma.join("schema.prisma"), schema_before).unwrap();
        git_in(dir, &["add", "-A"]);
        git_in(dir, &["commit", "-q", "-m", "baseline"]);
        std::fs::write(prisma.join("schema.prisma"), schema_after).unwrap();
        git_in(dir, &["add", "prisma/schema.prisma"]);
        git_in(dir, &["commit", "-q", "-m", "schema change"]);
    }

    // (a) Branch A — ALLOW_DATA_LOSS=true bypass: bypass echo verbatim + exit 0
    // golden/check-schema-safety.sh:18-20, anchor #10 ✅
    #[test]
    fn probe_a_allow_data_loss_bypass() {
        let tmp = tempfile::tempdir().unwrap();
        // Build a repo with destructive schema change (would be exit 1 without bypass)
        let before = "model User {\n  id Int @id\n  email String\n}\n";
        let after = "model User {\n  id Int @id\n}\n";
        build_2commit_repo(tmp.path(), before, after);

        let (code, stdout) = run_schema_in(tmp.path(), Some("true"));
        assert_eq!(code, 0, "bypass should exit 0, got stdout: {}", stdout);
        assert!(
            stdout.contains("ALLOW_DATA_LOSS=true") && stdout.contains("bypass"),
            "expected bypass echo, got: {}", stdout
        );
        // Verify em dash present (UTF-8 U+2014)
        assert!(stdout.contains('\u{2014}'), "expected em dash in bypass echo, got: {}", stdout);
    }

    // (b) ALLOW_DATA_LOSS with non-"true" values → NOT bypass
    // golden/check-schema-safety.sh:18 — exact string "true", case-sensitive
    #[test]
    fn probe_b_allow_data_loss_not_bypass() {
        let tmp = tempfile::tempdir().unwrap();
        let before = "model User {\n  id Int @id\n  email String\n}\n";
        let after = "model User {\n  id Int @id\n}\n";
        build_2commit_repo(tmp.path(), before, after);

        // These should NOT bypass (not exact "true")
        for val in &["TRUE", "1", "false", "yes", ""] {
            let (code, stdout) = run_schema_in(tmp.path(), if val.is_empty() { None } else { Some(val) });
            assert_ne!(code, 0, "ALLOW_DATA_LOSS={:?} should NOT bypass (destructive schema), stdout: {}", val, stdout);
            assert!(!stdout.contains("bypass"), "expected no bypass for ALLOW_DATA_LOSS={:?}", val);
        }
    }

    // (c) Branch B — schema.prisma missing → ❌ ... not found + exit 1
    // golden/check-schema-safety.sh:25-28
    #[test]
    fn probe_c_schema_missing() {
        let tmp = tempfile::tempdir().unwrap();
        // No prisma/schema.prisma
        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 1, "missing schema should exit 1, got: {}", stdout);
        assert!(stdout.contains("prisma/schema.prisma") && stdout.contains("not found"),
            "expected 'not found' message, got: {}", stdout);
    }

    // (d) Branch C/F — 1-commit repo AND not-a-git-repo → fallback chain → safe
    // golden/check-schema-safety.sh:32-39 (anchor #11 ✅, O1.2)
    #[test]
    fn probe_d_one_commit_fallback_safe() {
        // 1-commit repo: HEAD~1 does not exist → both git calls fail → echo "" fires
        let tmp = tempfile::tempdir().unwrap();
        let schema = "model User {\n  id Int @id\n  email String\n}\n";
        build_1commit_repo(tmp.path(), schema);

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 0, "1-commit repo should be safe (fallback chain), got: {}", stdout);
        assert!(stdout.contains("safe"), "expected 'safe', got: {}", stdout);
    }

    #[test]
    fn probe_d_not_a_repo_fallback_safe() {
        // Not a git repo: both git calls fail → echo "" fires → Branch C
        let tmp = tempfile::tempdir().unwrap();
        let prisma = tmp.path().join("prisma");
        std::fs::create_dir_all(&prisma).unwrap();
        std::fs::write(prisma.join("schema.prisma"), "model User { id Int @id }\n").unwrap();

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 0, "not-a-repo should be safe via fallback chain, got: {}", stdout);
        assert!(stdout.contains("safe"), "expected 'safe', got: {}", stdout);
    }

    // (e) header-skip: diff `---` header → NOT false-positive; closing `}` alone → NOT match
    // golden/check-schema-safety.sh:47 — grep -vE '^---|^-\+\+\+'
    #[test]
    fn probe_e_header_skip_and_closing_brace() {
        use regex::Regex;

        let skip_re = Regex::new(HEADER_SKIP_RE).unwrap();
        let dest_re = Regex::new(DESTRUCTIVE_RE).unwrap();

        // Header lines: must be skipped
        for header in &["--- a/prisma/schema.prisma", "-+++ b/prisma/schema.prisma"] {
            assert!(skip_re.is_match(header), "header should match skip: {}", header);
        }

        // Closing brace removed: `- }` → should NOT match destructive
        let closing = "- }";
        assert!(!dest_re.is_match(closing), "closing brace should not match destructive");

        // But a field removal should match
        assert!(dest_re.is_match("-  email String?"), "field removal should match");
        // And model removal should match
        assert!(dest_re.is_match("-model AuditLog {"), "model removal should match");
    }

    // (f) each alternative in destructive pattern `:48` — field, model, enum
    // golden/check-schema-safety.sh:48 — ^-\s*(model|enum)\s+\w+|^-\s+\w+\s+\S+
    #[test]
    fn probe_f_destructive_patterns_individual() {
        use regex::Regex;
        let dest_re = Regex::new(DESTRUCTIVE_RE).unwrap();

        // Field delete
        assert!(dest_re.is_match("-  legacyToken String?"), "field delete should match");
        // Model delete
        assert!(dest_re.is_match("-model AuditLog {"), "model delete should match");
        // Enum delete (parity-blind — fixture does not exercise enum)
        assert!(dest_re.is_match("-enum Status {"), "enum delete should match");
        assert!(dest_re.is_match("-enum UserRole {"), "enum delete should match");
        // Field inside removed model (indented)
        assert!(dest_re.is_match("-  id      Int    @id @default(autoincrement())"), "indented field delete should match");
        assert!(dest_re.is_match("-  payload String"), "payload field delete should match");
    }

    // (g) additive-only / no-diff → Branch D / C correct output + exit 0
    // golden/check-schema-safety.sh:63 (Branch D) and :36-39 (Branch C)
    #[test]
    fn probe_g_additive_only_safe() {
        // 2-commit repo: add a field only (not remove) → Branch D safe
        let tmp = tempfile::tempdir().unwrap();
        let before = "model User {\n  id Int @id\n  email String\n}\n";
        let after = "model User {\n  id Int @id\n  email String\n  displayName String?\n}\n";
        build_2commit_repo(tmp.path(), before, after);

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 0, "additive-only should be safe, got: {}", stdout);
        assert!(
            stdout.contains("no destructive pattern") && stdout.contains("safe"),
            "expected Branch D safe message, got: {}", stdout
        );
    }

    // ── P010 probes — empty-tree SHA fix ────────────────────────────────────────

    // P010-(a) routing delta: 1-commit repo → should route Branch D post-fix
    // TDD RED phase: on old code (bad 15-char SHA) Step 2 fails → Branch C fires instead.
    // After fix (40-char SHA) Step 2 succeeds → diff = additions → Branch D fires.
    // golden cite: Branch C = :36-39, Branch D = :63
    #[test]
    fn p010_probe_a_routing_1commit_branch_d() {
        let tmp = tempfile::tempdir().unwrap();
        let schema = "model User {\n  id Int @id\n  email String\n}\n";
        build_1commit_repo(tmp.path(), schema);

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 0, "1-commit repo should be safe, got: {}", stdout);
        // Post-fix: Step 2 succeeds → diff contains additions → Branch D wording
        assert!(
            stdout.contains("no destructive pattern"),
            "expected Branch D (no destructive pattern) not Branch C (No schema diff), got: {}",
            stdout
        );
    }

    // P010-(b) not-a-repo: Branch C/F unchanged post-fix (both git calls still fail)
    #[test]
    fn p010_probe_b_not_a_repo_still_branch_c() {
        let tmp = tempfile::tempdir().unwrap();
        let prisma = tmp.path().join("prisma");
        std::fs::create_dir_all(&prisma).unwrap();
        std::fs::write(prisma.join("schema.prisma"), "model User { id Int @id }\n").unwrap();

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 0, "not-a-repo should be safe via fallback chain, got: {}", stdout);
        // Step 2 also fails (no git repo), so Branch C fires as before
        assert!(
            stdout.contains("No schema diff"),
            "expected Branch C (No schema diff), got: {}",
            stdout
        );
    }

    // P010-(c) empirical anchor #8: diff so empty-tree is all additions → exit-1 unreachable
    // golden:48 DESTRUCTIVE_RE only matches lines starting with '-'; empty-tree diff has only '+' lines
    #[test]
    fn p010_probe_c_1commit_empirical_no_finding() {
        let tmp = tempfile::tempdir().unwrap();
        let schema = "model User {\n  id Int @id\n  email String\n}\n";
        build_1commit_repo(tmp.path(), schema);

        let (code, stdout) = run_schema_in(tmp.path(), None);
        // Post-fix: 1-commit repo → Branch D, no destructive findings, exit 0
        assert_eq!(code, 0, "1-commit repo with sane schema should exit 0, got: {}", stdout);
        // Confirm no ❌ (destructive findings indicator)
        assert!(
            !stdout.contains('\u{274C}'),
            "expected no destructive finding (exit-1 unreachable so empty-tree), got: {}",
            stdout
        );
    }

    // P010-(d) 2-commit destructive → exit 1 (Step 1 path, fix does not touch this)
    #[test]
    fn p010_probe_d_2commit_destructive_exit1() {
        let tmp = tempfile::tempdir().unwrap();
        let before = "model User {\n  id Int @id\n  email String\n}\n";
        let after = "model User {\n  id Int @id\n}\n"; // removed email field
        build_2commit_repo(tmp.path(), before, after);

        let (code, stdout) = run_schema_in(tmp.path(), None);
        assert_eq!(code, 1, "2-commit destructive should exit 1, got stdout: {}", stdout);
        assert!(stdout.contains('\u{274C}'), "expected ❌ in destructive output, got: {}", stdout);
    }

    // P010-(e) oracle guard: git hash-object -t tree /dev/null == EMPTY_TREE_SHA constant
    // If the hardcoded constant is wrong (typo), this test fails on every machine.
    // This is why hardcoding is safe: the oracle runs at test-time.
    #[test]
    fn p010_probe_e_oracle_guard_empty_tree_sha() {
        let output = std::process::Command::new("git")
            .args(["hash-object", "-t", "tree", "/dev/null"])
            .output()
            .expect("git hash-object must be available");
        assert!(output.status.success(), "git hash-object failed");
        let oracle = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(
            oracle,
            EMPTY_TREE_SHA,
            "EMPTY_TREE_SHA constant does not match oracle output"
        );
    }
}
