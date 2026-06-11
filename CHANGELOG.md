# Changelog

Format loosely follows Keep a Changelog.

## [Unreleased] — P004 check port + schema — 2026-06-11

### Added
- `src/checks/port.rs` — INV-001 port of `golden/check-port-bind.py` (parity 1:1). COMPOSE_FILES (3 hardcoded paths — order is output order), ALLOWED_PUBLIC set (`80:80`, `443:443`), 4-layer line-based parse mechanism (PORT_LINE_RE / numeric filter / is_in_ports_block backward-walk / classify 2/3/N-part), WARN-stderr on missing file, output format `{fname}:{lineno}: INV-001 violated -- {reason}` — all verbatim with `golden:line` citations. Non-UTF-8: error-path exit non-zero (no panic-101).
- `src/checks/schema.rs` — Prisma schema-safety port of `golden/check-schema-safety.sh` (64 LOC bash, `set -u`, NO `set -e`) — first bash script ported. ALLOW_DATA_LOSS bypass (exact string `"true"`, case-sensitive, em-dash in bypass echo), 3-step git fallback chain via `std::process::Command` (no git2 dep), header-skip + destructive grep pipeline, 6-branch table (A bypass / B schema missing / C no-diff / D diff-safe / E destructive / F not-a-repo) — all verbatim with `golden:line` citations.
- `src/main.rs` — variants `Port` + `Schema` added to `CheckCommand`; dispatch to `checks::port::run()` / `checks::schema::run()`.
- `src/checks/mod.rs` — `pub mod port;` + `pub mod schema;` added.
- `tests/parity_port.rs` — 2 parity tests (dirty/clean): stdout+stderr BYTE-EXACT vs `tests/golden/pins/` (port stderr pins ARE non-empty — 2-line WARN per fixture); 12 mandatory unit probes (a-g, f1-f6).
- `tests/parity_schema.rs` — 2 parity tests (dirty/clean): env-reconstruction 2-commit git repo per repin.sh:34-88, `env_remove("ALLOW_DATA_LOSS")` hermetic; stdout BYTE-EXACT vs pins; stderr empty; 7 mandatory unit probes (a-g).

### Stderr contract (P004 — differs from P002/P003)
Port check emits WARN lines to stderr for each missing compose file — `port--{dirty,clean}.stderr.txt` pins are 108 bytes each (2 lines). Parity asserts stderr BYTE-EXACT (not empty). Schema check stderr is empty (git stderr suppressed via `Stdio::null()`). Per-check stderr contract documented here for P005 `gate --all` aggregator.

### Bash-port precedent + git-via-Command pattern
`check-schema-safety.sh` is the first bash script ported. Pattern: `std::process::Command` for git calls, `Stdio::null()` for stderr suppression (mirrors `2>/dev/null`), `|| true` grep equivalent = DESTRUCTIVE vector empty is not an error. P005 should reuse this pattern.

### Fallback chain deviation note (O1.2 — parity-first, improve later)
`golden/check-schema-safety.sh:33` uses SHA `4b825dc8669f8c0` (15 chars) — NOT the standard empty-tree SHA `4b825dc642cb6eb9a060e54bf8d69288fbee4904`. Both git calls fail on 1-commit/fresh repo → `echo ""` fires. Ported AS-IS per Luật chơi 1 (parity before improvement). Improvement candidate added to BACKLOG.

## [Unreleased] — P003 check runtime — 2026-06-11

### Added
- `src/checks/runtime.rs` — INV-010 port of `golden/check-runtime-secrets.py` (parity 1:1). RUNTIME_FILES (`.git/config`, `.mcp.json`, `.claude/settings.local.json`), INFRA_GLOBS (read_dir+sort — Python 3.12+ sorted glob, no `glob` crate), INFRA_TOP_LEVEL, 15 prefix patterns + 1 generic, allowlist (golden:119-135), SKIP_EXTENSIONS, masking, output format — all verbatim with `golden:line` citations. Sub-mech F (dotfile token leak, golden:39) classification noted in Discovery.
- `src/main.rs` — variant `Runtime` added to `CheckCommand`; dispatch to `checks::runtime::run()`.
- `src/checks/mod.rs` — `pub mod runtime;` added.
- `tests/parity_runtime.rs` — 2 parity tests (dirty/clean) byte-exact vs `tests/golden/pins/`; 14 mandatory unit tests (V2): all pattern classes (a-g) including proof tests g1-g4 for db-conn equivalence.

### Pattern deviation (V2 — O1.1, security surface — Giám sát review required)
4 `db-conn-*` patterns (`golden:100-103`) contained negative lookahead `(?!\$)` — unsupported by `regex` crate. Resolution: **drop exactly the token `(?!\$)`**, keep every other character verbatim. Equivalence: the immediately-following char class `[^@/\s\$]{8,}` already excludes `$`, so the lookahead is redundant (the formal language accepted by both versions is identical). Proof: 15/15 adversarial oracle cases (Debate Log Turn 2) + proof tests g1-g4 (green). No dep added, no behavior change.

## [Unreleased] — P002 check secrets — 2026-06-11

### Added
- `src/checks/secrets.rs` — INV-009 port of `golden/check-hardcoded-secrets.py` (parity 1:1). All patterns, allowlist (8 entries), skip rules (test-file / path-level / comment-line), masking, and output format preserved verbatim with `golden:line` citations.
- `src/main.rs` — clap derive CLI skeleton: `inv-gate check secrets` subcommand. Exit 2 on unknown subcommand/flag (anchor #12 verified).
- `src/checks/mod.rs` — checks module layout (1 module / INV, per ARCHITECTURE.md).
- `tests/parity_secrets.rs` — 2 parity tests (dirty/clean) byte-exact vs `tests/golden/pins/`; 2 mandatory unit tests (V2): `should_skip_path(src/generated/)` + comment-line skip.

## [Unreleased] — P001 pin golden oracle — 2026-06-11

### Added
- `tests/golden/fixtures/` — fixture set (dirty/clean) for 4 INV checks + orchestrator gate.
- `tests/golden/repin.sh` — hermetic pin harness: temp git repos, fixed dates, path normalization.
- `tests/golden/pins/` — frozen stdout/stderr/exit codes per check × fixture.
- `tests/golden/MANIFEST.md` — fixture provenance, invocation contracts, unit-spec table (F06).
- `docs/DISCOVERIES.md` + `docs/discoveries/P001.md` — discovery index.

### Deviation from exit-code contract (document per CLAUDE.md rule)
The 4 individual check scripts (`check-hardcoded-secrets.py`, `check-port-bind.py`,
`check-runtime-secrets.py`, `check-schema-safety.sh`) have **no exit-2 mode**. They
exit 0 (clean) or 1 (violations) only. Exit 2 (`usage/config error`) applies ONLY to
`security-gate.sh` (unknown flag path `golden/security-gate.sh:14`). This is the
oracle reality — deviation noted per anchor #5/#8.

## v0.0.0 — sos adopt — 2026-06-11

- Spine retrofitted via `sos adopt`.
