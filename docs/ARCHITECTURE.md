# Architecture — inv-gate

> Port map: `golden/` (frozen tarot gate, 797 LOC Python+Bash) → Rust dual-mode binary.

## Overview

Single Rust binary, clap-derive CLI. Each mechanical INV check = 1 subcommand under
`check`; `gate` aggregates; `serve` (Phase 3) exposes the same via rmcp stdio MCP.
Golden-oracle method: outputs pinned from `golden/` scripts in `tests/golden/` BEFORE
porting (doc-rotate precedent).

## Components

- `src/main.rs` — clap dispatch: `check secrets|port|runtime|schema` · `gate` · `serve`
- `src/checks/` — 1 module / INV:
  - `secrets.rs` (INV-009) — **shipped P002**: parity port of `golden/check-hardcoded-secrets.py`; all scan targets, 7 prefix patterns + 1 generic, 8-entry allowlist, path-level + test-file + comment-line skip rules verbatim from golden.
  - `port.rs` (INV-001) — **shipped P004**: parity port of `golden/check-port-bind.py` (golden:12-80); COMPOSE_FILES 3 hardcoded paths, ALLOWED_PUBLIC nginx set, 4-layer line-based parse (PORT_LINE_RE / numeric filter `^[\d.:]+$` / is_in_ports_block backward-walk / classify 2/3/N-part), WARN-stderr missing file, output `{fname}:{lineno}: INV-001 violated -- {reason}`. Non-UTF-8: error-path exit non-zero.
  - `runtime.rs` (INV-010) — **shipped P003**: parity port of `golden/check-runtime-secrets.py` (golden:40-245); RUNTIME_FILES + INFRA_GLOBS (read_dir+sort, no glob crate) + INFRA_TOP_LEVEL, 15 prefix patterns + 1 generic, allowlist (golden:119-135), SKIP_EXTENSIONS (golden:77), masking (golden:169-173), errors="ignore" byte-strip (golden:180). V2 deviation: 4 db-conn patterns (golden:100-103) transcribed — `(?!\$)` dropped (equivalence-proven, proof tests g1-g4). Sub-mech F: dotfile token leak classification (golden:39).
  - `schema.rs` (Prisma schema-safety) — **shipped P004**: parity port of `golden/check-schema-safety.sh` (golden:16-64); ALLOW_DATA_LOSS bypass (exact `"true"`, em-dash echo), 3-step git fallback chain via `std::process::Command` (Stdio::null() suppresses stderr), header-skip + destructive grep pipeline, 6-branch table A-F. First bash script ported. O1.2: 15-char SHA in fallback ported as-is.
- `src/gate.rs` (gate --all) — **shipped P005, P007**: orchestrator parity port of `golden/security-gate.sh --mechanical-only`. In-process: `checks::{port,secrets,runtime}::run_core()` (buffered-core — P006 refactor). 6 inline private fns (INV-002..006 + INV-008); INV-008 Python→Rust-native (no python3 subprocess). INV-007 skipped in mechanical-only. Accumulator: all sections run, PASS/FAIL/WARN counters, summary verbatim `:204-210`. Flag mapping: `gate --all` ≡ `--mechanical-only`; `--include-ssh`/`--mechanical-only` not in Rust CLI (Sprint 2). Cite range per section in source. **Buffered-core (P006):** `run_core(skip_absent: bool) -> CheckOutput` accumulates all section output; `run(skip_absent: bool) -> i32` thin CLI wrapper. **P007 `--skip-absent`:** `State.skipped_invs` tracks skipped INV names. Guard clauses before INV-005 and INV-008 (only): INV-005 uses guard kép (SKIP only when BOTH `src/lib/sentry.ts` AND `sentry.*.config.*` absent — fail-closed on repos with sentry config present); INV-008 SKIP when `docker-compose.yml` absent. SKIP line printed per INV (LOUD); warn counter incremented. `Skipped invariants: ...` summary line appended when skips > 0 (only reachable with flag — parity runs unaffected). Default (false) path is BYTE-IDENTICAL to pre-P007. Non-webapp repos: use `gate --all --skip-absent`.
- `src/serve.rs` — **shipped P006, P007**: MCP stdio server (rmcp 1.7.0). `ServerHandler` impl with `ToolRouter` of 5 `ToolRoute::new_dyn()` routes. Each route calls `run_core()` in-process, wraps result in 4-field JSON response via `make_response()`. Runtime: `tokio::runtime::Builder::new_current_thread().enable_time()` (current-thread, time feature for rmcp internals). Blocks on `RunningService::waiting()` until stdin closed. **P007:** tool `gate` input schema updated to 1 optional property `skip_absent: boolean` (handcrafted JsonObject, no schemars dep). Handler parses `ctx.arguments`: missing/null → false; bool → use; wrong type → `isError: true` (fail-closed). 4 other tools: schema + handler unchanged.
- `golden/` — FROZEN oracle scripts (read-only reference)
- `tests/golden/` (P001) — pinned oracle outputs; parity tests compare Rust vs pin
- `tests/gate_skip.rs` (P007) — 8 skip-absent probes: (a) SKIP + exit 0; (b) default FAIL; (c) fail-closed per-INV; (c2) guard kép sentry.client.config.ts; (d) flag no-op byte-identical; (e) INV-004 not skippable; (f) skip + fail coexist. All fixtures synthetic in-code (F07).
- `tests/mcp_serve.rs` (P006, P007) — MCP integration tests: raw JSON-RPC pipe via `std::process::Command`, `ServeSession` struct, `read_response()` skips notifications, 5+3 tests covering tools/list + tools/call (dirty + clean) + stderr field + unknown-flag exit 2 + P007: skip_absent=true exit 0, backward-compat no-args exit 1, wrong-type isError. Helpers: `call_tool`, `call_tool_with_args`, `call_tool_raw`.

