# PHIẾU P004: `check port` (INV-001) + `check schema` — port `check-port-bind.py` + `check-schema-safety.sh` + parity

> **Loại:** Feature (Rust port — phiếu port thứ 3+4, NỐI vào skeleton P002/P003)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) scan-target patterns (port-bind rules, schema destructive patterns) là SECURITY surface (CLAUDE.md: AUTO Tầng 1, Giám sát review PR); (b) schema-safety = guard chống data loss (migration surface); (c) exit-code contract 0/1/2 là API pre-commit hook depend; (d) `ALLOW_DATA_LOSS` bypass = cơ chế override security gate — port sai chiều = gate hổng.
> **Ảnh hưởng:** `src/main.rs`, `src/checks/mod.rs`, `src/checks/port.rs` (MỚI), `src/checks/schema.rs` (MỚI), `tests/parity_port.rs` (MỚI), `tests/parity_schema.rs` (MỚI), `CHANGELOG.md`, `docs/ARCHITECTURE.md`
> **Dependency:** P001 (pins `port--*`, `schema--*`) + P002 (CLI skeleton) + **P003 (variant `Runtime` + `build_fixture_repo()` harness — anchor #1 ✅)**. Anchor #1b ✅ — P004 STACK trên branch `feat/P003-check-runtime` (HEAD `8c7bda3 feat(P003)`).

> *Note đánh số: BACKLOG Active sprint (single source — Rule 0) đặt P004 = `check port` + `check schema` gộp 1 phiếu. PROJECT.md §Roadmap (ghi rõ "placeholder") đánh số khác — BACKLOG thắng, không sửa PROJECT.md trong phiếu này (precedent P003).*

---

## Context

### Vấn đề hiện tại

Oracle đã pin (P001): `tests/golden/pins/{port,schema}--{dirty,clean}` + `exit_codes.json` (`port--dirty: 1`, `port--clean: 0`, `schema--dirty: 1`, `schema--clean: 0` — anchor #2 ✅). MANIFEST §1/§2/§5 ghi invocation contract + unit-spec cho cả 2 script. CLI skeleton có `Secrets` + `Runtime` (P002/P003). Chưa có dòng Rust nào cho INV-001 / schema-safety.

**Điểm khác P002/P003 — hai check, hai cơ chế khác nhau:**

1. **`check port`** (`golden/check-port-bind.py`, 82 LOC Python): scan TĨNH 3 compose file hardcoded — KHÔNG cần git, KHÔNG walk tree, **KHÔNG parse YAML thật** (line-based 4 layer — anchor #4 ✅, xem Task 2). Đặc thù: missing compose file → **WARN ra stderr** (`:60`), và **pin CÓ capture stderr** (`port--{dirty,clean}.stderr.txt` 108 bytes, 2 dòng WARN — anchor #5 ✅) ⇒ parity port assert **stderr BYTE-EXACT với pin**, khác contract "stderr rỗng" của P002/P003.
2. **`check schema`** (`golden/check-schema-safety.sh`, 64 LOC **BASH**, `set -u` KHÔNG `set -e` — script bash đầu tiên được port): cơ chế = `git diff HEAD~1..HEAD -- prisma/schema.prisma` với fallback chain 3 bước (`:32-34`, anchor #11 ✅) rồi grep dòng xóa pattern destructive (`:47-48`). Rust gọi `git` qua `std::process::Command` — **KHÔNG thêm dep `git2`**. Parity test DỰNG git repo thật 2 commit trong tempdir (MANIFEST §4 rule 4 — reuse `build_fixture_repo()` `tests/parity_runtime.rs:71-132`, BỎ bước remote-inject vì schema check không đọc remote — O1.4 Tầng 2).
3. Bypass `ALLOW_DATA_LOSS=true` → exit 0 sớm (`:18-20`, anchor #10 ✅ — exact string `"true"`, case-sensitive; `TRUE`/`1`/`yes` KHÔNG bypass). **Parity-blind** → unit test bắt buộc + test env `env_remove("ALLOW_DATA_LOSS")`.

**Lookahead gate (bài học P003 O1.1):** anchor #9 ✅ — grep `(?=`/`(?!`/`(?<` cả 2 script = 0 hit; `Regex::new` probe cả 4 pattern compile sạch (oracle SOUND, self-closed Turn 1). KHÔNG cần transcription.

### Giải pháp

Bốn phần, đúng method CLAUDE.md (parity trước, cải tiến sau):

1. **CLI nối skeleton** — thêm 2 variant `Port` + `Schema` vào `CheckCommand`. KHÔNG đụng `Secrets`/`Runtime`.
2. **Port logic INV-001** — `src/checks/port.rs` giữ NGUYÊN behavior `golden/check-port-bind.py`: COMPOSE_FILES order, 4-layer parse, classify 1/2/3/4+-part, nginx exception, WARN-stderr, format `:75`, exit code.
3. **Port logic schema-safety** — `src/checks/schema.rs` giữ NGUYÊN behavior `golden/check-schema-safety.sh`: bypass, fallback chain 3 bước (kể cả bad SHA — O1.2), header-skip + destructive grep, đủ 6 branch A-F (anchor #12 ✅), raw-diff-line output, exit code.
4. **Parity tests** — `tests/parity_port.rs` (fixture-copy tĩnh, stderr byte-exact) + `tests/parity_schema.rs` (env-reconstruction 2-commit). Stdout/stderr/exit **BYTE-EXACT** với pins. Kèm unit probes synthetic cho **MỌI** cơ chế parity-blind (port ≥12, schema ≥7 — enumerate Turn 1, xem Task 4/5).

### Scope

- CHỈ sửa/tạo: `src/main.rs`, `src/checks/mod.rs`, `src/checks/port.rs`, `src/checks/schema.rs`, `tests/parity_port.rs`, `tests/parity_schema.rs`, `CHANGELOG.md`, `docs/ARCHITECTURE.md` (+ `tests/golden/MANIFEST.md` §4 note conditional — Task 6).
- KHÔNG sửa: `golden/**` (read-only tuyệt đối), `tests/golden/pins/**` + `fixtures/**` + `repin.sh` (oracle), `src/checks/secrets.rs` + `runtime.rs` + `tests/parity_secrets.rs` + `parity_runtime.rs` (P002/P003 shipped — regression only; ngoại lệ DUY NHẤT: nếu Worker chọn extract helper chung `tests/common/mod.rs`, được phép sửa `parity_runtime.rs` CHỈ để import helper, diff hành vi = 0 — Tầng 2, Worker quyết), `Cargo.toml` (KHÔNG dep mới — cấm `git2`, `serde_yaml`, `glob`, `fancy_regex`).
- KHÔNG implement: `gate` (P005), `serve`, JSON output (P006), cải tiến pattern (sau parity, phiếu riêng — kể cả "sửa" bad SHA `:33`, xem O1.2).

---

## Task 0 — Verification Anchors

> Bước 0 capability: không network/POST — filesystem scan + `git` CLI trong test harness + cargo. Toàn bộ anchors đã được Worker CHALLENGE Turn 1 verify bằng grep/đọc golden + pins + repin.sh + fixtures + `Regex::new` probe.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Base có pins + skeleton P003 + `build_fixture_repo()` | ls + grep | ✅ RESOLVED (Turn 1) — 4 pin files đủ; `src/main.rs:22-26` enum `{ Secrets, Runtime }`; `tests/parity_runtime.rs:71` có helper `[verified — Worker Turn 1]` |
| 1b | P003 merged vào base P004 | git log/branch | ✅ RESOLVED (Turn 1) — P004 stack trên `feat/P003-check-runtime` (HEAD `8c7bda3`) `[verified — Worker Turn 1]` |
| 2 | Exit codes pin | cat exit_codes.json | ✅ RESOLVED (Turn 1) — `port--dirty:1`, `port--clean:0`, `schema--dirty:1`, `schema--clean:0` verbatim `[verified — Worker Turn 1]` |
| 3 | COMPOSE_FILES + nginx set | đọc golden | ✅ RESOLVED (Turn 1) — `golden/check-port-bind.py:12-16` 3 path đúng order; `:19` set `{"80:80","443:443"}` `[verified — Worker Turn 1]` |
| 4 | Port parse mechanism exhaustive | Worker đọc 82 LOC | ✅ RESOLVED (Turn 1) — line-based 4 layer, KHÔNG YAML parse. Chi tiết fold vào Task 2 mục 3-4. Parity-blind: 12 probes (Task 4) `[verified — Worker Turn 1]` |
| 5 | Missing file WARN stderr + pin capture | đọc golden + ls pins + fixtures | ✅ RESOLVED (Turn 1) — `port--{clean,dirty}.stderr.txt` TỒN TẠI (108 bytes mỗi file): `WARN: docker-compose.dev.yml not found, skipping` + `WARN: astro-service/docker-compose.yml not found, skipping`. Fixtures chỉ có `docker-compose.yml` → parity assert **stderr byte-exact pin** `[verified — Worker Turn 1]` |
| 6 | Split semantics | đọc golden | ✅ RESOLVED (Turn 1) — `classify()` `:41-53`: `spec.split(":")` đơn giản (KHÔNG rsplit/partition); IP so sánh `== "127.0.0.1"` exact; 1-part và 4+-part (IPv6 `::1:8080`) → violation `unrecognized format: {spec}` `[verified — Worker Turn 1]` |
| 7 | Output formats | đọc golden + pins | ✅ RESOLVED (Turn 1) — dirty: `docker-compose.yml:8: INV-001 violated -- implicit 0.0.0.0 bind: 8001:8001\n` (`:75`, `{fname}:{idx+1}`); clean: `INV-001: PASS (port bindings clean)\n` (36 bytes, 1 LF — KHÔNG count) `[verified — Worker Turn 1]` |
| 8 | Non-UTF-8/unreadable behavior port | đọc golden | ✅ RESOLVED (Turn 1) — `path.read_text()` `:62` KHÔNG try/except, KHÔNG encoding arg → UTF-8 strict, non-UTF-8 = uncaught `UnicodeDecodeError` → traceback stderr + exit non-zero (Python uncaught = 1). Spec Rust: Task 2 mục 7 `[verified — Worker Turn 1]` |
| 9 | KHÔNG lookaround, regex crate compile sạch | grep + `Regex::new` probe | ✅ RESOLVED (Turn 1) — 0 hit lookaround cả 2 script; probe compile clean cả 4 pattern. **[O1.1] self-closed via oracle (SOUND)** `[verified — Worker Turn 1]` |
| 10 | Bypass semantics | đọc golden `:18-20` | ✅ RESOLVED (Turn 1) — `"${ALLOW_DATA_LOSS:-false}" == "true"` exact case-sensitive; bypass echo VERBATIM: `ALLOW_DATA_LOSS=true — bypass schema safety check (Sếp explicit ack).` (chứa em dash UTF-8) → exit 0 `[verified — Worker Turn 1]` |
| 11 | Schema mechanism + patterns | đọc golden `:32-48` | ✅ RESOLVED (Turn 1) — diff `HEAD~1..HEAD` với **fallback chain 3 bước** `:33-34`: `\|\| git diff 4b825dc8669f8c0..HEAD -- prisma/schema.prisma 2>/dev/null \|\| echo ""`. Bad SHA 15-char (KHÔNG phải empty-tree SHA chuẩn) → cả 2 git call fail trên fresh repo → `DIFF=""` (O1.2). Header-skip `:47`, destructive `:48`, `\|\| true` `:49` neutralize grep-no-match `[verified — Worker Turn 1]` |
| 12 | Schema branches exhaustive | Worker đọc 64 LOC | ✅ RESOLVED (Turn 1) — 6 branch A-F enumerate đủ, fold vào Task 3 mục 2 (bảng). Parity-blind: A, B, F (+C nhánh 1-commit) → probes bắt buộc `[verified — Worker Turn 1]` |
| 13 | Dirty pin raw diff lines | xxd pin | ✅ RESOLVED (Turn 1) — banner `❌ DESTRUCTIVE...` + 4 matched lines (`-  legacyToken String?`, `-model AuditLog {`, `-  id      Int    @id @default(autoincrement())`, `-  payload String`) + blank + instruction block. KHÔNG SHA. Closing `}` correctly EXCLUDED `[verified — Worker Turn 1]` |
| 14 | repin.sh 2-commit flow + reuse | đọc repin.sh + parity_runtime.rs | ✅ RESOLVED (Turn 1) — `repin.sh:34-88` hermetic config + fixed dates; `parity_runtime.rs:71-132` mirror exact. Schema reuse được — BỎ remote-inject (O1.4, Tầng 2) `[verified — Worker Turn 1]` |
| 15 | `\w`/`\s` transcription | Worker confirm | ✅ RESOLVED (Turn 1) — nội dung scan ASCII identifiers; Python re / grep ERE (locale UTF-8) / Rust regex (Unicode) match identical. KHÔNG drift, giữ `\w`/`\s` nguyên văn `[verified — Worker Turn 1]` |
| 16 | Env vars khác | grep | ✅ RESOLVED (Turn 1) — port: 0 hit `os.environ`; schema: chỉ `$ALLOW_DATA_LOSS` (input) + vars nội bộ `[verified — Worker Turn 1]` |
| 17 | Cargo.toml deps đủ | cat | ✅ RESOLVED (Turn 1) — clap, regex, walkdir, serde, serde_json, anyhow, thiserror, tokio, rmcp; dev: assert_cmd, predicates, tempfile. KHÔNG dep mới `[verified — Worker Turn 1]` |
| 18 | `port.rs`/`schema.rs` chưa tồn tại | ls + grep | ✅ RESOLVED (Turn 1) — `src/checks/` chỉ có `mod.rs`, `runtime.rs`, `secrets.rs`; enum 0 hit `Port`/`Schema` `[verified — Worker Turn 1]` |

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P004
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V2 (Turn 1 — Worker accepted V1, KHÔNG Tầng 1 objection; Architect fold facts verified Turn 1 vào anchors + Task 2/3/4/5. KHÔNG đổi architecture.)

### Turn 1 — Worker Challenge (vs V1)

**Verdict:** ACCEPTED — 18/18 anchors ✅, KHÔNG Tầng 1 objection.

- **[O1.1]** anchor #9 lookahead → **self-closed via oracle** (`Regex::new` probe, SOUND): cả 4 pattern compile sạch với regex crate. Không cần transcription.
- **[O1.2]** Observation (không blocking): `golden/check-schema-safety.sh:33` dùng SHA `4b825dc8669f8c0` (15 chars) — KHÔNG phải empty-tree SHA chuẩn `4b825dc642cb6eb9a060e54bf8d69288fbee4904`. Cả 2 git invocation fail trên fresh/1-commit repo → `echo ""` fires. Resolution: **port fallback chain 3 bước AS-IS** (kể cả bad SHA), ghi Discovery. KHÔNG "fix" trong phiếu này (parity-first — Luật chơi 1).
- **[O1.3]** Probe list anchor #4 chưa enumerate đủ trong V1 → Tầng 2, fold V2: Task 4 liệt kê đủ 12 probe.
- **[O1.4]** `build_fixture_repo()` reuse: schema check không đọc remote → BỎ bước remote-inject. Tầng 2, Worker quyết cấu trúc helper.

**Key findings fold V2:** stderr pins TỒN TẠI cho port (anchor #5 — parity assert stderr byte-exact, dùng pattern `expected_stderr()` từ `parity_runtime.rs`); port parse = 4 layer line-based (anchor #4); schema = 6 branch A-F, `set -u` không `set -e`, `|| true` grep, git stderr `2>/dev/null` (anchor #11/#12); bypass string verbatim có em dash (anchor #10); 1-part/4+-part port spec → violation `unrecognized format` (anchor #6).

**Status:** ✅ ACCEPTED — phiếu bumped to V2 (fold-only). **READY FOR CHỦ NHÀ APPROVAL GATE.**

### Final consensus
- Phiếu version: V2
- Total turns: 1
- Approved by Chủ nhà: 11/06/2026 (Quản đốc self-approve theo ủy quyền; Chủ nhà nghiệm thu retro khi merge sprint 11/06/2026)

---

## Nhiệm vụ

### Task 1: Nối CLI skeleton — thêm variant `Port` + `Schema`

**File:** `src/main.rs` (sửa — chỉ thêm, không reshape enum P002/P003)

**Thêm:**

```rust
// Vào enum CheckCommand hiện có (src/main.rs:22-26 — anchor #1 ✅):
/// INV-001 — docker-compose host-bind check (parity port of golden/check-port-bind.py)
Port,
/// Prisma schema-safety — destructive migration guard (parity port of golden/check-schema-safety.sh)
Schema,
```

- Dispatch `Check { Port }` → `checks::port::run()`; `Check { Schema }` → `checks::schema::run()` → exit code → `std::process::exit(code)` (cùng pattern `Secrets`/`Runtime`).

**File:** `src/checks/mod.rs` (sửa): thêm `pub mod port;` + `pub mod schema;`

**Lưu ý:**
1. KHÔNG đụng `Secrets`/`Runtime` + module đã ship — diff phải rỗng.
2. Tên type/binding nội bộ = Tầng 2, Worker theo convention P002/P003.

### Task 2: Port logic INV-001 — `src/checks/port.rs`

**File:** `src/checks/port.rs` (MỚI)

**Thêm:** Port 1:1 từ `golden/check-port-bind.py` (82 LOC). Mỗi surface kèm comment cite `golden/check-port-bind.py:<line>`. Cơ chế 4 layer (anchor #4 ✅ — chép EXACT, kể cả chỗ "dở"; nghi sai → DISCOVERY, cải tiến phiếu sau):

1. **COMPOSE_FILES** (`:12-16`): 3 path hardcoded đúng array order — order quyết định output order.
2. **Nginx exception set** (`:19`): `{"80:80","443:443"}` — chép exact, KHÔNG nới.
3. **Parse 4 layer:** **Layer 1 — PORT_LINE_RE** `^\s*-\s*"?([^"]+?)"?\s*$`: match YAML list item (leading `-`), strip double-quote, KHÔNG strip single-quote. **Layer 2 — numeric filter** `^[\d.:]+$`: chỉ digits/dots/colons pass → silently SKIP: `/udp`/`/tcp` suffix, port range `8000-8001:...` (hyphen), long syntax `target: 80` (letters), single-quoted `'8001:8001'` (quote còn trong captured group). **Layer 3 — `is_in_ports_block()`**: walk ngược từ dòng hiện tại, dừng ở dòng `key:` đầu tiên (không bắt đầu `-`), `True` chỉ khi key đó là `ports:` — items trong `volumes:`/`environment:` bị loại. **Layer 4 — `classify()`** (`:41-53`): `spec.split(":")` đơn giản. 3-part: `ip == "127.0.0.1"` → ok, else violation `public IP bind: {spec}` (kể cả `0.0.0.0` explicit). 2-part: không thuộc nginx set → violation `implicit 0.0.0.0 bind: {spec}`. **1-part và 4+-part (IPv6) → violation `unrecognized format: {spec}`** — golden behavior, giữ nguyên. (Regex layer 1/2 Worker chép exact từ source khi viết — text trên theo Turn 1 report.)
4. **Missing file** (`:60`, anchor #5 ✅): WARN ra **stderr**, wording VERBATIM `WARN: {fname} not found, skipping`, KHÔNG exit 1, KHÔNG tính violation.
5. **Output**: stdout, dirty format `{fname}:{lineno}: INV-001 violated -- {result}` (`:75`, line-only 1-indexed, F06); clean PASS line VERBATIM: `INV-001: PASS (port bindings clean)` (KHÔNG count — anchor #7 ✅).
6. **Đọc file** (`:62`, anchor #8 ✅): golden = `read_text()` UTF-8 strict KHÔNG try/except → non-UTF-8 = uncaught exception, traceback stderr, exit 1 (Python uncaught). Rust: **KHÔNG panic** (panic = exit 101, lệch contract) — error path in message ra stderr + exit non-zero khớp golden observable (error text Rust ≠ Python traceback chấp nhận được — stderr này KHÔNG nằm trong pin; exit code phải khớp). Ghi Discovery.
7. **Exit**: ≥1 violation → 1; 0 → 0 (WARN không ảnh hưởng). Không exit path khác (MANIFEST §1).

**Lưu ý:**
1. KHÔNG dùng YAML parser (`serde_yaml` cấm) — golden là line-based (anchor #4 ✅); Rust mirror đúng mechanism, không "tốt hơn".
2. Security surface: từng rule/set entry/wording lệch 1 ký tự so golden = vi phạm Luật chơi 1.

### Task 3: Port logic schema-safety — `src/checks/schema.rs`

**File:** `src/checks/schema.rs` (MỚI)

**Thêm:** Port 1:1 từ `golden/check-schema-safety.sh` (64 LOC bash, `set -u` KHÔNG `set -e`). Mỗi surface kèm comment cite `golden/check-schema-safety.sh:<line>`:

1. **Bypass** (`:18-20`, anchor #10 ✅): env `ALLOW_DATA_LOSS` == `"true"` exact (case-sensitive; default `false` khi unset). Bypass → echo VERBATIM `ALLOW_DATA_LOSS=true — bypass schema safety check (Sếp explicit ack).` (giữ em dash UTF-8) → exit 0.
2. **Branch table (anchor #12 ✅ — chép ĐỦ, đúng output + exit từng branch, KHÔNG thêm branch mới):**

   | Branch | Điều kiện | Output (Worker chép verbatim từ golden khi viết) | Exit | Pin? |
   |---|---|---|---|---|
   | A | `ALLOW_DATA_LOSS=true` | bypass echo (mục 1) | 0 | blind → probe |
   | B | `prisma/schema.prisma` không tồn tại | `❌ prisma/schema.prisma not found — cannot check schema safety.` | 1 | blind → probe |
   | C | `DIFF=""` (no parent / cả 2 git fail / không đổi) | `No schema diff vs HEAD~1 — safe.` | 0 | blind → probe |
   | D | DIFF non-empty, DESTRUCTIVE empty | `Schema diff present but no destructive pattern (field/model removed) detected — safe.` | 0 | clean pin |
   | E | DIFF non-empty, DESTRUCTIVE non-empty | banner `❌ DESTRUCTIVE...` + raw lines + instruction block | 1 | dirty pin |
   | F | Not-a-git-repo | cả 2 git call fail → `DIFF=""` → Branch C | 0 | blind → probe |

3. **Git diff fallback chain** (`:32-34`, anchor #11 ✅ + O1.2): `std::process::Command`, cwd repo root, 3 bước AS-IS: (i) `git diff HEAD~1..HEAD -- prisma/schema.prisma`; (ii) fail → `git diff 4b825dc8669f8c0..HEAD -- prisma/schema.prisma` (**bad SHA 15-char — PORT NGUYÊN TRẠNG**, không "fix" thành empty-tree SHA chuẩn; ghi Discovery per O1.2); (iii) fail → `DIFF = ""`. Git stderr suppress (`2>/dev/null` ≡ `Stdio::null()` hoặc capture-and-discard). Semantics "fail" của bash `||` (non-zero exit) chép đúng.
4. **Filter** (`:47-48`): header-skip `grep -vE '^---|^-\+\+\+'` rồi destructive `grep -E '^-\s*(model|enum)\s+\w+|^-\s+\w+\s+\S+'` — chép VERBATIM từng ký tự sang regex crate (giữ `\w`/`\s` nguyên văn — anchor #15 ✅, nội dung ASCII không drift), per-LINE, đúng pipeline order (skip trước, match sau). Grep no-match KHÔNG abort (`|| true` `:49` — Rust: DESTRUCTIVE rỗng là path bình thường, KHÔNG error).
5. **Output** (`:52-55`): Branch E = banner + echo raw matching diff lines (KHÔNG reformat, KHÔNG `path:lineno:` — MANIFEST §5) + blank + instruction block, khớp pin dirty (anchor #13 ✅: 4 lines gồm cả `-  id ...` + `-  payload ...` của model bị xóa; closing `}` KHÔNG match).
6. **Exit**: theo branch table — chỉ 0/1.

**Lưu ý:**
1. Script bash đầu tiên được port — nếu phát hiện behavior phụ thuộc bash/grep version (khác pin env Darwin §3) → DISCOVERY, pin nguyên trạng.
2. KHÔNG sáng tác guard mới (vd "git installed?") — golden không có thì Rust không có; lệch bất khả kháng do `Command` semantics → Debate Log.

### Task 4: Parity test `check port` — `tests/parity_port.rs`

**File:** `tests/parity_port.rs` (MỚI — assert_cmd + tempfile + serde_json, deps sẵn)

**Thêm:** 2 parity case (dirty/clean):

1. `tempfile::tempdir()` → copy fixture tree port (KHÔNG cần git — scan tĩnh). File set ĐÚNG fixture P001: chỉ `docker-compose.yml` (anchor #5 ✅ — 2 file thiếu để WARN fire đúng pin).
2. Chạy binary `["check", "port"]`, cwd = temp dir.
3. So sánh: exit code == `port--<fixture>` trong `exit_codes.json` (serde_json, KHÔNG hardcode); **stdout BYTE-EXACT** pin (PASS/violation line + trailing LF); **stderr BYTE-EXACT** pin `port--<fixture>.stderr.txt` (2 dòng WARN — dùng pattern `expected_stderr()` từ `parity_runtime.rs`, anchor #5 ✅).

**Unit probes — BẮT BUỘC, 12 probe (enumerate Turn 1, anchor #4 ✅; synthetic in-code, F07 — KHÔNG invent fixture file trong `tests/golden/fixtures/`):**
   - (a) 3-part public IP (vd `203.0.113.5:8001:8001`) → violation `public IP bind`;
   - (b) 3-part loopback `127.0.0.1:8001:8001` → 0 violation;
   - (c) nginx `80:80` → 0 violation (fixture chỉ exercise `443:443`);
   - (d) 2-part non-nginx `8001:8001` → violation `implicit 0.0.0.0 bind` (format + wording exact);
   - (e) missing compose file → WARN wording exact trên stderr, exit theo file còn lại, không panic;
   - (f1) single-quoted `'8001:8001'` → SKIP (không violation — khác double-quoted);
   - (f2) bare 1-part `8080` → violation `unrecognized format: 8080`;
   - (f3) `0.0.0.0:8001:8001` explicit → violation `public IP bind` (KHÔNG `implicit`);
   - (f4) long syntax `target: 80` → SKIP silent;
   - (f5) port range `8000-8001:8000-8001` → SKIP silent;
   - (f6) list item trong `volumes:`/`environment:` block trông giống port → KHÔNG violation (is_in_ports_block);
   - (g) non-UTF-8 compose file → error stderr + exit non-zero, KHÔNG panic (Task 2 mục 6).
   - Mapping cơ-chế→test ghi vào Discovery (precedent P003). Probe thêm (IPv6 4-part `unrecognized format`, `/udp` skip) = optional Tầng 2.

### Task 5: Parity test `check schema` — `tests/parity_schema.rs`

**File:** `tests/parity_schema.rs` (MỚI)

**Thêm:** 2 parity case (dirty/clean):

1. `tempfile::tempdir()` → **tái tạo git repo 2 commit theo `tests/golden/repin.sh:34-88`** (anchor #14 ✅): git init + hermetic config (`user.name "P001 Pin Harness"`, `commit.gpgsign false`, fixed `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE = "2026-01-01T00:00:00 +0000"`) → commit `schema.before` → apply `schema.after` → commit 2. Dirty: schema.after xóa `legacyToken` + model `AuditLog`; clean: chỉ thêm `displayName` (MANIFEST §1). Helper: reuse `build_fixture_repo()` pattern P003 **BỎ bước remote-inject** (O1.4); copy hay extract `tests/common/mod.rs` = Tầng 2, Worker quyết (xem Scope).
2. Chạy binary `["check", "schema"]`, cwd = temp repo root, **`env_remove("ALLOW_DATA_LOSS")`** trên Command (env dev có thể set sẵn).
3. So sánh: exit code == `schema--<fixture>` pin; **stdout BYTE-EXACT** pin (anchor #13 ✅ — banner + 4 raw diff lines git-diff order + instruction block); stderr rỗng (git stderr đã suppress — Task 3 mục 3).

**Unit probes — BẮT BUỘC, 7 probe (anchor #12 ✅):**
   - (a) **Branch A — `ALLOW_DATA_LOSS=true` bypass**: env set + schema destructive → bypass echo verbatim + exit 0 (cơ chế parity-blind QUAN TRỌNG NHẤT phiếu);
   - (b) `ALLOW_DATA_LOSS` giá trị khác (`TRUE`, `1`, `false`, rỗng) → KHÔNG bypass (exact-match `"true"` — anchor #10 ✅);
   - (c) **Branch B — schema.prisma missing** → `❌ ... not found ...` + exit 1;
   - (d) **Branch C/F — 1-commit repo VÀ not-a-git-repo** → `No schema diff vs HEAD~1 — safe.` + exit 0 (fallback chain đủ 3 bước);
   - (e) header-skip: diff có `---`-header → KHÔNG false-positive; xóa `}` đơn thuần → KHÔNG match (P001 anchor #12 precedent);
   - (f) mỗi alternative pattern `:48` riêng lẻ: field-delete riêng, model-delete riêng, **enum-delete** (parity-blind — fixture không exercise enum) → match đúng;
   - (g) additive-only / no-diff → Branch D / C đúng output + exit 0.
   - Mapping cơ-chế→test ghi vào Discovery.

**Lưu ý:**
1. Pin là oracle: test ĐỎ → sửa `port.rs`/`schema.rs` hoặc bước dựng env cho ĐÚNG repin.sh — KHÔNG sửa pin/fixture/repin.sh (Luật chơi 3).
2. Order findings khớp pin EXACT (P003 lesson — không sort lại).
3. Git version drift (pin env git 2.50.1 — MANIFEST §3): nếu diff output local lệch pin → Debate Log objection, KHÔNG tự normalize.

### Task 6: Docs gate

**File:** `CHANGELOG.md` — entry P004 (Unreleased): port INV-001 + schema-safety, variants `check port`/`check schema`, parity vs pins (gồm stderr-pin contract port), note bash-script đầu tiên + git-via-Command + bad-SHA fallback ported as-is (O1.2). KHÔNG bump version `Cargo.toml` (F13).
**File:** `docs/ARCHITECTURE.md` — Components: `port.rs` + `schema.rs` từ "planned" → shipped (P004), format như `secrets.rs`/`runtime.rs` (cite range, không count — IG-04).
**File:** `tests/golden/MANIFEST.md` §4 — append note NẾU chốt rule mới reuse được cho P005 (candidates từ Turn 1: stderr-pin contract per-check; git-via-Command + stderr-suppress pattern; bash-port precedent). Không có gì mới → không đụng.
**File:** `docs/discoveries/P004.md` + index 1-line `docs/DISCOVERIES.md` — gồm: (i) port parse 4-layer + bảng branch schema A-F kèm cite; (ii) stderr-pin contract (anchor #5); (iii) **O1.2 bad SHA `:33` ported as-is** + lý do; (iv) non-UTF-8 mapping (Python crash → Rust error-path exit, anchor #8); (v) mapping cơ-chế→unit-test cả 2 check; (vi) note reuse cho P005 `gate` (orchestrator aggregate cả 4 check — stderr contract giờ KHÁC nhau giữa các check).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Task 1: thêm variant `Port` + `Schema` + dispatch |
| `src/checks/mod.rs` | Task 1: thêm `pub mod port;` + `pub mod schema;` |
| `src/checks/port.rs` | Task 2: MỚI — port INV-001 4-layer, mọi rule cite golden:line |
| `src/checks/schema.rs` | Task 3: MỚI — port schema-safety 6 branch, git via Command, cite golden:line |
| `tests/parity_port.rs` | Task 4: MỚI — 2 parity (stdout+stderr byte-exact) + 12 probes |
| `tests/parity_schema.rs` | Task 5: MỚI — 2 parity (env-reconstruction) + 7 probes |
| `CHANGELOG.md` | Task 6: entry P004 |
| `docs/ARCHITECTURE.md` | Task 6: port.rs + schema.rs shipped |
| `tests/golden/MANIFEST.md` | Task 6: §4 note — CONDITIONAL |
| `docs/discoveries/P004.md` + `docs/DISCOVERIES.md` | Discovery report + 1-line index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng khi nghiệm thu (kể cả bad SHA `:33` — KHÔNG "fix") |
| `tests/golden/pins/**`, `tests/golden/fixtures/**`, `tests/golden/repin.sh` | Oracle — `git diff` rỗng. Parity fail → sửa Rust/env-setup |
| `src/checks/secrets.rs`, `src/checks/runtime.rs`, `tests/parity_secrets.rs` | P002/P003 shipped — diff rỗng, test xanh |
| `tests/parity_runtime.rs` | Diff rỗng — NGOẠI LỆ: refactor import helper chung (Scope), behavior diff = 0 |
| `Cargo.toml` | KHÔNG dep mới (`git2`/`serde_yaml`/`glob`/`fancy_regex` cấm — escalate Chủ nhà nếu bất khả), không bump version. Diff rỗng |
| `CLAUDE.md` | Chỉ đụng nếu phát hiện deviation exit-code MỚI — kỳ vọng: không |

---

## Luật chơi (Constraints)

1. **Parity-first, security surface:** KHÔNG đổi/thêm/nới rule, exception set, pattern, output wording, count logic, WARN wording so với oracle. Mỗi surface mang comment cite golden:line. Nghi golden "sai" (gồm bad SHA O1.2, 1-part `unrecognized format`) → pin nguyên trạng + DISCOVERY, cải tiến để phiếu sau. PR cần Giám sát review (CLAUDE.md).
2. **regex crate only, KHÔNG dep mới** (anchor #9 ✅ compile sạch — nếu EXECUTE phát hiện ngược lại → DỪNG, Debate Log objection, equivalence proof MỚI theo MANIFEST §4 rule 6, KHÔNG auto-reuse transcription db-conn). Git CHỈ qua `std::process::Command` — `git2` cấm, bất khả → objection + `⚠️ AWAITING CHỦ NHÀ`.
3. **Pin bất khả xâm phạm:** parity đỏ → sửa Rust hoặc env-setup. Sửa pin/fixture/repin.sh/MANIFEST-để-test-xanh = vi phạm scope (`MANIFEST.md` ngoại lệ: Task 6 conditional note).
4. **Byte-exact** stdout VÀ stderr (port — pin có stderr) gồm banner/PASS line + trailing newline. Schema stderr rỗng (git suppress). Không print gì ngoài golden.
5. **Exit contract:** check logic chỉ exit 0/1 (gồm bypass exit 0). Exit 2 = clap usage error duy nhất (deviation đã document P001). Non-UTF-8 port: error-path exit non-zero, KHÔNG panic-101 (Task 2 mục 6).
6. **CLI tối thiểu:** chỉ thêm `check port` + `check schema`. Không flag, không JSON (P006), không đụng `secrets`/`runtime`, không stub `gate`.
7. **Test env hermetic:** mọi Command trong test `env_remove("ALLOW_DATA_LOSS")` (anchor #16 ✅ — không env nào khác). Harness/fixtures LF (precedent P002).
8. **Cite RANGE, không count** (IG-04): comment + docs ghi `golden:<line-range>`, KHÔNG ghi "N entries".
9. `cargo test` xanh toàn bộ trước commit — gồm regression `parity_secrets` + `parity_runtime`.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — gồm 4 parity test: `port--dirty` exit 1 + stdout/stderr byte-exact, `port--clean` exit 0 + stdout/stderr byte-exact (stderr = 2 dòng WARN pin), `schema--dirty` exit 1 + stdout byte-exact (banner + 4 raw lines + instruction block), `schema--clean` exit 0 + byte-exact
- [ ] Unit probes Task 4 (a)-(g, f1-f6) đủ 12 + Task 5 (a)-(g) đủ 7 — MỌI cơ chế parity-blind có test (đặc biệt: **ALLOW_DATA_LOSS bypass**, schema branch B/C/F, enum-delete, single-quote skip, bare-port `unrecognized format`, `0.0.0.0` explicit, is_in_ports_block)
- [ ] Regression: `parity_secrets` + `parity_runtime` xanh; `git diff src/checks/secrets.rs src/checks/runtime.rs tests/parity_secrets.rs` rỗng (`parity_runtime.rs` rỗng hoặc import-only refactor)
- [ ] `cargo check` / build sạch
- [ ] `git diff golden/ tests/golden/fixtures/ tests/golden/pins/ tests/golden/repin.sh` rỗng
- [ ] `git diff Cargo.toml` rỗng

### Manual Testing
- [ ] Dogfood: `cargo run -- check port` tại repo root inv-gate — không có compose file → 3 dòng WARN stderr + PASS stdout + exit 0 (0 violation); `cargo run -- check schema` — không có `prisma/` → Branch B `❌ ... not found` + exit 1 (golden behavior — ghi nhận, KHÔNG "sửa")
- [ ] `cargo run -- check bogus` → clap usage error, exit 2 (regression contract P002)
- [ ] `ALLOW_DATA_LOSS=true cargo run -- check schema` trong repo test destructive → bypass echo + exit 0 (mắt thường)
- [ ] Mỗi rule/pattern/exception constant trong `port.rs`/`schema.rs` có comment cite golden:line; fallback chain có comment O1.2 (bad SHA as-is)

### Regression
- [ ] `bash tests/golden/repin.sh` vẫn chạy được + `git diff tests/golden/pins/` rỗng sau khi chạy

### Docs Gate
- [ ] `CHANGELOG.md` — entry P004 (gồm note O1.2 + stderr contract)
- [ ] `docs/ARCHITECTURE.md` — `port.rs` + `schema.rs` shipped, hết "planned"
- [ ] `tests/golden/MANIFEST.md` §4 — note nếu có (kèm citation), không thì ghi "None" trong discovery
- [ ] Discovery hook P005: stderr contract per-check + git-via-Command + bash-port precedent ghi rõ cho `gate --all`

### Discovery Report
- [ ] `docs/discoveries/P004.md`: assumptions ĐÚNG/SAI (cite file:line — đặc biệt #4 4-layer, #5 stderr pin, #8 non-UTF-8 crash mapping, #11/O1.2 bad SHA, #12 branch table), mapping cơ-chế→unit-test cả 2 check, tier escalation ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
