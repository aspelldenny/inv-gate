// src/checks/secrets.rs — Port of golden/check-hardcoded-secrets.py (INV-009)
//
// PARITY CONTRACT: every constant below cites golden/check-hardcoded-secrets.py:<line>.
// DO NOT change any pattern, allowlist entry, skip rule, masking, or output wording
// without a separate phiếu + Tầng 1 + Giám sát review (security surface).

use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

// golden/check-hardcoded-secrets.py:33-35
const SRC_DIRS_JS: &[&str] = &["src"];
const SRC_DIRS_PY: &[&str] = &["astro-service"];
const JS_EXTS: &[&str] = &[".ts", ".tsx", ".js", ".jsx"];
const PY_EXTS: &[&str] = &[".py"];

// golden/check-hardcoded-secrets.py:39-48
// Skip these path substring patterns (test + build artifact + generated)
const SKIP_PATH_SUBSTR: &[&str] = &[
    "node_modules/",
    ".next/",
    "__pycache__/",
    "dist/",
    "build/",
    "target/",
    ".claude/worktrees/",
    "src/generated/", // Prisma client WASM base64 blob (coincidental AKIA collision)
];

// golden/check-hardcoded-secrets.py:97-99
// Comment line prefixes (minimal — single-line only)
const JS_COMMENT_PREFIX: &str = "//";
const PY_COMMENT_PREFIX: &str = "#";

// golden/check-hardcoded-secrets.py:84-95
// Allowlist substrings (skip violation if match line contains)
const ALLOWLIST_SUBSTRINGS: &[&str] = &[
    "c993dc1e",        // Sentry DSN public default (INV-003 allowlist consistency)
    "google/gemini",   // Model routing slug
    "anthropic/claude",// Model routing slug
    "process.env.",    // Node env reference (not literal)
    "os.environ",      // Python env reference (not literal)
    "your-",           // Placeholder convention (your-api-key)
    "xxx",             // Common placeholder
    "REPLACE",         // Common placeholder
    "PLACEHOLDER",     // Common placeholder — golden/check-hardcoded-secrets.py:94
];

// golden/check-hardcoded-secrets.py:106-107
fn should_skip_path(path_str: &str) -> bool {
    SKIP_PATH_SUBSTR.iter().any(|s| path_str.contains(s))
}

// golden/check-hardcoded-secrets.py:51-58
// Test file patterns
fn is_test_file(path_str: &str) -> bool {
    // golden/check-hardcoded-secrets.py:52
    let re_test_ext_ts = Regex::new(r"\.test\.(ts|tsx|js|jsx|py)$").unwrap();
    // golden/check-hardcoded-secrets.py:53
    let re_spec_ext = Regex::new(r"\.spec\.(ts|tsx|js|jsx|py)$").unwrap();
    // golden/check-hardcoded-secrets.py:54
    let re_tests_dir = Regex::new(r"/__tests__/").unwrap();
    // golden/check-hardcoded-secrets.py:55
    let re_mocks_dir = Regex::new(r"/__mocks__/").unwrap();
    // golden/check-hardcoded-secrets.py:56
    let re_tests_path = Regex::new(r"/tests/").unwrap();
    // golden/check-hardcoded-secrets.py:57
    let re_prisma_seed = Regex::new(r"prisma/seed.*\.ts$").unwrap();

    re_test_ext_ts.is_match(path_str)
        || re_spec_ext.is_match(path_str)
        || re_tests_dir.is_match(path_str)
        || re_mocks_dir.is_match(path_str)
        || re_tests_path.is_match(path_str)
        || re_prisma_seed.is_match(path_str)
}

// golden/check-hardcoded-secrets.py:110-114
fn is_comment_line(line: &str, ext: &str) -> bool {
    let stripped = line.trim_start();
    if ext == ".py" {
        stripped.starts_with(PY_COMMENT_PREFIX)
    } else {
        stripped.starts_with(JS_COMMENT_PREFIX)
    }
}

// golden/check-hardcoded-secrets.py:117-118
fn is_allowlisted(text: &str) -> bool {
    ALLOWLIST_SUBSTRINGS.iter().any(|s| text.contains(s))
}

// golden/check-hardcoded-secrets.py:121-125
fn mask(secret: &str) -> String {
    if secret.len() <= 12 {
        let last4: String = if secret.len() >= 4 {
            secret[secret.len() - 4..].to_string()
        } else {
            String::new()
        };
        format!("***{}", last4)
    } else {
        let first4 = &secret[..4];
        let last4 = &secret[secret.len() - 4..];
        format!("{}...{}", first4, last4)
    }
}

