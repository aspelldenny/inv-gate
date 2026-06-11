# MANIFEST — Golden Oracle Pin (P001)

> Source of truth for fixture provenance, invocation contracts, exit-code table,
> normalization rules, and unit-spec (F06). Every trigger in `fixtures/` derives
> from a pattern cited here (F07 compliance).

---

## 1. Per-check pattern table

### check-hardcoded-secrets.py (INV-009)

Scan targets: `["src"]` (JS/TS/JSX/TSX) + `["astro-service"]` (Python) — `golden/check-hardcoded-secrets.py:33-34`.

| Pattern class | Regex (abridged) | golden:line | Fixture trigger |
|---|---|---|---|
| `github-pat` | `gh[pous]_[A-Za-z0-9]{36}` | `golden/check-hardcoded-secrets.py:69` | `src/config.ts` line 3 with `ghp_` token literal |
| `generic-entropy` | `(api_key\|secret\|token)[:=]"[A-Za-z0-9_\-]{32,}"` | `golden/check-hardcoded-secrets.py:80-81` | `src/config.ts` line 5 with `api_key = "..."` 32+ chars |
| allowlist skip | `your-`, `xxx`, `REPLACE`, `PLACEHOLDER`, `process.env.`, `os.environ`, `c993dc1e` | `golden/check-hardcoded-secrets.py:85-95` | clean fixture uses `process.env.` reference |
| test-file skip | `/tests/`, `*.test.*`, `*.spec.*`, `/__tests__/`, `/__mocks__/`, `prisma/seed-*.ts` | `golden/check-hardcoded-secrets.py:51-58` | not triggered in P001 fixture |

**Note (F07 deviation):** script scans `["src"]` for `.ts/.tsx/.js/.jsx` only (`golden/check-hardcoded-secrets.py:33-35`), and `["astro-service"]` for `.py`. Fixture uses `src/config.ts` (not `.py`) to match actual scan extensions. Python fixture would require `astro-service/` directory — not in P001 scope.

### check-port-bind.py (INV-001)

Scan targets: `COMPOSE_FILES = ["docker-compose.yml", "docker-compose.dev.yml", "astro-service/docker-compose.yml"]` — `golden/check-port-bind.py:12-16`.

| Pattern class | Rule | golden:line | Fixture trigger |
|---|---|---|---|
| `implicit 0.0.0.0 bind` | `HOST:CONTAINER` (no IP prefix) not in `{"80:80","443:443"}` | `golden/check-port-bind.py:48-53` | `docker-compose.yml` service with `- 8001:8001` |
| `public IP bind` | 3-part `IP:HOST:CONTAINER` where IP != `127.0.0.1` | `golden/check-port-bind.py:43-47` | not triggered in P001 fixture |
| allowed: loopback | IP prefix `127.0.0.1:*` | `golden/check-port-bind.py:44-45` | clean fixture uses `127.0.0.1:8001:8001` |
| allowed: nginx | `80:80` or `443:443` | `golden/check-port-bind.py:19` | clean fixture uses nginx service with `443:443` |

**Note:** script prints WARN to stderr for missing COMPOSE_FILES (not exit 1) — `golden/check-port-bind.py:60`.

### check-runtime-secrets.py (INV-010)

Scan targets (runtime files): `.git/config`, `.mcp.json`, `.claude/settings.local.json` — `golden/check-runtime-secrets.py:40-44`.
Scan targets (infra globs): `scripts/*.sh`, `.github/workflows/*.yml`, `.github/workflows/*.yaml`, `hooks/*` — `golden/check-runtime-secrets.py:47-52`.
Top-level infra: `Dockerfile.nextjs`, `docker-compose.yml`, `docker-compose.staging.yml`, `astro-service/Dockerfile`, `astro-service/docker-compose.yml` — `golden/check-runtime-secrets.py:56-63`.

