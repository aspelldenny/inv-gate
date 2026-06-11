# Changelog

Format loosely follows Keep a Changelog.

## [Unreleased] — P009 — CI actions bump (Node 24) — 2026-06-11

### Changed
- `.github/workflows/release.yml`: bump `actions/checkout@v4` → `@v5` (node20 → node24).
  GitHub Node.js 20 forced migration deadline 16/06/2026 (annotation run 27356594756).
- `.github/workflows/release.yml`: bump `softprops/action-gh-release@v2` → `@v3` (node20 → node24).
  v3.0.0 release notes: only runtime change, no input renames.
- `.github/workflows/release.yml`: add `prerelease: ${{ contains(github.ref_name, '-rc') }}` to
  `with:` block of gh-release step (O1.2 resolution — latest-pointer guard). Rc tags evaluate true
  → GitHub never promotes prerelease to Latest; non-rc releases evaluate false → behavior unchanged.
- `dtolnay/rust-toolchain@stable` not changed (composite action, not in Node.js 20 annotation).
- Verify method: rc tag `v0.1.1-rc1` → CI 3 jobs green, 0 Node deprecation annotation,
  `isPrerelease=true`, latest pointer stayed `v0.1.0`, rc release+tag deleted post-verify.

## [0.1.0] — 2026-06-11

### P008 — Release CI 3-target + version freeze (this release)

- Release CI (`.github/workflows/release.yml`): 3-target matrix confirmed mirror-siblings convention
  (mac-arm64 / linux-x64 / win-x64). Chủ nhà ruling 11/06: scope = windows + macos + linux.
  BACKLOG wording `mac-x64` = lỗi diễn giải (cũ), corrected. Matrix was already correct at
  bootstrap (commit f2946c4) — Task 1 verify diff = EMPTY (zero workflow changes).
- Asset naming contract: `inv-gate-<target-triple>` (+`.exe` for win), consumed by
  `sos-kit install.sh:65 ${bin}-${TARGET}${EXT}`. EXT logic preserved verbatim.
- Version freeze F13: `Cargo.toml` version = `0.1.0` (set at bootstrap) = CHANGELOG latest release.
  No Cargo.toml bump needed — confirm only.
- Tag `v0.1.0` pushed → GitHub Release CI (3 jobs parallel) → 3 assets:
  `inv-gate-aarch64-apple-darwin`, `inv-gate-x86_64-unknown-linux-gnu`,
  `inv-gate-x86_64-pc-windows-msvc.exe`.
- Ship-test transcript: `docs/discoveries/P008.md` (acceptance (c) — CHẠY ĐƯỢC ≠ SHIP ĐƯỢC).

### P007 — gate --skip-absent — 2026-06-11

### Added (behavior change — first deviation from golden parity; method rule 3 CLAUDE.md)
- `gate --all --skip-absent` (bool flag, default off): opt-in skip for allowlisted INVs whose
  prerequisite file is absent. Allowlist: **INV-005 + INV-008 only** (hardcoded, closed — adding
  any INV requires a separate phiếu + Tầng 1 review).
  - INV-005 (Sentry scrubber): SKIP only when **both** `src/lib/sentry.ts` AND `sentry.*.config.*`
    glob are absent (guard kép — fail-closed: repo with `sentry.client.config.ts` present still
    runs the check; probe c2).
  - INV-008 (internal ports): SKIP only when `docker-compose.yml` absent.
  - Output per skipped INV: `  SKIP (...)` line (LOUD — not silent), counts as WARN.
  - Summary appends `Skipped invariants: ...` line (only when skips > 0 — unreachable in parity runs).
  - File present but check failing → FAIL as usual (fail-closed principle; probes c, c2).
  - INV-004 / INV-009 / INV-010: **never skippable** (universal, no file-absence dependency).
- `src/gate.rs`: `run_core(skip_absent: bool)` — added param. Default path (`false`) is
  BYTE-IDENTICAL to prior parity; all new code gated on `if skip_absent` branches.
  `run(skip_absent: bool)` CLI wrapper threaded from `Gate { skip_absent }`.
  `State.skipped_invs: Vec<String>` added for `Skipped invariants` line.
- `src/main.rs`: `--skip-absent` flag on `Gate` variant (default false).
- `src/serve.rs`: MCP tool `gate` now accepts optional arg `skip_absent: boolean` (default false).
  Wrong type → `isError: true` (fail-closed; no silent default). Input schema handcrafted
  (no schemars dep). 4 tools unchanged.
