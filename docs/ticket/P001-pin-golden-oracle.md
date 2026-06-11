# PHIẾU P001: Pin golden oracle — fixture set + freeze findings/exit codes vào `tests/golden/`

> **Loại:** Feature (test-infra / oracle pinning)
> **Ưu tiên:** P1
> **Tầng:** 1 — pinned oracle là shared contract cho toàn bộ P002-P005 (sai pin → LAN vào mọi parity test); scan-target patterns là SECURITY surface (CLAUDE.md). AUTO Tầng 1.
> **Ảnh hưởng:** `tests/golden/**` (toàn bộ là file MỚI), `CHANGELOG.md`
> **Dependency:** None (phiếu đầu sprint — P002..P005 đều block trên phiếu này)

---

## Context

### Vấn đề hiện tại

`golden/` chứa 5 script frozen từ tarot (797 LOC Python+Bash) — đây là oracle cho Rust port. Method bắt buộc (CLAUDE.md + golden/README.md, doc-rotate precedent): **PIN TRƯỚC, PORT SAU**. Hiện chưa có fixture set, chưa có pinned output — không phiếu port nào (P002-P005) được phép chạy trước khi P001 xong.

### Giải pháp

1. Worker đọc 5 script `golden/` → trích pattern class + invocation contract của từng check.
2. Dựng fixture set 2 nhánh: `dirty/` (repo mẫu có secret + port-bind + token-in-url + schema-deletion) và `clean/` (bản sạch cùng cấu trúc). Schema-safety pin qua **git repo thật trong temp** (2 commit — xem Task 2/3, hệ quả O1.2).
3. Viết harness `repin.sh` chạy 5 script trên fixture, capture stdout/stderr/exit code → freeze vào `tests/golden/pins/`.
4. Viết `MANIFEST.md`: provenance từng fixture (golden `file:line` của pattern nó trigger), invocation contract, exit-code table, và **unit-spec table cho mọi offset field (F06: char vs byte vs line)**.

### Scope

