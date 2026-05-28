# PROJECT — inv-gate

> **Status:** Bootstrap (2026-05-28). Cargo skeleton + vision only.
> **Full setup deferred:** post-pilot retrospective of `advisory-inbox` (Workflow v2.1 pilot, ETA ~3 days). At that point: port v2.1 doctrine + agents + skills symlink + CI workflow from sos-kit golden template.
> **Workflow v2.1 spec:** `~/sos-kit/docs/WORKFLOW_V2.1.md` (durable doctrine).

---

## Vision (1 câu)

Rust binary thay **5 mechanical INV check** (~794 dòng Python+Bash) trong `tarot/scripts/security-gate.sh` + helpers — `check secrets / port / runtime / schema` + orchestrator `gate` — dual mode **CLI** (pre-commit hook gọi) + **MCP** (em hoặc Giám sát agent gọi `mcp__inv__check_all`).

---

## Why this exists

Tarot security gate hiện có 5 file mechanical check:

| File | LOC | INV | Việc |
|------|-----|-----|------|
| `scripts/security-gate.sh` | 210 | orchestrator | Run all INV-001..010 mechanical, exit 0/1 |
| `scripts/check-hardcoded-secrets.py` | 192 | INV-009 | Grep secret pattern in source code |
| `scripts/check-port-bind.py` | 82 | INV-001 | Docker host-bind 0.0.0.0 except nginx 80/443 |
| `scripts/check-runtime-secrets.py` | 246 | INV-010 | `.git/config` token-in-URL scan (P305 Sub-mech F) |
| `scripts/check-schema-safety.sh` | 64 | INV (Prisma) | Migration DROP/DELETE/DROP COLUMN safety |

**~794 dòng Python + Bash** = LLM phải nhớ command path + flag mỗi lần. Catalog đang grow (P273 ship INV-001..008, P305 add INV-010 — sẽ còn INV-011+).

Rust binary `inv-gate` replace với:
- Single CLI: `inv-gate check secrets` thay vì `python3 scripts/check-hardcoded-secrets.py`
- Composite: `inv-gate gate --all` thay vì `bash scripts/security-gate.sh`
- Catalog growth: add new INV = add subcmd, không phải nhớ path file mới
- MCP mode: em call `mcp__inv__check_all` 1 lệnh thay vì 5 path lookup
- Compile-time test catalog completeness

---

## Scope cứng

### IN scope (Phase 1-3)

**Initial subcommand (Phase 1):**

1. **`check secrets`** — scan source code for hardcoded API keys/tokens (INV-009)
2. **`check port`** — docker-compose host-bind pattern check (INV-001)
3. **`check runtime`** — `.git/config` + env runtime state token leak (INV-010, Sub-mech F)
4. **`check schema`** — Prisma migration destructive operation guard

**Composite (Phase 2):**

5. **`gate --all`** — orchestrate all check subcmd, exit code aggregate
6. **`gate --mechanical-only`** — skip SSH-required check
7. **`gate --include-ssh`** — include remote check (vps connection)

**MCP (Phase 3):**

8. **`serve`** — rmcp stdio JSON-RPC, expose check + gate subcmd as tools

### OUT scope (NOT building)

- INV-101..108 judgment invariants — đó là `boundary-check` agent surface, NOT mechanical
- Auto-fix violations — INV-gate is REPORT only, fix là phiếu mới
- CI/CD integration — install qua pre-commit hook, KHÔNG Bash wrapper
- Multi-project parallel scan — 1 project 1 run
- Web UI — terminal đủ

---

## Success criteria

1. **CLI parity:** mỗi `check <inv>` cho cùng exit code + violation list như Python+Bash counterpart cho 100% test fixtures
2. **Composite:** `gate --all` orchestrates ≥4 check subcmd, returns aggregate exit
3. **MCP mode:** `serve` expose tools, JSON-RPC handshake clean
4. **Test:** `cargo test --all` ≥ 40 tests pass (1 test per INV × 8-10 case mỗi cái)
5. **Binary size:** `< 6 MB` release build
6. **Performance:** all INV check < 2s on tarot-size codebase (vs ~5-10s Python+Bash hiện tại)
7. **Tarot smoke test:** install vào tarot, replace 5 file, security-gate run clean
8. **Catalog future-proof:** add new INV-011 trong < 100 LOC new subcmd

---

## Tech Stack

- Rust edition 2024, MSRV 1.85
- clap 4.x derive (subcmd nested: `check secrets`, `check port`, etc.)
- serde + serde_json (output JSON for tooling consumption)
- regex (secret pattern + path pattern matching)
- walkdir (recursive file scan)
- tokio (only for MCP `serve`)
- rmcp 1.7.0 (MCP server)
- anyhow + thiserror
- assert_cmd + predicates + tempfile (CLI integration tests)

---

## Roadmap (placeholder — refine post-pilot retrospective)

### Phase 1 — Core 4 check (P001-P006)

- P001 scaffold CLI + nested subcmd `check <name>`
- P002 INV-009 `check secrets` (port Python `check-hardcoded-secrets.py`)
- P003 INV-001 `check port` (port Python `check-port-bind.py`)
- P004 INV-010 `check runtime` (port Python `check-runtime-secrets.py`, Sub-mech F)
- P005 INV `check schema` (port Bash `check-schema-safety.sh`)
- P006 violation report struct + JSON output

### Phase 2 — Composite (P007-P008)

- P007 `gate` composite subcmd (3 modes: all / mechanical-only / include-ssh)
- P008 exit code aggregation + violation summary

### Phase 3 — MCP (P009-P010)

- P009 `serve` subcmd (rmcp stdio)
- P010 MCP tool dispatch (8 tools, schema validate)

### Phase 4 — Ship (P011-P012)

- P011 README + ARCHITECTURE polish + `cargo publish`
- P012 install in tarot — replace 5 file trong `tarot/scripts/`

---

## Constraints

- KHÔNG mutate source code (REPORT only, fix via phiếu)
- KHÔNG fetch network (local mechanical scan)
- KHÔNG depend OS-specific (cross-platform macOS + Linux)
- KHÔNG silently expand INV catalog — new INV phải có phiếu + doctrine entry trong INVARIANTS.md trước

---

## Notes for future session resume

- Bootstrap commit này CHỈ ship Cargo skeleton + this PROJECT.md
- Full Workflow v2.1 doctrine port sẽ ship **POST-PILOT** sau khi `advisory-inbox` Phase 1-4 xong + retrospective
- INV catalog source: `~/tarot/docs/security/INVARIANTS.md` (mechanical INV-001..010) — port nội dung khi bắt đầu code phase

---

## Cross-reference

- Pilot precedent: `~/advisory-inbox/` (Workflow v2.1 first test bed)
- Reference precedent: `~/advisory-cron/` (Rust binary template structure)
- Spec: `~/sos-kit/docs/WORKFLOW_V2.1.md`
- INV catalog source: `~/tarot/docs/security/INVARIANTS.md`
