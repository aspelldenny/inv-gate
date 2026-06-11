// src/gate.rs — Orchestrator port of golden/security-gate.sh --mechanical-only branch.
//
// PARITY CONTRACT: every section cite golden/security-gate.sh:<line>.
// Accumulator semantics: run all sections, count PASS/FAIL/WARN, exit 1 iff FAIL>0.
// DO NOT add parallel execution, change section order, or alter output wording without
// a separate phiếu + Tầng 1 review (security surface — CLAUDE.md).

use std::io::{self, Write};

// golden/security-gate.sh:18-21 — PASS/FAIL/WARN counters + FAILED_INVS list
struct State {
    pass: u32,
    fail: u32,
    warn: u32,
    failed_invs: Vec<String>,
}

impl State {
    fn new() -> Self {
        State {
            pass: 0,
            fail: 0,
            warn: 0,
            failed_invs: Vec::new(),
        }
    }
}

// golden/security-gate.sh:23-37 — run() wrapper: print header + PASS/FAIL + blank line,
// increment counters. Returns exit code of inner fn (0=pass, non-0=fail).
fn run_section<F>(state: &mut State, inv: &str, desc: &str, f: F)
where
    F: FnOnce() -> i32,
{
    // golden/security-gate.sh:27
    println!("--- {}: {} ---", inv, desc);
    let code = f();
    if code == 0 {
        // golden/security-gate.sh:29-30
        println!("  PASS");
        state.pass += 1;
    } else {
        // golden/security-gate.sh:32-34
        println!("  FAIL");
        state.fail += 1;
        state.failed_invs.push(inv.to_string());
    }
    // golden/security-gate.sh:36 — blank line after each section
    println!();
}

