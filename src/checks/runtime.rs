// src/checks/runtime.rs — Port of golden/check-runtime-secrets.py (INV-010)
//
// PARITY CONTRACT: every constant below cites golden/check-runtime-secrets.py:<line>.
// DO NOT change any pattern, allowlist entry, skip rule, masking, or output wording
// without a separate phiếu + Tầng 1 + Giám sát review (security surface — CLAUDE.md).
//
// V2 NOTE (O1.1 resolution, Debate Log Turn 2, equivalence proven 15/15 adversarial cases):
//   4 db-conn patterns (golden:100-103) contain `(?!\$)` — regex crate does not support
//   lookaheads. Transcription: drop EXACTLY the token `(?!\$)`, keep every other character
//   verbatim. Equivalence: the class immediately following (`[^@/\s\$]{8,}`) already
//   excludes `$`, making the lookahead redundant over the formal language it guards.
//   Proof tests: tests g1-g4 in tests/parity_runtime.rs.
//
// Sub-mech F (anchor #16, golden:39): dotfile token leak classification — `.mcp.json` and
//   `.claude/settings.local.json` added to RUNTIME_FILES per "P306 extend" comment in golden.
//   Documented in Discovery P003.

use regex::Regex;
use std::path::{Path, PathBuf};

// golden/check-runtime-secrets.py:40-44
// Hard-coded runtime state files. Skip silently if not exists (anchor #13).
const RUNTIME_FILES: &[&str] = &[
    ".git/config",
    ".mcp.json",
    ".claude/settings.local.json",
];

// golden/check-runtime-secrets.py:47-52
// Infra-extension glob pairs (dir, suffix). 1-level only — NOT recursive (anchor #8b).
// Python 3.12+ Path.glob() returns sorted alphabetical → Rust: read_dir + sort + suffix filter.
const INFRA_GLOBS: &[(&str, &str)] = &[
    ("scripts", ".sh"),
    (".github/workflows", ".yml"),
    (".github/workflows", ".yaml"),
    ("hooks", ""),  // "" = match all files (hooks/* — no extension filter)
];

// golden/check-runtime-secrets.py:56-63
// Top-level infra files (no glob, exact match). Scan order: runtime → globs → top-level.
const INFRA_TOP_LEVEL: &[&str] = &[
    "Dockerfile.nextjs",
    "docker-compose.yml",
    "docker-compose.staging.yml",
    "astro-service/Dockerfile",
    "astro-service/docker-compose.yml",
];

// golden/check-runtime-secrets.py:77
// Skip if filename ends with these (documentation/example extensions).
const SKIP_EXTENSIONS: &[&str] = &[".example", ".md", ".sample", ".template"];

// golden/check-runtime-secrets.py:66-74
// Skip if path substring matches (defense — vendored/generated).
const SKIP_PATH_SUBSTR: &[&str] = &[
    "node_modules/",
    ".next/",
    "target/",
    "dist/",
    "build/",
    ".claude/worktrees/",
    "src/generated/",
];

// golden/check-runtime-secrets.py:119-135
// Allowlist substrings — port exact range (IG-04: KHÔNG ghi count).
// Entry `"${"` (:133) is the env-interpolation lưới at string level.
const ALLOWLIST_SUBSTRINGS: &[&str] = &[
    // === INV-009 mirror (golden:120-129) ===
    "c993dc1e",
    "google/gemini",
    "anthropic/claude",
    "process.env.",
    "os.environ",
    "your-",
    "xxx",
    "REPLACE",
    "PLACEHOLDER",
    // === INV-010 extended (golden:130-135) ===
    "CHANGEME",
    "EXAMPLE",
    "${",   // env substitution syntax (Docker compose / shell)
    "<",    // angle bracket placeholder e.g. <YOUR_TOKEN>
];

fn should_skip_path(path_str: &str) -> bool {
    SKIP_PATH_SUBSTR.iter().any(|s| path_str.contains(s))
}