**Nhánh A (anchor #13 verified):** `.git/config` is in `RUNTIME_FILES` → harness injects token-in-URL via `git remote add`. No static fixture file needed.

| Pattern class | Regex (abridged) | golden:line | Fixture trigger |
|---|---|---|---|
| `token-in-url` | `https://[...]+:(gh[pous]_[A-Za-z0-9]{36}\|github_pat_...)@` | `golden/check-runtime-secrets.py:95-97` | harness `git remote add origin https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git` |
| `github-pat-classic` | `gh[pous]_[A-Za-z0-9]{36}` | `golden/check-runtime-secrets.py:86` | same remote URL (token also matches prefix pattern) |
| allowlist skip | `CHANGEME`, `EXAMPLE`, `${`, `<`, `your-`, `xxx`, `REPLACE`, `PLACEHOLDER` | `golden/check-runtime-secrets.py:119-135` | "FAKE" not in allowlist → triggers |
| `.example` skip | `SKIP_EXTENSIONS = (".example", ".md", ".sample", ".template")` | `golden/check-runtime-secrets.py:77` | not triggered in P001 |

**Token value (O2.1):** `ghp_FAKETOKEN000000000000000000000000000` = `ghp_` + 36 alphanum (`FAKETOKEN` 9 chars + `000000000000000000000000000` 27 chars). `echo -n 'ghp_FAKETOKEN000000000000000000000000000' | wc -c` = 40. Verified against `golden/check-runtime-secrets.py:96`.

### check-schema-safety.sh (Prisma schema-safety)

Scan target: `prisma/schema.prisma` — `golden/check-schema-safety.sh:23`.
Mechanism: `git diff HEAD~1..HEAD -- prisma/schema.prisma` (NOT migration SQL) — `golden/check-schema-safety.sh:32`.
Bypass: `ALLOW_DATA_LOSS=true` → exit 0 early — `golden/check-schema-safety.sh:18`. **Harness must `unset ALLOW_DATA_LOSS` (O2 Tầng 2 note).**

| Pattern class | Regex (grep -E) | golden:line | Fixture trigger |
|---|---|---|---|
| field-delete | `^-\s+\w+\s+\S+` (removal line with field name + type) | `golden/check-schema-safety.sh:48` | `schema.after.prisma` removes `legacyToken String?` from User |
| model-delete | `^-\s*(model\|enum)\s+\w+` (removal line with model/enum keyword) | `golden/check-schema-safety.sh:48` | `schema.after.prisma` removes entire `AuditLog` model |
| skip: diff header | `grep -vE '^---|^-\+\+\+'` | `golden/check-schema-safety.sh:47` | closing brace `}` removal does NOT false-positive (anchor #12 verified) |
| additive: no trigger | field/model added only (no `-` removal lines) | `golden/check-schema-safety.sh:51-52` | clean `schema.after.prisma` adds `displayName String?` only |

### security-gate.sh (orchestrator)

Flags: `--mechanical-only` (skip INV-007 SSH, `golden/security-gate.sh:171-174`), `--include-ssh`, unknown flag → exit 2 (`golden/security-gate.sh:14`).
**P001 pin: nhánh `--mechanical-only` only** (anchor #7 verified — SSH to `root@103.167.150.178:1994` at line 147 would fail in CI/sandboxed env).
INV-007 skip: when `--mechanical-only`, INV-007 block skipped entirely (line 174 `else` clause — not even WARN).

Hardcoded script paths: `python3 scripts/check-port-bind.py` (line 55), `python3 scripts/check-hardcoded-secrets.py` (line 197), `python3 scripts/check-runtime-secrets.py` (line 201) → harness MUST copy scripts to `$tmp/scripts/` layout (anchor #6 verified).

**no usage-error mode for the 4 individual check scripts:** they accept no arguments and have no `exit 2` path. Confirmed: `check-hardcoded-secrets.py`, `check-port-bind.py`, `check-runtime-secrets.py`, `check-schema-safety.sh` — all assume cwd = repo root, output to stdout, exit 0/1 only (anchor #5/#8 verified). Usage-error pin ONLY applies to `security-gate.sh` (unknown flag).

---

## 2. Invocation contract

| Script | CWD | Args accepted | Hardcoded targets | Output stream | Exit codes |
|---|---|---|---|---|---|
| `check-hardcoded-secrets.py` | repo root | none | `["src"]` JS/TS, `["astro-service"]` py | stdout | 0=clean, 1=violations |
| `check-port-bind.py` | repo root | none | `["docker-compose.yml","docker-compose.dev.yml","astro-service/docker-compose.yml"]` | stdout (violations), stderr (WARN missing file) | 0=clean, 1=violations |
| `check-runtime-secrets.py` | repo root | none | `.git/config`, `.mcp.json`, `.claude/settings.local.json`, infra globs | stdout | 0=clean, 1=violations |
| `check-schema-safety.sh` | repo root | none | `prisma/schema.prisma` (via git diff) | stdout | 0=safe, 1=destructive |
| `security-gate.sh` | repo root | `--mechanical-only`, `--include-ssh` | delegates to above scripts at `scripts/` subpath | stdout | 0=all pass, 1=any fail, 2=unknown flag |

---

## 3. Oracle environment provenance

Pin valid in environments with equivalent versions:

- Python: `3.14.3`
- Bash: `GNU bash 3.2.57(1)-release (arm64-apple-darwin25)`
- Git: `2.50.1` (Apple Git-155)
- OS: Darwin 25.5.0 arm64

Pins may differ on Linux/amd64 if filesystem ordering of `rglob()` differs — see normalization rule §4.

---

## 4. Normalization rules

Applied by `repin.sh` to each captured output before writing to `pins/`:

1. **Absolute temp path → fixture-relative:** `sed "s|$tmp/||g"` — replaces temp dir prefix in all finding lines so output is reproducible across machines/runs.
2. **Commit SHA:** pinned SHAs are deterministic (fixed `GIT_AUTHOR_DATE` + `GIT_COMMITTER_DATE = "2026-01-01T00:00:00 +0000"` + local config). If SHA appears in output and proves non-deterministic across machines, apply `sed 's/[0-9a-f]\{40\}/<SHA>/g'` and record here. As of first pin: SHA determinism confirmed — no normalization applied.
3. **No sorting:** output line order preserved as-is. Parity tests for P002-P005 should compare order-insensitively OR verify `rglob()` is deterministic on the test machine (Python `Path.rglob()` traversal order is OS-dependent — verified consistent on single-platform pin).

### P003 rules (applicable to P004-P005)

4. **env-reconstruction for INV-010 parity:** parity test must build a real git repo in `tempdir` per `repin.sh:34-88` (hermetic config, fixed dates, 2-commit flow, remote inject). Simple fixture-copy is not sufficient — `.git/config` is generated by git commands, not a static file. See `tests/parity_runtime.rs` `build_fixture_repo()` as the reference implementation.
5. **INFRA_GLOBS sorted alphabetical:** `golden/check-runtime-secrets.py:215-221` uses `root.glob(pattern)` which in Python 3.12+ returns sorted results. Rust port: `read_dir` + collect + `.sort()` before iterating. NO `glob` crate needed — pairs are 1-level non-recursive. P004-P005 checks that glob over infra dirs must follow the same pattern.
6. **db-conn transcription precedent (INV-010 only):** 4 patterns (`golden:100-103`) had `(?!\$)` dropped (equivalence-proven, see P003 Debate Log Turn 2 + proof tests g1-g4). This transcription is specific to these 4 patterns; no other check currently exercises negative lookahead. If P004-P005 encounter lookahead in `golden/check-port-bind.py` or `golden/check-schema-safety.sh`, run a fresh equivalence proof before applying the same approach.

---

## 5. Unit-spec table (F06 compliance)

All numeric fields in pinned outputs must have explicit unit declaration.

| Check | Field | Unit | Derive from (golden:line) |
|---|---|---|---|
| `check-hardcoded-secrets.py` | `lineno` in `path:lineno:` | line number (1-indexed), line-only — no col/offset field | `golden/check-hardcoded-secrets.py:182`: `f"{path_str}:{lineno}: INV-009 violated..."` |
| `check-port-bind.py` | `idx+1` in `fname:lineno:` | line number (1-indexed), line-only — no col/offset field | `golden/check-port-bind.py:75`: `f"{fname}:{idx + 1}: INV-001 violated..."` |
| `check-runtime-secrets.py` | `lineno` in `path:lineno:` | line number (1-indexed), line-only — no col/offset field | `golden/check-runtime-secrets.py:240`: `f"{path_str}:{lineno}: INV-010 violated..."` |
| `check-schema-safety.sh` | n/a — output is raw diff lines, no `path:lineno:` format | n/a (diff format: `^-  field_name type` or `^- model Name`) | `golden/check-schema-safety.sh:52-55`: echoes raw `$DESTRUCTIVE` lines |

**Pre-fill from anchor #9 (verified):** all 4 Python/shell checks output `path:lineno:` format — LINE-ONLY, no col/byte-offset field. schema-safety outputs raw diff lines (not `path:lineno:` format at all — no numeric offset field).

---

## 6. Pinned run inventory

| Key (pins/exit_codes.json) | Script | Fixture | Flags | Expected exit |
|---|---|---|---|---|
| `secrets--dirty` | check-hardcoded-secrets.py | dirty | none | 1 |
| `secrets--clean` | check-hardcoded-secrets.py | clean | none | 0 |
| `port--dirty` | check-port-bind.py | dirty | none | 1 |
| `port--clean` | check-port-bind.py | clean | none | 0 |
| `runtime--dirty` | check-runtime-secrets.py | dirty | none | 1 |
| `runtime--clean` | check-runtime-secrets.py | clean | none | 0 |
| `schema--dirty` | check-schema-safety.sh | dirty | none | 1 |
| `schema--clean` | check-schema-safety.sh | clean | none | 0 |
| `gate--dirty` | security-gate.sh | dirty | --mechanical-only | 1 |
| `gate--clean` | security-gate.sh | clean | --mechanical-only | 0 |
| `gate--usage-error` | security-gate.sh | n/a | --no-such-flag | 2 |
