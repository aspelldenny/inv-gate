// src/checks/port.rs — Port of golden/check-port-bind.py (INV-001)
//
// PARITY CONTRACT: every constant below cites golden/check-port-bind.py:<line>.
// DO NOT change any pattern, allowlist entry, exception set, output wording, or count
// logic without a separate phiếu + Tầng 1 + Giám sát review (security surface — CLAUDE.md).
//
// Mechanism: line-based 4-layer (anchor #4 ✅) — NOT YAML parse (serde_yaml banned).
// Port is 1:1 behavioral copy including "golden dở" edges (1-part/4+-part `unrecognized format`,
// single-quote skip, is_in_ports_block walk) — see O1.2/anchor #4 in phiếu Debate Log.
//
// Non-UTF-8: golden read_text() UTF-8 strict (no try/except) → uncaught UnicodeDecodeError →
// traceback stderr + Python exits with code 1 (uncaught exception). Rust: error-path exit
// non-zero + error message to stderr. KHÔNG panic (exit 101 ≠ golden exit 1). Discovery P004.

use regex::Regex;
use std::io::{self, Write};

// golden/check-port-bind.py:12-16 — hardcoded compose file list, ORDER matters for output order
const COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.dev.yml",
    "astro-service/docker-compose.yml",
];

// golden/check-port-bind.py:19 — nginx public ports allowlist (exact set)
const ALLOWED_PUBLIC: &[&str] = &["80:80", "443:443"];

// golden/check-port-bind.py:21 — PORT_LINE_RE: match YAML list item with optional double-quote.
// Captures between double-quotes or bare. KHÔNG strip single-quote (anchor #6 ✅).
// Pattern: ^\s*-\s*"?([^"]+?)"?\s*$
// Layer 1 of 4-layer parse mechanism.
fn port_line_re() -> Regex {
    Regex::new(r#"^\s*-\s*"?([^"]+?)"?\s*$"#).unwrap()
}

// golden/check-port-bind.py:69 — numeric filter: only digits/dots/colons pass.
// Layer 2: silently SKIP /udp|/tcp suffix, port-range hyphen, long-syntax letters, single-quoted items.
// Pattern: ^[\d.:]+$
fn numeric_re() -> Regex {
    Regex::new(r"^[\d.:]+$").unwrap()
}

// golden/check-port-bind.py:23-37 — is_in_ports_block(): walk backward from idx.
// Returns true iff the nearest enclosing key is `ports:`.
// Layer 3 of 4-layer parse.
fn is_in_ports_block(lines: &[&str], idx: usize) -> bool {
    let mut i = idx as isize - 1;
    while i >= 0 {
        let line = lines[i as usize];
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            i -= 1;
            continue;
        }
        // If we hit another `key:` at same/less indent -> not in ports:
        // golden:31 — check: ends with ":" and does NOT start with "-"
        if stripped.ends_with(':') && !stripped.starts_with('-') {
            return stripped == "ports:";
        }
        // Still in list items — golden:34
        if stripped.starts_with('-') {
            i -= 1;
            continue;
        }
        return false;
    }
    false
}

// golden/check-port-bind.py:39-53 — classify(): split on ":", handle 1/2/3/4+ parts.
// Layer 4 of 4-layer parse.
// Returns "ok" or a violation reason string.
fn classify(spec: &str) -> String {
    let parts: Vec<&str> = spec.split(':').collect();
    match parts.len() {
        3 => {
            // IP:HOST:CONTAINER — golden:43-47
            let ip = parts[0];
            if ip == "127.0.0.1" {
                "ok".to_string()
            } else {
                // golden:47 — kể cả 0.0.0.0 explicit → public IP bind (KHÔNG implicit)
                format!("public IP bind: {}", spec)
            }
        }
        2 => {
            // HOST:CONTAINER — golden:48-52
            if ALLOWED_PUBLIC.contains(&spec) {
                "ok".to_string()
            } else {
                format!("implicit 0.0.0.0 bind: {}", spec)
            }
        }
        _ => {
            // 1-part and 4+-part (IPv6 ::1:8080) — golden:53
            format!("unrecognized format: {}", spec)
        }
    }
}

