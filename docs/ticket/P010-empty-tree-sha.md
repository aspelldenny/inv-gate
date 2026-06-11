# PHIẾU P010: Fix bad-SHA empty-tree fallback trong `check schema` (deviation có chủ đích đầu tiên khỏi golden)

> **Loại:** Bugfix / behavior change post-parity (CLAUDE.md method rule 3 — parity xong rồi mới sửa behavior, phiếu riêng = chính là phiếu này)
> **Ưu tiên:** P1 (DEBT — Sếp pick 11/06; không deadline ngoài, làm SAU P009)
> **Tầng:** 1 — `src/checks/schema.rs` = **security-gate surface** (CLAUDE.md: Giám sát review trên PR) + đây là **deviation behavior có chủ đích ĐẦU TIÊN khỏi golden** (P007 là additive flag; P010 sửa hành vi sẵn có vì golden mang bug) → precedent-setting, docs Tầng 1.
> **Ảnh hưởng:** `src/checks/schema.rs`, tests (probe mới + có thể UPDATE probe d cũ — anchor #6), `CHANGELOG.md`, `CLAUDE.md` (deviation note — lần đầu file này được Worker đụng, có authorization rõ bên dưới), `tests/golden/MANIFEST.md` (note §4 — KHÔNG đụng pin .txt), `docs/ARCHITECTURE.md` (chỉ nếu cite SHA cũ — anchor #9), `docs/discoveries/P010.md`, `docs/DISCOVERIES.md`.
> **Dependency:** Độc lập P009 (file không giao nhau ngoài CHANGELOG — rebase trivial). Branch `feat/P010-empty-tree-sha` từ main mới nhất tại thời điểm execute.

> *Authorization sửa CLAUDE.md: rule có sẵn trong CLAUDE.md — "document any deviation from the golden scripts in CHANGELOG + here" — đây là deviation thật đầu tiên → Worker THÊM note ngắn theo Task 3.2, không sửa gì khác trong file đó.*

---

## Context

### Vấn đề hiện tại (O1.2 — docs/discoveries/P004.md)

`golden/check-schema-safety.sh:33` dùng SHA `4b825dc8669f8c0` (**15 ký tự — SHA cụt, không phải object hợp lệ**) làm fallback khi `git diff HEAD~1..HEAD` fail (repo 1 commit / không phải repo) `[verified — docs/discoveries/P004.md §O1.2]`. P004 đã port AS-IS đúng luật parity. Hệ quả hiện tại trong Rust (`src/checks/schema.rs` `get_diff()` Step 2 `[unverified — anchor #2, Worker grep]`):

- Repo 1 commit: Step 1 fail (không có parent) → Step 2 `git diff 4b825dc8669f8c0..HEAD` CŨNG fail (invalid object) → diff rỗng → **Branch C "safe" do LỖI LỆNH, không phải do diff sạch** — outcome đúng, mechanism sai (P004 wording).
- Empty-tree SHA đúng là `4b825dc642cb6eb9a060e54bf8d69288fbee4904` (40 ký tự) — hằng số git SHA-1, oracle: `git hash-object -t tree /dev/null`.

### Giải pháp

1. **Thay SHA cụt bằng SHA chuẩn 40 ký tự — HARDCODE + comment cite oracle (Architect đã quyết, chốt):**
   - **A (CHỌN): hardcode** `4b825dc642cb6eb9a060e54bf8d69288fbee4904` + comment `// git empty-tree SHA (SHA-1 constant); oracle: git hash-object -t tree /dev/null; fixes golden:33 truncated 15-char (P010, see CLAUDE.md deviations)`.
   - B (LOẠI): gọi `git hash-object -t tree /dev/null` runtime — bị loại vì: thêm 1 process spawn mỗi lần fallback fire + thêm 1 failure mode mới (command fail thì sao?) để lấy 1 HẰNG SỐ bất biến của git. Bù rủi ro typo: **probe (e) Task 2 chạy oracle lúc test-time** — hằng số sai thì test đỏ ngay trên mọi máy.
   - Giới hạn biết trước: repo init `--object-format=sha256` có empty-tree hash KHÁC. Golden chưa bao giờ hỗ trợ sha256-repo → ngoài scope, ghi 1 dòng trong comment/Discovery, KHÔNG xử lý.
2. **Hệ quả behavior — Architect phân tích trước, Worker verify bằng probe (anchor #7, #8):**
   - Repo 1 commit, post-fix: Step 2 THÀNH CÔNG → diff empty-tree..HEAD = toàn dòng `+` (additions) → route sang **Branch D** (đánh giá nội dung thật) thay vì Branch C (fallback mù). Đây là delta quan sát được duy nhất.
   - **QUAN TRỌNG — đọc kỹ:** diff so empty-tree KHÔNG THỂ chứa dòng deletion `-` (mọi thứ là added; header `---` bị `HEADER_SKIP_RE` bỏ qua), mà `DESTRUCTIVE_RE` match dòng deletion `[verified — P004 anchor #13: cả 4 matched lines bắt đầu bằng `-`]` → **case "1-commit repo + destructive → exit 1" là UNREACHABLE by construction.** Brief BACKLOG specced test exit-1 cho nhánh này — Architect kết luận test đó không thể viết đúng; thay bằng probe routing C→D (Task 2). Nếu Worker grep `DESTRUCTIVE_RE` thấy pattern match cả dòng KHÔNG bắt đầu `-` (phân tích này sai) → Debate Log, vì khi đó fix mở ra nhánh exit-1 thật và test plan phải viết lại.
   - Not-a-repo: Step 1 fail → Step 2 CŨNG fail (không có git repo) → Branch C nguyên trạng — fix không đổi nhánh này.
   - Repo ≥2 commit: Step 1 thành công → Step 2 không bao giờ chạy → **parity pins KHÔNG đổi** (fixture schema có 2 commit — anchor #5).
3. **Test trước-fix-fail (chứng minh bug thật):** probe routing 1-commit-repo viết TRƯỚC khi sửa → chạy → ĐỎ (đang ra Branch C) → fix → XANH (ra Branch D). Transcript red→green vào Discovery. Điều kiện: wording stdout Branch C ≠ Branch D (anchor #7); nếu trùng wording → assert ở seam `get_diff()` (diff rỗng vs non-empty) — Worker chọn seam, Tầng 2.
4. **Docs:** CHANGELOG `[Unreleased] — P010` + CLAUDE.md deviation note + MANIFEST.md §4 note update (O1.2 → fixed in Rust, golden giữ bug làm reference).
5. **Ship (IG-09):** commit branch → Giám sát (security-gate surface) → merge → push → xóa branch.

### Scope

- CHỈ sửa: `src/checks/schema.rs` (1 hằng SHA + comment), tests (probe mới + update probe d cũ NẾU anchor #6 buộc), `CHANGELOG.md`, `CLAUDE.md` (note deviation — text chốt Task 3.2), `tests/golden/MANIFEST.md` (1 note §4), `docs/ARCHITECTURE.md` (chỉ nếu anchor #9 dương), `docs/discoveries/P010.md`, `docs/DISCOVERIES.md`.
- KHÔNG sửa: `golden/**` (**NEVER "fix" golden** — CLAUDE.md method rule 4; golden giữ bug làm reference vĩnh viễn), `tests/golden/*.txt` pins (byte read-only — fix không chạm fixture path, pins PHẢI byte-identical), `src/` ngoài `schema.rs`, `src/gate.rs`/`src/serve.rs` (gọi run_core schema — hưởng fix tự động, không sửa), `.github/**`, `Cargo.toml`/`Cargo.lock` (không dep mới, không version bump — F13 fire lúc release thật), `hooks/**`, `scripts/**`.
- KHÔNG implement: hỗ trợ sha256-repo, pattern destructive mới/tốt hơn (behavior change khác = phiếu khác), sửa wording Branch C/D (giữ nguyên output text — chỉ đổi ROUTING cho 1-commit repo), repin pins (không có gì để repin).

---

## Task 0 — Verification Anchors

> **Bước 0 capability (P285 Layer 1):** phiếu local-only (cargo + git + temp dir) — Worker có đủ. Không network. ✅ không có capability assumption mở.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Baseline: main mới nhất (≥ `aa262e4`; nếu P009 merge trước → bao gồm nó), `cargo test` xanh (95 hoặc baseline mới), tree clean | `git log -1` + `git status` + `cargo test` | `[unverified — Worker confirm, count = baseline]` |
| 2 | `src/checks/schema.rs` `get_diff()` Step 2 chứa literal `4b825dc8669f8c0` đúng 1 chỗ | `grep -n '4b825dc' src/checks/schema.rs` → kỳ vọng đúng 1 hit | `[unverified — per P004 discovery §O1.2; >1 hit hoặc 0 hit → Debate Log]` |
| 3 | Fallback chain shape: Step 1 `git diff HEAD~1..HEAD` → fail → Step 2 `git diff <SHA>..HEAD` → fail → diff rỗng → Branch C | Worker đọc `get_diff()` | `[unverified — per P004 anchor #11; lệch shape → Debate Log]` |
| 4 | Empty-tree SHA đúng = `4b825dc642cb6eb9a060e54bf8d69288fbee4904` | `git hash-object -t tree /dev/null` `[oracle: git, SOUND]` — chạy TRƯỚC khi edit + probe (e) chạy lại lúc test-time | ⏳ TO VERIFY |
| 5 | Parity fixture schema có 2 commit → Step 1 luôn thành công trong fixture path → fallback KHÔNG nằm trong pinned paths → pins byte-identical sau fix | Worker đọc harness `build_fixture_repo` (P004 cite: `tests/parity_runtime.rs:71-132`, reuse tại `tests/parity_schema.rs`) + chạy full parity suite | `[unverified — per P004 anchor #14 + BACKLOG wording; nếu fixture hóa ra 1 commit → DỪNG, Debate Log: fix sẽ đổi pin = vi phạm]` |
| 6 | Probe d cũ của P004 ("1-commit repo / not-a-repo → fallback chain → safe") tồn tại trong tests — assertion của nó cho nhánh 1-commit có thể ĐỎ post-fix (routing C→D) | `grep -rn 'fallback\|1-commit\|one_commit\|probe_d' tests/` + đọc probe | `[needs Worker verify — nếu probe assert wording Branch C cho 1-commit → UPDATE probe sang Branch D (đây là expected-test-update, ghi Discovery); nếu chỉ assert exit 0 → giữ nguyên; nhánh not-a-repo giữ assert C]` |
| 7 | Wording stdout Branch C ≠ Branch D (để probe routing red→green quan sát được từ CLI output) | Worker đọc schema.rs branch outputs (golden cite: C = `golden:32-39` fallback-safe, D = `golden:63` additive-safe) | `[needs Worker verify — nếu trùng wording → assert ở seam get_diff() (rỗng vs non-empty), Tầng 2 Worker chọn]` |
| 8 | `DESTRUCTIVE_RE` chỉ match dòng deletion (bắt đầu `-`) → "1-commit + destructive → exit 1" unreachable so empty-tree | Worker đọc pattern trong schema.rs (port từ `golden:48`) + probe empirical | `[unverified — per P004 anchor #13/probe f; nếu pattern match cả dòng không-`-` → Debate Log, test plan đổi]` |
| 9 | `docs/ARCHITECTURE.md` / `tests/golden/MANIFEST.md` §4 có cite SHA cụt / O1.2 | `grep -rn '4b825dc\|O1.2' docs/ARCHITECTURE.md tests/golden/MANIFEST.md` | `[needs Worker verify — P004 discovery nói MANIFEST §4 rule có O1.2 note → update note; ARCHITECTURE chưa rõ]` |
| 10 | Sau fix: full suite xanh (baseline + probes mới), pins byte-identical, `git diff` chỉ chạm files trong scope | `cargo test` + `git diff --stat` + `git diff tests/golden/` rỗng (trừ MANIFEST.md note) | ⏳ TO VERIFY |

**Anchor mở (Worker verify): TẤT CẢ trừ Bước 0. ✅ Verified từ docs: nguồn P004 discovery cho #2/#3/#5/#8 (vẫn cần grep xác nhận code thật). 0 anchor ❌.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P010
mkdir -p ".backup/${PHIEU_ID}"
cp src/checks/schema.rs ".backup/${PHIEU_ID}/schema.rs.orig"
cp CHANGELOG.md ".backup/${PHIEU_ID}/CHANGELOG.md.orig"
cp CLAUDE.md ".backup/${PHIEU_ID}/CLAUDE.md.orig"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

> Điểm Architect MUỐN Worker challenge kỹ nhất: anchor #8 (unreachability của exit-1 so empty-tree) — nếu phân tích sai, test plan phải viết lại; và anchor #6 (probe d cũ có vỡ không).

### Turn 1 — Worker Challenge (vs V1)

_(Worker CHALLENGE điền)_

**Status:** ⏳ PENDING CHALLENGE

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: 11/06/2026 (Sếp pick trực tiếp + Quản đốc approve theo ủy quyền; nghiệm thu qua Giám sát APPROVE + merge cùng ngày)

---

## Nhiệm vụ

> Thứ tự TDD: Task 1 (probe routing viết trước, chạy → ĐỎ) → Task 2 (fix SHA → XANH + probes còn lại) → Task 3 (docs) → Task 4 (ship).

### Task 1: Probe routing 1-commit repo — viết TRƯỚC fix, chứng minh bug

**File:** tests — Worker chọn: file mới `tests/schema_fallback.rs` HOẶC extend file probe schema hiện có (Tầng 2; reuse pattern `build_fixture_repo` per P004).

1. Probe (a) — **routing delta**: temp git repo **1 commit duy nhất** chứa `schema.prisma` hợp lệ (nội dung additive bình thường) → chạy `check schema` → assert **Branch D** wording + exit 0 (post-fix kỳ vọng). Điều kiện anchor #7; nếu wording C=D → assert seam `get_diff()` non-empty thay thế.
2. Chạy probe (a) trên code CHƯA fix → PHẢI ĐỎ (hiện ra Branch C) → chép output đỏ vào Discovery (bằng chứng bug thật). Nếu probe (a) XANH trước fix → anchor #3/#7 sai → DỪNG, Debate Log.

### Task 2: Fix SHA + probes còn lại

**File:** `src/checks/schema.rs`

- **Tìm:** literal `4b825dc8669f8c0` trong `get_diff()` Step 2 (anchor #2 — đúng 1 chỗ).
- **Thay bằng:** `4b825dc642cb6eb9a060e54bf8d69288fbee4904` + comment 1-2 dòng: empty-tree SHA-1 constant; oracle `git hash-object -t tree /dev/null`; deviates from `golden/check-schema-safety.sh:33` (truncated 15-char = golden bug, P010 — see CLAUDE.md §deviations); sha256-repos out of scope.
- **KHÔNG đổi:** shape fallback chain (Step 1 → Step 2 → empty), wording mọi branch, pattern `DESTRUCTIVE_RE`/`HEADER_SKIP_RE`, bypass `ALLOW_DATA_LOSS`.

**Probes thêm (cùng file test Task 1):**

| Probe | Setup | Assert |
|---|---|---|
| (a) | 1-commit repo, schema sạch | Branch D + exit 0 (Task 1 — giờ XANH) |
| (b) | not-a-repo (temp dir không init git) | Branch C/F nguyên trạng — fix KHÔNG đổi nhánh này |
| (c) | 1-commit repo — empirical confirm anchor #8 | diff so empty-tree toàn `+`, không finding, exit 0; ghi Discovery "exit-1 unreachable by construction, brief test spec điều chỉnh có lý do" |
| (d) | 2-commit repo destructive (reuse fixture pattern) | exit 1 + finding nguyên — Step 1 path không bị fix đụng (parity suite cũng cover, probe này rẻ nên giữ) |
| (e) | **oracle guard:** chạy `git hash-object -t tree /dev/null` trong test | output == hằng số trong schema.rs (chống typo hằng số trên mọi máy — đây là lý do hardcode an toàn) |

Probe d CŨ của P004 (anchor #6): nếu vỡ vì routing C→D → update assertion 1-commit sang Branch D, GIỮ not-a-repo assert C. Mọi test-update kiểu này liệt kê trong Discovery (expected, không phải regression).

### Task 3: Docs deviation (Tầng 1 — precedent đầu tiên)

1. **`CHANGELOG.md`:** section `## [Unreleased] — P010 — fix empty-tree SHA fallback (check schema)` trên cùng (trên/dưới P009 tùy thứ tự merge — rebase trivial): SHA cụt golden:33 → SHA chuẩn 40-char; hệ quả = 1-commit repo route Branch C→D (outcome vẫn safe — exit-1 unreachable so empty-tree, có probe chứng minh); not-a-repo + ≥2-commit KHÔNG đổi; pins byte-identical; deviation có chủ đích ĐẦU TIÊN khỏi golden behavior (golden bug, không port bug).
2. **`CLAUDE.md`:** thêm dưới §Method (sau rule 4) một note ngắn — đề xuất text (Worker chỉnh wording nhẹ OK, ý GIỮ):
   ```
   - **Deviations from golden (intentional):** P010 — `check schema` fallback dùng empty-tree
     SHA chuẩn 40-char thay SHA cụt 15-char của `golden:33` (golden bug — O1.2,
     docs/discoveries/P004.md). golden/ giữ nguyên làm reference, KHÔNG sửa.
   ```
3. **`tests/golden/MANIFEST.md` §4:** update note O1.2 (anchor #9): "fixed in Rust P010; golden retains the bug; fallback path không nằm trong pinned fixture path — pins unchanged". KHÔNG đụng file pin .txt nào.
4. **`docs/ARCHITECTURE.md`:** chỉ nếu anchor #9 thấy cite SHA/O1.2 → sync 1 dòng.

### Task 4: Ship (IG-09 — thứ tự CỨNG)

1. `cargo test` full xanh (baseline + probes mới; pins byte-identical — anchor #10) + docs-gate `check_all` → commit trên `feat/P010-empty-tree-sha` (hook chạy thật, KHÔNG `--no-verify`).
2. **GIÁM SÁT REVIEW DIFF — BẮT BUỘC TRƯỚC MERGE** (security-gate surface): trọng tâm = diff schema.rs đúng 1 hằng + comment (không lệnh git mới, không đổi pattern/wording/bypass), `golden/` + pins untouched, probe d update có lý do, CLAUDE.md note đúng scope. Verdict ghi Discovery.
3. Merge `main` → push → XÓA branch (local + remote). Không stack.

### Task 5: Discovery report

**File:** `docs/discoveries/P010.md` + 1-line index `docs/DISCOVERIES.md` — gồm: (i) anchors verdict (đặc biệt #6 probe-d-update, #7 wording, #8 unreachability — kèm output probe (c)); (ii) transcript red→green probe (a) (bằng chứng bug thật); (iii) oracle output `git hash-object -t tree /dev/null`; (iv) Giám sát verdict; (v) ghi rõ deviation #1 precedent: quy trình đã theo (parity trước → behavior phiếu riêng → CLAUDE.md note → golden untouched) để phiếu deviation sau noi theo; (vi) tier escalations ("None" nếu không); (vii) note brief-spec điều chỉnh: BACKLOG specced test "destructive → exit 1" cho 1-commit — không khả thi by construction, thay bằng probe routing + probe empirical (c), Chủ nhà thấy ở approval gate.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/checks/schema.rs` | Task 2: 1 hằng SHA + comment — KHÔNG gì khác |
| tests (file probe — Worker chọn tên) | Task 1+2: probes (a)-(e); update probe d cũ nếu anchor #6 buộc |
| `CHANGELOG.md` | Task 3.1: `## [Unreleased] — P010` |
| `CLAUDE.md` | Task 3.2: deviation note (authorized — rule deviation có sẵn trong file) |
| `tests/golden/MANIFEST.md` | Task 3.3: note O1.2 §4 (text note ONLY) |
| `docs/ARCHITECTURE.md` | CHỈ nếu anchor #9 dương |
| `docs/discoveries/P010.md` + `docs/DISCOVERIES.md` | Task 5: report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY TUYỆT ĐỐI — never fix golden (method rule 4); golden GIỮ bug làm reference |
| `tests/golden/*.txt` (pins) | Byte-identical sau fix — `git diff tests/golden/` chỉ được chạm MANIFEST.md |
| `src/gate.rs`, `src/serve.rs`, `src/checks/` khác | Diff RỖNG — hưởng fix qua `run_core` schema, không sửa |
| `.github/**`, `hooks/**`, `scripts/**`, `Cargo.toml`, `Cargo.lock` | Diff RỖNG — không dep mới, không version bump |

---

## Luật chơi (Constraints)

1. **Diff code = 1 hằng số + comment.** Mọi thèm muốn "tiện tay" sửa wording/pattern/chain shape → Debate Log, không tự quyết (security-gate surface).
2. **Parity pins là bất khả xâm phạm:** full parity suite phải xanh KHÔNG sửa pin nào. Anchor #5 fail (fixture hóa ra 1 commit, fallback nằm trong pinned path) → DỪNG NGAY, Debate Log — phiếu sai tiền đề, không được repin để ép xanh.
3. **golden/ không bao giờ sửa** — kể cả thêm comment. Deviation sống ở Rust + docs.
4. **Red→green bắt buộc:** probe (a) phải được chứng kiến ĐỎ trên code chưa fix (transcript Discovery). Không có red = không có bằng chứng bug = giá trị phiếu giảm một nửa.
5. **Hằng số phải có oracle guard trong test** (probe e) — hardcode được phép VÌ có guard này.
6. **Exit-code contract API (0/1/2) không đổi** — fix chỉ đổi routing nội bộ C→D cho 1-commit repo, mọi exit code giữ semantics.
7. **Giám sát review BẮT BUỘC trước merge** (security-gate surface), verdict vào Discovery.
8. **Escape hatch:** anchor #2 (0 hoặc >1 hit), #3 (chain shape lệch), #5 (fixture 1 commit), #8 (DESTRUCTIVE_RE match dòng không-`-`) — bất kỳ cái nào fail → DỪNG, Debate Log với evidence grep, KHÔNG improvise.
9. F13 KHÔNG fire: không version bump (entry nằm `[Unreleased]`; release sau gom).

---

## Nghiệm thu

### Automated
- [ ] `cargo test` full xanh: baseline + probes (a)-(e) mới; số test mới ghi Discovery
- [ ] Probe (a) có transcript RED trên code cũ → GREEN sau fix
- [ ] `git diff tests/golden/` chỉ chạm `MANIFEST.md` (pins byte-identical); `git diff golden/` RỖNG TUYỆT ĐỐI
- [ ] docs-gate `check_all` pass (CHANGELOG entry P010)

### Manual Testing (acceptance Chủ nhà)
- [ ] Temp repo 1 commit + schema.prisma: `cargo run -- check schema` → Branch D wording + exit 0 (trước fix: Branch C)
- [ ] `git hash-object -t tree /dev/null` trên máy = hằng số trong schema.rs (oracle, probe e tự động hóa)
- [ ] Repo này (≥2 commit): `cargo run -- check schema` output BYTE-IDENTICAL trước/sau fix (Step 1 path không đụng)

### Regression
- [ ] Full parity suite xanh, 0 pin sửa
- [ ] `cargo run -- gate --all` + `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree (dogfood nguyên)
- [ ] `ALLOW_DATA_LOSS=true` bypass + schema-missing Branch B nguyên (probe P004 cũ vẫn xanh)

### Docs Gate
- [ ] `CHANGELOG.md` — `[Unreleased] — P010` đủ ý (deviation #1, routing C→D, pins nguyên)
- [ ] `CLAUDE.md` — deviation note đúng text-ý Task 3.2
- [ ] `tests/golden/MANIFEST.md` §4 — note O1.2 updated

### Discovery Report
- [ ] `docs/discoveries/P010.md`: anchors verdict, red→green transcript, oracle output, probe (c) unreachability evidence + note brief-spec điều chỉnh, Giám sát verdict, precedent deviation #1, tier escalations
- [ ] Append 1-line index `docs/DISCOVERIES.md`