- `tests/gate_skip.rs` (new): 8 probes — (a) SKIP + exit 0; (b) default FAIL; (c) fail-closed
  per-INV; (c2) guard kép sentry.client.config.ts; (d) flag no-op byte-identical when files present;
  (e) INV-004 not skippable; (f) skip + fail coexist.
- `tests/mcp_serve.rs`: 3 new probes — (g1) skip_absent=true → exit_code 0; (g2) no-args backward
  compat → exit_code 1; (g3) wrong-type → isError true. Helper `call_tool_with_args` + `call_tool_raw`.
- `.gitignore`: added 4 entries INV-004 demands (`.env.production .env.staging .env.backup .env.local`).
  git history verified CLEAN before add (anchor #10 re-run: `git log --all --diff-filter=A -- '.env*'` empty).

### Not changed
- Default behavior (no flag): BYTE-IDENTICAL to P006 — 84 old tests + pins untouched.
- `src/checks/*.rs`, `golden/`, `tests/golden/`: diff rỗng tuyệt đối (flag lives in gate orchestrator only).
- `Cargo.toml`, `Cargo.lock`: no new deps, no version bump (F13 — P008).

### P006 — serve MCP stdio — 2026-06-11

### Added
- `src/serve.rs` — MCP stdio server via rmcp 1.7.0. Exposes 5 tools: `check_secrets` / `check_runtime` / `check_port` / `check_schema` / `gate`. Each tool calls the corresponding `run_core()` in-process (zero subprocess, same code path as CLI). Response: 1 JSON text content item with 4 fields: `exit_code` (i32), `is_clean` (bool), `findings` (stdout text), `stderr` (stderr text, e.g. WARN from port). `isError: false` for check findings (tool ran successfully); `true` only on internal error. CWD contract: client launches server with cwd = repo to scan.
- `src/main.rs` — `Commands::Serve` variant; dispatches to `serve::run()`.
- `tests/mcp_serve.rs` — 5 integration tests via raw JSON-RPC pipe (initialize → notifications/initialized → tools/list → tools/call): `mcp_tools_list_five_tools`, `mcp_five_tools_dirty_match_cli`, `mcp_clean_fixture_exit_zero`, `mcp_check_port_stderr_field_is_string`, `serve_with_unknown_flag_exits_2`. Oracle: live CLI `run_core()` on same fixture.

### Changed (buffered-core refactor — parity byte-exact)
- `src/checks/mod.rs` — added `pub struct CheckOutput { stdout: String, stderr: String, code: i32 }`.
- `src/checks/secrets.rs`, `port.rs`, `runtime.rs`, `schema.rs` — each gains `run_core() -> CheckOutput` (pure, no I/O side effects) + thin `run() -> i32` CLI wrapper using `print!`/`eprint!`. Parity: BYTE-EXACT vs all prior pins (79 old tests green).
- `src/gate.rs` — `run_core() -> CheckOutput` accumulates all sections via `run_section_buf()` helper; `run()` thin CLI wrapper. 79 old tests remain BYTE-EXACT.
- Stdout-poisoning grep: 0 `println!` in core or serve (only `print!`/`eprint!` in `run()` wrappers + `eprintln!` in serve error paths). JSON-RPC framing safe.

### Changed (dep pinning — security surface)
- `Cargo.toml` — pinned exact `=version` for: `serde`, `serde_json`, `tokio` (added `time` feature for rmcp internal timeouts), `anyhow`, `thiserror`, `rmcp`. Rationale: supply-chain reproducibility; Giám sát review on every upgrade (CLAUDE.md).

### Deviation — tokio `time` feature
rmcp 1.7.0 uses `tokio::time` internally (not documented in crate README). Added `"time"` to tokio features to satisfy runtime requirement. No behavior change to CLI paths.

### P005 — gate --all + dogfood swap — 2026-06-11

### Added
- `src/gate.rs` — orchestrator port of `golden/security-gate.sh --mechanical-only` branch (P005). In-process calls: `checks::{port,secrets,runtime}::run()` (tuần tự, single-thread, no subprocess). 6 inline check private fns: `inv_002` (`:latest` tag — INV-002), `inv_003` (.env.example real value — INV-003), `inv_004` (.env.* gitignored + history — INV-004), `inv_005` (Sentry beforeSend/beforeBreadcrumb — INV-005), `inv_006` (astro-service CORS wildcard — INV-006), `inv_008` (internal services expose vs ports — INV-008, Python→Rust-native). INV-007 skipped (mechanical-only — zero output). Accumulator semantics: all sections run, exit 1 iff FAIL>0. Summary verbatim: `====...` + `Security gate: $PASS passed, $FAIL failed, $WARN warnings` + `Failed invariants: ...` only when FAIL>0.
- `src/main.rs` — `Gate { --all }` variant; `mod gate;`; dispatch to `gate::run()`. `gate` bare or unknown flag → clap exit 2.
- `tests/parity_gate.rs` — parity tests (dirty/clean) stdout+stderr BYTE-EXACT vs `gate--{dirty,clean}` pins; usage-error exit 2; unit probes: accumulator, clean-all-sections, bare-exit-2, ≥1/inline-check (8 probes for INV-002..006+INV-008), summary conditional.

### Dogfood swap — `scripts/security-gate.sh` (adapted 99 LOC)
Per-check swap (Chủ nhà decision (b)): replaced `"$PY" scripts/check-hardcoded-secrets.py` → `"$INV_GATE" check secrets` and `"$PY" scripts/check-runtime-secrets.py` → `"$INV_GATE" check runtime`. Build-guard fail-closed added (binary missing → exit 1 with guided message). Original python3 calls preserved as comments (reversible). Python detection block retained for backward compat. `hooks/pre-commit` not modified.

### Deviation — flag surface (document per CLAUDE.md)
`gate --all` (Rust) ≡ golden `security-gate.sh --mechanical-only`. Full SSH mode (`--include-ssh`) not implemented in Phase 1. `--mechanical-only` / `--include-ssh` flags not exposed in Rust CLI (Sprint 2). Usage-error text deviation from golden `:14` is pre-documented (P001, anchor #9) — exit code 2 matches, text wording differs (clap auto-generated vs golden echo).

### P004 — check port + schema — 2026-06-11

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

### P003 — check runtime — 2026-06-11

### Added
- `src/checks/runtime.rs` — INV-010 port of `golden/check-runtime-secrets.py` (parity 1:1). RUNTIME_FILES (`.git/config`, `.mcp.json`, `.claude/settings.local.json`), INFRA_GLOBS (read_dir+sort — Python 3.12+ sorted glob, no `glob` crate), INFRA_TOP_LEVEL, 15 prefix patterns + 1 generic, allowlist (golden:119-135), SKIP_EXTENSIONS, masking, output format — all verbatim with `golden:line` citations. Sub-mech F (dotfile token leak, golden:39) classification noted in Discovery.
- `src/main.rs` — variant `Runtime` added to `CheckCommand`; dispatch to `checks::runtime::run()`.
- `src/checks/mod.rs` — `pub mod runtime;` added.
- `tests/parity_runtime.rs` — 2 parity tests (dirty/clean) byte-exact vs `tests/golden/pins/`; 14 mandatory unit tests (V2): all pattern classes (a-g) including proof tests g1-g4 for db-conn equivalence.

### Pattern deviation (V2 — O1.1, security surface — Giám sát review required)
4 `db-conn-*` patterns (`golden:100-103`) contained negative lookahead `(?!\$)` — unsupported by `regex` crate. Resolution: **drop exactly the token `(?!\$)`**, keep every other character verbatim. Equivalence: the immediately-following char class `[^@/\s\$]{8,}` already excludes `$`, so the lookahead is redundant (the formal language accepted by both versions is identical). Proof: 15/15 adversarial oracle cases (Debate Log Turn 2) + proof tests g1-g4 (green). No dep added, no behavior change.

### P002 — check secrets — 2026-06-11

### Added
- `src/checks/secrets.rs` — INV-009 port of `golden/check-hardcoded-secrets.py` (parity 1:1). All patterns, allowlist (8 entries), skip rules (test-file / path-level / comment-line), masking, and output format preserved verbatim with `golden:line` citations.
- `src/main.rs` — clap derive CLI skeleton: `inv-gate check secrets` subcommand. Exit 2 on unknown subcommand/flag (anchor #12 verified).
- `src/checks/mod.rs` — checks module layout (1 module / INV, per ARCHITECTURE.md).
- `tests/parity_secrets.rs` — 2 parity tests (dirty/clean) byte-exact vs `tests/golden/pins/`; 2 mandatory unit tests (V2): `should_skip_path(src/generated/)` + comment-line skip.

### P001 — pin golden oracle — 2026-06-11

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
