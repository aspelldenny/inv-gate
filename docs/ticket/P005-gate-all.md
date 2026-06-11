# PHIẾU P005: `gate --all` — orchestrator port `security-gate.sh` (210 LOC) + dogfood swap PER-CHECK trong `scripts/security-gate.sh`

> **Loại:** Feature (Rust port — phiếu CUỐI Sprint 1: orchestrator + dogfood wiring)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) orchestrator = TOÀN BỘ security surface (gọi 3 script-check + 6 inline check, aggregate); (b) exit-code contract 0/1/2 là API pre-commit hook depend (CLAUDE.md); (c) Part 2 sửa `scripts/security-gate.sh` — file mà `hooks/pre-commit [4/7]` đang gọi = enforcement point của repo — sai 1 dòng là gate hổng hoặc block mọi commit; (d) flag mapping `--all` ↔ `--mechanical-only` là CLI contract deviation phải document.
> **Ảnh hưởng:** `src/main.rs`, `src/gate.rs` (MỚI), `tests/parity_gate.rs` (MỚI), `scripts/security-gate.sh` (dogfood per-check — QUYẾT ĐỊNH CHỦ NHÀ Turn 1), `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `CLAUDE.md` (1 dòng mapping), `tests/golden/MANIFEST.md` (§4 conditional)
> **Dependency:** P001 (pins `gate--*`) + P002/P003/P004 (4 check modules + parity harness). BACKLOG: P001-P004 ĐỀU shipped. Base branch: stack trên `feat/P004-check-port-schema` (HEAD `683098c` per BACKLOG) `[unverified — Worker confirm anchor #15]`.

> *Note scope: BACKLOG Active sprint (Rule 0, single source) đặt P005 = `gate --all` + dogfood swap. PROJECT.md §Roadmap (placeholder) đánh số khác (P007-P008) — BACKLOG thắng, precedent P003/P004. `--include-ssh` / SSH mode = NGOÀI scope (Sprint 2+). Profile/flag mode cho gate đa-repo = phiếu riêng Sprint 2 (Chủ nhà decision (c) — xem Sprint close-out).*

---

## Context

### Vấn đề hiện tại

4 check đã port + parity xanh (63 tests): `check secrets|runtime|port|schema`. Oracle gate đã pin (P001): `gate--{dirty,clean}.{stdout,stderr}.txt` + `gate--usage-error.stdout.txt`, exit codes `gate--dirty: 1`, `gate--clean: 0`, `gate--usage-error: 2` `[verified — MANIFEST §6]`. **Pin là nhánh `--mechanical-only`** — INV-007 SSH (`root@103.167.150.178:1994` tại `:147`) bị skip HẲN khi `--mechanical-only` (`:171-174` else clause, zero output) `[verified — MANIFEST §1 + Turn 1 enumeration]`. Chưa có dòng Rust nào cho `gate`.

**Điểm khác P002-P004 — đây là phiếu port ORCHESTRATOR, không phải check:**

1. `golden/security-gate.sh` (210 LOC bash) gọi 3 script python (`:55` port, `:197` secrets, `:201` runtime) `[verified — MANIFEST §1]` + **6 inline check** (INV-002..006 bash + INV-008 Python, ≈83 LOC tổng, mỗi cái <40 LOC) `[verified — Worker Turn 1 enumeration]`. **Schema check KHÔNG nằm trong gate** (pin confirm — Turn 1) → gate.rs KHÔNG gọi `checks::schema::run()`. Rust thay subprocess bằng **in-process function call** — quan sát được (stdout/stderr/exit) phải identical, cơ chế bên trong khác là OK.
2. **Aggregate = ACCUMULATOR** `[verified — Turn 1]`: `run()` wrapper (`:27-36`) in section header + `  PASS`/`  FAIL` + blank line, tăng FAIL, KHÔNG exit sớm; exit 1 cuối nếu FAIL>0 (`:206-209`). Summary `:204-210`: `====...` + `Security gate: $PASS passed, $FAIL failed, $WARN warnings` + `Failed invariants: ${FAILED_INVS[*]}` CHỈ khi FAIL>0.
3. **Stderr contract per-check KHÁC NHAU** (MANIFEST §4 rule 7, P004 discovery §Hook): port emit 2 dòng WARN ra stderr, secrets/runtime stderr rỗng. Gate preserve từng stream độc lập — parity assert **stdout VÀ stderr byte-exact** vs pins.
4. **Part 2 dogfood — QUYẾT ĐỊNH CHỦ NHÀ (Turn 1, AskUserQuestion):** repo này wire hook qua `core.hooksPath=hooks` ✅, `hooks/pre-commit` block `[4/7]` (lines 207-220) gọi `bash scripts/security-gate.sh --mechanical-only > /tmp/sos-secgate.log 2>&1` `[verified — Turn 1 anchor #11]`. `scripts/security-gate.sh` của repo là bản **ADAPTED 99 LOC** — KHÁC golden 210 LOC. Swap = **PER-CHECK**: thay các call `python3 scripts/check-*.py` BÊN TRONG bản adapted bằng `inv-gate check secrets|runtime|port` tương ứng — kill python3, giữ nguyên phần bash còn lại, KHÔNG coverage loss, KHÔNG sửa `hooks/pre-commit`.

**Blocker schema (anchor #7 cũ): RESOLVED MOOT** — schema không nằm trong gate, và per Chủ nhà decision (a): acceptance của `gate --all` = fixture-based (pins `gate--*`), KHÔNG yêu cầu pass trên repo này.

### Giải pháp

1. **CLI:** thêm subcommand `Gate` với flag `--all` (required). `inv-gate gate --all` ≡ golden `security-gate.sh --mechanical-only` (nhánh đã pin). KHÔNG implement `--mechanical-only`/`--include-ssh` riêng trong P005. Deviation flag-surface document ở: CHANGELOG + `docs/ARCHITECTURE.md` + **1 dòng `CLAUDE.md` §Rules**. Unknown flag → clap usage error exit 2 (golden: echo + exit 2 tại `:14` `[verified — Turn 1]`; text deviation đã document P001 — anchor #10).
2. **`src/gate.rs`:** port 1:1 nhánh mechanical-only theo bảng enumeration Turn 1 (Task 2): 3 check script → in-process `checks::{port,secrets,runtime}::run()`, 6 inline check (INV-002..006 + INV-008) → port Rust thuần TRONG `gate.rs` (layout: private fn per INV — quyết định Architect V2; KHÔNG tạo module con `src/checks/` mới, inline checks là ruột orchestrator, không phải check CLI public). INV-008 inline Python → Rust thuần, KHÔNG gọi python3 (mục tiêu kill python3); parity chống pin. Accumulator + summary đúng `:27-36`/`:204-210`.
3. **Parity:** `tests/parity_gate.rs` — fixture env mirror đúng section gate của `repin.sh` (anchor #8): git repo 2-commit + remote-inject + compose + `src/config.ts`; thành phần thêm cho inline checks → theo đúng repin.sh (Worker confirm khi EXECUTE; schema-flow chỉ giữ nếu repin.sh có — gate không gọi schema). Stdout + stderr + exit BYTE-EXACT vs pins `gate--{dirty,clean}`. Usage-error: exit 2. Kèm unit probes (Task 3).
4. **Dogfood swap PER-CHECK** (Chủ nhà decision (b)): sửa `scripts/security-gate.sh` (adapted 99 LOC) — build-guard fail-closed + thay từng call python3 bằng binary, comment giữ dòng python3 cũ (reversible). Commit P005 = proof-commit, transcript hook vào Discovery.
5. **Sprint close-out:** mục riêng cuối phiếu — owner = ORCHESTRATOR (Worker không edit BACKLOG), gồm append item Sprint 2 "profile/flag mode gate đa-repo" (decision (c)).

### Scope

- CHỈ sửa/tạo: `src/main.rs`, `src/gate.rs`, `tests/parity_gate.rs`, `scripts/security-gate.sh` (CHỈ build-guard + các cặp comment/call — Task 4), `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `CLAUDE.md` (1 dòng), `tests/golden/MANIFEST.md` (§4 conditional), `docs/discoveries/P005.md`, `docs/DISCOVERIES.md`.
- Ngoại lệ có điều kiện: `src/checks/{secrets,runtime,port}.rs` + `src/checks/mod.rs` — CHỈ visibility/signature tối thiểu nếu `run()` chưa public/chưa return code (anchor #6); behavior diff = 0, parity P002-P004 xanh nguyên. Tương tự `tests/parity_runtime.rs`/`tests/common/` nếu extract helper dùng chung (precedent P004 — Tầng 2, Worker quyết).
- KHÔNG sửa: `golden/**` (read-only tuyệt đối), `tests/golden/pins/**` + `fixtures/**` + `repin.sh` (oracle), **`hooks/pre-commit`** (verify wiring only — decision (b) không chạm), `scripts/check-*.py` (đường lui dogfood — GIỮ NGUYÊN, không xóa), `src/checks/schema.rs` (gate không gọi schema), `Cargo.toml` (KHÔNG dep mới, KHÔNG bump version — v0.1.0 là Sprint 2).
- KHÔNG implement: `serve` (Sprint 2), JSON output, SSH mode, `--mechanical-only`/`--include-ssh` flags, profile/flag mode đa-repo (Sprint 2 — decision (c)), cải tiến pattern/SHA (BACKLOG [DEBT]), parallel execution.

---

## Task 0 — Verification Anchors

> Bước 0 capability: không network — filesystem + `git` CLI trong harness + cargo + sửa 1 file bash. Anchors ✅ = chốt Turn 1 (Worker CHALLENGE đã verify, evidence trong Debate Log). Anchors ⏳ còn lại Worker verify khi EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Pins gate đủ 5 file: `gate--{dirty,clean}.{stdout,stderr}.txt` + `gate--usage-error.stdout.txt`; exit_codes `1/0/2` | `ls tests/golden/pins/gate--*` + `cat exit_codes.json` | exit codes `[verified — MANIFEST §6]`; file set stderr `[unverified — Worker ls khi EXECUTE]` |
| 2 | Execution path mechanical-only enumerate đủ (order + mọi output line + golden:line + stream) | Worker đọc 210 LOC | ✅ Turn 1 — bảng fold vào Task 2 mục 1 |
| 3 | Inline check giữa các script call | enumeration #2 | ✅ RESOLVED — 5 inline bash (INV-002..006) + 1 inline Python (INV-008), ≈83 LOC, mỗi cái <40 LOC → TRONG scope, port hết vào `gate.rs` |
| 4 | Aggregate exit logic | đọc golden + pins | ✅ ACCUMULATOR — run() tăng FAIL không exit sớm; exit 1 cuối nếu FAIL>0 (`:206-209`) |
| 5 | `gate--dirty`: check nào fail; `gate--clean`: all pass | `cat` pins + đối chiếu repin.sh | ⏳ TO VERIFY khi EXECUTE `[needs Worker verify — parity assert sẽ chốt]` |
| 6 | 3 module secrets/runtime/port expose `pub fn run() -> <code>` (return, KHÔNG tự `process::exit`) — schema KHÔNG cần (không trong gate) | grep signatures `src/checks/*.rs` + `src/main.rs` dispatch | `[unverified — per P004 Task 1 design; Worker confirm]` |
| 7 | ~~Golden gate guard schema~~ | — | ✅ MOOT — schema KHÔNG nằm trong gate (pin confirm Turn 1) |
| 8 | repin.sh harness cho `gate--*`: fixture layout (union surface), copy scripts vào `$tmp/scripts/`, remote-inject, flag `--mechanical-only` | đọc `tests/golden/repin.sh` section gate, cite lines | ⏳ TO VERIFY — `$tmp/scripts/` layout `[verified — MANIFEST §1]`, phần còn lại `[needs Worker verify]` |
| 9 | Usage-error parity = exit-code-only (deviation P001) | precedent `tests/parity_secrets.rs` + CHANGELOG | `[unverified — per P004 Luật chơi 5; Worker confirm]` |
| 10 | `gate--usage-error` pin chỉ có `.stdout.txt` — Rust KHÔNG cần match text | ls pins | `[unverified — Worker confirm]` |
| 11 | Hook `[4/7]` gọi gì | đọc `hooks/pre-commit` | ✅ Turn 1 — block `[4/7]` lines 207-220: `bash scripts/security-gate.sh --mechanical-only > /tmp/sos-secgate.log 2>&1` |
| 12 | Hook wiring | `git config core.hooksPath` | ✅ Turn 1 — `core.hooksPath=hooks`, file đang chạy = `hooks/pre-commit` |
| 13 | (REFRAMED) Sau swap per-check: `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree repo này | chạy tại repo root sau `cargo build --release` | ⏳ TO VERIFY — Task 4 mục 5 |
| 14 | `src/gate.rs` chưa tồn tại; `main.rs` chưa có variant `Gate`; deps đủ | ls + grep | `[unverified — ARCHITECTURE ghi "planned"; Worker confirm]` |
| 15 | Base branch `feat/P004-check-port-schema` đủ P002-P004 (63 tests xanh) | git log + `cargo test` | `[unverified — BACKLOG ghi shipped `683098c`; Worker confirm]` |
| 16 | ~~Schema `run()` env passthrough~~ | — | ✅ MOOT — gate không gọi schema |
| 17 | `scripts/security-gate.sh` adapted = 99 LOC, kỳ vọng 3 call `python3 scripts/check-*.py` (secrets/runtime/port) | Worker enumerate 99 LOC khi EXECUTE, map verbatim từng call → binary call (Discovery) | ⏳ TO VERIFY — file SOS-KIT, KHÔNG dùng cite golden cho file này |
| 18 | `scripts/check-*.py` (spine copy) identical golden counterpart hay adapted | `diff scripts/check-*.py golden/...` | ⏳ TO VERIFY — chỉ ghi Discovery, không block (hook depend exit code + log capture, không parse text) |

**Anchor ⚠️ còn lại (Worker verify khi EXECUTE): #1 file-set, #5, #8, #13, #17, #18.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P005
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp scripts/security-gate.sh ".backup/${PHIEU_ID}/security-gate.sh.orig"   # đường lui dogfood
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V2 (Turn 1 folded — enumeration + Chủ nhà decision dogfood per-check)

### Turn 1 — Worker Challenge (vs V1)

**Anchor verification:**
- #2 ✅ enumeration ĐẦY ĐỦ (bảng đầy đủ trong challenge report, Architect fold vào Task 2 mục 1): 9 INV section + run() wrapper `:27-36` + unknown-flag `:14` + summary `:204-210`.
- #3 ✅ RESOLVED — 5 inline bash (INV-002 `:58-71`, INV-003 `:75-85`, INV-004 `:89-110`, INV-005 `:113-122`, INV-006 `:126-135`) + 1 inline Python (INV-008 `:179-192`), ≈83 LOC, mỗi cái <40 LOC.
- #4 ✅ ACCUMULATOR — run() tăng FAIL, không exit sớm; exit 1 cuối nếu FAIL>0 (`:206-209`).
- #7 ✅ MOOT — schema check KHÔNG nằm trong gate (pin confirm).
- #11/#12 ✅ — `core.hooksPath=hooks`; `hooks/pre-commit` block `[4/7]` lines 207-220 gọi `bash scripts/security-gate.sh --mechanical-only > /tmp/sos-secgate.log 2>&1`; `scripts/security-gate.sh` repo = bản ADAPTED **99 LOC** ≠ golden 210 LOC.

**Objections (Tầng 1):**
- [O1.1] Dogfood semantics: hook đang chạy bản adapted 99 LOC, KHÔNG phải golden 210. Swap `[4/7]` sang `gate --all` (port golden) = ĐỔI coverage của hook, không phải swap tương đương. Cần Chủ nhà quyết: per-check swap / gate --all chấp nhận diff / profile mode.

**Status:** ✅ RESPONDED (Turn 1 Architect Response, phiếu V2)

### Turn 1 — Chủ nhà Decision (qua AskUserQuestion)

- **(a)** `gate --all` vẫn port parity ĐẦY ĐỦ với golden; acceptance = fixture-based (pins `gate--*`), KHÔNG yêu cầu pass trên repo này.
- **(b)** Dogfood swap = **PER-CHECK**: thay các call `python3 scripts/check-*.py` BÊN TRONG `scripts/security-gate.sh` (bản adapted 99 LOC) bằng binary per-check (`inv-gate check secrets` / `check runtime` / `check port`) — kill python3, giữ nguyên phần bash còn lại, KHÔNG coverage loss. KHÔNG sửa `hooks/pre-commit`.
- **(c)** Profile/flag mode cho gate đa-repo → phiếu riêng Sprint 2 (orchestrator append BACKLOG ở Sprint close-out).

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → DEFER TO CHỦ NHÀ → RESOLVED per decision (b): Task 4 viết lại per-check swap trong `scripts/security-gate.sh`; file này chuyển sang Files cần sửa; `hooks/pre-commit` → verify-only.
- Anchors #2/#3/#4 → ACCEPT, fold: bảng enumeration vào Task 2 mục 1; inline checks (INV-002..006 + INV-008) thuộc scope, layout = private fn trong `src/gate.rs` (quyết định Architect); INV-008 Python → Rust thuần; aggregate = accumulator + summary verbatim.
- Anchor #7/#16 → ACCEPT (MOOT): schema KHÔNG trong gate — bỏ `checks::schema::run()` khỏi gate, bỏ probe ALLOW_DATA_LOSS + schema-guard, bỏ blocker Context.
- Anchor #13 → REFRAME: điểm đo dogfood = `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree SAU swap (Task 4 mục 5).
- Binary path trong hook → Architect quyết: binary tĩnh `target/release/inv-gate` + build-guard fail-closed đầu script (KHÔNG `cargo run --release` trong hook — build lúc commit chậm + output cargo lẫn log); hook chạy ở repo root (core.hooksPath) nên path tương đối OK.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Final consensus
- Phiếu version: V2
- Total turns: 1 (+ 1 Chủ nhà decision via AskUserQuestion)
- Approved by Chủ nhà: [pending — dogfood approach đã approve Turn 1; full phiếu approval tại APPROVAL_GATE]

---

## Nhiệm vụ

### Task 1: CLI — subcommand `gate --all`

**File:** `src/main.rs` (sửa — chỉ thêm, không reshape enum `check` P002-P004)

**Thêm:**
- Variant `Gate` (ngang hàng `Check`) với arg bool `--all` **required** (`gate` trần → clap usage error exit 2).
- `mod gate;` + dispatch `Gate { all: true }` → `gate::run()` → `std::process::exit(code)` (pattern P002-P004).
- Doc-comment variant: `/// Orchestrator — parity port of golden/security-gate.sh --mechanical-only branch`.

**Lưu ý:**
1. KHÔNG flag `--mechanical-only`/`--include-ssh` (scope). KHÔNG đụng `check` variants.
2. Unknown flag (`gate --no-such-flag`) → clap error exit 2 = parity exit-code pin `gate--usage-error` (golden `:14`; text deviation đã document P001 — KHÔNG cố match usage text golden).
3. Tên type/binding nội bộ = Tầng 2.

### Task 2: Port orchestrator — `src/gate.rs`

**File:** `src/gate.rs` (MỚI)

**Thêm:** Port 1:1 nhánh `--mechanical-only` theo bảng enumeration ✅ Turn 1. Mỗi dòng output + mỗi quyết định flow kèm comment cite `golden/security-gate.sh:<line>`:

1. **Thứ tự + cơ chế (bảng chốt Turn 1):**

   | # | Section | Golden lines | Cơ chế port |
   |---|---------|-------------|-------------|
   | 0 | Unknown flag → echo + exit 2 | `:14` | clap (Task 1, deviation P001) |
   | 0b | `run()` wrapper: section header, `  PASS`/`  FAIL`, blank line; tăng PASS/FAIL + FAILED_INVS | `:27-36` | helper fn trong `gate.rs` |
   | 1 | INV-001 port-bind (script) | `:55` | `checks::port::run()` in-process |
   | 2 | INV-002 inline bash | `:58-71` | private fn Rust trong `gate.rs` |
   | 3 | INV-003 inline bash | `:75-85` | private fn Rust |
   | 4 | INV-004 inline bash | `:89-110` | private fn Rust |
   | 5 | INV-005 inline bash | `:113-122` | private fn Rust |
   | 6 | INV-006 inline bash | `:126-135` | private fn Rust |
   | 7 | INV-007 SSH | `:147`, skip `:171-174` | SKIP HẲN — zero output trong mechanical-only |
   | 8 | INV-008 inline Python | `:179-192` | private fn **Rust thuần — KHÔNG gọi python3** |
   | 9 | INV-009 secrets (script) | `:197` | `checks::secrets::run()` in-process |
   | 10 | INV-010 runtime (script) | `:201` | `checks::runtime::run()` in-process |
   | 11 | Summary | `:204-210` | `====...` + `Security gate: $PASS passed, $FAIL failed, $WARN warnings` + `Failed invariants: ${FAILED_INVS[*]}` CHỈ khi FAIL>0 |

   Logic bên trong từng inline check (INV-002..006, INV-008): Worker enumerate verbatim khi EXECUTE từ chính range đã cite, parity chống pin. Cơ chế tăng `$WARN` `[needs Worker verify — đọc golden, kỳ vọng từ output check hoặc wrapper]`.
2. **Mọi dòng echo/banner/PASS-FAIL/summary:** VERBATIM từng byte (kể cả emoji/em-dash/khoảng trắng), đúng stream (stdout vs stderr), đúng vị trí. Nguồn byte-truth = pins `gate--{dirty,clean}` + golden source. Lệch 1 ký tự = parity đỏ.
3. **Gọi check in-process:** `checks::{port,secrets,runtime}::run()` — TUẦN TỰ, single-thread, capture return code (anchor #6). **KHÔNG gọi `checks::schema::run()`** (schema không trong gate — Turn 1). KHÔNG spawn binary chính mình, KHÔNG spawn python. Check tự in stdout/stderr như khi chạy lẻ — gate KHÔNG capture/reformat.
4. **Aggregate exit = ACCUMULATOR (✅ Turn 1):** chạy hết mọi section, exit 1 cuối nếu FAIL>0 (`:206-209`), exit 0 nếu không. Chỉ exit 0/1 từ logic gate (2 là clap).
5. **Layout (quyết định Architect V2):** tất cả inline check = private fn trong `src/gate.rs` (vd `inv_002()`...), KHÔNG tạo module con mới — inline là ruột orchestrator, `src/checks/` chỉ cho check CLI public. Tên fn nội bộ = Tầng 2.

**Lưu ý:**
1. Đặt file tại `src/gate.rs` đúng ARCHITECTURE.md (đã ghi "planned").
2. Nghi golden "sai" → pin nguyên trạng + DISCOVERY, cải tiến để phiếu sau (Luật chơi 1).
3. Inline nào hóa ra >40 LOC hoặc cần surface fixture chưa pin (trái với Turn 1) → DỪNG, Debate Log objection — KHÔNG tự port mở rộng im lặng.

### Task 3: Parity test — `tests/parity_gate.rs`

**File:** `tests/parity_gate.rs` (MỚI — assert_cmd + tempfile + serde_json, deps sẵn)

**Thêm:** 2 parity case (dirty/clean) + 1 usage-error case:

1. **Harness = mirror đúng section gate của `repin.sh`** (anchor #8): tempdir → git repo 2-commit hermetic (config + fixed dates `2026-01-01T00:00:00 +0000`) + **remote-inject** (runtime check đọc `.git/config`) + `src/config.ts` + `docker-compose.yml` (1 file — 2 WARN port fire đúng pin) + mọi thành phần repin.sh dựng cho inline checks INV-002..006/008 (Worker đối chiếu khi EXECUTE; schema files chỉ nếu repin.sh có — gate không gọi schema). Reuse/extend `build_fixture_repo()` (`tests/parity_runtime.rs:71-132` `[verified — P004 anchor #14]`); cấu trúc helper = Tầng 2.
2. Chạy binary `["gate", "--all"]`, cwd = temp root, `env_remove("ALLOW_DATA_LOSS")` (hermetic — Luật chơi 8).
3. Assert: exit == `gate--<fixture>` từ `exit_codes.json` (KHÔNG hardcode); **stdout BYTE-EXACT** pin; **stderr BYTE-EXACT** pin (chứa WARN port — MANIFEST §4 rule 7).
4. Usage-error: `["gate", "--no-such-flag"]` → exit 2, KHÔNG assert text (anchor #9).

**Unit probes — BẮT BUỘC cho mọi cơ chế parity-blind (synthetic in-code, F07 — KHÔNG invent fixture trong `tests/golden/fixtures/`):**
   - (a) đúng 1 check fail → các section SAU vẫn chạy (accumulator evidence) + gate exit 1 (`:206-209`);
   - (b) all clean → exit 0 + đủ mọi section line + summary `0 failed`;
   - (c) `gate` trần (thiếu `--all`) → exit 2;
   - (d) **inline checks: ≥1 probe / inline mechanism** (INV-002..006 + INV-008 = tối thiểu 6 probes) — mỗi probe trigger đúng cơ chế fail/pass của inline đó;
   - (e) summary conditional: FAIL>0 → CÓ dòng `Failed invariants: ...`; FAIL=0 → KHÔNG có dòng đó (`:204-210`).
   - Mapping cơ-chế→test ghi vào Discovery (precedent P003/P004).

**Lưu ý:**
1. Pin là oracle: test ĐỎ → sửa `gate.rs` hoặc harness cho đúng repin.sh — KHÔNG sửa pin/fixture/repin.sh (Luật chơi 3).
2. Ordering output khớp pin EXACT — không sort, không parallel.
3. Regression: `parity_{secrets,runtime,port,schema}` + 63 tests hiện có xanh nguyên.

### Task 4: Dogfood swap PER-CHECK — `scripts/security-gate.sh` (adapted, 99 LOC)

**File:** `scripts/security-gate.sh` (sửa — CHỈ: 1 build-guard block + các cặp comment/call python3→binary. Mọi bash khác giữ nguyên BYTE. KHÔNG sửa `hooks/pre-commit` — decision (b))

> File này là bản ADAPTED của SOS-KIT (99 LOC) — KHÁC `golden/security-gate.sh` (210 LOC). KHÔNG dùng cite golden:line cho file này. `golden/` vẫn read-only tuyệt đối.

1. **Enumerate trước khi sửa (anchor #17):** Worker đọc đủ 99 LOC, map từng call `python3 scripts/check-*.py` → binary call tương ứng: `check-hardcoded-secrets.py` → `check secrets`; `check-runtime-secrets.py` → `check runtime`; `check-port-bind.py` → `check port`. Số call thực tế + args verbatim ghi vào Discovery làm record đường lui. Call nào KHÔNG map được vào 1 trong 3 check đã port → DỪNG, Debate Log.
2. **Build-guard fail-closed đầu script** (sau shebang/setup — quyết định Architect, trong options Chủ nhà cho):
   ```bash
   # P005 dogfood — hook chạy ở repo root (core.hooksPath=hooks), path tương đối OK
   INV_GATE="target/release/inv-gate"
   [ -x "$INV_GATE" ] || { echo "❌ inv-gate chưa build — chạy: cargo build --release (đường lui: uncomment các dòng python3, hoặc .backup/P005/security-gate.sh.orig)" >&2; exit 1; }
   ```
   Fail-closed: thiếu binary → exit 1 → hook block commit, KHÔNG silent-skip. KHÔNG dùng `cargo run --release` trong hook (build-at-commit chậm + output cargo lẫn `/tmp/sos-secgate.log`). Wording/vị trí align convention script — Tầng 2.
3. **Mỗi call: comment dòng python3 cũ NGAY TRÊN dòng mới** (reversible — Chủ nhà decision (b)):
   ```bash
   # P005: python3 scripts/check-hardcoded-secrets.py <args cũ verbatim>
   "$INV_GATE" check secrets
   ```
   Giữ NGUYÊN cách script tiêu thụ exit code (wrapper/`||`/`if` hiện có) — exit contract 0/1 identical (CLAUDE.md), output per-check identical vì P002-P004 parity byte-exact. Args cũ nếu binary không có flag tương đương → DỪNG, Debate Log (kỳ vọng không có — check CLI P002-P004 không nhận args ngoài subcommand `[unverified — Worker confirm]`).
4. **KHÔNG coverage loss:** `git diff scripts/security-gate.sh` CHỈ được chứa guard block + các cặp comment/call. Phần bash inline còn lại của adapted gate giữ nguyên byte.
5. **Pre-check trước proof-commit (anchor #13 reframed):** `cargo build --release` rồi `bash scripts/security-gate.sh --mechanical-only; echo $?` tại repo root trên clean tree → kỳ vọng exit 0. Khác 0 → DỪNG, Debate Log objection (KHÔNG commit gate hỏng).
6. **Proof-commit:** commit P005 chạy hook thật (KHÔNG `--no-verify`) — transcript bước `[4/7]` + nội dung `/tmp/sos-secgate.log` vào `docs/discoveries/P005.md` §Dogfood evidence. Hook bị bypass vì lý do bất kỳ → KHÔNG tính proof.
7. **Reversibility test:** tạm rename `target/release/inv-gate` → guard fire LOUD + commit thử BỊ BLOCK (fail-closed proof), rồi khôi phục. Đường lui đầy đủ: uncomment python3 lines hoặc restore `.backup/P005/security-gate.sh.orig`.
8. **Anchor #18:** diff nhanh `scripts/check-*.py` vs golden counterpart — khác biệt (nếu có) chỉ ghi Discovery, không block (hook depend exit code + log capture, không parse text).
9. **Lưu ý vui mà thật:** `scripts/security-gate.sh` có thể nằm trong scan surface của chính các check — nếu gate tự bắt edit của mình khi proof-commit → đó là dogfood hoạt động, ghi Discovery (đừng nhét chuỗi token-like vào comment).

### Task 5: Docs gate

**File:** `CHANGELOG.md` — entry P005 (Unreleased): `gate --all` port (in-process orchestration, 3 script-check + 6 inline check kể cả INV-008 Python→Rust, accumulator semantics theo golden), **deviation: `gate --all` ≡ golden `--mechanical-only`; full/SSH mode không implement Phase 1**; usage-error text deviation (đã document P001, nhắc 1 dòng); dogfood per-check swap trong `scripts/security-gate.sh` (kill python3 trong gate chain repo này). KHÔNG bump version `Cargo.toml` (F13).
**File:** `docs/ARCHITECTURE.md` — `src/gate.rs` từ "planned" → shipped (P005): order, in-process call, inline fns, accumulator, flag mapping, cite range (IG-04 — không count). Data flow: pre-commit `[4/7]` → `scripts/security-gate.sh` (adapted) → `inv-gate check secrets|runtime|port` per-check (dogfood); `gate --all` = parity-proven fixture-based.
**File:** `CLAUDE.md` §Rules — thêm 1 dòng: `gate --all` (Rust) ≡ golden `--mechanical-only`; SSH mode chưa port; dogfood repo = per-check trong `scripts/security-gate.sh`.
**File:** `tests/golden/MANIFEST.md` §4 — append rule MỚI nếu chốt được precedent reuse cho Sprint 2 (candidates: in-process-vs-subprocess observable-parity; union-fixture harness; per-check-swap reversible pattern). Không có gì mới → không đụng, ghi "None" trong Discovery.
**File:** `docs/discoveries/P005.md` + index 1-line `docs/DISCOVERIES.md` — gồm: (i) bảng enumeration golden 210 LOC (bản chốt sau parity) + bảng map 99 LOC adapted (anchor #17); (ii) aggregate semantics evidence; (iii) inline checks logic verbatim + probe mapping; (iv) hook wiring + call cũ verbatim + transcript proof-commit + reversibility-block transcript; (v) anchor #18 diff finding; (vi) mapping cơ-chế→unit-test; (vii) hooks cho Sprint 2 (`serve` expose `gate` tool; profile/flag mode đa-repo; sos-kit global swap).

---

## Sprint close-out (owner: ORCHESTRATOR — KHÔNG phải Worker)

> Worker KHÔNG edit `docs/BACKLOG.md` (footer BACKLOG: Architect/Worker chỉ đọc). Sau khi P005 merge + Chủ nhà nghiệm thu:

1. Orchestrator move item P005 → done checkbox; move tóm tắt Sprint 1 (1 dòng: 4 check + gate ported, parity byte-exact, dogfood per-check live — python3 killed, N tests) vào BACKLOG §Recently shipped (rule 2+3 BACKLOG).
2. Orchestrator **append item mới vào BACKLOG (Chủ nhà decision (c)):** "Profile/flag mode cho `gate` đa-repo (adapted-gate parity surface) — Sprint 2".
3. Orchestrator ghi sprint summary vào `CHANGELOG.md` (mục Sprint 1 tổng kết — khác entry P005 của Worker).
4. Orchestrator confirm với Chủ nhà: Sprint 2 (MCP + distribution + profile mode) promote lên Active hay chưa — KHÔNG tự promote.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Task 1: variant `Gate { --all }` + `mod gate;` + dispatch |
| `src/gate.rs` | Task 2: MỚI — orchestrator port (3 in-process call + 6 inline fn), mọi output line cite golden:line |
| `tests/parity_gate.rs` | Task 3: MỚI — 2 parity (union harness, stdout+stderr byte-exact) + usage-error + probes (a)-(e) |
| `scripts/security-gate.sh` | Task 4: CHỈ build-guard + cặp comment/call python3→binary (per-check — Chủ nhà decision (b)) |
| `CHANGELOG.md` | Task 5: entry P005 + deviation notes |
| `docs/ARCHITECTURE.md` | Task 5: gate.rs shipped + data flow per-check dogfood |
| `CLAUDE.md` | Task 5: 1 dòng flag mapping + dogfood note |
| `tests/golden/MANIFEST.md` | Task 5: §4 note — CONDITIONAL |
| `docs/discoveries/P005.md` + `docs/DISCOVERIES.md` | Discovery report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng khi nghiệm thu |
| `tests/golden/pins/**`, `fixtures/**`, `repin.sh` | Oracle — diff rỗng. Parity đỏ → sửa Rust/harness |
| `hooks/pre-commit` | KHÔNG đụng (decision (b)) — diff rỗng; verify block `[4/7]` lines 207-220 vẫn gọi `scripts/security-gate.sh` |
| `scripts/check-*.py` | Đường lui dogfood — diff rỗng, không xóa (deprecate là việc Sprint 2 sos-kit) |
| `src/checks/*.rs`, `src/checks/mod.rs` | Diff rỗng — NGOẠI LỆ visibility/signature-only nếu anchor #6 đòi (behavior diff = 0, 63 tests xanh). `schema.rs`: diff rỗng tuyệt đối (không trong gate) |
| `tests/parity_{secrets,runtime,port,schema}.rs` | Regression xanh — NGOẠI LỆ import-helper refactor (precedent P004) |
| `Cargo.toml` | KHÔNG dep mới, KHÔNG bump version. Diff rỗng |
| `docs/BACKLOG.md` | Worker/Architect KHÔNG edit — close-out là việc orchestrator |

---

## Luật chơi (Constraints)

1. **Parity-first, security surface:** KHÔNG đổi order check, banner wording, aggregate logic so với golden mechanical-only. Mỗi output line + flow decision trong `gate.rs` comment cite `golden/security-gate.sh:<line>`. Nghi golden "dở" → pin nguyên trạng + DISCOVERY. PR cần Giám sát review (CLAUDE.md — gate = security surface).
2. **In-process, tuần tự, single-thread:** gate gọi `checks::{port,secrets,runtime}::run()` trực tiếp + inline fns. CẤM spawn chính binary, CẤM spawn python (kể cả INV-008), CẤM parallel/async. Gate không capture/reformat output check. KHÔNG gọi schema.
3. **Pin bất khả xâm phạm:** parity đỏ → sửa `gate.rs`/harness. Sửa pin/fixture/repin.sh-để-test-xanh = vi phạm scope.
4. **Byte-exact CẢ stdout VÀ stderr** vs pins `gate--{dirty,clean}` (stderr chứa WARN port — MANIFEST §4 rule 7). Usage-error: exit-code-only (anchor #9).
5. **Exit contract:** gate logic chỉ 0/1; 2 duy nhất từ clap (deviation P001). Deviation MỚI → CHANGELOG + CLAUDE.md.
6. **Dogfood fail-closed + reversible + zero coverage loss (decision (b)):** build-guard exit 1 khi thiếu binary (KHÔNG silent fallback/skip); dòng python3 cũ giữ dạng comment ngay trên call mới; backup `.orig` trong snapshot; `git diff scripts/security-gate.sh` CHỈ chứa guard + cặp comment/call; KHÔNG sửa `hooks/pre-commit`.
7. **KHÔNG dep mới**; git/hook thao tác qua tay Worker + `std::process::Command` nếu cần trong test (pattern MANIFEST §4 rule 8).
8. **Test env hermetic:** mọi Command trong test `env_remove("ALLOW_DATA_LOSS")`; harness LF (precedent P002).
9. **Cite RANGE, không count** (IG-04). Cite golden:line CHỈ cho `src/gate.rs`; `scripts/security-gate.sh` adapted cite theo chính file đó.
10. `cargo test` xanh TOÀN BỘ trước commit — gồm regression 63 tests P001-P004.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — gồm parity gate: `gate--dirty` exit 1 + stdout/stderr byte-exact, `gate--clean` exit 0 + stdout/stderr byte-exact, usage-error exit 2
- [ ] Unit probes Task 3 (a)-(e) — đặc biệt: accumulator evidence, ≥1 probe / inline check (6 inline), summary conditional `Failed invariants`
- [ ] Regression: 63 tests P001-P004 xanh; `git diff src/checks/` rỗng hoặc visibility-only (anchor #6, schema.rs rỗng tuyệt đối); `git diff golden/ tests/golden/` rỗng; `git diff Cargo.toml hooks/ scripts/check-*.py` rỗng
- [ ] `git diff scripts/security-gate.sh` CHỈ chứa guard + cặp comment/call (Luật chơi 6)
- [ ] `cargo build --release` sạch

### Manual Testing
- [ ] Dogfood pre-check: `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree SAU swap (Task 4 mục 5)
- [ ] Proof-commit: commit P005 chạy hook thật, transcript `[4/7]` + `/tmp/sos-secgate.log` (binary path, không python3) vào Discovery — KHÔNG `--no-verify`
- [ ] Reversibility: tạm rename `target/release/inv-gate` → guard fire LOUD + commit thử BỊ BLOCK (fail-closed), rồi khôi phục
- [ ] `cargo run -- gate --no-such-flag` → exit 2; `cargo run -- gate` (trần) → exit 2
- [ ] Mỗi output line trong `gate.rs` có comment cite golden:line
- [ ] (Informational, KHÔNG acceptance — decision (a)): chạy `target/release/inv-gate gate --all` tại repo root, ghi kết quả vào Discovery

### Regression
- [ ] `bash tests/golden/repin.sh` vẫn chạy + `git diff tests/golden/pins/` rỗng sau khi chạy
- [ ] `cargo run -- check secrets|runtime|port|schema` từng cái behavior nguyên (so P004 manual notes)

### Docs Gate
- [ ] `CHANGELOG.md` — entry P005 (flag-mapping deviation + dogfood per-check)
- [ ] `docs/ARCHITECTURE.md` — gate.rs shipped, hết "planned"; data flow per-check dogfood
- [ ] `CLAUDE.md` — 1 dòng mapping + dogfood note
- [ ] `tests/golden/MANIFEST.md` §4 — note nếu có, không thì "None" trong Discovery

### Discovery Report
- [ ] `docs/discoveries/P005.md`: assumptions ĐÚNG/SAI (cite — đặc biệt enumeration chốt, #17 map adapted 99 LOC, #18 diff), dogfood evidence transcript + reversibility-block transcript, mapping cơ-chế→test, tier escalations ("None" nếu không)
- [ ] Append 1-line index `docs/DISCOVERIES.md`
