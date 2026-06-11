# PHIẾU P009: CI actions bump — Node.js 20 deprecation (DEADLINE 16/06/2026)

> **Loại:** Chore / CI maintenance (workflow only — KHÔNG đổi code Rust)
> **Ưu tiên:** P0-deadline (GitHub forced migration 16/06/2026 — sau ngày đó release CI có thể FAIL; hôm nay 11/06 → còn 5 ngày)
> **Tầng:** 1 — `.github/workflows/release.yml` = **supply-chain surface** (build + publish binary người khác cài; CLAUDE.md rule: Giám sát review diff BẮT BUỘC trước merge). Diff là version bump action = chính xác loại thay đổi supply-chain cần soi nhất.
> **Ảnh hưởng:** `.github/workflows/release.yml`, `CHANGELOG.md`, `docs/discoveries/P009.md`, `docs/DISCOVERIES.md`. `docs/ARCHITECTURE.md` KHÔNG đụng (anchor #9 âm `[verified — Worker Turn 1]`). Ngoài tree: tag thử `v0.1.1-rc1` + release rc TẠM (xóa sau verify).
> **Dependency:** P008 merged (main `aa262e4`, tag `v0.1.0` live) `[verified — docs/discoveries/P008.md]`. Độc lập với P010 — branch `feat/P009-ci-actions-bump` từ main mới nhất. Khuyến nghị làm TRƯỚC P010 (deadline).

> *Authorization tag-thử push: rc tag + rc release = action ngoài working tree nhưng nằm trong BACKLOG Active sprint item P009 ("Verify bằng tag thử `v0.1.1-rc1` ... → xóa tag rc") Chủ nhà pick 11/06 → **authorized-by-sprint-goal**. Điều kiện cứng: rc tag/release PHẢI XÓA sau verify (Task 4.5) — không để rc tồn tại qua đêm.*

---

## Context

### Vấn đề hiện tại

CI run 27356594756 (release v0.1.0, 11/06) kèm annotation Node.js 20 deprecation. Nguyên văn `[verified — Worker Turn 1, run log]`:

> `Node.js 20 actions are deprecated. The following actions are running on Node.js 20 and may not work as expected: actions/checkout@v4, softprops/action-gh-release@v2. Actions will be forced to run with Node.js 24 by default starting June 16th, 2026. Node.js 20 will be removed from the runner on September 16th, 2026.`

**Forced** migration sang **Node.js 24** ngày **16/06/2026** — không phải soft-warn. Sau 16/06, runner có thể từ chối chạy action Node-20 → release CI đỏ → không ship được version mới.

Workflow hiện tại `[verified — Architect Read + Worker Turn 1]` dùng đúng 3 action — bảng version **CHỐT V2** (mọi runtime oracle-verified Turn 1 qua `gh api action.yml`):

| Dòng | Action | Pin hiện tại | Runtime | Pin đích (CHỐT) |
|---|---|---|---|---|
| 28 | `actions/checkout` | `@v4` | node20 `[verified — oracle Turn 1]` | **`@v5`** — action.yml ref=v5: `using: node24` (smallest major tag có node24; v6.0.3 cũng node24 nhưng KHÔNG cần — O1.1) |
| 29 | `dtolnay/rust-toolchain` | `@stable` | `using: composite` — không Node, KHÔNG trong annotation `[verified — Turn 1]` | **KHÔNG đổi** |
| 41 | `softprops/action-gh-release` | `@v2` | node20, tag KHÔNG trôi `[verified — oracle Turn 1]` | **`@v3`** — action.yml ref=v3: `using: "node24"`; release notes v3.0.0: "only runtime change, no input renames" |

`.github/workflows/` chỉ có 1 file `release.yml` `[verified — Architect Glob]` → không workflow nào khác cần sửa.

**Lưu ý nguồn (đã chốt):** brief Quản đốc ghi "Node-22-compatible" — SAI. Ground truth = annotation log run 27356594756 `[verified — Worker Turn 1]`: target = **Node.js 24**, forced 16/06/2026.

### Giải pháp

1. **Inventory từ log — ĐÃ XONG Turn 1:** annotation nguyên văn đã chép vào Debate Log (trên đây). Task 1 còn lại = chép vào Discovery.
2. **Version đích — ĐÃ CHỐT Turn 1 (bảng trên):** checkout `@v4`→`@v5`, gh-release `@v2`→`@v3`, rust-toolchain nguyên. Không cần re-run oracle trừ khi EXECUTE cách Turn 1 quá vài ngày.
3. **Bump tối thiểu (Task 3):** đổi version suffix 2 dòng `uses:` + **1 dòng O1.2 resolution**: thêm `prerelease: ${{ contains(github.ref_name, '-rc') }}` vào `with:` block của gh-release step (Quản đốc quyết — xem Debate Log Turn 1 Response). Giữ pin-style major tag `@vX` — KHÔNG chuyển sang SHA-pin (đổi pin-style = quyết định supply-chain riêng, ngoài scope). KHÔNG đụng action không bị warning.
4. **Verify bằng rc tag (Task 4) — Architect đã quyết, chốt phương án A:**
   - **A (CHỌN): tag thử `v0.1.1-rc1`** — match trigger `tags: ["v*"]`, chạy ĐÚNG full path thật (build 3 target → tạo release → upload asset), zero thay đổi workflow ngoài version bump + prerelease conditional.
   - B (LOẠI): thêm `workflow_dispatch` — bị loại vì: (i) thêm trigger surface VĨNH VIỄN vào workflow supply-chain (minimalism — CLAUDE.md); (ii) `action-gh-release` cần tag context, dispatch-run không có tag → step fail hoặc phải thêm conditional → diff phình to hơn; (iii) không exercise path thật tag→release.
   - **Latest pointer được bảo vệ bởi prerelease conditional (O1.2):** rc tag chứa `-rc` → `prerelease: true` → GitHub KHÔNG cho prerelease làm Latest → `v0.1.0` giữ pointer tuyệt đối trong khung test.
   - Sau CI xanh + 0 deprecation warning → **XÓA release rc** (`gh release delete v0.1.1-rc1 --yes`) + **XÓA tag rc** (local + remote). Chưa có consumer: sos-kit join còn nằm Open backlog `[verified — BACKLOG.md KIT-HARVEST]`; release `v0.1.0` thật KHÔNG đụng.
5. **Ship (Task 5, IG-09):** commit branch → **Giám sát review diff (BẮT BUỘC — workflow diff = supply-chain, nay gồm cả dòng prerelease conditional)** → merge main → push → xóa branch. Chốt thứ tự: **Giám sát → merge → tag rc từ main → verify → xóa rc**. Nếu CI đỏ phải sửa → sửa trên main bằng commit mới (diff nhỏ, qua hook), re-tag rc2 (cap 2 lần).

### Scope

- CHỈ sửa: `.github/workflows/release.yml` (2 dòng version suffix + 1 dòng `prerelease:` conditional — tổng 3 dòng), `CHANGELOG.md` (section `## [Unreleased] — P009`), `docs/discoveries/P009.md`, `docs/DISCOVERIES.md`.
- KHÔNG sửa: `docs/ARCHITECTURE.md` (anchor #9 âm — đã verify), `src/**`, `tests/**`, `golden/**`, `hooks/**`, `scripts/**`, `Cargo.toml`/`Cargo.lock` (KHÔNG bump version — 0.1.1 thật chưa release, F13 chỉ fire lúc cắt release heading), matrix targets / runner labels / trigger / permissions / EXT logic / asset naming (tất cả là contract P008 — nguyên trạng).
- KHÔNG implement: SHA-pinning actions (Park nếu thấy cần — ghi Discovery), `workflow_dispatch` trigger (loại có lý do ở trên), bump runner image (`ubuntu-22.04`→`24.04` v.v. — ngoài warning, ngoài scope), release `v0.1.1` THẬT.

---

## Task 0 — Verification Anchors

> **Bước 0 capability (P285 Layer 1):** phiếu cần **network qua `gh` CLI** (api read + tag push + release delete). Worker có Bash+gh; P008 đã verify ADMIN + scopes `repo`+`workflow` nhưng trạng thái có thể đổi → Worker re-verify đầu phiếu: `gh auth status` + `gh api repos/aspelldenny/inv-gate --jq .permissions`. KHÔNG cần WebFetch (GET-only không đủ cho việc này — mọi thứ đi qua `gh`).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Baseline: main = `aa262e4` (hoặc mới hơn nếu P010 merge trước), `cargo test` 95 xanh, tree clean | `git log -1` + `git status` + `cargo test` | ✅ **VERIFIED (partial)** — HEAD = `b93f295` (post-P008 doc commit, newer than aa262e4 ✅); tree clean except untracked ticket files. `cargo test` count deferred to EXECUTE (not run in CHALLENGE mode). |
| 2 | `release.yml` đúng 3 action: `checkout@v4` (dòng 28), `dtolnay/rust-toolchain@stable` (dòng 29), `softprops/action-gh-release@v2` (dòng 41); không workflow file nào khác | đọc file + `ls .github/workflows/` | ✅ **VERIFIED (Architect Read + Glob 11/06)** — Worker re-confirm 1 lệnh |
| 3 | Annotation run 27356594756 nêu tên `checkout@v4` + `action-gh-release@v2`, migration target Node **24**, deadline 16/06/2026 | `gh run view 27356594756` (annotations) và/hoặc `--log` grep -i 'node' | ✅ **VERIFIED (Turn 1)** — annotation exact text trong Debate Log + Context; "forced to run with Node.js 24 by default starting June 16th, 2026"; brief "Node 22" sai |
| 4 | `actions/checkout` có version mới hơn declare `using: node24` (kỳ vọng v5) | `gh api repos/actions/checkout/releases/latest --jq .tag_name` + `gh api 'repos/actions/checkout/contents/action.yml?ref=<tag>' --jq .content \| base64 -d \| grep using` `[oracle: gh api action.yml, SOUND cho claim runtime]` | ✅ **VERIFIED (Turn 1)** — v4 = node20; v5 major tag = node24; v6.0.3 = node24. **Pin đích CHỐT = `@v5`** (smallest-major với node24 — O1.1 self-closed) |
| 5 | `softprops/action-gh-release`: tag trôi `@v2` có thể ĐÃ node24, hoặc cần version mới | `gh api 'repos/softprops/action-gh-release/contents/action.yml?ref=v2' --jq .content \| base64 -d \| grep using`; nếu node20 → check `releases/latest` + action.yml của tag đó | ✅ **VERIFIED (Turn 1)** — v2 = node20 (NOT drifted — cần bump); latest = v3.0.0; v3 major tag = `"node24"`. **Pin đích CHỐT = `@v3`.** Release notes v3.0.0: only runtime change, no input renames. |
| 6 | `dtolnay/rust-toolchain@stable` = composite action (không Node) → không bị warning → KHÔNG đụng | annotation list (anchor #3) + `gh api 'repos/dtolnay/rust-toolchain/contents/action.yml?ref=stable' ... grep using` → kỳ vọng `composite` | ✅ **VERIFIED (Turn 1)** — using: composite; NOT in annotation; no action needed |
| 7 | Chưa có consumer release nào ngoài tay mình (sos-kit BINARIES join chưa làm) → rc release tồn tại vài phút rồi xóa = an toàn | BACKLOG.md KIT-HARVEST còn ở Open backlog | ✅ **VERIFIED (Architect Read BACKLOG 11/06)** |
| 8 | gh capable: auth OK, quyền push tag + delete release | `gh auth status` + permissions check (Bước 0) | ✅ **VERIFIED (Turn 1)** — auth OK (aspelldenny), scopes: repo+workflow+gist+read:org, permissions admin=true+push=true |
| 9 | `docs/ARCHITECTURE.md` §Distribution có cite action version cụ thể không | `grep -n 'checkout\|gh-release' docs/ARCHITECTURE.md` | ✅ **VERIFIED (Turn 1)** — no output (no citations); file NOT touched |
| 10 | Sau bump: `git diff` CHỈ chạm files trong scope; workflow diff = đúng 3 dòng (2 bump + 1 prerelease); `cargo test` vẫn xanh nguyên (zero code change) | `git diff --stat` + `git diff .github/` + `cargo test` trước commit | ⏳ TO VERIFY |
| 11 | Run rc: 3 job XANH + **0 annotation Node deprecation** + release rc có đúng 3 asset đúng tên (contract P008 nguyên) | `gh run watch <id>` + `gh run view <id>` (annotations rỗng phần Node) + `gh release view v0.1.1-rc1 --json assets` | ⏳ TO VERIFY — acceptance chính |
| 12 | Sau cleanup: `gh release list` chỉ còn `v0.1.0`; `git ls-remote --tags origin` không còn `v0.1.1-rc1` | 2 lệnh đó | ⏳ TO VERIFY |
| 13 | O1.2 resolution hoạt động: rc release mang cờ Pre-release (conditional fire vì tag chứa `-rc`) + latest pointer GIỮ NGUYÊN `v0.1.0` trong suốt khung test | `gh release view v0.1.1-rc1 --json isPrerelease` → `true`; `gh release view --json tagName` (latest, không args tag) → `v0.1.0` | ⏳ TO VERIFY — acceptance O1.2 |

**Anchor mở (Worker verify ở EXECUTE): #1 (cargo test count), #10, #11, #12, #13. ✅ Verified Turn 1: #2–#9. 0 anchor ❌.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P009
mkdir -p ".backup/${PHIEU_ID}"
cp .github/workflows/release.yml ".backup/${PHIEU_ID}/release.yml.orig"
cp CHANGELOG.md ".backup/${PHIEU_ID}/CHANGELOG.md.orig"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Turn 1 resolved — versions chốt @v5/@v3, +1 dòng prerelease conditional)

### Turn 1 — Worker Challenge (vs V1)

**Anchor verification:**
- #1 HEAD = `b93f295` (post-P008 doc commit, clean tree modulo untracked ticket files) ✅
- #2 release.yml lines 28/29/41 confirmed ✅
- #3 Annotation exact text confirmed from run log ✅ — see below
- #4 actions/checkout: v4 major tag = node20; latest = v6.0.3 = node24; v5 major tag exists = node24 ✅
- #5 softprops/action-gh-release: v2 major tag = node20 (STILL — NOT drifted); latest = v3.0.0 = node24; v3 major tag = node24 ✅ — needs bump
- #6 dtolnay/rust-toolchain@stable = composite — NOT in annotation, no action needed ✅
- #7 KIT-HARVEST still Open ✅
- #8 gh auth OK, permissions admin+push+workflow scope ✅
- #9 `docs/ARCHITECTURE.md` — no citation of checkout/gh-release versions (no output from grep) ✅ → file NOT touched
- #10/#11/#12 — TO VERIFY in EXECUTE (post-bump)

**Annotation nguyên văn (run 27356594756, tất cả 3 job):**
> `##[warning]Node.js 20 actions are deprecated. The following actions are running on Node.js 20 and may not work as expected: actions/checkout@v4, softprops/action-gh-release@v2. Actions will be forced to run with Node.js 24 by default starting June 16th, 2026. Node.js 20 will be removed from the runner on September 16th, 2026.`
Deadline = **June 16, 2026** (forced; không phải soft-warn). Migration target = **Node.js 24** (brief "Node 22" sai — log wins).

**Objections:**

- [O1.1] **Phiếu Task 3 nói pin đích = major tag (`@v5`) cho checkout.** Thực tế: v5 major tag = node24 (xác nhận) nhưng latest release = v6.0.3 (cũng node24). Phiếu nói "version NHỎ NHẤT có using: node24" — đó là v5, không phải v6. Không có breaking-change risk thêm từ đây vì cả hai đều node24. Claim Worker muốn self-close: "v5 là smallest-major-tag có node24".
  Claim: v5 major tag tồn tại và có using: node24
  Oracle: `gh api 'repos/actions/checkout/contents/action.yml?ref=v5' --jq .content | base64 -d | grep using:` → SOUND
  Soundness: SOUND
  Verdict: self-closed via oracle — v5 xác nhận node24. Pin đích = `@v5`.

- [O1.2] **CRITICAL — rc release "latest" pointer risk.** Phiếu Task 5 step 4 tạo `v0.1.1-rc1` tag → action-gh-release v3 sẽ tạo GitHub Release. Input `prerelease:` KHÔNG được set trong release.yml (xem dòng 41-43). action-gh-release v3 default `prerelease: false` (action.yml: "Defaults to false") → release sẽ tạo như *non-prerelease*. GitHub API `make_latest` không được set → GitHub default behavior: xét semver, nếu `v0.1.1-rc1 > v0.1.0` (theo GitHub semver legacy comparison) → **"Latest" pointer nhảy từ v0.1.0 sang v0.1.1-rc1**. Rủi ro: `install.sh` người dùng gọi `releases/latest` trong vài phút trước cleanup → tải binary rc (chứa version string "0.1.0" bên trong — F13). Phiếu ghi "an toàn vì chưa có consumer" nhưng **KHÔNG spec `prerelease: true`** trong workflow bump → lỗ hổng thật.
  Claim: "rc release sẽ không steal latest pointer" → FALSE nếu không add `prerelease: true`
  Oracle: action-gh-release v3 action.yml `prerelease: description: "Defaults to false"` — SOUND cho claim "input chưa set = false = non-prerelease"
  Soundness: SOUND (input default confirmed; GitHub API behavior confirmed — "prereleases cannot be set as latest" chỉ áp dụng khi prerelease=true)
  Verdict: OBJECTION STANDS — phiếu cần spec cách né; self-close không đủ (quyết định về workflow diff = supply-chain Tầng 1)

**Proposed alternatives cho O1.2:**
- **A. Thêm `prerelease: true` vào `with:` block của step softprops/action-gh-release trong release.yml (Recommended).** Diff = 1 dòng thêm vào `with:`, chỉ fire khi tag chứa rc/alpha/beta → thực tế: tag `v0.1.1` thật cần thêm bước bỏ flag sau. NHƯNG: release.yml hiện tại KHÔNG có `with:` block cho softprops step — chỉ có `files:`. Thêm `prerelease: true` vĩnh viễn sẽ đánh dấu mọi release là prerelease (kể cả v0.1.1 thật) → SAI. Cần conditional hoặc không dùng cách này.
- **B. Thêm `make_latest: false` vào `with:` block (Recommended — safer).** `make_latest: false` = release được tạo nhưng KHÔNG steal "Latest" pointer. Áp dụng mọi release (rc lẫn thật) — nhưng release thật cũng bị. Cần `make_latest: true` (explicit) cho release thật → lại phức tạp.
- **C. Dùng `draft: true` cho toàn bộ rc run — LOẠI** (draft release không upload asset đúng cách với immutable constraint).
- **D. Chủ nhà approve: CHẤP NHẬN RỦI RO vì window cleanup rất ngắn (<5 phút), consumer chưa có (anchor #7 ✅), và brief đã note "an toàn vì chưa có consumer".** Nếu Chủ nhà approve D: ghi Discovery + CHANGELOG note rõ, không sửa workflow thêm. Worker lean = **D** (window ngắn, anchor #7 confirmed, thêm `make_latest`/`prerelease` vào workflow = diff supply-chain ngoài scope ban đầu; Architect đã quyết "diff = version suffix ONLY").

**Status:** ✅ RESPONDED (see Turn 1 — Architect Response)

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT** (Worker self-closed via oracle, hợp lệ) — pin đích checkout CHỐT `@v5` (smallest major tag có node24), KHÔNG nhảy v6. Bảng version Context + anchor #4 cập nhật. Gh-release CHỐT `@v3` (anchor #5). Folded vào Task 2/3.
- [O1.2] → **ACCEPT (modified) — Quản đốc quyết theo ủy quyền, ghi nhận tại đây.** KHÔNG chọn option D (accept risk) của Worker — chọn **conditional prerelease**: thêm vào `with:` block của gh-release step:
  ```yaml
  prerelease: ${{ contains(github.ref_name, '-rc') }}
  ```
  Lý do: (i) 1 dòng, permanent fix cho MỌI rc tag tương lai (không chỉ lần test này); (ii) release thật (`v0.1.1`, `v0.2.0`...) không chứa `-rc` → prerelease=false như cũ — zero behavior change cho path thật; (iii) GitHub Actions expression trong `with:` là cơ chế chuẩn — Worker option A nhầm "không conditional được, cần bỏ flag sau" — expression CHÍNH LÀ conditional expressible; (iv) latest pointer được bảo vệ TUYỆT ĐỐI trong khung test (prerelease không bao giờ thành Latest). Supply-chain diff +1 dòng → Giám sát soi nguyên văn (Task 5.2). Cập nhật: Task 3 (thêm bước 3b), Constraints #3, Nghiệm thu + anchor #13 mới (isPrerelease=true + latest giữ v0.1.0).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Sẵn sàng APPROVAL_GATE / EXECUTE (mọi objection đóng, không DEFER).

### Final consensus
- Phiếu version: V2
- Total turns: 1
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

> Thứ tự: Task 1 (chép annotation) → Task 2 (version đích — ĐÃ CHỐT) → Task 3 (bump + prerelease) → Task 4 docs → Task 5 (ship: Giám sát → merge → rc tag verify → cleanup) → Task 6 (Discovery).

### Task 1: Annotation run 27356594756 — ĐÃ ĐỌC Turn 1

Annotation nguyên văn đã verify + chép vào Debate Log Turn 1 (và Context). Việc còn lại: chép nguyên văn vào `docs/discoveries/P009.md` (Task 6), cite run id 27356594756. KHÔNG cần re-run `gh run view` trừ khi muốn double-check.

### Task 2: Version đích — ĐÃ CHỐT V2 (oracle-verified Turn 1, KHÔNG cần re-run)

| Action | Pin cũ | Pin mới | Evidence (Turn 1) |
|---|---|---|---|
| `actions/checkout` | `@v4` (node20) | **`@v5`** | action.yml ref=v5: `using: node24` |
| `softprops/action-gh-release` | `@v2` (node20, tag KHÔNG trôi) | **`@v3`** | action.yml ref=v3: `using: "node24"`; release notes v3.0.0: only runtime change, no input renames |
| `dtolnay/rust-toolchain` | `@stable` | **KHÔNG đổi** | `using: composite`, không trong annotation |

Chép evidence (output grep `using:`) vào Discovery. Re-run oracle CHỈ nếu EXECUTE cách Turn 1 nhiều ngày (hôm nay cùng ngày → skip).

### Task 3: Bump version + prerelease conditional trong `.github/workflows/release.yml`

**File:** `.github/workflows/release.yml`

**3a — Version bump (2 dòng):**
- **Tìm:** dòng `uses: actions/checkout@v4` (dòng 28 `[verified]`) và `uses: softprops/action-gh-release@v2` (dòng 41 `[verified]`).
- **Thay bằng:** cùng dòng, `@v4`→`@v5` và `@v2`→`@v3` (CHỐT Task 2).

**3b — Prerelease conditional (1 dòng — O1.2 resolution, Quản đốc quyết):**
- **Tìm:** `with:` block của step `softprops/action-gh-release` (hiện chỉ có `files:` `[verified — Worker Turn 1]`).
- **Thêm:** 1 dòng vào `with:` block:
  ```yaml
  prerelease: ${{ contains(github.ref_name, '-rc') }}
  ```
- **Lưu ý:** indent khớp `files:` cùng block. Release thật không chứa `-rc` → evaluate false → behavior y như cũ. Rc tag → true → GitHub không bao giờ trỏ Latest vào prerelease.

**Lưu ý chung:**
1. Diff = ĐÚNG 3 dòng (2 version suffix + 1 prerelease). KHÔNG thêm input/step/trigger/secret nào khác. KHÔNG đổi matrix, EXT logic, asset naming (contract P008 — Tầng 1).
2. Header comment file (dòng 1-4): không cite version → không đụng `[verified]`.
3. Breaking-change check ĐÃ XONG Turn 1: gh-release v3 release notes = "only runtime change, no input renames"; checkout v5 = node24 runtime bump. Nếu EXECUTE phát hiện diff cần vượt 3 dòng → ghi Debate Log trước khi sửa, không tự quyết (supply-chain).

### Task 4: Docs — `CHANGELOG.md`

**File:** `CHANGELOG.md`

Thêm section MỚI TRÊN CÙNG (trên `## [0.1.0]`): `## [Unreleased] — P009 — CI actions bump (Node 24)` — ghi: checkout `@v4`→`@v5`, gh-release `@v2`→`@v3` (GitHub Node 20 EOL forced 16/06/2026, annotation run 27356594756), + 1 dòng `prerelease:` conditional cho rc tag (O1.2 — latest-pointer guard), verify method (rc tag `v0.1.1-rc1`, CI xanh 0 warning, isPrerelease=true, latest giữ v0.1.0, đã xóa rc). KHÔNG bump `Cargo.toml` (chưa release 0.1.1 thật — F13 fire lúc cắt heading release). `docs/ARCHITECTURE.md` KHÔNG sửa (anchor #9 âm).

### Task 5: Ship + rc verify (IG-09 — thứ tự CỨNG)

1. `cargo test` xanh (anchor #10) + docs-gate `check_all` → commit trên `feat/P009-ci-actions-bump` (hook chạy thật, KHÔNG `--no-verify`).
2. **GIÁM SÁT REVIEW DIFF — BẮT BUỘC TRƯỚC MERGE** (orchestrator chạy; supply-chain): trọng tâm = ĐÚNG 3 dòng (28: `@v5`; 41: `@v3`; with-block: dòng `prerelease: ${{ contains(github.ref_name, '-rc') }}` nguyên văn), version đích khớp Task 2, không action/step/secret/input nào khác, contract asset+EXT nguyên. Verdict ghi Discovery.
3. Merge `main` → push `origin main` → XÓA branch (local + remote).
4. **Tag rc từ main đã merge:** `git tag v0.1.1-rc1 && git push origin v0.1.1-rc1` (lightweight đủ — rc sẽ xóa; annotated cũng OK, Tầng 2).
5. `gh run watch <id>` đến khi 3 job xanh → `gh run view <id>` xác nhận **0 annotation Node deprecation** (anchor #11) → `gh release view v0.1.1-rc1 --json assets` đúng 3 asset đúng tên → **anchor #13:** `gh release view v0.1.1-rc1 --json isPrerelease` → `true` VÀ `gh release view --json tagName` (latest) → `v0.1.0` (pointer không nhảy).
6. **Cleanup BẮT BUỘC (anchor #12):** `gh release delete v0.1.1-rc1 --yes` → `git push origin :refs/tags/v0.1.1-rc1` → `git tag -d v0.1.1-rc1`. Verify: `gh release list` chỉ còn `v0.1.0`; remote tags sạch.
7. **Retry protocol (CI đỏ hoặc warning còn):** sửa CHỈ workflow file bằng commit mới trên main (qua hook) → xóa rc hỏng (release + tag) → re-tag `v0.1.1-rc2`. **Cap = 2 lần re-tag (rc1, rc2 + 1 lần sửa giữa); lần 3 → DỪNG, escalate Chủ nhà kèm log CI.**

### Task 6: Discovery report

**File:** `docs/discoveries/P009.md` + 1-line index `docs/DISCOVERIES.md` — gồm: (i) annotation nguyên văn (Turn 1) + verdict node-target (node24, không phải node22 như brief); (ii) bảng action: pin cũ → pin mới → `using:` evidence (output grep); (iii) O1.2 resolution: prerelease conditional 1 dòng + verdict Quản đốc; (iv) rc run id + link + 3 job verdict + xác nhận 0 deprecation warning + isPrerelease=true + latest giữ v0.1.0; (v) cleanup evidence (release list + ls-remote sau xóa); (vi) Giám sát verdict; (vii) anchors ĐÚNG/SAI; (viii) tier escalations ("None" nếu không); (ix) hook sos-kit harvest: siblings (doctor/docs-gate/claude-hooks/doc-rotate) gần như chắc chắn CÙNG bị deprecation này — kit-level note, owner = orchestrator, NGOÀI scope phiếu.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.github/workflows/release.yml` | Task 3: ĐÚNG 3 dòng — dòng 28 `@v4`→`@v5`, dòng 41 `@v2`→`@v3`, +1 dòng `prerelease: ${{ contains(github.ref_name, '-rc') }}` vào `with:` block gh-release |
| `CHANGELOG.md` | Task 4: section `## [Unreleased] — P009` trên cùng |
| `docs/discoveries/P009.md` + `docs/DISCOVERIES.md` | Task 6: report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `docs/ARCHITECTURE.md` | Anchor #9 âm — KHÔNG đụng |
| `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock` | Diff RỖNG TUYỆT ĐỐI — zero code change; tests xanh nguyên |
| `golden/**`, `tests/golden/**` | READ-ONLY tuyệt đối |
| `hooks/**`, `scripts/**` | Dogfood nguyên trạng |
| Trigger / permissions / matrix / EXT / asset naming trong release.yml | Contract P008 — diff các phần này = vi phạm, Giám sát reject (ngoại lệ DUY NHẤT đã approve: 1 dòng prerelease conditional — O1.2) |

---

## Luật chơi (Constraints)

1. **Deadline thật:** 16/06/2026. Phiếu phải merge + verify TRƯỚC ngày đó. Block không gỡ được trong 1 phiên → escalate ngay, không ngâm.
2. **Zero code change:** `src/` + `tests/` diff rỗng; `cargo test` xanh nguyên trước commit.
3. **Supply-chain kỷ luật:** diff workflow = ĐÚNG 3 dòng (2 version suffix + 1 dòng `prerelease:` conditional — O1.2, Quản đốc approve). Mọi nhu cầu sửa thêm `with:`/input/step → Debate Log trước, không tự quyết. KHÔNG SHA-pinning trong phiếu này (ghi Park nếu muốn đề xuất).
4. **Verify = rc tag, KHÔNG đốt version thật:** `v0.1.1-rc1` → xanh + 0 warning + isPrerelease=true + latest giữ v0.1.0 → XÓA cả release lẫn tag. `workflow_dispatch` đã loại (lý do trong Context §Giải pháp 4). Release `v0.1.0` thật KHÔNG đụng dưới mọi hình thức.
5. **Cleanup là acceptance, không phải dọn dẹp tùy hứng:** phiếu CHƯA XONG chừng nào `gh release list` còn rc hoặc remote còn tag rc (anchor #12).
6. **Giám sát review BẮT BUỘC trước merge** (workflow diff = supply-chain, gồm cả dòng prerelease conditional; verdict ghi Discovery).
7. **Retry cap:** 2 lần re-tag rc; lần 3 → escalate kèm log.
8. **Escape hatch:** action bị warning mà KHÔNG có version node24 nào tồn tại (cả latest vẫn node20) → DỪNG, Discovery + escalate Chủ nhà với options (chờ upstream / đổi action thay thế = phiếu riêng) — KHÔNG tự đổi sang action khác (supply-chain). *(Đã loại trừ thực tế bởi Turn 1 — cả 2 action có node24 — giữ làm guard nếu EXECUTE thấy khác.)*
9. F13 KHÔNG fire: Cargo.toml giữ `0.1.0` (rc tag không phải release thật; binary rc mang version 0.1.0 bên trong = chấp nhận, rc bị xóa).

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh nguyên (count = baseline anchor #1; zero code change)
- [ ] `git diff src/ tests/ golden/ hooks/ scripts/ Cargo.toml Cargo.lock` RỖNG
- [ ] `git diff .github/workflows/release.yml` = ĐÚNG 3 dòng (anchor #10)
- [ ] docs-gate `check_all` pass (CHANGELOG có entry P009)

### Manual Testing (acceptance Chủ nhà)
- [ ] Run rc `v0.1.1-rc1`: 3 job XANH + **0 annotation Node deprecation** trong `gh run view` (so với run 27356594756 có warning)
- [ ] Release rc có đúng 3 asset đúng tên contract (`inv-gate-<target>[.exe]`) — chứng minh bump không vỡ asset path
- [ ] **Release rc có cờ Pre-release:** `gh release view v0.1.1-rc1 --json isPrerelease` → `true` (anchor #13 — O1.2 conditional fire)
- [ ] **Latest pointer KHÔNG nhảy:** `gh release view --json tagName` (latest) → `v0.1.0` trong suốt khung test (anchor #13)
- [ ] Cleanup verified: `gh release list` chỉ `v0.1.0`; `git ls-remote --tags origin` không còn rc

### Regression
- [ ] Workflow phần KHÔNG đụng (trigger/permissions/matrix/EXT/naming) diff rỗng so `.backup/P009/release.yml.orig`
- [ ] `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree (dogfood nguyên)

### Docs Gate
- [ ] `CHANGELOG.md` — `## [Unreleased] — P009` với bảng pin cũ→mới + dòng prerelease conditional + lý do + verify method

### Discovery Report
- [ ] `docs/discoveries/P009.md`: annotation nguyên văn, bảng version + `using:` evidence, O1.2 resolution, rc run link, isPrerelease + latest evidence, cleanup evidence, Giám sát verdict, anchors verdict, kit-level hook siblings cùng bệnh (owner=orchestrator), tier escalations
- [ ] Append 1-line index `docs/DISCOVERIES.md`
