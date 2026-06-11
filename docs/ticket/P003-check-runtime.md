# PHIẾU P003: `check runtime` (INV-010) — port `check-runtime-secrets.py` + parity test chống pin

> **Loại:** Feature (Rust port — phiếu port thứ 2, NỐI vào skeleton P002)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) scan-target patterns (token-in-url, github-pat, db-conn, stripe, …) là SECURITY surface (CLAUDE.md: AUTO Tầng 1, Giám sát review PR); (b) `.git/config` token scan = runtime-secrets surface; (c) exit-code contract 0/1/2 là API pre-commit hook depend; (d) parity-contract quyết định ở Task 3 (env reconstruction) áp cho cả P004-P005; (e) **V2: db-conn lookahead transcription** (Task 2 mục 4b) — đổi text regex trên security surface, dù equivalence-proven, vẫn cần Giám sát soi kỹ.
> **Ảnh hưởng:** `src/main.rs`, `src/checks/mod.rs`, `src/checks/runtime.rs` (MỚI), `tests/parity_runtime.rs` (MỚI), `CHANGELOG.md`, `docs/ARCHITECTURE.md`
> **Dependency:** P001 (pins `runtime--*`) + P002 (CLI skeleton). Anchor #1 ✅ — base có đủ pins + skeleton (`src/main.rs:22-25` enum `CheckCommand { Secrets }`).

> *Note đánh số: BACKLOG Active sprint (single source — Rule 0) đặt `check runtime` = P003. PROJECT.md §Roadmap (ghi rõ "placeholder") đặt runtime = P004 — BACKLOG thắng, không sửa PROJECT.md trong phiếu này.*

---

## Context

### Vấn đề hiện tại

Oracle đã pin (P001): `tests/golden/pins/runtime--dirty.stdout.txt` (exit 1) + `runtime--clean.stdout.txt` (exit 0) + `exit_codes.json`, kèm MANIFEST §1/§2/§5 ghi invocation contract (no args, cwd=repo-root, RUNTIME_FILES + INFRA_GLOBS hardcoded) và unit-spec (line-only, 1-indexed, `:240`). CLI skeleton đã có từ P002. Chưa có dòng Rust nào cho INV-010.