// golden/check-hardcoded-secrets.py:128-153
// Returns list of (lineno, masked_match, pattern_name)
fn scan_file(path: &Path) -> Vec<(usize, String, String)> {
    // golden/check-hardcoded-secrets.py:60-76 — PREFIX_PATTERNS
    let prefix_patterns: &[(&str, &str)] = &[
        // golden/check-hardcoded-secrets.py:62-63 — Anthropic API key
        ("anthropic", r"sk-ant-[A-Za-z0-9_\-]{40,}"),
        // golden/check-hardcoded-secrets.py:64-65 — OpenAI-style API key
        ("openai", r"sk-[A-Za-z0-9]{48}"),
        // golden/check-hardcoded-secrets.py:66-67 — AWS access key ID
        ("aws", r"AKIA[0-9A-Z]{16}"),
        // golden/check-hardcoded-secrets.py:68-69 — GitHub personal access token
        ("github-pat", r"gh[pous]_[A-Za-z0-9]{36}"),
        // golden/check-hardcoded-secrets.py:70-71 — Google API key
        ("google-api", r"AIza[0-9A-Za-z_\-]{35}"),
        // golden/check-hardcoded-secrets.py:72-73 — Resend API key
        ("resend", r"re_[A-Za-z0-9_\-]{30,}"),
        // golden/check-hardcoded-secrets.py:74-75 — Telegram bot token
        ("telegram-bot", r"\b\d{8,12}:[A-Za-z0-9_\-]{35}\b"),
    ];

    // golden/check-hardcoded-secrets.py:79-82 — GENERIC_PATTERN
    // Note: backtick in character class ['"` ] requires raw string with # delimiter
    let generic_pattern = Regex::new(
        r#"(?i)(api[_\-]?key|apikey|secret|password|token)\s*[:=]\s*['"`]([A-Za-z0-9_\-+/=]{32,})['"`]"#
    ).unwrap();

    let compiled_prefix: Vec<(&str, Regex)> = prefix_patterns
        .iter()
        .map(|(name, pat)| (*name, Regex::new(pat).unwrap()))
        .collect();

    let mut violations = Vec::new();

    // golden/check-hardcoded-secrets.py:131-134 — non-UTF-8/unreadable → skip file silent
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return violations,
    };

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let ext_with_dot = format!(".{}", ext);

    // golden/check-hardcoded-secrets.py:136 — enumerate(content.splitlines(), start=1)
    for (lineno, line) in content.lines().enumerate().map(|(i, l)| (i + 1, l)) {
        // golden/check-hardcoded-secrets.py:137 — is_comment_line check
        if is_comment_line(line, &ext_with_dot) {
            continue;
        }

        // golden/check-hardcoded-secrets.py:139-145 — Prefix-based detection
        for (name, pat) in &compiled_prefix {
            for m in pat.find_iter(line) {
                let hit = m.as_str();
                if is_allowlisted(line) || is_allowlisted(hit) {
                    continue;
                }
                violations.push((lineno, mask(hit), name.to_string()));
            }
        }

        // golden/check-hardcoded-secrets.py:147-152 — Generic high-entropy fallback
        for caps in generic_pattern.captures_iter(line) {
            // golden/check-hardcoded-secrets.py:148 — m.group(2) = captures[2]
            let hit = &caps[2];
            let full = &caps[0];
            if is_allowlisted(line) || is_allowlisted(full) || is_allowlisted(hit) {
                continue;
            }
            violations.push((lineno, mask(hit), "generic-entropy".to_string()));
        }
    }
    violations
}