/// `inv-gate gate --all` — parity port of golden/security-gate.sh --mechanical-only branch.
///
/// Returns 0 (all pass) or 1 (any fail). Exit 2 is clap-only (src/main.rs).
pub fn run() -> i32 {
    let mut state = State::new();

    // golden/security-gate.sh:54-55 — INV-001: port-bind check (in-process, not subprocess)
    run_section(&mut state, "INV-001", "No host-bind 0.0.0.0 except nginx 80/443", || {
        crate::checks::port::run()
    });

    // golden/security-gate.sh:57-72 — INV-002: no :latest tag (inline bash)
    run_section(&mut state, "INV-002", "No :latest tag (except umami/portainer exception)", || {
        inv_002()
    });

    // golden/security-gate.sh:74-86 — INV-003: no real secret in .env.example (inline bash)
    run_section(&mut state, "INV-003", "No real secret value in .env.example", || {
        inv_003()
    });

    // golden/security-gate.sh:88-111 — INV-004: .env.* gitignored + never committed (inline bash)
    run_section(&mut state, "INV-004", ".env.{production,staging,backup,local} gitignored + never committed", || {
        inv_004()
    });

    // golden/security-gate.sh:112-123 — INV-005: Sentry config scrubs Authorization (inline bash)
    run_section(&mut state, "INV-005", "Sentry config has beforeSend/beforeBreadcrumb scrubber", || {
        inv_005()
    });

    // golden/security-gate.sh:125-136 — INV-006: astro-service CORS not wildcard (inline bash)
    run_section(&mut state, "INV-006", "astro-service CORS not wildcard", || {
        inv_006()
    });

    // golden/security-gate.sh:169-174 — INV-007: skip entirely in --mechanical-only
    // "else --mechanical-only: skip INV-007 entirely (don't even WARN — pre-commit don't need it)"
    // → ZERO output in mechanical-only mode

    // golden/security-gate.sh:176-193 — INV-008: internal services use expose: not ports: (inline Python→Rust)
    run_section(&mut state, "INV-008", "Internal services use expose: not ports:", || {
        inv_008()
    });

    // golden/security-gate.sh:195-197 — INV-009: secrets check (in-process, not subprocess)
    run_section(&mut state, "INV-009", "No hardcoded secret in src/ + astro-service/ source files", || {
        crate::checks::secrets::run()
    });

    // golden/security-gate.sh:199-201 — INV-010: runtime secrets check (in-process, not subprocess)
    run_section(&mut state, "INV-010", "No secret in runtime state + infra files", || {
        crate::checks::runtime::run()
    });

    // golden/security-gate.sh:203-209 — Summary
    // golden:204
    println!("====================================");
    // golden:205
    println!("Security gate: {} passed, {} failed, {} warnings", state.pass, state.fail, state.warn);
    if state.fail > 0 {
        // golden:206-208
        let inv_list: Vec<&str> = state.failed_invs.iter().map(|s| s.as_str()).collect();
        println!("Failed invariants: {}", inv_list.join(" "));
        // golden:208-209
        return 1;
    }
    // golden:210
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline check private fns — each ports one bash function from golden
// golden/security-gate.sh:<cite-range>
// ─────────────────────────────────────────────────────────────────────────────

/// INV-002 — no :latest tag (except umami/portainer exception)
/// golden/security-gate.sh:58-71
fn inv_002() -> i32 {
    // golden:60 — scan these compose files
    let compose_files = &[
        "docker-compose.yml",
        "astro-service/docker-compose.yml",
        "docker-compose.dev.yml",
    ];

    let mut findings: Vec<String> = Vec::new();

    for fname in compose_files {
        let content = match std::fs::read_to_string(fname) {
            Ok(c) => c,
            Err(_) => continue, // golden: 2>/dev/null — skip missing files silently
        };
        for line in content.lines() {
            // golden:60 — grep -E '^\s+image:.*:latest$'
            let trimmed = line.trim();
            if trimmed.starts_with("image:") && line.trim_end().ends_with(":latest") {
                // golden:62-63 — Known exceptions: umami postgresql-latest, portainer latest
                // grep -vE 'ghcr\.io/umami-software/umami:postgresql-latest|portainer/portainer-ce:latest'
                if line.contains("ghcr.io/umami-software/umami:postgresql-latest")
                    || line.contains("portainer/portainer-ce:latest")
                {
                    continue;
                }
                findings.push(line.to_string());
            }
        }
    }

    if findings.is_empty() {
        // golden:65 — return 0
        0
    } else {
        // golden:68-69 — echo remaining, return 1
        for f in &findings {
            println!("{}", f);
        }
        1
    }
}

/// INV-003 — no real secret value in .env.example
/// golden/security-gate.sh:75-85
fn inv_003() -> i32 {
    // golden:76-77 — grep -E '^[A-Z_]+=[^#[:space:]]' .env.example 2>/dev/null
    let content = match std::fs::read_to_string(".env.example") {
        Ok(c) => c,
        Err(_) => return 0, // 2>/dev/null → missing file = no violations
    };

    // golden:77-78 — allowlist: patterns that are OK even if they match the base regex
    // grep -vE '=postgresql://\.\.\.|=Soul Signature|=c993dc1e|=http://soulsign-|=http://localhost|=http://[0-9]|=google/gemini|=anthropic/claude|=1$'
    let allowlist_patterns = [
        "=postgresql://...",
        "=Soul Signature",
        "=c993dc1e",
        "=http://soulsign-",
        "=http://localhost",
        "=1",
    ];

    let mut findings: Vec<String> = Vec::new();

    for line in content.lines() {
        // golden:76 — match ^[A-Z_]+=[^#[:space:]]
        // Must start with uppercase letters/underscore, then =, then non-# non-space char
        let bytes = line.as_bytes();
        let mut i = 0;
        // Check leading [A-Z_]+
        while i < bytes.len() && (bytes[i].is_ascii_uppercase() || bytes[i] == b'_') {
            i += 1;
        }
        if i == 0 || i >= bytes.len() || bytes[i] != b'=' {
            continue;
        }
        // After '=', must be non-# non-space
        if i + 1 >= bytes.len() {
            continue;
        }
        let after_eq = bytes[i + 1];
        if after_eq == b'#' || after_eq == b' ' || after_eq == b'\t' {
            continue;
        }

        // golden:77-78 — apply allowlist (grep -v)
        let mut allowed = false;
        for pat in &allowlist_patterns {
            if line.contains(pat) {
                allowed = true;
                break;
            }
        }
        // golden:77 extra patterns with regex: =http://[0-9], =google/gemini, =anthropic/claude
        if !allowed {
            let value_part = &line[i + 1..];
            if value_part.starts_with("http://")
                && value_part[7..].chars().next().map_or(false, |c| c.is_ascii_digit())
            {
                allowed = true;
            } else if value_part.starts_with("google/gemini") || value_part.starts_with("anthropic/claude") {
                allowed = true;
            }
        }

        if !allowed {
            findings.push(line.to_string());
        }
    }

    if findings.is_empty() {
        0
    } else {
        for f in &findings {
            println!("{}", f);
        }
        1
    }
}

/// INV-004 — .env.{production,staging,backup,local} gitignored + never committed
/// golden/security-gate.sh:89-110
fn inv_004() -> i32 {
    // golden:91-99 — check .gitignore for each env file
    let env_files = &["production", "staging", "backup", "local"];
    let mut missing: Vec<String> = Vec::new();

    let gitignore_content = std::fs::read_to_string(".gitignore").unwrap_or_default();

    for f in env_files {
        // golden:93 — Accept exact match OR glob pattern covering this file
        // grep -qE "^\.env\.${f}$|^\.env\*\.${f}$"
        let exact = format!(".env.{}", f);
        let glob_pat = format!(".env*.{}", f);
        let found = gitignore_content.lines().any(|line| {
            line == exact.as_str() || line == glob_pat.as_str()
        });
        if !found {
            missing.push(format!(".env.{}", f));
        }
    }

    if !missing.is_empty() {
        // golden:97-99
        println!("Missing in .gitignore: {}", missing.join(" "));
        return 1;
    }

    // golden:101-108 — git log history check
    // git log --all --diff-filter=A --name-only 2>/dev/null | grep -E '\.env\.(production|staging|backup|local)$'
    let git_out = std::process::Command::new("git")
        .args([
            "log", "--all", "--diff-filter=A", "--name-only",
        ])
        .stderr(std::process::Stdio::null())
        .output();

    match git_out {
        Ok(output) => {
            let log_text = String::from_utf8_lossy(&output.stdout);
            let leaked: Vec<&str> = log_text
                .lines()
                .filter(|line| {
                    line.ends_with(".env.production")
                        || line.ends_with(".env.staging")
                        || line.ends_with(".env.backup")
                        || line.ends_with(".env.local")
                })
                .collect();
            if !leaked.is_empty() {
                // golden:105-107
                println!("Historic leak detected:");
                for l in leaked {
                    println!("{}", l);
                }
                return 1;
            }
        }
        Err(_) => {} // golden: 2>/dev/null — git not available → no leak
    }

    // golden:109
    0
}

/// INV-005 — Sentry config has beforeSend/beforeBreadcrumb scrubber
/// golden/security-gate.sh:113-122
fn inv_005() -> i32 {
    // golden:115-116 — grep -rnE "beforeBreadcrumb|beforeSend" src/lib/sentry.ts sentry.*.config.* 2>/dev/null
    let targets: Vec<String> = {
        let mut v = vec!["src/lib/sentry.ts".to_string()];
        // golden:116 — sentry.*.config.* glob
        if let Ok(entries) = std::fs::read_dir(".") {
            let mut sentry_configs: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with("sentry.") && name.contains(".config.") {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
            sentry_configs.sort();
            v.extend(sentry_configs);
        }
        v
    };

    let mut found_any = false;
    for target in &targets {
        let content = match std::fs::read_to_string(target) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if content.contains("beforeBreadcrumb") || content.contains("beforeSend") {
            found_any = true;
            break;
        }
    }

    if found_any {
        // golden:121 — return 0
        0
    } else {
        // golden:118-119
        println!("No beforeSend/beforeBreadcrumb handler found in Sentry config files");
        1
    }
}

/// INV-006 — astro-service CORS not wildcard
/// golden/security-gate.sh:126-135
fn inv_006() -> i32 {
    // golden:127-128 — grep -nE "origins.*\*|allow_origin.*\*|CORS\(.*\*" astro-service/app.py 2>/dev/null
    let content = match std::fs::read_to_string("astro-service/app.py") {
        Ok(c) => c,
        Err(_) => return 0, // 2>/dev/null → missing = PASS
    };

    let mut findings: Vec<String> = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        // golden:127 — grep patterns
        let matched = (line.contains("origins") && line.contains('*'))
            || (line.contains("allow_origin") && line.contains('*'))
            || (line.contains("CORS(") && line.contains('*'));
        if matched {
            // golden uses -n flag → include line number
            findings.push(format!("{}:{}", idx + 1, line));
        }
    }

    if findings.is_empty() {
        // golden:133 — return 0
        0
    } else {
        // golden:131-132
        for f in &findings {
            println!("{}", f);
        }
        1
    }
}

/// INV-008 — internal services use expose: not ports: (Rust-native YAML parse, no python3)
/// golden/security-gate.sh:177-192 (inline Python → ported to Rust, no python3 subprocess)
///
/// INV-008 devation note: golden uses `python3 -c "import yaml, sys; ..."`.
/// Rust port: manual line-based parse of docker-compose.yml services block.
/// Parity: same service list, same output format, same exit codes.
/// No `serde_yaml` dep — manual line-based state machine (TIDAK dep mới, phiếu constraint).
fn inv_008() -> i32 {
    // golden:181 — open docker-compose.yml
    let content = match std::fs::read_to_string("docker-compose.yml") {
        Ok(c) => c,
        Err(_) => {
            // golden: python yaml.safe_load — FileNotFoundError → uncaught → exit 1 + traceback
            // Rust: mirror failure semantics (no file → can't check → fail safe)
            // However since golden exits 1 on any exception, we match that
            // Actually, looking at the pins: both dirty/clean have no violations (fixture services
            // are 'app'/'nginx', neither is in internal list). Missing file → PASS (no violations
            // to report) in the fixture context. But fail-safe is correct for missing compose.
            // We use exit 1 to match golden's python exception behavior.
            let stderr = io::stderr();
            let mut err = stderr.lock();
            let _ = writeln!(err, "docker-compose.yml: file not found");
            return 1;
        }
    };

    // golden:182 — internal services list
    let internal = &[
        "nextjs",
        "postgres",
        "astro-service",
        "umami-db",
        "nextjs-staging",
        "postgres-staging",
    ];

    // Manual YAML parse: find service names under `services:`, check for `ports:` directive.
    // golden:183-189 — for svc in internal: cfg = data.get('services',{}).get(svc,{}); if 'ports' in cfg
    //
    // Simple line-based state machine:
    // - Find `services:` block
    // - Under each service name, detect `ports:` key
    let mut violations: Vec<String> = Vec::new();
    let mut in_services = false;
    let mut current_service: Option<String> = None;

    for line in content.lines() {
        if line.trim_end() == "services:" {
            in_services = true;
            continue;
        }
        if !in_services {
            continue;
        }

        // Detect top-level service name (2-space indent key with colon, no value)
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let leading_spaces = line.len() - line.trim_start().len();

        // A service name under services: has exactly 2 leading spaces and ends with ':'
        if leading_spaces == 2 {
            // Check if this is a new section (non-services key at indent 0 would break out,
            // but indent 2 under services: = service name)
            let key = trimmed.trim_end_matches(':').trim();
            if !key.is_empty() && !key.starts_with('#') {
                current_service = Some(key.to_string());
            }
            continue;
        }

        // If we see a key at indent 0, we've left `services:` block
        if leading_spaces == 0 && !trimmed.starts_with('#') {
            in_services = false;
            current_service = None;
            continue;
        }

        // Check for `ports:` key inside a service that is in internal list
        if let Some(ref svc) = current_service {
            if internal.contains(&svc.as_str()) {
                // golden:185 — if 'ports' in cfg
                let field = trimmed.trim_start().split(':').next().unwrap_or("").trim();
                if field == "ports" {
                    violations.push(format!(
                        "{}: has ports: directive (should be expose:)",
                        svc
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        // golden: sys.exit(0) path (implicitly — no violations)
        0
    } else {
        // golden:188-189 — print violations + sys.exit(1)
        for v in &violations {
            println!("{}", v);
        }
        1
    }
}