// golden/check-port-bind.py:55-80 — main logic
pub fn run() -> i32 {
    let port_re = port_line_re();
    let num_re = numeric_re();
    let mut violations: Vec<String> = Vec::new();

    for fname in COMPOSE_FILES {
        let path = std::path::Path::new(fname);

        // golden/check-port-bind.py:59-61 — missing file: WARN stderr, continue (no violation, no exit 1)
        if !path.exists() {
            eprintln!("WARN: {} not found, skipping", fname);
            continue;
        }

        // golden/check-port-bind.py:62 — read_text() UTF-8 strict, no try/except.
        // Rust: map error to stderr + non-zero exit (anchor #8 ✅ — KHÔNG panic=101).
        let content = match std::fs::read(path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => {
                    // Non-UTF-8: error to stderr, exit non-zero — mirrors Python uncaught UnicodeDecodeError.
                    // golden crash exit code = 1 (Python uncaught exception default).
                    eprintln!("error reading {}: {}", fname, e);
                    return 1;
                }
            },
            Err(e) => {
                eprintln!("error reading {}: {}", fname, e);
                return 1;
            }
        };

        // golden/check-port-bind.py:63 — splitlines()
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            // golden/check-port-bind.py:64-65 — PORT_LINE_RE match
            let m = match port_re.captures(line) {
                Some(m) => m,
                None => continue,
            };
            // golden/check-port-bind.py:67 — captured group 1, stripped
            let spec = m[1].trim();

            // golden/check-port-bind.py:69-70 — numeric filter (Layer 2)
            if !num_re.is_match(spec) {
                continue;
            }

            // golden/check-port-bind.py:71 — is_in_ports_block (Layer 3)
            if !is_in_ports_block(&lines, idx) {
                continue;
            }

            // golden/check-port-bind.py:72-75 — classify + emit violation
            let result = classify(spec);
            if result != "ok" {
                // golden/check-port-bind.py:75 — format: {fname}:{idx+1}: INV-001 violated -- {result}
                violations.push(format!("{}:{}: INV-001 violated -- {}", fname, idx + 1, result));
            }
        }
    }

    // golden/check-port-bind.py:76-79
    if !violations.is_empty() {
        println!("{}", violations.join("\n"));
        // Flush stdout before exit
        let _ = io::stdout().flush();
        1
    } else {
        // golden/check-port-bind.py:79
        println!("INV-001: PASS (port bindings clean)");
        0
    }
}