// golden/check-hardcoded-secrets.py:156-170 — collect_files()
// Mirrors Python: for each target dir, for each ext in order, rglob("*{ext}")
// Returns files in same traversal order as Python Path.rglob() on macOS (alphabetical depth-first).
fn collect_files() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    // golden/check-hardcoded-secrets.py:157-163 — JS/TS dirs
    for dir in SRC_DIRS_JS {
        let root = Path::new(dir);
        if !root.exists() {
            continue;
        }
        for ext in JS_EXTS {
            // golden/check-hardcoded-secrets.py:161 — root.rglob(f"*{ext}")
            // WalkDir with sort gives alphabetical depth-first order matching Python rglob on macOS
            let mut matched: Vec<_> = WalkDir::new(root)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(ext))
                        .unwrap_or(false)
                })
                .map(|e| e.path().to_path_buf())
                .collect();
            // Python rglob on macOS: depth-first, then alphabetical within a directory.
            // WalkDir sort_by_file_name gives alphabetical at each level — same traversal.
            // However Python rglob yields parent dir entries before subdirs alphabetically,
            // and WalkDir does too (parent files before subdir entries when sorted).
            files.append(&mut matched);
        }
    }

    // golden/check-hardcoded-secrets.py:164-169 — Python dirs
    for dir in SRC_DIRS_PY {
        let root = Path::new(dir);
        if !root.exists() {
            // golden/check-hardcoded-secrets.py:166 — `if not root.exists(): continue`
            continue;
        }
        for ext in PY_EXTS {
            let mut matched: Vec<_> = WalkDir::new(root)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(ext))
                        .unwrap_or(false)
                })
                .map(|e| e.path().to_path_buf())
                .collect();
            files.append(&mut matched);
        }
    }

    files
}

// golden/check-hardcoded-secrets.py:173-188 — main()
/// Buffered core — no stdout/stderr side effects; returns CheckOutput.
/// Called by both CLI run() and MCP serve tools.
pub fn run_core() -> crate::checks::CheckOutput {
    let mut all_violations: Vec<String> = Vec::new();

    for path in collect_files() {
        let path_str = path.to_string_lossy();
        // golden/check-hardcoded-secrets.py:176-177 — should_skip_path
        if should_skip_path(&path_str) {
            continue;
        }
        // golden/check-hardcoded-secrets.py:178-179 — is_test_file
        if is_test_file(&path_str) {
            continue;
        }
        // golden/check-hardcoded-secrets.py:180-184 — collect violations
        for (lineno, masked, pat_name) in scan_file(&path) {
            // golden/check-hardcoded-secrets.py:182 — output format
            all_violations.push(format!(
                "{path_str}:{lineno}: INV-009 violated -- {masked} ({pat_name})"
            ));
        }
    }

    if !all_violations.is_empty() {
        // golden/check-hardcoded-secrets.py:185-186
        let text = format!("{}\n", all_violations.join("\n"));
        crate::checks::CheckOutput { stdout: text, stderr: String::new(), code: 1 }
    } else {
        // golden/check-hardcoded-secrets.py:187
        crate::checks::CheckOutput {
            stdout: "INV-009: PASS (0 hardcoded secrets)\n".to_string(),
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
    out.code
}

// ── Unit tests (Task 3 Lưu ý 4 — V2, BẮT BUỘC) ─────────────────────────────
// F07: synthetic in-code, no new fixture files in tests/golden/fixtures/
#[cfg(test)]
mod tests {
    use super::*;

    // Unit test (a) — V2 O1.1: should_skip_path with src/generated/ path → skipped
    // golden/check-hardcoded-secrets.py:39-48 (SKIP_PATH_SUBSTR) + :106-107 (should_skip_path)
    #[test]
    fn test_skip_path_src_generated() {
        assert!(should_skip_path("src/generated/prisma-client.ts"));
        assert!(should_skip_path("src/generated/"));
        // Verify non-skipped path is not caught
        assert!(!should_skip_path("src/config.ts"));
        assert!(!should_skip_path("src/services/auth.ts"));
    }

    // Unit test (b) — V2 O1.2: comment line with secret-like string → skipped
    // golden/check-hardcoded-secrets.py:98-114 (is_comment_line) + :137 (applied)
    #[test]
    fn test_comment_line_skip() {
        // JS/TS single-line comment: // api_key = "FAKEKEY..."
        assert!(is_comment_line("// api_key = \"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456\"", ".ts"));
        assert!(is_comment_line("  // token: ABCDEFGHIJKLMNOPQRSTUVWXYZ123456", ".tsx"));
        // Python single-line comment: # api_key = "..."
        assert!(is_comment_line("# secret = \"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456\"", ".py"));
        assert!(is_comment_line("  # token = \"secret_value_here\"", ".py"));
        // Non-comment lines are NOT skipped
        assert!(!is_comment_line("const api_key = \"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456\"", ".ts"));
        assert!(!is_comment_line("api_key = \"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456\"", ".py"));
    }
}