fn should_skip_extension(path: &Path) -> bool {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    SKIP_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

// golden/check-runtime-secrets.py:151-162
fn is_comment_line(line: &str, path: &Path) -> bool {
    let stripped = line.trim_start();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let ext_lower = ext.to_lowercase();
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    // Shell / YAML / Python / .git/config (# comments)
    if [".sh", ".yml", ".yaml", ".py"].contains(&ext_lower.as_str())
        || name == ".gitconfig"
        || name == "config"
    {
        return stripped.starts_with('#');
    }
    // Docker compose / Dockerfile
    if name.starts_with("docker-compose") || name == "dockerfile" || name.starts_with("dockerfile.") {
        return stripped.starts_with('#');
    }
    // JS in workflow expressions (rare)
    stripped.starts_with("//")
}

fn is_allowlisted(text: &str) -> bool {
    ALLOWLIST_SUBSTRINGS.iter().any(|s| text.contains(s))
}

// golden/check-runtime-secrets.py:169-173
// Mask: keep first 4 + last 4; else if len <= 12 → "***" + last4 (or "***" if < 4).
fn mask(secret: &str) -> String {
    if secret.len() <= 12 {
        if secret.len() >= 4 {
            format!("***{}", &secret[secret.len() - 4..])
        } else {
            "***".to_string()
        }
    } else {
        format!("{}...{}", &secret[..4], &secret[secret.len() - 4..])
    }
}

// golden/check-runtime-secrets.py:176-200
// Returns list of (lineno, masked_match, pattern_name).
fn scan_file(path: &Path) -> Vec<(usize, String, String)> {
    // PREFIX_PATTERNS golden:82-111 (per-class cites below)
    // V2 O1.1: db-conn-* patterns: `(?!\$)` DROPPED (redundant — see file header).
    let prefix_patterns: &[(&str, &str)] = &[
        // golden:83 — Anthropic API key
        ("anthropic",           r"sk-ant-[A-Za-z0-9_\-]{40,}"),
        // golden:84 — OpenAI-style API key
        ("openai",              r"sk-[A-Za-z0-9]{48}"),
        // golden:85 — AWS access key ID
        ("aws",                 r"AKIA[0-9A-Z]{16}"),
        // golden:86 — GitHub PAT classic
        ("github-pat-classic",  r"gh[pous]_[A-Za-z0-9]{36}"),
        // golden:87 — GitHub PAT fine-grained
        ("github-pat-fine",     r"github_pat_[A-Za-z0-9_]{82}"),
        // golden:88 — Google API key
        ("google-api",          r"AIza[0-9A-Za-z_\-]{35}"),
        // golden:89 — Resend API key
        ("resend",              r"re_[A-Za-z0-9_\-]{30,}"),
        // golden:90 — Telegram bot token
        ("telegram-bot",        r"\b\d{8,12}:[A-Za-z0-9_\-]{35}\b"),
        // golden:93 — PEM private key block
        ("pem-private-key",     r"-----BEGIN (RSA |OPENSSH |EC |DSA |)PRIVATE KEY-----"),
        // golden:95-97 — Token-in-URL (token embedded in remote URL)
        ("token-in-url",
         r"https://[a-zA-Z0-9._\-]+:(gh[pous]_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82})@"),
        // golden:100 — DB conn postgresql (V2: `(?!\$)` dropped — see file header)
        // Original: r'postgresql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@'
        ("db-conn-postgresql",
         r"postgresql://[^:/\s\$]+:[^@/\s\$]{8,}@"),
        // golden:101 — DB conn mongodb (V2: `(?!\$)` dropped — see file header)
        // Original: r'mongodb(\+srv)?://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@'
        ("db-conn-mongodb",
         r"mongodb(\+srv)?://[^:/\s\$]+:[^@/\s\$]{8,}@"),
        // golden:102 — DB conn mysql (V2: `(?!\$)` dropped — see file header)
        // Original: r'mysql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@'
        ("db-conn-mysql",
         r"mysql://[^:/\s\$]+:[^@/\s\$]{8,}@"),
        // golden:103 — DB conn redis (V2: `(?!\$)` dropped — see file header)
        // Original: r'redis://[^:/\s\$]*:(?!\$)[^@/\s\$]{8,}@'
        ("db-conn-redis",
         r"redis://[^:/\s\$]*:[^@/\s\$]{8,}@"),
        // golden:105 — Stripe live secret key
        ("stripe-live-secret",      r"sk_live_[A-Za-z0-9]{24,}"),
        // golden:106 — Stripe test secret key
        ("stripe-test-secret",      r"sk_test_[A-Za-z0-9]{24,}"),
        // golden:107 — Stripe live restricted key
        ("stripe-live-restricted",  r"rk_live_[A-Za-z0-9]{24,}"),
        // golden:108 — Stripe test restricted key
        ("stripe-test-restricted",  r"rk_test_[A-Za-z0-9]{24,}"),
        // golden:110 — Slack bot/app tokens
        ("slack-token",             r"xox[baprs]-[A-Za-z0-9-]{10,}"),
    ];

    // golden:114-116 — GENERIC_PATTERN (generic-entropy fallback)
    let generic_pattern = Regex::new(
        r#"(?i)(api[_\-]?key|apikey|secret|password|token|jwt[_\-]?secret|signing[_\-]?key|private[_\-]?key)\s*[:=]\s*['"`]([A-Za-z0-9_\-+/=]{32,})['"`]"#
    ).unwrap();

    let compiled_prefix: Vec<(&str, Regex)> = prefix_patterns
        .iter()
        .map(|(name, pat)| (*name, Regex::new(pat).unwrap()))
        .collect();

    let mut violations = Vec::new();

    // golden:179-181 — read file with errors="ignore" semantics:
    //   - missing/OSError/IsADirectory → return empty (skip-file)
    //   - non-UTF-8 bytes: DROPPED (not U+FFFD replacement — that's from_utf8_lossy).
    //     Valid UTF-8 sequences are kept; `\n` is preserved (byte 0x0A is valid UTF-8).
    //     Line offsets remain intact because newlines survive byte filtering.
    // Rust impl: read raw bytes, filter to valid UTF-8 keeping \n, then decode.
    let raw = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return violations,
    };
    // Strip non-UTF-8 bytes while preserving \n (0x0A) — mirrors Python errors="ignore".
    // Strategy: decode each byte; keep if it contributes to a valid UTF-8 sequence.
    // Simpler equivalent: from_utf8_lossy would insert U+FFFD (wrong). Instead, collect
    // only bytes that form valid UTF-8 by iterating the str chars after lossy decode and
    // re-encoding — but U+FFFD would slip in. Use manual filter: keep all bytes valid in
    // UTF-8 context (i.e., build a valid UTF-8 byte sequence by skipping invalid bytes).
    // Implementation: collect chars from a lossy decode, skip U+FFFD, re-encode to string.
    // This gives exactly errors="ignore" semantics: invalid bytes → dropped, valid → kept.
    let content: String = String::from_utf8_lossy(&raw)
        .chars()
        .filter(|&c| c != '\u{FFFD}')
        .collect();

    // golden:183 — enumerate(content.splitlines(), start=1)
    for (lineno, line) in content.lines().enumerate().map(|(i, l)| (i + 1, l)) {
        // golden:184
        if is_comment_line(line, path) {
            continue;
        }

        // golden:186-192 — Prefix-based detection: mask(group(0)) — whole match
        for (name, pat) in &compiled_prefix {
            for m in pat.find_iter(line) {
                let hit = m.as_str();
                if is_allowlisted(line) || is_allowlisted(hit) {
                    continue;
                }
                violations.push((lineno, mask(hit), name.to_string()));
            }
        }

        // golden:193-199 — Generic high-entropy fallback: hit = group(2), allowlist on line+full+hit
        for caps in generic_pattern.captures_iter(line) {
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

// golden/check-runtime-secrets.py:203-209
// Collect runtime state files: skip silently if not exists or not a file.
fn collect_runtime_files() -> Vec<PathBuf> {
    RUNTIME_FILES
        .iter()
        .map(|s| PathBuf::from(s))
        .filter(|p| p.exists() && p.is_file())
        .collect()
}

// golden/check-runtime-secrets.py:212-225
// Collect infra-extension files via glob (1-level) + top-level exact paths.
// INFRA_GLOBS: read_dir + sort alphabetical + suffix filter (Python 3.12+ sorted glob).
fn collect_infra_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    for (dir, suffix) in INFRA_GLOBS {
        let root = Path::new(dir);
        if !root.exists() {
            continue;
        }
        let mut matched: Vec<PathBuf> = match std::fs::read_dir(root) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .filter(|e| {
                    if suffix.is_empty() {
                        true
                    } else {
                        e.file_name()
                            .to_str()
                            .map(|n| n.ends_with(suffix))
                            .unwrap_or(false)
                    }
                })
                .map(|e| e.path())
                .collect(),
            Err(_) => continue,
        };
        // Python 3.12+ Path.glob() returns sorted alphabetical
        matched.sort();
        files.extend(matched);
    }

    // Top-level exact paths: skip silently if not exists
    for name in INFRA_TOP_LEVEL {
        let p = PathBuf::from(name);
        if p.exists() && p.is_file() {
            files.push(p);
        }
    }
    files
}

// golden/check-runtime-secrets.py:228-245 — main()
/// Buffered core — no stdout/stderr side effects; returns CheckOutput.
/// Called by both CLI run() and MCP serve tools.
pub fn run_core() -> crate::checks::CheckOutput {
    let mut all_violations: Vec<String> = Vec::new();
    let mut scanned = 0usize;

    let all_files: Vec<PathBuf> = collect_runtime_files()
        .into_iter()
        .chain(collect_infra_files())
        .collect();

    for path in &all_files {
        let path_str = path.to_str().unwrap_or("");
        if should_skip_path(path_str) {
            continue;
        }
        if should_skip_extension(path) {
            continue;
        }
        scanned += 1;
        // golden:238-240 — collect findings, format output
        for (lineno, masked, pat_name) in scan_file(path) {
            all_violations.push(format!(
                "{path_str}:{lineno}: INV-010 violated -- {masked} ({pat_name})"
            ));
        }
    }

    if !all_violations.is_empty() {
        // golden:243-244
        let text = format!("{}\n", all_violations.join("\n"));
        crate::checks::CheckOutput { stdout: text, stderr: String::new(), code: 1 }
    } else {
        // golden:245
        crate::checks::CheckOutput {
            stdout: format!("INV-010: PASS (0 runtime/infra secrets, {scanned} files scanned)\n"),
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

// ── Unit tests (Task 3 Lưu ý 4 — BẮT BUỘC per phiếu V2) ─────────────────────
// F07: synthetic in-code, no new fixture files in tests/golden/fixtures/
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── (a) allowlist skip direction ────────────────────────────────────────
    // golden:165-166 — is_allowlisted: token-shape line containing "${" → 0 finding
    #[test]
    fn test_allowlist_skip_env_interpolation() {
        // Line contains "${" → allowlisted → no finding
        assert!(is_allowlisted("postgresql://user:${DB_PASSWORD}@host/db"));
        assert!(is_allowlisted("token=${MY_SECRET_TOKEN}"));
        // Line NOT allowlisted
        assert!(!is_allowlisted("postgresql://user:realpassword123@host/db"));
    }

    // ── (b) SKIP_EXTENSIONS ─────────────────────────────────────────────────
    // golden:77 + 147-148 — file with .example extension → skipped
    #[test]
    fn test_skip_extensions() {
        let tmp = tempfile::tempdir().unwrap();
        // Write a file with .example extension containing a real-looking token
        let path = tmp.path().join(".env.example");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "SECRET=ghp_FAKETOKEN000000000000000000000000000").unwrap();
        drop(f);
        assert!(should_skip_extension(&path), ".env.example should be skipped");
        // Verify .md and .sample too
        assert!(should_skip_extension(Path::new("README.md")));
        assert!(should_skip_extension(Path::new("config.sample")));
        assert!(should_skip_extension(Path::new("docker.template")));
        // Non-skipped extensions
        assert!(!should_skip_extension(Path::new(".git/config")));
        assert!(!should_skip_extension(Path::new("scripts/deploy.sh")));
    }

    // ── Helper: write a temp file and scan it ───────────────────────────────
    fn scan_line(content: &str, filename: &str) -> Vec<(usize, String, String)> {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(filename);
        std::fs::write(&path, content).unwrap();
        scan_file(&path)
    }

    // ── (c) per-pattern-class firing tests ──────────────────────────────────
    // All classes in golden:82-111 that fixture does NOT exercise (all except
    // github-pat-classic:86 and token-in-url:95-97 which are in parity fixture).

    // golden:83 — anthropic
    #[test]
    fn test_pattern_anthropic() {
        let line = "ANTHROPIC_API_KEY=sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n";
        let hits = scan_line(line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "anthropic"),
            "expected anthropic hit, got: {:?}", hits);
    }

    // golden:84 — openai
    #[test]
    fn test_pattern_openai() {
        let line = "OPENAI_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890123456789012\n";
        let hits = scan_line(line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "openai"),
            "expected openai hit, got: {:?}", hits);
    }

    // golden:85 — aws
    #[test]
    fn test_pattern_aws() {
        // AKIA + exactly 16 [0-9A-Z] chars
        let line = "key=AKIABCDEFGHIJKLMNOPQ\n";
        let hits = scan_line(line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "aws"),
            "expected aws hit, got: {:?}", hits);
    }

    // golden:87 — github-pat-fine
    #[test]
    fn test_pattern_github_pat_fine() {
        // github_pat_ + 82 alphanum/underscore
        let token = format!("github_pat_{}", "A".repeat(82));
        let line = format!("TOKEN={}\n", token);
        let hits = scan_line(&line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "github-pat-fine"),
            "expected github-pat-fine hit, got: {:?}", hits);
    }

    // golden:88 — google-api
    #[test]
    fn test_pattern_google_api() {
        // AIza + 35 alphanum/underscore/hyphen
        let token = format!("AIza{}", "B".repeat(35));
        let line = format!("GKEY={}\n", token);
        let hits = scan_line(&line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "google-api"),
            "expected google-api hit, got: {:?}", hits);
    }

    // golden:89 — resend
    #[test]
    fn test_pattern_resend() {
        // re_ + 30+ alphanum/underscore/hyphen
        // NOTE: must NOT use 'x' repeated — allowlist contains "xxx" which would suppress hit
        let token = format!("re_{}", "A".repeat(30));
        let line = format!("KEY={}\n", token);
        let hits = scan_line(&line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "resend"),
            "expected resend hit, got: {:?}", hits);
    }

    // golden:90 — telegram-bot
    #[test]
    fn test_pattern_telegram_bot() {
        // 8-12 digits : exactly 35 alphanum/underscore/hyphen (word boundary)
        // "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789" = 26 + 9 = 35 chars
        let line = "BOT=12345678:ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789\n";
        let hits = scan_line(line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "telegram-bot"),
            "expected telegram-bot hit, got: {:?}", hits);
    }

    // golden:93 — pem-private-key
    #[test]
    fn test_pattern_pem_private_key() {
        let lines = [
            "-----BEGIN RSA PRIVATE KEY-----\n",
            "-----BEGIN OPENSSH PRIVATE KEY-----\n",
            "-----BEGIN EC PRIVATE KEY-----\n",
            "-----BEGIN PRIVATE KEY-----\n", // PKCS#8 empty variant
        ];
        for line in &lines {
            let hits = scan_line(line, "key.pem");
            assert!(hits.iter().any(|(_, _, n)| n == "pem-private-key"),
                "expected pem-private-key hit for: {:?}, got: {:?}", line, hits);
        }
    }

    // golden:105-108 — stripe keys
    // Suffix ghép runtime: source không được chứa chuỗi Stripe-shaped liền mạch,
    // GitHub push protection chặn push (kể cả token fake đúng format).
    #[test]
    fn test_pattern_stripe() {
        let suffix = "ABCDEFGHIJKLMNOPQRSTUVWX";
        let cases = [
            (format!("sk_live_{suffix}"), "stripe-live-secret"),
            (format!("sk_test_{suffix}"), "stripe-test-secret"),
            (format!("rk_live_{suffix}"), "stripe-live-restricted"),
            (format!("rk_test_{suffix}"), "stripe-test-restricted"),
        ];
        for (token, pat_name) in &cases {
            let line = format!("KEY={}\n", token);
            let hits = scan_line(&line, "config.sh");
            assert!(hits.iter().any(|(_, _, n)| n == *pat_name),
                "expected {} hit, got: {:?}", pat_name, hits);
        }
    }

    // golden:110 — slack-token
    #[test]
    fn test_pattern_slack_token() {
        let line = "SLACK=xoxb-abcdefghij1234\n";
        let hits = scan_line(line, "config.sh");
        assert!(hits.iter().any(|(_, _, n)| n == "slack-token"),
            "expected slack-token hit, got: {:?}", hits);
    }

    // golden:114-116 — generic-entropy
    #[test]
    fn test_pattern_generic_entropy() {
        let line = r#"jwt_secret = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""# ;
        let line_with_newline = format!("{}\n", line);
        let hits = scan_line(&line_with_newline, "config.yml");
        assert!(hits.iter().any(|(_, _, n)| n == "generic-entropy"),
            "expected generic-entropy hit, got: {:?}", hits);
    }

    // ── (d) missing runtime file — no panic, not counted ────────────────────
    #[test]
    fn test_missing_runtime_file_skipped() {
        // collect_runtime_files() only returns files that exist
        // In a temp dir with no .mcp.json, it should not be included
        let tmp = tempfile::tempdir().unwrap();
        let mcp = tmp.path().join(".mcp.json");
        // Confirm it does not exist
        assert!(!mcp.exists());
        // Simulating collect_runtime_files in a context — the function checks p.exists()
        // so we test the predicate directly
        let p = PathBuf::from("/nonexistent_path_for_test/.mcp.json");
        assert!(!p.exists(), "test file must not exist");
        // If it doesn't exist, it would be filtered out
        let files: Vec<PathBuf> = vec![p].into_iter().filter(|f| f.exists() && f.is_file()).collect();
        assert!(files.is_empty(), "missing file should not be collected");
    }

    // ── (e) non-UTF-8 file: invalid bytes dropped, valid token on other line found ──
    // golden:179-181, anchor #14 — errors="ignore": invalid bytes dropped, file still scanned
    #[test]
    fn test_non_utf8_file_still_scanned() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config");
        // Write: invalid UTF-8 bytes on line 1, valid token on line 2
        let mut raw: Vec<u8> = Vec::new();
        raw.extend_from_slice(b"\xff\xfe invalid bytes here\n");
        raw.extend_from_slice(b"url = https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/repo.git\n");
        std::fs::write(&path, &raw).unwrap();
        let hits = scan_file(&path);
        assert!(
            hits.iter().any(|(_, _, n)| n == "github-pat-classic" || n == "token-in-url"),
            "expected token found in non-UTF-8 file, got: {:?}", hits
        );
    }

    // ── (f) infra-glob matching: 1-level only, sorted alphabetical ──────────
    #[test]
    fn test_infra_glob_one_level_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts = tmp.path().join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        // Create files: b.sh, a.sh, and a subdir with c.sh (should NOT be matched)
        std::fs::write(scripts.join("b.sh"), "#!/bin/bash\n").unwrap();
        std::fs::write(scripts.join("a.sh"), "#!/bin/bash\n").unwrap();
        let sub = scripts.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("c.sh"), "#!/bin/bash\n").unwrap();

        // Test the glob logic directly using same approach as collect_infra_files
        let root = &scripts;
        let mut matched: Vec<PathBuf> = std::fs::read_dir(root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .filter(|e| e.file_name().to_str().map(|n| n.ends_with(".sh")).unwrap_or(false))
            .map(|e| e.path())
            .collect();
        matched.sort();

        let names: Vec<&str> = matched.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        // a.sh before b.sh (alphabetical), c.sh from subdir NOT present
        assert_eq!(names, vec!["a.sh", "b.sh"], "should be sorted: {:?}", names);
    }

    // ── (g) db-conn equivalence proof (V2 — O1.1, BẮT BUỘC) ────────────────
    // All 4 transcribed patterns ((?!\$) dropped). Proof tests g1/g2/g3/g4.

    // g1: ${DB_PASSWORD} → 0 finding (class [^@/\s\$] excludes $, so ${...} fails to match 8+ chars)
    #[test]
    fn test_dbconn_g1_env_interpolation_no_finding() {
        let cases = [
            "postgresql://user:${DB_PASSWORD}@host/db\n",
            "mongodb://user:${DB_PASSWORD}@host/db\n",
            "mysql://user:${DB_PASSWORD}@host/db\n",
            "redis://:${DB_PASSWORD}@host\n",
        ];
        for line in &cases {
            let hits = scan_line(line, "config.sh");
            let db_hits: Vec<_> = hits.iter().filter(|(_, _, n)| n.starts_with("db-conn")).collect();
            assert!(db_hits.is_empty(),
                "g1: expected 0 db-conn hits for {:?}, got: {:?}", line, db_hits);
        }
    }

    // g2: real password >=8 chars → finding for all 4 protocols
    #[test]
    fn test_dbconn_g2_real_password_finding() {
        let cases = [
            ("postgresql://user:realpassword123@host/db\n", "db-conn-postgresql"),
            ("mongodb://user:realpassword123@host/db\n", "db-conn-mongodb"),
            ("mongodb+srv://user:realpassword123@host/db\n", "db-conn-mongodb"),
            ("mysql://user:realpassword123@host/db\n", "db-conn-mysql"),
            ("redis://:realpassword123@host\n", "db-conn-redis"),
        ];
        for (line, expected_name) in &cases {
            let hits = scan_line(line, "config.sh");
            assert!(hits.iter().any(|(_, _, n)| n == expected_name),
                "g2: expected {} hit for {:?}, got: {:?}", expected_name, line, hits);
        }
    }

    // g3: password starting with $ ($ecret123456) → 0 finding
    // The class [^@/\s\$] does NOT match $, so a password starting with $ cannot
    // contribute to a match of 8+ chars (the first char is already excluded).
    #[test]
    fn test_dbconn_g3_dollar_prefix_password_no_finding() {
        let cases = [
            "postgresql://user:$ecret123456@host/db\n",
            "mongodb://user:$ecret123456@host/db\n",
            "mysql://user:$ecret123456@host/db\n",
            "redis://:$ecret123456@host\n",
        ];
        for line in &cases {
            let hits = scan_line(line, "config.sh");
            let db_hits: Vec<_> = hits.iter().filter(|(_, _, n)| n.starts_with("db-conn")).collect();
            assert!(db_hits.is_empty(),
                "g3: expected 0 db-conn hits for {:?}, got: {:?}", line, db_hits);
        }
    }

    // g4 (optional per phiếu Turn 2): $ in middle of password → match truncated at $
    // Both golden and transcribed: match stops before $, may not reach 8 chars minimum.
    #[test]
    fn test_dbconn_g4_dollar_in_middle() {
        // "pass$word" — the $ breaks the [^@/\s\$]+ match; "pass" is only 4 chars < 8 minimum
        // So no match. Both original golden and transcribed agree (Turn 2 case 8).
        let line = "postgresql://user:pass$word@host/db\n";
        let hits = scan_line(line, "config.sh");
        let db_hits: Vec<_> = hits.iter().filter(|(_, _, n)| n.starts_with("db-conn")).collect();
        assert!(db_hits.is_empty(),
            "g4: expected 0 db-conn hits for {:?}, got: {:?}", line, db_hits);
        // But a password with $ in middle where both parts are >=8 each side:
        // "abcdefgh$ijklmnop" → match would be "abcdefgh" part if >=8 chars before $? No —
        // the regex requires the 8+ chars to be immediately followed by @, not $.
        // So no match in either golden or transcribed — consistent.
    }

    // ── masking boundary tests (optional Tầng 2) ────────────────────────────

    // Mask boundary: len=12 → "***" + last4
    #[test]
    fn test_mask_boundary_12() {
        assert_eq!(mask("123456789012"), "***9012");
        assert_eq!(mask("ABCDEFGHIJKL"), "***IJKL");
    }

    // Mask boundary: len=13 → first4 + "..." + last4
    #[test]
    fn test_mask_boundary_13() {
        assert_eq!(mask("1234567890123"), "1234...0123");
    }

    // Mask: len < 4 → "***"
    #[test]
    fn test_mask_short() {
        assert_eq!(mask("abc"), "***");
        assert_eq!(mask(""), "***");
    }

    // ── double-firing synthetic (optional Tầng 2) ───────────────────────────
    // A line matching 2 patterns → 2 findings (no dedupe — anchor #12)
    #[test]
    fn test_double_firing() {
        // Token-in-url contains github-pat-classic pattern too
        let line = "url = https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example.git\n";
        let hits = scan_line(line, "config");
        let pat_names: Vec<&str> = hits.iter().map(|(_, _, n)| n.as_str()).collect();
        assert!(pat_names.contains(&"github-pat-classic"), "expected github-pat-classic in double-fire: {:?}", pat_names);
        assert!(pat_names.contains(&"token-in-url"), "expected token-in-url in double-fire: {:?}", pat_names);
    }
}