- CHỈ tạo file mới dưới `tests/golden/` + 1 entry `CHANGELOG.md` + discovery report.
- **PIN ONLY — KHÔNG port Rust nào trong phiếu này.** Không sửa `src/`, không sửa `Cargo.toml`.
- KHÔNG sửa bất kỳ file nào trong `golden/` (read-only reference — hard rule CLAUDE.md). Nếu script cần path shim để chạy → copy ra temp dir, không bao giờ edit gốc.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | 5 script golden tồn tại đúng tên: `security-gate.sh`, `check-hardcoded-secrets.py`, `check-port-bind.py`, `check-runtime-secrets.py`, `check-schema-safety.sh` `[verified — Architect Glob golden/*]` | `ls golden/` | ✅ Glob confirmed (+ `README.md`) |
| 2 | Mapping script→INV→port-target đúng như bảng golden/README.md `[verified — đã đọc README]` | đọc `golden/README.md` | ✅ |
| 3 | `tests/golden/` CHƯA tồn tại (Architect bị envelope chặn glob `tests/**`) `[needs Worker verify]` | `ls tests/ 2>/dev/null` | ⏳ TO VERIFY — nếu đã có file, DISCOVERY_REPORT trước khi ghi đè |
| 4 | `python3` + `bash` có trên máy; 3 script `.py` chạy stdlib-only (không pip dep) `[needs Worker verify]` `[oracle: python3 chạy thật — SOUND]` | `python3 --version && bash --version`; chạy thử từng `.py` không arg, xem ImportError | ⏳ TO VERIFY (Bước 0 capability — không pin trên giả định "chắc chạy được") |
| 5 | Invocation contract từng script `[verified — Worker Turn 1]` | đã đọc source 5 file | ✅ Cả 4 script check KHÔNG nhận args, assume cwd = repo root, hardcode target: `["src"]`, `["astro-service"]`, `COMPOSE_FILES`, `prisma/schema.prisma`. KHÔNG có exit-2 mode. Chỉ `security-gate.sh` nhận flags (`--mechanical-only`, `--include-ssh`) + exit 2 trên unknown flag. Worker ghi mapping script→target vào MANIFEST |
| 6 | `security-gate.sh` hardcode path sibling scripts `[verified — Worker Turn 1]` | grep trong `security-gate.sh` | ✅ Hardcode `python3 scripts/check-*.py` (lines 55/197/201) → harness PHẢI copy script ra temp `scripts/` layout |
| 7 | `security-gate.sh` chứa bước SSH/remote `[verified — Worker Turn 1]` | grep `ssh` | ✅ INV-007 SSH `root@103.167.150.178:1994` tại line 147; `--mechanical-only` skip hẳn (line 174) → pin nhánh mechanical-only, ghi bước skip vào MANIFEST |
| 8 | Exit-code contract 0/1/2 per script `[verified một phần — Worker Turn 1]` `[oracle: chạy script + `echo $?` — SOUND]` | chạy clean/dirty/usage-sai | ✅ 4 script check: chỉ có 0/1, KHÔNG có exit-2 mode (ghi "no usage-error mode" vào MANIFEST). `security-gate.sh`: exit 2 trên unknown flag → pin run này. ⏳ giá trị 0/1 thật trên fixture vẫn pin ở Task 4 |
| 9 | Output format `[verified — Worker Turn 1]` | đọc format string | ✅ `path:lineno: INV-xxx violated -- <masked> (<pattern_name>)` — LINE-ONLY, không có col/offset field, cả 4 check. Task 5 ghi explicit "line-only" per check |
| 10 | `CHANGELOG.md` nằm ở repo root `[verified — Glob + `.docs-gate.toml` changelog = "../CHANGELOG.md"]` | `ls CHANGELOG.md` | ✅ |
| 11 | `docs/DISCOVERIES.md` chưa tồn tại (Glob không thấy) — Worker tạo mới khi viết index entry `[verified — Architect Glob **/DISCOVERIES.md rỗng]` | `ls docs/DISCOVERIES.md` | ✅ not found |
| 12 | `check-schema-safety.sh` detect cả 2 class xóa: field-deletion VÀ model-deletion `[verified — Worker Turn 2]` | Worker cite rule lines trong MANIFEST | ✅ Pattern `^-\s*(model\|enum)\s+\w+\|^-\s+\w+\s+\S+` bắt cả field-delete lẫn model-delete; closing brace không false-positive |
| 13 | `check-runtime-secrets.py` đọc `.git/config` (token-in-url) hay scan `astro-service/`? `[verified — Worker Turn 2]` | đọc source, ghi mapping vào MANIFEST | ✅ `.git/config` nằm trong `RUNTIME_FILES` → **nhánh A** (harness inject remote URL). Nhánh B bị BỎ |
| 14 | `COMPOSE_FILES` của check-port-bind có chứa tên `docker-compose.yml` `[verified — Worker Turn 2]` | đọc constant trong source | ✅ `docker-compose.yml` là entry ĐẦU của `COMPOSE_FILES` |

**Anchor #3 mà ra ❌ khác giả định → handling đã ghi sẵn ở cột Result; không cần re-spawn Architect trừ khi đổi cấu trúc fixture.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V3 (revised sau Turn 2)

### Turn 1 — Worker Challenge
- **[O1.1]** Task 3 step 3 usage-error run: 4 script check không nhận args, không có exit-2 mode → **SELF-CLOSED** `[oracle: đọc source + chạy script — SOUND]`. Escape hatch sẵn trong phiếu V1 ("no usage-error mode" vào MANIFEST). Chỉ `security-gate.sh` có exit 2 (unknown flag).
- **[O1.2] BLOCKING** — Prisma fixture sai cơ chế oracle: `golden/check-schema-safety.sh` KHÔNG scan migration SQL; nó chạy `git diff HEAD~1..HEAD -- prisma/schema.prisma` tìm field/model bị XÓA trong declarative schema. Fixture V1 (`migration.sql` + DROP TABLE) → exit 0 trên dirty (NGƯỢC intended) → pin sai baseline cho toàn bộ P002-P005.
- Facts verified kèm theo: anchor #5/#6/#7/#9 (đã fold vào bảng Task 0, cột Result).

