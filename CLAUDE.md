# CLAUDE.md — inv-gate

> Project context for Claude Code. Workflow doctrine: sos-kit (3 roles + Quản đốc orchestrator).
> **PRD / single source of truth: `docs/PROJECT.md`** — written 2026-05-28, read it before any phiếu or code.

## What this is

Rust binary replacing tarot's **5-file mechanical security gate** (~797 LOC Python+Bash,
frozen at `golden/`): INV-009 secrets / INV-001 port-bind / INV-010 runtime-secrets /
Prisma schema-safety + the `security-gate.sh` orchestrator. Dual mode: **CLI** (pre-commit
hook calls `inv-gate gate --all`) + **MCP** (`serve` — rmcp stdio, tools `check_*` + `gate`).

Kit-family Rust tool — sibling of doctor/docs-gate/claude-hooks/doc-rotate. When v0.1.0
ships: release-CI tag → GitHub Releases 3-target → join sos-kit `install.sh` BINARIES →
sos-kit pre-commit `[4/7]` swaps Python call for the binary (kills the last python3
dependency in the gate chain).

## ⚠️ Dogfood duty (đọc TRƯỚC khi chạy Sprint 1)

This sprint doubles as sos-kit's end-to-end workflow dogfood. **Open `docs/SOS_KIT_FEEDBACK.md`
FIRST** — 23-point watchlist (W1-W23). Tick items as the run passes them; log kit-findings as
`[IG-NN]` in that file (kit-bug → routed to ~/sos-kit at harvest; inv-gate tool-bug → own BACKLOG).
Pre-cleared 2026-06-11: W15 cargo-check phase OK · W16 security gate passes over golden/*.py.

## Method — golden-oracle port (doc-rotate precedent)

1. **P001 pins the oracle FIRST**: run each `golden/` script on fixtures → freeze findings
   + exit codes into `tests/golden/`. No porting before pinning.
2. Port one INV per phiếu. Parity test = Rust output vs pinned oracle (same findings,
   same exit codes). Spec units explicitly (char vs byte — F06 lesson).
3. Behavior changes (better patterns, new INV) come AFTER parity, as separate phiếu.
4. `golden/` is read-only reference — never "fix" it.

## Rules

- Stack: Rust edition 2024, clap derive, regex, rmcp (serve = Phase 3). `cargo test` green before commit.
- Worker KHÔNG invent fixture files — synthetic in-code instances for parity probes OK (F07).
- Exit-code contract is API: `0` clean, `1` findings, `2` usage/config error — pre-commit
  hooks depend on it; document any deviation from the golden scripts in CHANGELOG + here.
- **P001 deviation (anchor #5/#8):** the 4 individual check scripts exit 0/1 ONLY — no
  exit-2 mode. Exit 2 applies only to `security-gate.sh` (unknown flag). Rust port must
  match this per-script behavior; do NOT add exit-2 to individual checks without a
  separate phiếu + Tầng 1 approval.
- CHANGELOG bump → `Cargo.toml` version sync (F13).
- Scan-target patterns (what counts as a secret) are a SECURITY surface → Tầng 1 docs +
  Giám sát review on the PR.
- **P007 flag deviation:** `gate --all --skip-absent` = opt-in deviation from golden parity.
  Skip allowlist: INV-005 (guard kép — absent only when BOTH `src/lib/sentry.ts` AND
  `sentry.*.config.*` absent) + INV-008 (absent only when `docker-compose.yml` absent).
  Default (no flag) = golden `--mechanical-only` parity as before. INV-004/009/010 never skip.
  Adding any INV to the allowlist requires a separate phiếu + Tầng 1 review.
- **P005 flag mapping:** `inv-gate gate --all` (Rust) ≡ `security-gate.sh --mechanical-only` (golden). SSH mode (`--include-ssh`) not ported Phase 1 (Sprint 2). Dogfood repo: per-check swap in `scripts/security-gate.sh` (INV-009/010 call binary; INV-007 + inline checks retained in bash or skipped).
- **P003 pattern transcription (INV-010 only):** `golden/check-runtime-secrets.py:100-103`
  — 4 `db-conn-*` patterns had `(?!\$)` (negative lookahead, unsupported by `regex` crate)
  dropped. Equivalence proven: the immediately-following class `[^@/\s\$]{8,}` already
  excludes `$`, making the lookahead redundant. Proof: 15/15 adversarial oracle cases +
  proof tests g1-g4 in `tests/parity_runtime.rs`. Future patterns with lookahead require a
  fresh equivalence proof — do NOT apply this transcription blindly.

## Sos-kit v2 — 3-role envelope

Chủ nhà (Sếp) — vision, approve, nghiệm thu · Kiến trúc sư (`architect` subagent) — đọc docs,
viết phiếu, KHÔNG đọc code · Thợ (`worker` subagent) — execute phiếu, full code, KHÔNG vision docs.
Orchestrator contract: `agents/orchestrator.md` (condensed) + `docs/ORCHESTRATION.md` (spec).
All spawnable agents carry `background: true` (frontmatter) — spawns don't block the main session.
Session start: banner shows `docs/BACKLOG.md` Active sprint → pick item → DRAFT → CHALLENGE →
APPROVAL_GATE → EXECUTE. Markers: `.sos-state/architect-active` / `worker-active` (2 chiều).
