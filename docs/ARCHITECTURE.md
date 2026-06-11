# Architecture — inv-gate

> Port map: `golden/` (frozen tarot gate, 797 LOC Python+Bash) → Rust dual-mode binary.

## Overview

Single Rust binary, clap-derive CLI. Each mechanical INV check = 1 subcommand under
`check`; `gate` aggregates; `serve` (Phase 3) exposes the same via rmcp stdio MCP.
Golden-oracle method: outputs pinned from `golden/` scripts in `tests/golden/` BEFORE
porting (doc-rotate precedent).

## Components

- `src/main.rs` — clap dispatch: `check secrets|port|runtime|schema` · `gate` · `serve`
- `src/checks/` (planned) — 1 module / INV: `secrets.rs` (INV-009), `port.rs` (INV-001), `runtime.rs` (INV-010), `schema.rs` (Prisma)
- `src/gate.rs` (planned) — orchestrator: run all, aggregate exit code (0 clean / 1 findings / 2 config error)
- `golden/` — FROZEN oracle scripts (read-only reference)
- `tests/golden/` (P001) — pinned oracle outputs; parity tests compare Rust vs pin

## Data flow

repo files → check module (regex/walk, same patterns as golden) → findings (path:line + pattern id)
→ stdout report + exit code → consumed by pre-commit hook `[4/7]` (CLI) or Giám sát/Quản đốc (MCP).