// ── Unit tests (Task 4 — 12 probes BẮT BUỘC per phiếu V2) ───────────────────
// F07: synthetic in-code, KHÔNG invent fixture files in tests/golden/fixtures/
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Helper: write compose file content to temp dir and run check against it.
    fn run_with_compose(content: &str) -> (i32, String, String) {
        let tmp = tempfile::tempdir().unwrap();
        let compose_path = tmp.path().join("docker-compose.yml");
        std::fs::write(&compose_path, content).unwrap();

        // Run binary
        let output = assert_cmd::Command::cargo_bin("inv-gate")
            .unwrap()
            .args(["check", "port"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        (code, stdout, stderr)
    }

    // (a) 3-part public IP → violation `public IP bind`
    // golden/check-port-bind.py:43-47
    #[test]
    fn probe_a_3part_public_ip_violation() {
        let content = "services:\n  app:\n    ports:\n      - \"203.0.113.5:8001:8001\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 1, "expected exit 1 for public IP bind");
        assert!(stdout.contains("public IP bind: 203.0.113.5:8001:8001"),
            "expected 'public IP bind' in stdout, got: {}", stdout);
    }

    // (b) 3-part loopback → 0 violation
    // golden/check-port-bind.py:45
    #[test]
    fn probe_b_3part_loopback_ok() {
        let content = "services:\n  app:\n    ports:\n      - \"127.0.0.1:8001:8001\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "expected exit 0 for loopback bind");
        assert!(stdout.contains("PASS"), "expected PASS, got: {}", stdout);
    }

    // (c) nginx 80:80 → 0 violation (ALLOWED_PUBLIC)
    // golden/check-port-bind.py:19,50
    #[test]
    fn probe_c_nginx_80_ok() {
        let content = "services:\n  nginx:\n    ports:\n      - \"80:80\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "expected exit 0 for 80:80 nginx");
        assert!(stdout.contains("PASS"), "expected PASS, got: {}", stdout);
    }

    // (d) 2-part non-nginx → violation `implicit 0.0.0.0 bind` (format+wording exact)
    // golden/check-port-bind.py:48-52
    #[test]
    fn probe_d_2part_non_nginx_violation() {
        let content = "services:\n  app:\n    ports:\n      - \"8001:8001\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 1, "expected exit 1 for 2-part non-nginx");
        assert!(stdout.contains("INV-001 violated -- implicit 0.0.0.0 bind: 8001:8001"),
            "expected 'implicit 0.0.0.0 bind: 8001:8001', got: {}", stdout);
    }

    // (e) missing compose file → WARN wording exact on stderr, no panic
    // golden/check-port-bind.py:59-61 (anchor #5 ✅)
    #[test]
    fn probe_e_missing_file_warn_stderr() {
        // Empty temp dir — no compose files at all → all 3 WARN
        let tmp = tempfile::tempdir().unwrap();
        let output = assert_cmd::Command::cargo_bin("inv-gate")
            .unwrap()
            .args(["check", "port"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("WARN: docker-compose.yml not found, skipping"),
            "expected WARN for docker-compose.yml, got: {}", stderr);
        assert!(stderr.contains("WARN: docker-compose.dev.yml not found, skipping"),
            "expected WARN for docker-compose.dev.yml, got: {}", stderr);
        assert!(stderr.contains("WARN: astro-service/docker-compose.yml not found, skipping"),
            "expected WARN for astro-service, got: {}", stderr);
        // No panic
        assert!(output.status.code().unwrap_or(-1) != 101, "must not panic");
    }

    // (f1) single-quoted '8001:8001' → SKIP (not violation — Layer 2 numeric filter)
    // golden/check-port-bind.py:21 — PORT_LINE_RE does NOT strip single-quote → captured group
    // contains "'8001:8001'" → numeric filter fails (apostrophe not in [\d.:]) → skipped.
    #[test]
    fn probe_f1_single_quoted_skip() {
        let content = "services:\n  app:\n    ports:\n      - '8001:8001'\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "single-quoted port should be skipped, got stdout: {}", stdout);
        assert!(stdout.contains("PASS"), "expected PASS for single-quoted, got: {}", stdout);
    }

    // (f2) bare 1-part `8080` → violation `unrecognized format: 8080`
    // golden/check-port-bind.py:53 — 1-part (not 2 or 3 parts after split)
    #[test]
    fn probe_f2_bare_1part_unrecognized() {
        let content = "services:\n  app:\n    ports:\n      - \"8080\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 1, "expected exit 1 for bare 1-part port");
        assert!(stdout.contains("unrecognized format: 8080"),
            "expected 'unrecognized format: 8080', got: {}", stdout);
    }

    // (f3) 0.0.0.0:8001:8001 explicit → violation `public IP bind` (NOT `implicit`)
    // golden/check-port-bind.py:47 — 3-part, ip != "127.0.0.1" → public IP bind
    #[test]
    fn probe_f3_explicit_000_public_ip() {
        let content = "services:\n  app:\n    ports:\n      - \"0.0.0.0:8001:8001\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 1, "expected exit 1 for 0.0.0.0 explicit");
        assert!(stdout.contains("public IP bind: 0.0.0.0:8001:8001"),
            "expected 'public IP bind' (not implicit), got: {}", stdout);
        assert!(!stdout.contains("implicit"), "must NOT say 'implicit' for 0.0.0.0 explicit");
    }

    // (f4) long syntax `target: 80` → SKIP silent (Layer 2 — letters fail numeric filter)
    // golden/check-port-bind.py:69 — "target: 80" has letters → ^[\d.:]+ fails
    #[test]
    fn probe_f4_long_syntax_skip() {
        let content = "services:\n  app:\n    ports:\n      - target: 80\n        published: 80\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "long-syntax target should be skipped, got stdout: {}", stdout);
        assert!(stdout.contains("PASS"), "expected PASS for long-syntax, got: {}", stdout);
    }

    // (f5) port range `8000-8001:8000-8001` → SKIP silent (hyphen not in [\d.:])
    // golden/check-port-bind.py:69 — hyphen fails numeric filter
    #[test]
    fn probe_f5_port_range_skip() {
        let content = "services:\n  app:\n    ports:\n      - \"8000-8001:8000-8001\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "port-range should be skipped, got stdout: {}", stdout);
        assert!(stdout.contains("PASS"), "expected PASS for port range, got: {}", stdout);
    }

    // (f6) list item inside `volumes:` block trông giống port → NOT violation (is_in_ports_block)
    // golden/check-port-bind.py:23-37 — walk back hits `volumes:` not `ports:`
    #[test]
    fn probe_f6_volumes_block_not_violation() {
        let content = "services:\n  app:\n    volumes:\n      - \"8001:8001\"\n    ports:\n      - \"80:80\"\n";
        let (code, stdout, _stderr) = run_with_compose(content);
        assert_eq!(code, 0, "volumes item should not be violation, got stdout: {}", stdout);
        assert!(stdout.contains("PASS"), "expected PASS for volumes block item, got: {}", stdout);
    }

    // (g) non-UTF-8 compose file → error stderr + exit non-zero, KHÔNG panic
    // anchor #8 ✅ — golden crash behavior: error-path exit, not panic-101
    #[test]
    fn probe_g_non_utf8_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("docker-compose.yml");
        // Write invalid UTF-8 bytes
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"services:\n  app:\n    ports:\n      - \"\xff\xfe8001:8001\"\n").unwrap();
        drop(f);

        let output = assert_cmd::Command::cargo_bin("inv-gate")
            .unwrap()
            .args(["check", "port"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let code = output.status.code().unwrap_or(-1);
        assert_ne!(code, 0, "non-UTF-8 should exit non-zero");
        assert_ne!(code, 101, "must NOT panic (exit 101)");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.is_empty(), "expected error message on stderr for non-UTF-8");
    }

    // Unit-level classify() tests (avoid binary spawn overhead for simple cases)

    #[test]
    fn classify_public_ip() {
        assert!(classify("203.0.113.5:8001:8001").starts_with("public IP bind"));
    }

    #[test]
    fn classify_loopback_ok() {
        assert_eq!(classify("127.0.0.1:8001:8001"), "ok");
    }

    #[test]
    fn classify_explicit_000() {
        assert!(classify("0.0.0.0:8001:8001").starts_with("public IP bind"));
    }

    #[test]
    fn classify_nginx_allowed() {
        assert_eq!(classify("80:80"), "ok");
        assert_eq!(classify("443:443"), "ok");
    }

    #[test]
    fn classify_implicit_bind() {
        assert!(classify("8001:8001").starts_with("implicit 0.0.0.0 bind"));
    }

    #[test]
    fn classify_1part_unrecognized() {
        assert_eq!(classify("8080"), "unrecognized format: 8080");
    }

    #[test]
    fn classify_4part_unrecognized() {
        // IPv6-like ::1:8080 splits to 4 parts: ["", "", "1", "8080"]
        assert!(classify("::1:8080").starts_with("unrecognized format"));
    }

    #[test]
    fn is_in_ports_block_true() {
        let lines = vec!["    ports:", "      - \"8001:8001\""];
        assert!(is_in_ports_block(&lines, 1));
    }

    #[test]
    fn is_in_ports_block_false_volumes() {
        let lines = vec!["    volumes:", "      - \"8001:8001\""];
        assert!(!is_in_ports_block(&lines, 1));
    }
}