**Điểm khác P002:** INV-010 scan **runtime state**, không phải source tree. Hai hệ quả: (1) parity test phải TÁI TẠO môi trường harness (git remote + file set per `repin.sh:81-87` — anchor #11 ✅) chứ không chỉ copy fixture; (2) clean pin có count line `3 files scanned` — Turn 1 xác nhận count **deterministic** khi tái tạo đúng harness env (anchor #5 ✅): 3 file = `.git/config` + `scripts/check-schema-safety.sh` (repin.sh:53 copy, match `scripts/*.sh`) + `docker-compose.yml` (fixture, match INFRA_TOP_LEVEL).

**V2 — phát hiện CHALLENGE Turn 1:** 4 pattern `db-conn-*` (`golden/check-runtime-secrets.py:100-103`) dùng negative lookahead `(?!\$)` — regex crate không compile (anchor #3 ❌, O1.1). Resolution: Task 2 mục 4b — transcription equivalence-preserving, KHÔNG thêm dep, KHÔNG đổi behavior.

### Giải pháp

Ba phần, đúng method CLAUDE.md (parity trước, cải tiến sau):

1. **CLI nối skeleton** — thêm variant `Runtime` vào `CheckCommand`. KHÔNG đụng path `secrets`, KHÔNG stub `Port`/`Schema`.
2. **Port logic INV-010** — `src/checks/runtime.rs` giữ NGUYÊN behavior `golden/check-runtime-secrets.py` (246 LOC): RUNTIME_FILES, INFRA_GLOBS + top-level infra, đủ pattern inventory PREFIX `:82-111` + GENERIC `:114-116` (Turn 1 enumerate ✅), allowlist `:119-135`, SKIP_EXTENSIONS `:77`, masking `:169-173`, format `:240`, exit code. Ngoại lệ duy nhất: mục 4b (lookahead transcription, equivalence-proven + test-backed).
3. **Parity test** — tái tạo harness env per `repin.sh` → chạy binary → stdout + exit code **BYTE-EXACT** với pins. Kèm unit test synthetic cho MỌI cơ chế parity-blind (fixture chỉ exercise 2/20 pattern class — đều trên `.git/config:14`).

### Scope

- CHỈ sửa/tạo: `src/main.rs` (thêm variant), `src/checks/mod.rs` (thêm `pub mod runtime;`), `src/checks/runtime.rs`, `tests/parity_runtime.rs`, `CHANGELOG.md`, `docs/ARCHITECTURE.md` (+ `tests/golden/MANIFEST.md` §4 note conditional — Task 4).
- KHÔNG sửa: `golden/**` (read-only tuyệt đối), `tests/golden/pins/**` + `fixtures/**` + `repin.sh` (oracle), `src/checks/secrets.rs` + `tests/parity_secrets.rs` (P002 shipped — regression only), `Cargo.toml` (KHÔNG dep mới — kể cả `glob`/`fancy_regex`; xem O1.1 resolution + fallback).
- KHÔNG implement: `check port/schema`, `gate`, `serve`, JSON output (P006).

---

## Task 0 — Verification Anchors

> Bước 0 capability: không network/POST — filesystem scan + `git` CLI trong test harness + cargo. Toàn bộ anchors đã được Worker CHALLENGE Turn 1 verify bằng grep/đọc golden + pins + repin.sh.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Pins runtime + P002 skeleton có trong base | `ls` + `cat exit_codes.json` + grep enum | ✅ RESOLVED (Turn 1) — pins đủ, `runtime--dirty: 1` / `runtime--clean: 0`; `src/main.rs:22-25` có `CheckCommand { Secrets }` `[verified — Worker Turn 1]` |
| 2 | Pattern inventory exhaustive | Worker đọc toàn bộ pattern section golden | ✅ RESOLVED (Turn 1) — PREFIX_PATTERNS `:82-111` (per-class cites: xem Task 2 mục 4) + GENERIC_PATTERN `:114-116` (`generic-entropy`, scan pass riêng `captures_iter`, hit = group(2)). Fixture exercise CHỈ `github-pat-classic` `:86` + `token-in-url` `:95-97` (cùng `.git/config:14`) → mọi class còn lại parity-blind (Task 3 Lưu ý 4) `[verified — Worker Turn 1]` |
| 3 | Golden regex compile được bằng regex crate | grep lookaround + `Regex::new` probe | ❌ CONFIRMED (Turn 1) — 4 pattern `db-conn-*` `:100-103` dùng `(?!\$)`. → **O1.1, resolution Task 2 mục 4b (V2)**. Các pattern còn lại compile sạch `[verified — Worker Turn 1]` |
| 4 | Masking algorithm runtime | đọc golden + đối chiếu pin | ✅ RESOLVED (Turn 1) — `mask()` `:169-173`: `len > 12 → first4 + "..." + last4`; áp `mask(group(0))` cho prefix patterns. Pin dirty khớp: `ghp_...0000` (40 chars), `http...000@` (token-in-url whole match) `[verified — Worker Turn 1]` |
| 5 | Count line "N files scanned" deterministic? | đọc pins + repin.sh | ✅ RESOLVED (Turn 1) — clean pin: `INV-010: PASS (0 runtime/infra secrets, 3 files scanned)`. 3 file enumerated (Context). Count deterministic khi tái tạo đúng harness env — KHÔNG cần normalize `[verified — Worker Turn 1]` |
| 6 | Paths trong pin relative | đọc pin | ✅ RESOLVED (Turn 1) — `.git/config:14: INV-010 violated -- ...`; repin.sh:116-117 normalize `sed "s|${tmp}/||g"` `[verified — Worker Turn 1]` |
| 7 | `runtime.rs` chưa tồn tại; enum chỉ có `Secrets` | ls + grep | ✅ RESOLVED (Turn 1) `[verified — Worker Turn 1]` |
| 8 | Cargo.toml deps đủ | — | ✅ — clap, regex, walkdir, serde_json, anyhow, thiserror; dev: assert_cmd, predicates, tempfile. KHÔNG thêm dep `[verified — Worker Turn 1]` |
| 8b | INFRA_GLOBS implement được không cần `glob` crate | đọc golden + Python glob semantics | ✅ RESOLVED (Turn 1) — pairs 1-level; Python 3.12+ `Path.glob()` trả **sorted alphabetical** → Rust `read_dir` + collect + **sort** + suffix filter `[verified — Worker Turn 1]` |
| 9 | Allowlist `:119-135` port nguyên range | Worker chép exact | ✅ RESOLVED (Turn 1) — INV-009 mirror + extended (gồm `"${"` entry `:133`). Port exact range, KHÔNG ghi count (IG-04) `[verified — Worker Turn 1]` |
| 10 | SKIP_EXTENSIONS `:77` semantics | đọc golden | ✅ RESOLVED (Turn 1) — `(".example", ".md", ".sample", ".template")`, `path.name.endswith(tuple)` = suffix match. Parity-blind → unit test bắt buộc `[verified — Worker Turn 1]` |
| 11 | Harness env tái tạo deterministic | đọc repin.sh | ✅ RESOLVED (Turn 1) — `build_fixture_repo` dirty `repin.sh:81-87`: `git remote add origin "https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git"`; hermetic config (`user.name "P001 Pin Harness"`, `commit.gpgsign false`, fixed dates). Clean: remote không token `[verified — Worker Turn 1]` |
| 12 | Double-firing dirty pin | đối chiếu pin | ✅ RESOLVED (Turn 1) — 2 finding lines cùng `.git/config:14` (`github-pat-classic` + `token-in-url`), không dedupe `[verified — Worker Turn 1]` |
| 13 | Missing file behavior | đọc golden | ✅ RESOLVED (Turn 1) — `collect_runtime_files()` `:203-209`: `if p.exists() and p.is_file()` — vắng → skip silent, KHÔNG tính vào `scanned`, không stderr `[verified — Worker Turn 1]` |
| 14 | Non-UTF-8/unreadable behavior | đọc golden | ✅ RESOLVED (Turn 1) — `scan_file()` `:179-181`: `read_text(encoding="utf-8", errors="ignore")` + `except (UnicodeDecodeError, OSError, IsADirectoryError): return violations`. **Architect note V2:** `errors="ignore"` = file non-UTF-8 vẫn ĐƯỢC SCAN với bytes không decode được bị DROP — KHÁC skip-file của secrets P002, và KHÁC `from_utf8_lossy` (replace U+FFFD). Chỉ OSError/IsADirectory mới skip-file. Spec: Task 2 mục 10 `[verified — Worker Turn 1; semantics note: Architect, needs Worker confirm Turn 2]` |
| 15 | Order findings | đọc golden + pin | ✅ RESOLVED (Turn 1) — `main()` `:231`: `collect_runtime_files() + collect_infra_files()` — RUNTIME_FILES array order `:40-44` → INFRA_GLOBS `:47-52` (sorted per glob) → INFRA_TOP_LEVEL `:56-63` `[verified — Worker Turn 1]` |
| 16 | Sub-mech F là gì | grep golden comments | ✅ RESOLVED (Turn 1) — `:39` comment: "Sub-mech F instance #11 — dotfile token leak 2026-05-28" = phân loại scan-target (dotfile config token leak: `.mcp.json`, `.claude/settings.local.json`), KHÔNG phải cơ chế regex. Ghi Discovery, không ảnh hưởng port `[verified — Worker Turn 1]` |

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P003
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V2 (Turn 1 — Architect accepted O1.1, resolution = equivalence-preserving transcription, không dep mới)

### Turn 1 — Worker Challenge (vs V1)

**Verdict:** FAIL — anchor #3 ❌ (CRITICAL). Worker đã verify 16/16 anchors bằng đọc golden + pins + repin.sh.

**Objection (Tầng 1):**
- [O1.1] `golden/check-runtime-secrets.py:100-103` — 4 pattern `db-conn-{postgresql,mongodb,mysql,redis}` dùng negative lookahead `(?!\$)`; regex crate không support. Oracle `Regex::new` → Err (SOUND). Options Worker đề xuất: **A** thêm `fancy_regex` (parity exact, dep mới = Tầng 1 supply-chain — Worker recommended); **B** rewrite pattern bỏ lookahead VÀ đổi char class (deviation hành vi thật, dựa lưới allowlist `"${"` `:133`).

**Notes (fold vào V2):** pattern inventory đầy đủ (anchor #2); count line deterministic (anchor #5); `read_dir` phải sort alphabetical khớp Python 3.12+ `Path.glob()` (anchor #8b); env reconstruction steps repin.sh:81-87 (anchor #11); Sub-mech F = dotfile token leak classification `:39` (anchor #16).

**Status:** ✅ RESPONDED (xem Architect Response bên dưới)

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT** (blocker thật) — nhưng resolution KHÔNG phải A hay B mà là **C: equivalence-preserving transcription**:
  - Quan sát: trong cả 4 pattern, `(?!\$)` đứng NGAY TRƯỚC class `[^@/\s\$]{8,}` — class này **đã exclude `$`**. Lookahead zero-width assert "ký tự kế ≠ `$`" trước một class không thể match `$` ⇒ **redundant về formal language**: bỏ `(?!\$)` mà giữ NGUYÊN VĂN mọi phần khác (kể cả `\$` trong các char class) cho ra regex match đúng cùng tập input, cùng span. (Trường hợp biên EOS: lookahead pass nhưng class fail vì cần 8+ chars — hai bản vẫn giống nhau.)
  - Khác option B của Worker: B đổi cả char class (`[^:/\s\$]` → `[^:/\s]`) = deviation thật. C chỉ xóa đúng token `(?!\$)`, mọi ký tự khác verbatim.
  - Claim equivalence là **oracle-checkable** → Task 2 mục 4b spec transcription + Task 3 Lưu ý 4(g) spec proof tests bắt buộc.
  - **Fallback:** nếu Worker Turn 2 tìm được counterexample equivalence (input mà golden và bản transcribed khác kết quả) → DỪNG, set `⚠️ AWAITING CHỦ NHÀ`, DEFER chọn A (fancy_regex, supply-chain) vs B (deviation + document). Worker KHÔNG tự thêm dep.
- Notes Tầng 1/2 → fold: anchors 1-16 cập nhật ✅ với cites; Task 2 mục 2 (read_dir+sort), mục 4 (inventory per-class), mục 9 (mask `:169-173`, `mask(group(0))`), mục 10 (errors="ignore" = strip-bytes-then-scan, KHÔNG lossy, KHÔNG skip-file — Architect semantics note, Worker confirm Turn 2); Task 3 Lưu ý 1 (env steps cụ thể), Lưu ý 4 (probe mọi class `:82-111` + GENERIC trừ `:86`/`:95-97`).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Next: Worker CHALLENGE Turn 2 verify consensus — trọng tâm: (i) equivalence claim mục 4b, (ii) semantics `errors="ignore"` anchor #14.

### Turn 2 — Worker Challenge (phiếu V2)

**Anchor verification:** All previously-verified anchors remain ✅. Two open items from Architect Response confirmed below.

**Probe (i) — equivalence claim mục 4b (`(?!\$)` drop):**

Golden source confirmed (`golden/check-runtime-secrets.py:100-103`): the 4 patterns verbatim:
- `:100` `postgresql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@`
- `:101` `mongodb(\+srv)?://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@`
- `:102` `mysql://[^:/\s\$]+:(?!\$)[^@/\s\$]{8,}@`
- `:103` `redis://[^:/\s\$]*:(?!\$)[^@/\s\$]{8,}@`

Structure confirmed: in every case `(?!\$)` stands IMMEDIATELY before `[^@/\s\$]{8,}` with no other token between them. Architect's structural claim is correct for all 4.

Oracle run (`/tmp/probe_equivalence.py`): 15/15 adversarial cases PASS — original and transcribed return identical match-list + span on every case including: `${VAR}` interpolation, real password >=8 chars, password starting with `$`, password exactly 8 chars (boundary), multi-match on one line, all 4 protocol variants (postgresql/mongodb/mysql/redis), `$VAR` (no braces), `$` in middle of password.

Claim: `[O1.1]` equivalence is SOUND.
Oracle: Python regex `finditer` on original vs transcribed across 15 adversarial inputs.
Soundness: SOUND — oracle directly tests the claim.
Verdict: **self-closed via oracle.**

Phiếu proof tests g1/g2/g3: all confirmed correct (g1 → 0 match, g2 → 1 match, g3 → 0 match, both original and transcribed identical). Minor gap: `$` in middle of password (case 8) not in g1-g3 — the class truncates the match at `$`, both versions agree. Recommendation: add as optional g4 in Task 3 Lưu ý 4(g) (belt-and-suspenders, Tầng 2 — Worker may add without re-escalating).

**Probe (ii) — `errors="ignore"` semantics anchor #14:**

Golden: `golden/check-runtime-secrets.py:180` `path.read_text(encoding="utf-8", errors="ignore")`.

Oracle run (`/tmp/probe_errors_ignore.py`) on file with `\xff\xfe` prefix bytes + valid token on next line:
- Invalid bytes: DROPPED (not in decoded string). `U+FFFD` NOT inserted.
- `\n` at end of invalid-byte line: PRESERVED after drop → `splitlines()` still splits correctly.
- Token on line 2: matched at `lineno=2` — line offset UNCHANGED.
- Confirmed: `errors='ignore'` ≠ `from_utf8_lossy` (which inserts `U+FFFD`). Phiếu spec "KHÔNG `from_utf8_lossy`" is correct and necessary.

Rust impl note (Tầng 2, Worker decides at execute): strip invalid bytes via `String::from_utf8` with manual filtering (keep bytes that are valid UTF-8 leads/continuations, preserve `\n`), or collect valid UTF-8 chars only. Either preserves `\n` → line offsets intact. `from_utf8_lossy` is explicitly wrong per spec.

**Status:** ✅ WORKER ACCEPTED V2 — both consensus points verified via oracle. No new objections. Ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: 11/06/2026 (Quản đốc self-approve theo ủy quyền gate kỹ thuật của Chủ nhà cùng ngày; Chủ nhà nghiệm thu retro khi merge sprint)

---

## Nhiệm vụ

### Task 1: Nối CLI skeleton — thêm variant `Runtime`

**File:** `src/main.rs` (sửa — chỉ thêm, không reshape enum P002)

**Thêm:**

```rust
// Vào enum CheckCommand hiện có (src/main.rs:22-25 — anchor #7 ✅):
/// INV-010 — runtime secrets scan (parity port of golden/check-runtime-secrets.py)
Runtime,
```

- Dispatch `Check { Runtime }` → `checks::runtime::run()` → exit code → `std::process::exit(code)` (cùng pattern `Secrets`).

**File:** `src/checks/mod.rs` (sửa): thêm `pub mod runtime;`

**Lưu ý:**
1. KHÔNG đụng variant `Secrets` / `src/checks/secrets.rs` — diff phải rỗng.
2. KHÔNG thêm `Port`/`Schema` stub (precedent P002 Lưu ý 1).
3. Tên type/binding nội bộ = Tầng 2, Worker chọn theo convention P002.

### Task 2: Port logic INV-010 — `src/checks/runtime.rs`

**File:** `src/checks/runtime.rs` (MỚI)

**Thêm:** Port 1:1 từ `golden/check-runtime-secrets.py`. Mỗi surface kèm comment cite `golden/check-runtime-secrets.py:<line>`:

1. **RUNTIME_FILES** (`:40-44`): `.git/config`, `.mcp.json`, `.claude/settings.local.json` — đúng array order. Vắng mặt → skip silent, KHÔNG tính vào `scanned` (`:203-209`, anchor #13 ✅).
2. **INFRA_GLOBS** (`:47-52`): `scripts/*.sh`, `.github/workflows/*.yml`, `.github/workflows/*.yaml`, `hooks/*` — implement bằng `read_dir` + collect + **sort alphabetical** + suffix filter (khớp Python 3.12+ `Path.glob()` sorted — anchor #8b ✅). KHÔNG dep `glob`. 1-level, không recursive.
3. **INFRA_TOP_LEVEL** (`:56-63`): `Dockerfile.nextjs`, `docker-compose.yml`, `docker-compose.staging.yml`, `astro-service/Dockerfile`, `astro-service/docker-compose.yml` — đúng order. Scan order tổng: runtime → globs → top-level (`:231`, anchor #15 ✅).
4. **PREFIX_PATTERNS** (`:82-111`) — chép exact regex + pattern_name từng class từ golden (KHÔNG dùng abridged): `anthropic :83`, `openai :84`, `aws :85`, `github-pat-classic :86`, `github-pat-fine :87`, `google-api :88`, `resend :89`, `telegram-bot :90`, `pem-private-key :93`, `token-in-url :95-97`, `db-conn-* :100-103` (→ mục 4b), `stripe-live-secret :105`, `stripe-test-secret :106`, `stripe-live-restricted :107`, `stripe-test-restricted :108`, `slack-token :110`.

   **4b — db-conn transcription (V2, O1.1 resolution):** 4 pattern `:100-103` chứa `(?!\$)` (regex crate không compile). Port = **xóa đúng token `(?!\$)`, giữ NGUYÊN VĂN mọi ký tự khác** (kể cả `\$` trong các char class `[^:/\s\$]` / `[^@/\s\$]`). Equivalence: class theo sau đã exclude `$` nên lookahead redundant — KHÔNG đổi tập match. Mỗi pattern kèm comment 2 dòng: cite golden:line + note `(?!\$) dropped — redundant, following class excludes '$'; proof: tests (g)`. CẤM mọi sửa đổi khác (không đổi class, không đổi quantifier). Proof tests bắt buộc: Task 3 Lưu ý 4(g).
5. **GENERIC_PATTERN** (`:114-116`, `generic-entropy`): scan pass riêng `captures_iter`, hit = `captures[2]` tương đương Python `group(2)` (anchor #2 ✅).
6. **Allowlist** (`:119-135`): chép exact NGUYÊN RANGE entries + matching semantics từ golden — phiếu cố ý KHÔNG liệt kê entries/count (IG-04). Lưu ý entry `"${"` `:133` là lưới env-interpolation ở string level.
7. **SKIP_EXTENSIONS** (`:77`): `(".example", ".md", ".sample", ".template")` — suffix match trên file name (`endswith` semantics, anchor #10 ✅).
8. **Masking** (`:169-173`, anchor #4 ✅): `len > 12 → first4 + "..." + last4` (nhánh else Worker chép exact từ golden); prefix patterns mask `group(0)` (whole match — token-in-url mask cả URL match: `http...000@`).
9. *(merged vào mục 8 — masking)*
10. **Đọc file** (anchor #14 ✅ + Architect note V2): missing/OSError/IsADirectory → skip silent. **Non-UTF-8 → vẫn SCAN, bytes không decode được bị DROP** (Python `errors="ignore"`). Rust: KHÔNG `from_utf8_lossy` (U+FFFD ≠ drop), KHÔNG skip-file — decode giữ các sequence UTF-8 hợp lệ, bỏ phần invalid (cách implement = Tầng 2, Worker chọn + confirm semantics khớp Python ở Turn 2). Unit test: Task 3 Lưu ý 4(e).
11. **Output**: stdout, format `path:lineno: INV-010 violated -- …` (`:240`, line-only, 1-indexed, F06). Banner/summary/count verbatim **bằng logic**: clean = `INV-010: PASS (0 runtime/infra secrets, N files scanned)` với N đếm từ file thật scan (anchor #5 ✅) — Worker chép exact format string từ golden, KHÔNG hardcode N.
12. **Double-firing** (anchor #12 ✅): 1 dòng match nhiều pattern → in đủ từng finding theo thứ tự pattern trong golden, KHÔNG dedupe (pin dirty: 2 lines cùng `.git/config:14`).
13. **Exit**: ≥1 finding → 1; 0 finding → 0. Không exit path khác (golden không có exit-2 mode — MANIFEST §1).

**Lưu ý:**
1. Harness/fixtures là LF — `lines()` ≡ Python `splitlines()`, 1-indexed (precedent P002 anchor #14).
2. Security surface: ngoài mục 4b (transcription được phép DUY NHẤT, có proof), từng regex/allowlist/skip entry lệch 1 ký tự so golden = vi phạm Luật chơi 1.
3. Nghi golden "sai" (pattern dở) → pin nguyên trạng, ghi DISCOVERY, cải tiến để phiếu sau.

### Task 3: Parity test chống pin

**File:** `tests/parity_runtime.rs` (MỚI — assert_cmd + tempfile + serde_json, deps sẵn)

**Thêm:** 2 test case (dirty / clean), mỗi case:

1. `tempfile::tempdir()` → **tái tạo harness env theo `tests/golden/repin.sh`** (anchor #11 ✅): copy fixture tree + thực hiện EXACT các bước repin.sh phần runtime — dirty: git init + hermetic config (`user.name "P001 Pin Harness"`, `commit.gpgsign false`, fixed `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE`) + `git remote add origin https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git` (`repin.sh:81-87`); clean: remote không token; copy `scripts/check-schema-safety.sh` vào `scripts/` (`repin.sh:53`) — file set quyết định count `3 files scanned`.
2. Chạy binary args `["check", "runtime"]`, **cwd = temp dir** (MANIFEST §2).
3. So sánh:
   - **exit code** == key `runtime--<fixture>` trong `pins/exit_codes.json` (serde_json — KHÔNG hardcode).
   - **stdout** == pin `runtime--<fixture>.stdout.txt` **BYTE-EXACT** (kể cả count line + trailing newline).
   - **stderr** == rỗng (anchor #13 ✅ — golden không in stderr).

**Lưu ý:**
1. Pin là oracle: test ĐỎ → sửa `runtime.rs` hoặc sửa bước dựng env cho ĐÚNG repin.sh — không sửa pin/fixture/repin.sh (Luật chơi 3).
2. Count line: deterministic khi env đúng (anchor #5 ✅) — KHÔNG normalize. Nếu thực thi phát hiện `.git/config` của git local sinh lineno ≠ 14 (git version drift so pin env git 2.50.1) → Debate Log objection, KHÔNG tự normalize (parity-contract Tầng 1 cho P004-P005).
3. Order: byte-exact = order EXACT như pin (anchor #15 ✅ — dirty pin: 2 findings cùng line, order = pattern order golden).
4. **Unit probes — BẮT BUỘC cho MỌI cơ chế parity-blind** (synthetic in-code, F07 — KHÔNG invent fixture file trong `tests/golden/fixtures/`):
   - (a) **allowlist skip direction**: dòng token-shape + entry allowlist (vd chứa `${`) → 0 finding;
   - (b) **SKIP_EXTENSIONS**: file `.example` chứa token thật → skipped;
   - (c) **mỗi pattern class trong `:82-111` + GENERIC `:114-116` mà fixture KHÔNG exercise** (tức tất cả TRỪ `github-pat-classic :86` + `token-in-url :95-97`): 1 firing test / class — synthetic line match → finding đúng format + pattern_name + masking;
   - (d) **missing runtime file**: env thiếu `.mcp.json` → không panic, không tính vào count;
   - (e) **non-UTF-8 file**: file chứa invalid bytes + token hợp lệ ở dòng khác → vẫn scan ra token (strip-bytes semantics, Task 2 mục 10), không panic;
   - (f) **infra-glob matching**: `scripts/a.sh` được scan, `scripts/sub/b.sh` KHÔNG (1-level glob); matched files sorted alphabetical;
   - (g) **db-conn equivalence proof (V2 — O1.1, BẮT BUỘC, cả 4 pattern transcribed)**: (g1) `postgresql://user:${DB_PASSWORD}@host` → 0 finding (class exclude `$` — đúng intent lookahead golden); (g2) `postgresql://user:realpassword123@host` → finding `db-conn-postgresql`; (g3) password bắt đầu bằng `$` (vd `$ecret123456`) → 0 finding. (g2) lặp cho mongodb/mysql/redis.
   - Mapping cơ-chế→test ghi vào Discovery. Probe thêm (double-firing synthetic, masking biên len 12/13) = optional Tầng 2.

### Task 4: Docs gate

**File:** `CHANGELOG.md` — entry P003 (Unreleased): port INV-010, variant `check runtime`, parity vs pin, **note db-conn `(?!\$)` transcription (equivalence-proven, tests (g))**. KHÔNG bump version `Cargo.toml`.
**File:** `docs/ARCHITECTURE.md` — Components: `runtime.rs` từ "planned" → shipped (P003), format dòng như `secrets.rs` (cite range, không count — IG-04). Giữ port/schema planned.
**File:** `tests/golden/MANIFEST.md` §4 — append note ngắn kèm citation NẾU chốt rule mới dùng lại được cho P004-P005 (env-reconstruction rule; glob sorted rule; db-conn transcription precedent). Không có gì mới → không đụng.
**File:** `docs/discoveries/P003.md` + index 1-line `docs/DISCOVERIES.md` — gồm mục riêng: (i) pattern inventory `:82-111`/`:114-116` per-class cites; (ii) Sub-mech F = dotfile token leak classification (`:39`, anchor #16 ✅); (iii) db-conn transcription + kết quả proof tests; (iv) `errors="ignore"` semantics đã port thế nào; (v) env-reconstruction approach + mapping cơ-chế→unit-test.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Task 1: thêm variant `Runtime` + dispatch |
| `src/checks/mod.rs` | Task 1: thêm `pub mod runtime;` |
| `src/checks/runtime.rs` | Task 2: MỚI — port INV-010, mọi pattern/skip cite golden:line; mục 4b transcription |
| `tests/parity_runtime.rs` | Task 3: MỚI — 2 parity test + unit probes (a)-(g) |
| `CHANGELOG.md` | Task 4: entry P003 + transcription note |
| `docs/ARCHITECTURE.md` | Task 4: runtime.rs shipped |
| `tests/golden/MANIFEST.md` | Task 4: §4 note — CONDITIONAL |
| `docs/discoveries/P003.md` + `docs/DISCOVERIES.md` | Discovery report + 1-line index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng khi nghiệm thu |
| `tests/golden/pins/**`, `tests/golden/fixtures/**`, `tests/golden/repin.sh` | Oracle — `git diff` rỗng. Parity fail → sửa Rust/env-setup |
| `src/checks/secrets.rs`, `tests/parity_secrets.rs` | P002 shipped — diff rỗng, test vẫn xanh |
| `Cargo.toml` | KHÔNG dep mới (`glob`/`fancy_regex` đều cấm trừ khi O1.1 fallback kích hoạt qua Chủ nhà), không bump version. Diff rỗng |
| `CLAUDE.md` | Chỉ đụng nếu phát hiện deviation exit-code MỚI — kỳ vọng: không |

---

## Luật chơi (Constraints)

1. **Parity-first, security surface:** KHÔNG đổi/thêm/nới pattern, pattern_name, allowlist entry, skip rule, masking, output wording, count logic so với oracle. **Ngoại lệ DUY NHẤT:** Task 2 mục 4b — xóa đúng token `(?!\$)` trong 4 pattern db-conn, equivalence-proven + proof tests (g) bắt buộc xanh. Mỗi surface mang comment cite golden:line. PR cần Giám sát review (CLAUDE.md) — highlight mục 4b trong PR description.
2. **regex crate only, KHÔNG dep mới.** Nếu Worker tìm được counterexample equivalence mục 4b → DỪNG, Debate Log objection, status `⚠️ AWAITING CHỦ NHÀ` — DEFER chọn fancy_regex (A) vs deviation-documented (B). Worker KHÔNG tự thêm dependency (supply-chain surface). Tương tự `glob` crate (không cần — anchor #8b ✅).
3. **Pin bất khả xâm phạm:** parity đỏ → sửa Rust hoặc env-setup. Sửa pin/fixture/repin.sh/MANIFEST-để-test-xanh = vi phạm scope. (`MANIFEST.md` ngoại lệ: Task 4 conditional note.)
4. **Byte-exact** gồm banner/count line + trailing newline. Count bằng LOGIC, không hardcode. Không print stdout ngoài golden; stderr rỗng trong parity run.
5. **Exit contract:** runtime logic chỉ exit 0/1. Exit 2 = clap usage error duy nhất (deviation đã document P001).
6. **CLI tối thiểu:** chỉ thêm `check runtime`. Không flag, không JSON (P006), không stub, không đụng `secrets`.
7. **PROJECT.md constraints:** không mutate file scan, không network, cross-platform path API (output dùng `/` như golden).
8. **Cite RANGE, không count** (IG-04): comment + docs ghi `golden:<line-range>`, KHÔNG ghi "N entries".
9. `cargo test` xanh toàn bộ trước commit — gồm parity_secrets regression.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — gồm 2 parity test: `runtime--dirty` exit 1 + stdout byte-exact pin (2 findings `.git/config:14`); `runtime--clean` exit 0 + stdout byte-exact pin (PASS + `3 files scanned`)
- [ ] Unit probes (a)-(f) xanh — MỌI pattern class `:82-111` + GENERIC không được fixture exercise đều có firing test
- [ ] **Proof tests (g1)-(g3) xanh cho cả 4 db-conn pattern transcribed (V2 — O1.1)**
- [ ] Regression: `tests/parity_secrets.rs` xanh, `git diff src/checks/secrets.rs tests/parity_secrets.rs` rỗng
- [ ] `cargo check` / build sạch
- [ ] `git diff golden/ tests/golden/fixtures/ tests/golden/pins/ tests/golden/repin.sh` rỗng
- [ ] `git diff Cargo.toml` rỗng

### Manual Testing
- [ ] Dogfood: `cargo run -- check runtime` tại repo root inv-gate — kỳ vọng exit 0. **NẾU ra finding thật trong `.git/config` của chính repo → DỪNG, báo Chủ nhà NGAY (token leak thật, không phải bug test)**
- [ ] `cargo run -- check bogus` → clap usage error, exit 2 (regression contract P002)
- [ ] Dựng env dirty thủ công theo repin.sh → mắt thường đối chiếu findings (double-firing `.git/config:14` — anchor #12) khớp pin
- [ ] Mỗi pattern/allowlist/skip constant trong `runtime.rs` có comment cite golden:line; 4 pattern db-conn có comment transcription note

### Regression
- [ ] `bash tests/golden/repin.sh` vẫn chạy được + `git diff tests/golden/pins/` rỗng sau khi chạy

### Docs Gate
- [ ] `CHANGELOG.md` — entry P003 + transcription note
- [ ] `docs/ARCHITECTURE.md` — `runtime.rs` shipped, hết "planned"
- [ ] `tests/golden/MANIFEST.md` §4 — note nếu có (kèm citation), không thì ghi "None" trong discovery
- [ ] Discovery hook P004+: env-reconstruction + glob-sorted rule + transcription precedent ghi rõ cho `check port`/`check schema` (P004) và `gate` (P005)

### Discovery Report
- [ ] `docs/discoveries/P003.md`: assumptions ĐÚNG/SAI (cite file:line — đặc biệt #3 lookahead + resolution, #5 count, #8b glob sorted, #14 errors="ignore" semantics, #16 Sub-mech F), pattern inventory, mapping cơ-chế→unit-test, tier escalation ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
