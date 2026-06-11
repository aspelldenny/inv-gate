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
  - `port.rs` (INV-001) — planned
  - `runtime.rs` (INV-010) — **shipped P003**: parity port of `golden/check-runtime-secrets.py` (golden:40-245); RUNTIME_FILES + INFRA_GLOBS (read_dir+sort, no glob crate) + INFRA_TOP_LEVEL, 15 prefix patterns + 1 generic, allowlist (golden:119-135), SKIP_EXTENSIONS (golden:77), masking (golden:169-173), errors="ignore" byte-strip (golden:180). V2 deviation: 4 db-conn patterns (golden:100-103) transcribed — `(?!\$)` dropped (equivalence-proven, proof tests g1-g4). Sub-mech F: dotfile token leak classification (golden:39).
  - `schema.rs` (Prisma schema-safety) — planned
- `src/gate.rs` (planned) — orchestrator: run all, aggregate exit code (0 clean / 1 findings / 2 config error)
- `golden/` — FROZEN oracle scripts (read-only reference)
- `tests/golden/` (P001) — pinned oracle outputs; parity tests compare Rust vs pin

## Data flow

repo files → check module (regex/walk, same patterns as golden) → findings (path:line + pattern id)
→ stdout report + exit code → consumed by pre-commit hook `[4/7]` (CLI) or Giám sát/Quản đốc (MCP).