## Data flow

### CLI path
repo files → `run_core()` (pure, buffered) → `CheckOutput { stdout, stderr, code }`
→ `run()` thin wrapper: `print!(stdout)` + `eprint!(stderr)` + `process::exit(code)`
→ consumed by pre-commit hook `[4/7]`.

### MCP path (P006)
MCP client (Claude / Giám sát) → JSON-RPC over stdin/stdout → `serve::run()` (tokio current-thread)
→ `ToolRouter::call()` → same `run_core()` in-process (zero subprocess)
→ `make_response()`: `CallToolResult` with 1 text content item = JSON `{ exit_code, is_clean, findings, stderr }`
→ JSON-RPC response to client. stdout = JSON-RPC ONLY (no print!/println! in serve or core).

**P005 dogfood data flow (per-check swap):**
pre-commit `[4/7]` → `bash scripts/security-gate.sh --mechanical-only` (adapted 99 LOC) →
  `target/release/inv-gate check secrets` (INV-009) + `target/release/inv-gate check runtime` (INV-010);
  python3 calls removed (reversible — original calls kept as comments).
`gate --all` = parity-proven fixture-based (pins `gate--{dirty,clean}`).

## Distribution (v0.1.0, shipped P008)

Release CI: `.github/workflows/release.yml` — trigger `push: tags: ["v*"]`, 3 jobs parallel,
no test step in CI (intentional: pre-commit + nghiệm thu local cover; workflow kept minimal
as kit-family convention).

**3-target matrix (kit-family mirror-siblings, Chủ nhà ruling 2026-06-11):**

| Target triple | Runner | Asset name |
|---|---|---|
| `aarch64-apple-darwin` | `macos-14` | `inv-gate-aarch64-apple-darwin` |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | `inv-gate-x86_64-unknown-linux-gnu` |
| `x86_64-pc-windows-msvc` | `windows-2022` | `inv-gate-x86_64-pc-windows-msvc.exe` |

**Asset naming contract:** `inv-gate-<target-triple>` (+`.exe` for Windows) —
consumed by `sos-kit install.sh:65`: `${bin}-${TARGET}${EXT}`. EXT logic in workflow MUST
be preserved on any workflow edit (Tầng 1 contract).

**Release profile:** `strip = true`, `lto = true`, `opt-level = 3` (Cargo.toml release profile).
Binary size target: < 6 MB (PROJECT.md success criterion #5). Local arm64 release binary: ~2.8 MB.

**Exit-code contract (pre-commit API):** `0` clean, `1` findings, `2` usage/config error.
Pre-commit hooks depend on this — document any deviation in CHANGELOG + CLAUDE.md.
See also CLAUDE.md §Rules for per-script exit-2 deviation note (P001).

**Intel-Mac gap (IG-10):** `x86_64-apple-darwin` not in matrix (no `Darwin-x86_64` mapping
in `sos-kit install.sh`). Kit-level feedback logged IG-10 for sos-kit harvest. Out of scope here.