**Status:** ✅ RESPONDED

### Turn 1 — Architect Response (phiếu V2)
- **[O1.1]** → đã self-closed bởi oracle, không cần Architect action. V2 ghi cứng vào Task 3 step 3: usage-error run CHỈ áp dụng cho `security-gate.sh`.
- **[O1.2]** → **ACCEPT**, chọn hướng **(a)**: harness dựng git repo THẬT trong temp dir với 2 commit (commit 1 = schema đủ field, commit 2 = xóa field + model → `git diff HEAD~1..HEAD` trigger thật). Lý do: oracle phán trên git history — fixture tĩnh không bao giờ trigger đúng claim. Spec chi tiết: Task 2 (nội dung schema before/after) + Task 3 step 1-2 (vị trí trong repin.sh, commit messages, determinism).
  - **Hệ quả dây chuyền (Architect reconcile):** temp dir nay là git repo thật → cơ chế `dot-git/` rename của V1 bị BỎ (sẽ clobber `.git/` thật). INV-010 trigger chuyển sang `git remote add origin <token-in-url FAKE>` trong harness, kèm nhánh dự phòng nếu script scan `astro-service/` (anchor #13).
- Anchor mới #12/#13/#14 thêm cho các điểm chưa chốt được từ docs.

**Status:** ✅ RESPONDED — phiếu bumped to V2.

### Turn 2 — Worker Challenge
- **[O2.1] BLOCKING** — Fake token V2 sai format: `golden/check-runtime-secrets.py:96` đòi password segment match `gh[pous]_[A-Za-z0-9]{36}` hoặc `github_pat_[A-Za-z0-9_]{82}`. `FAKE_TOKEN_000` (14 chars) không match → dirty INV-010 exit 0 → pin sai. Fix Worker đã pre-verify chống regex + allowlist: `ghp_` + 36 alphanum, "FAKE" không nằm trong `ALLOWLIST_SUBSTRINGS` nên vẫn trigger.
- **Note Tầng 2:** `golden/check-schema-safety.sh:18` exit 0 sớm nếu env `ALLOW_DATA_LOSS=true` → harness cần `unset ALLOW_DATA_LOSS` chống env leak pin sai.
- Facts verified kèm theo: anchor #12/#13/#14 ✅ (fold vào bảng Task 0); determinism git OK — config local + fixed dates đủ, branch name không ảnh hưởng SHA.

**Status:** ✅ RESPONDED

### Turn 2 — Architect Response (phiếu V3)
- **[O2.1]** → **ACCEPT** `[oracle: Worker đã chạy regex + allowlist — SOUND]`. Thay MỌI occurrence `FAKE_TOKEN_000` bằng `ghp_FAKETOKEN000000000000000000000000000` (`FAKETOKEN` 9 ký tự + 27 số `0` = đúng 36 alphanum sau `ghp_`; tổng 40 chars — Worker self-check `echo -n '<token>' | wc -c` → 40 trước khi inject). Citation `golden/check-runtime-secrets.py:96` ghi cạnh giá trị (F07). Sửa tại: Task 2 bảng INV-010 + Task 3 step 2. Vẫn thỏa Constraint 6 — fake rõ ràng, không trip push-protection.
- **[Note Tầng 2]** → **ACCEPT (fold)**: Task 3 step 1 thêm env hygiene `unset ALLOW_DATA_LOSS` kèm citation `golden/check-schema-safety.sh:18`.
- Anchor #13 ✅ nhánh A → bỏ nhánh B khỏi Task 2/Task 3 (giảm dead branch trong spec).

**Status:** ✅ RESPONDED — phiếu bumped to V3. Sẵn sàng Worker CHALLENGE Turn 3 (consensus check) hoặc approval gate.

---

## Nhiệm vụ

### Task 1: Trích pattern class + invocation contract từ golden → MANIFEST skeleton

**File:** `tests/golden/MANIFEST.md` (MỚI)

**Thêm:** Đọc 5 script `golden/` (Worker được phép — Architect thì không), ghi vào MANIFEST:

1. **Per-check pattern table:** mỗi check liệt kê pattern class nó bắt (secret regex classes, port-bind rule + nginx exception, token-in-url rule, **schema-deletion rule: field-delete / model-delete — anchor #12 verified, cite pattern line**) — mỗi dòng cite `golden/<file>:<line>` `[needs Worker verify — Architect không đọc được script]`.
2. **Invocation contract:** từng script — mapping script→hardcoded target (anchor #5: `["src"]`, `["astro-service"]`, `COMPOSE_FILES`, `prisma/schema.prisma`), cwd assumption (= repo root, verified), output stream (stdout vs stderr). Ghi rõ: 4 check không nhận args, không exit-2 mode; gate nhận `--mechanical-only` / `--include-ssh`, exit 2 unknown flag.
3. **Oracle environment provenance:** `python3 --version`, `bash --version`, `git --version` (schema check phụ thuộc git), OS — pin chỉ valid trong env tương đương.

**Lưu ý:** Đây là bước chống F07 — fixture ở Task 2 chỉ được chứa trigger string DERIVE từ pattern đã cite ở đây, không tự nghĩ. Mapping script→target ở mục 2 quyết định VỊ TRÍ đặt trigger file của Task 2.

### Task 2: Dựng fixture set

**File:** `tests/golden/fixtures/dirty/**` + `tests/golden/fixtures/clean/**` (MỚI)

**Thêm:** Hai cây thư mục cùng cấu trúc. Spec per-INV (class do Architect chốt từ PROJECT.md — exact string Worker derive từ golden, cite trong MANIFEST). **Vị trí file PHẢI khớp hardcoded target dir của script tương ứng (MANIFEST mục 2)**:

| INV | Fixture dirty (≥1 finding) | Fixture clean (0 finding) |
|---|---|---|
| INV-009 secrets | `src/config.py` (target verified = `["src"]`; đuôi file script scan `[needs Worker verify]`) chứa ≥2 instance secret-class match regex golden | cùng file, đọc secret từ env var |
| INV-001 port | `docker-compose.yml` (tên file verified = entry đầu `COMPOSE_FILES` — anchor #14 ✅): 1 service non-nginx bind `0.0.0.0:<port>` (violation per PROJECT.md `[unverified]` — rule thật lấy từ golden) | nginx bind 80/443 (exception được phép) + service khác bind `127.0.0.1` |
| INV-010 runtime | **Nhánh A verified (anchor #13 ✅ — `.git/config` trong `RUNTIME_FILES`):** harness inject `git remote add origin https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git` (token format per `golden/check-runtime-secrets.py:96` — `ghp_` + 36 alphanum; "FAKE" không nằm trong allowlist. Task 3 step 2, KHÔNG cần file fixture tĩnh) | Remote URL sạch `https://github.com/example/fixture.git` |
| Prisma schema | **Git-history fixture** (O1.2): `prisma/schema.before.prisma` + `prisma/schema.after.prisma` — harness materialize thành 2 commit (Task 3). `after` XÓA field `legacyToken` khỏi `User` VÀ XÓA nguyên model `AuditLog` (cover cả 2 class — anchor #12 ✅ pattern bắt cả hai) | `prisma/schema.before.prisma` (giống dirty) + `prisma/schema.after.prisma` ADDITIVE: giữ nguyên 2 model, thêm field `displayName String?` vào `User` |

**Nội dung schema fixture (Architect chốt):**

`schema.before.prisma` (chung cả 2 nhánh):

```prisma
datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

generator client {
  provider = "prisma-client-js"
}

model User {
  id          Int     @id @default(autoincrement())
  email       String  @unique
  legacyToken String?
}

model AuditLog {
  id      Int    @id @default(autoincrement())
  payload String
}
```

- `dirty/prisma/schema.after.prisma`: bản trên BỎ dòng `legacyToken String?` và BỎ toàn bộ block `model AuditLog`.
- `clean/prisma/schema.after.prisma`: bản trên + thêm dòng `displayName String?` vào `User`.

**Lưu ý (load-bearing):**
1. **KHÔNG còn `dot-git/`** (đổi so V1 — O1.2 hệ quả): temp dir là git repo thật do harness `git init`; mọi trigger liên quan `.git/config` do harness inject bằng lệnh git, không phải file fixture tĩnh.
2. **Secret/token giá trị FAKE rõ ràng** (prefix/suffix `FAKE`, entropy giả) nhưng VẪN match regex golden — áp cho cả token-in-url ở remote URL (giá trị đã chốt ở bảng trên, O2.1). Fixture không được chứa thứ giống credential thật (tự trip GitHub push-protection / chính inv-gate sau này).
3. **F07:** không thêm fixture file nào ngoài bảng trên. Cần thêm (vd golden scan thêm loại file khác) → ghi DISCOVERY_REPORT + thêm dòng MANIFEST với citation, không thêm âm thầm.

### Task 3: Harness `repin.sh`

**File:** `tests/golden/repin.sh` (MỚI, executable)

**Thêm:** Bash script, chạy từ repo root:

1. **Dựng temp git repo per fixture (dirty rồi clean) — đổi so V1 (O1.2):**
   - `tmp=$(mktemp -d)`; copy toàn bộ fixture (TRỪ `schema.before/after.prisma`) vào `$tmp`.
   - `git init -q "$tmp"`; config LOCAL (hermetic, không đụng global): `user.name "P001 Pin Harness"`, `user.email "pin@inv-gate.local"`, `commit.gpgsign false`.
   - Determinism: export `GIT_AUTHOR_DATE="2026-01-01T00:00:00 +0000"` và `GIT_COMMITTER_DATE` cùng giá trị → commit SHA ổn định giữa các lần chạy (idempotency Task 4).
   - **Env hygiene (O2 Tầng 2 note):** `unset ALLOW_DATA_LOSS` — `golden/check-schema-safety.sh:18` exit 0 sớm nếu env này `=true`; leak từ shell ngoài sẽ pin sai.
   - **Commit 1:** đặt `schema.before.prisma` vào `$tmp/prisma/schema.prisma`, `git add -A`, commit message `P001 fixture baseline`.
   - **Commit 2:** ghi đè `$tmp/prisma/schema.prisma` bằng `schema.after.prisma`, `git add`, commit message `P001 fixture schema change`. → `git diff HEAD~1..HEAD -- prisma/schema.prisma` chứa ĐÚNG VÀ CHỈ schema change.
   - Copy 5 script golden vào `$tmp/scripts/` layout (anchor #6 verified — gate hardcode `python3 scripts/check-*.py` lines 55/197/201). Copy, không sửa gốc.
2. **INV-010 remote inject (nhánh A — anchor #13 ✅):** dirty → `git -C "$tmp" remote add origin https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git` (token per `golden/check-runtime-secrets.py:96` — O2.1; self-check token = 40 chars trước khi inject); clean → URL sạch.
3. Với mỗi script × mỗi fixture: chạy từ `$tmp` (cwd contract verified), capture stdout → `tests/golden/pins/<check>--<fixture>.stdout.txt`, stderr (nếu non-empty) → `...stderr.txt`, exit code → gom vào `tests/golden/pins/exit_codes.json` (`{"<check>--<fixture>": <code>}`).
4. **Usage-error run — CHỈ `security-gate.sh`** (chốt từ O1.1/anchor #5): chạy với 1 unknown flag (vd `--no-such-flag`) → pin exit 2 thành `security-gate--usage-error`. 4 script check: ghi "no usage-error mode" vào MANIFEST, không có run này.
5. **Normalize path:** thay absolute temp path trong output bằng path fixture-relative (sed) — pin phải reproducible giữa các máy. CHỈ normalize path, không đụng nội dung finding nào khác; rule normalize ghi vào MANIFEST. Nếu output chứa commit SHA: SHA đã deterministic nhờ fixed dates (step 1) — nếu vẫn lệch giữa máy, thêm rule normalize SHA→`<SHA>` và ghi MANIFEST.
6. Deterministic: nếu output order phụ thuộc filesystem order `[needs Worker verify]` → KHÔNG sort output (đổi oracle = sai); thay vào đó ghi nhận xét vào MANIFEST để parity test sau này biết so sánh order-insensitive hay không.

**Lưu ý:** `security-gate.sh` pin theo nhánh `--mechanical-only` (anchor #7 verified — SSH INV-007 line 147 bị skip tại line 174); ghi rõ bước skip vào MANIFEST.

### Task 4: Chạy pin + freeze

**File:** `tests/golden/pins/**` (MỚI — output của Task 3)

**Thêm:** Chạy `repin.sh`. Acceptance tại chỗ:
- Mỗi check trên `dirty` → exit `1` + ≥1 finding trong stdout pin. Riêng schema check: finding phải phản ánh deletion (field `legacyToken` và/hoặc model `AuditLog` — per anchor #12).
- Mỗi check trên `clean` → exit `0` + 0 finding. Riêng schema check: clean CŨNG có diff (additive) — exit 0 chứng minh script phân biệt additive vs deletion, không phải "không có diff nên pass".
- `security-gate.sh` pinned cả 2 fixture (aggregate, nhánh mechanical-only) + run usage-error (exit 2).
- Chạy `repin.sh` lần 2 → `git diff tests/golden/pins/` rỗng (idempotent — fixed git dates ở Task 3 step 1 là điều kiện cần).

Nếu check nào KHÔNG ra finding trên dirty → fixture chưa trúng pattern → quay lại Task 2 chỉnh trigger string (vẫn trong class đã spec), KHÔNG nới pattern, KHÔNG sửa golden.

### Task 5: Unit-spec table (F06) — hoàn thiện MANIFEST

**File:** `tests/golden/MANIFEST.md`

**Thêm:** Bảng "Offset unit spec": mỗi field số xuất hiện trong pinned output → 1 dòng:

| Check | Field | Unit (line / char-col / byte-col / byte-offset) | Derive từ (golden file:line) |

**Pre-fill từ anchor #9 (verified):** cả 4 check output `path:lineno: INV-xxx violated -- <masked> (<pattern_name>)` — LINE-ONLY → 4 dòng explicit "`<check>`: line-only, no col/offset field". Worker đối chiếu lại với pins thật ở Task 4; lệch → sửa bảng theo reality + cite. **Không được để field số nào vắng mặt khỏi bảng** — acceptance cứng (bài học F06: char vs byte lệch nhau giữa Python và Rust là bug parity kinh điển).

### Task 6: Docs gate

**File:** `CHANGELOG.md` (root) — entry P001: pin golden oracle, liệt kê mọi deviation phát hiện so với contract 0/1/2 (đã biết 1 deviation từ anchor #5/#8: **4 script check không có exit-2 mode** — ghi rõ, kèm note vào CLAUDE.md per rule "document any deviation"). Không bump version `Cargo.toml` (phiếu không ship behavior — F13 chỉ áp khi bump version).
**File:** `docs/ARCHITECTURE.md` — verify-only: mục `tests/golden/ (P001)` đã mô tả đúng layout ship thật chưa; lệch thì sửa 1-2 dòng cho khớp reality.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `tests/golden/MANIFEST.md` | Task 1 + 5: pattern provenance, invocation contract + mapping script→target, env provenance, unit-spec table |
| `tests/golden/fixtures/dirty/**` | Task 2: trigger 4 class — gồm `prisma/schema.before.prisma` + `schema.after.prisma` (deletion) |
| `tests/golden/fixtures/clean/**` | Task 2: bản sạch cùng cấu trúc — schema after ADDITIVE |
| `tests/golden/repin.sh` | Task 3: harness pin reproducible, dựng temp git repo thật 2 commit |
| `tests/golden/pins/**` | Task 4: stdout/stderr/exit codes frozen |
| `CHANGELOG.md` | Task 6: entry P001 + deviation exit-code |
| `CLAUDE.md` | Task 6: note deviation exit-code contract (4 check không có exit 2) |
| `docs/discoveries/P001.md` + `docs/DISCOVERIES.md` | Discovery report + index (tạo mới — anchor #11) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | **READ-ONLY tuyệt đối** — kể cả khi script không chạy in-place (dùng temp copy). `git diff golden/` phải rỗng khi nghiệm thu |
| `src/**`, `Cargo.toml` | Không đụng — phiếu này KHÔNG port Rust. `cargo test` vẫn xanh như trước |
| `docs/ARCHITECTURE.md` | Chỉ sửa nếu layout `tests/golden/` ship khác mô tả hiện tại (Task 6) |

---

## Luật chơi (Constraints)

1. **PIN ONLY.** Bất kỳ dòng Rust nào xuất hiện trong diff → scope violation, dừng + escalate.
2. **`golden/` bất khả xâm phạm.** Path shim = copy ra temp. Output "sai" so với kỳ vọng = pin nguyên trạng + ghi MANIFEST, không patch oracle.
3. **Exit-code contract là API** (0/1/2). Pin behavior THẬT; deviation đã biết (4 check không có exit-2) + mọi deviation mới → CHANGELOG + CLAUDE.md note, không che.
4. **F07:** fixture file chỉ gồm những gì Task 2 spec; exact trigger string derive từ golden với citation `golden/<file>:<line>` trong MANIFEST.
5. **F06:** mọi offset field có unit spec, không ngoại lệ (đã biết line-only — vẫn ghi explicit per check).
6. **Secret/token fixture = fake rõ ràng** nhưng match regex. Không real-looking creds trong repo — kể cả trong remote URL inject bởi harness.
7. Normalization trên pin: CHỈ path temp→relative (+ SHA nếu cần, ghi MANIFEST). Không sort, không trim finding.
8. Pattern/fixture cho secrets là SECURITY surface (CLAUDE.md) → PR cần Giám sát review trước merge.
9. **Temp git repo phải hermetic:** config local-only, fixed dates, `unset ALLOW_DATA_LOSS` (Task 3 step 1), không phụ thuộc global gitconfig / env của máy.

---

## Nghiệm thu

### Automated
- [ ] `bash tests/golden/repin.sh` chạy sạch end-to-end từ repo root
- [ ] Chạy lần 2 → `git diff tests/golden/pins/` rỗng (idempotent / reproducible — fixed git dates)
- [ ] `cargo test` xanh (không Rust nào bị đụng)
- [ ] `git diff golden/` rỗng

### Manual Testing
- [ ] 4 check + orchestrator: dirty → exit 1 + ≥1 finding; clean → exit 0 + 0 finding (đối chiếu `exit_codes.json`)
- [ ] INV-010 dirty pin có finding token-in-url thật (token fake `ghp_…` match `golden/check-runtime-secrets.py:96` — O2.1)
- [ ] Schema check: dirty finding phản ánh deletion (field + model); clean có diff additive nhưng exit 0 (chứng minh phân biệt additive/deletion)
- [ ] MANIFEST có đủ: pattern citation per fixture trigger, invocation contract + mapping script→target 5 script, env provenance (gồm `git --version`), normalize rule
- [ ] Unit-spec table phủ 100% field số trong pins (4 check pre-fill "line-only" — đối chiếu pins thật)
- [ ] Usage-error run pinned cho `security-gate.sh` (exit 2); 4 check ghi "no usage-error mode" trong MANIFEST

### Regression
- [ ] Repo vẫn build/test như trước phiếu (không src change)

### Docs Gate
- [ ] `CHANGELOG.md` — entry P001 + deviation exit-code (4 check không có exit-2 mode)
- [ ] `CLAUDE.md` — note deviation per rule "document any deviation"
- [ ] `docs/ARCHITECTURE.md` — khớp layout `tests/golden/` thật

### Discovery Report
- [ ] `docs/discoveries/P001.md`: assumptions ĐÚNG/SAI (cite file:line — đặc biệt anchor #12/#13/#14 + O1.2/O2.1 root cause), scope expansion, edge case, docs updated, tier escalation ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md` (tạo file nếu chưa có)
