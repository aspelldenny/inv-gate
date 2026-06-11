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
// O1.2 (Debate Log Turn 1): golden/check-schema-safety.sh:33 uses SHA `4b825dc8669f8c0`
// (15 chars) — NOT the standard empty-tree SHA `4b825dc642cb6eb9a060e54bf8d69288fbee4904`.
// Fallback chain ported AS-IS: both git calls may fail on 1-commit/fresh repo → echo "" fires.
// This is intentional parity — DO NOT "fix" the SHA in this phiếu (parity-first, Luật chơi 1).
// Improvement candidate for BACKLOG after parity is shipped.

use std::io::{self, Write};
use std::process::{Command, Stdio};

// golden/check-schema-safety.sh:23
const SCHEMA_FILE: &str = "prisma/schema.prisma";

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

// golden/check-schema-safety.sh:32-34 — fallback chain 3 steps (AS-IS including bad SHA — O1.2)
// Step 1: git diff HEAD~1..HEAD -- prisma/schema.prisma
// Step 2: git diff 4b825dc8669f8c0..HEAD -- prisma/schema.prisma  ← 15-char SHA, NOT empty-tree standard
// Step 3: "" (echo "" fallback)
// "fail" semantics = bash || (non-zero exit from git) ↔ Rust: !success_bool
fn get_diff() -> String {
    // Step 1
    let (diff, ok) = git_diff(&["diff", "HEAD~1..HEAD", "--", SCHEMA_FILE]);
    if ok {
        return diff;
    }

    // Step 2 — O1.2: bad SHA 4b825dc8669f8c0 (15 chars) ported AS-IS, not fixed
    let (diff2, ok2) = git_diff(&["diff", "4b825dc8669f8c0..HEAD", "--", SCHEMA_FILE]);
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
pub fn run() -> i32 {
    // Branch A — golden/check-schema-safety.sh:18-20
    let allow_data_loss = std::env::var("ALLOW_DATA_LOSS").unwrap_or_else(|_| "false".to_string());
    if allow_data_loss == "true" {
        println!("{}", BYPASS_ECHO);
        return 0;
    }

    // Branch B — golden/check-schema-safety.sh:25-28
    if !std::path::Path::new(SCHEMA_FILE).exists() {
        println!("\u{274C} {} not found \u{2014} cannot check schema safety.", SCHEMA_FILE);
        return 1;
    }

    // Fallback chain — golden/check-schema-safety.sh:32-34 (Branch C/D/E/F)
    let diff = get_diff();

    // Branch C + F — golden/check-schema-safety.sh:36-39
    // DIFF empty: no parent commit / both git calls fail / no change in schema.
    if diff.trim().is_empty() {
        println!("No schema diff vs HEAD~1 \u{2014} safe.");
        return 0;
    }

    // golden/check-schema-safety.sh:46-49 — find destructive lines
    let destructive = find_destructive(&diff);

    if !destructive.is_empty() {
        // Branch E — golden/check-schema-safety.sh:51-60
        println!("\u{274C} DESTRUCTIVE SCHEMA CHANGE DETECTED in {}:", SCHEMA_FILE);
        println!("{}", destructive.join("\n"));
        println!();
        println!("Field/model removed \u{2192} may cause DROP COLUMN/TABLE on db push.");
        println!();
        println!("To proceed:");
        println!("  CI:    re-run workflow_dispatch with input data_loss_ack=true.");
        println!("  Local: verify backup < 24h, then ALLOW_DATA_LOSS=true bash scripts/check-schema-safety.sh");
        let _ = io::stdout().flush();
        1
    } else {
        // Branch D — golden/check-schema-safety.sh:63
        println!("Schema diff present but no destructive pattern (field/model removed) detected \u{2014} safe.");
        0
    }
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
}
