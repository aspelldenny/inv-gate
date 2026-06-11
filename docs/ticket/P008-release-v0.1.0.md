# PHIẾU P008: Release v0.1.0 — version freeze (F13) + release CI 3-target + tag + verify assets (CHỐT DỰ ÁN)

> **Loại:** Release / Chore (CI + docs — KHÔNG đổi code Rust)
> **Ưu tiên:** P1 (item cuối Active sprint — Sprint 2 chốt dự án theo PROJECT.md)
> **Tầng:** 1 — (a) workflow CI = **supply-chain surface** (build + publish binary người khác cài — Giám sát review diff BẮT BUỘC trước merge); (b) tag `v0.1.0` push lên GitHub = hành động public, KHÔNG reversible sạch sau khi consumer (sos-kit install.sh) trỏ vào; (c) asset naming `<bin>-<target-triple>[.exe]` = cross-repo contract sos-kit install.sh depend; (d) version 0.1.0 = API surface (F13: Cargo.toml = CHANGELOG).
> **Ảnh hưởng:** `.github/workflows/release.yml` (V2: verify-vs-spec — diff có thể RỖNG), `CHANGELOG.md`, `docs/ARCHITECTURE.md` (nhẹ), `docs/discoveries/P008.md`, `docs/DISCOVERIES.md`. Ngoài repo: tag `v0.1.0` + GitHub Release trên `aspelldenny/inv-gate` `[verified — Cargo.toml repository]`.
> **Dependency:** P007 merged main `2531343`, 95 tests xanh `[unverified — Worker confirm anchor #1]`. Branch: `feat/P008-release-v0.1.0` từ `main`.

> *Authorization tag push: tag/release = action ngoài working tree, NHƯNG nằm trong sprint goal Chủ nhà đã chốt (BACKLOG Active sprint: "tag v0.1.0 → release CI 3-target xanh trên GitHub" = exit criterion Sprint 2) → **authorized-by-sprint-goal**, không cần hỏi lại — TRỪ khi retry protocol vượt cap (Luật chơi 8).*

---

## Context

### Vấn đề hiện tại

Dự án xong vision code-side (CLI 4 check + `gate --all --skip-absent` + `serve` MCP, 95 tests) nhưng chưa ship: chưa có release nào trên GitHub (`gh release list` trống `[verified — Worker CHALLENGE]`), CHANGELOG toàn `[Unreleased]` (7 sections P001-P007), chưa có tag. Sos-kit install.sh không có gì để tải.

Repo ĐÃ CÓ `.github/workflows/release.yml` từ bootstrap (commit f2946c4) `[verified — Architect đã Read file]` với matrix mac-arm64 / linux-x64 / win-x64.

**V2 (Turn 1 resolved):** Matrix bootstrap **ĐÚNG spec**. V1 từng coi matrix lệch BACKLOG (thiếu mac-x86_64, thừa win-x64) — đó là **lỗi diễn giải BACKLOG của Quản đốc**, Chủ nhà đã xác nhận trực tiếp 11/06: *"scope của sos kit là window, macos, linux"* → 3-target MIRROR SIBLINGS đúng convention `~/sos-kit/install.sh:49-52`. BACKLOG đã được sửa. Phiếu = verify matrix hiện tại vs spec + freeze version + tag + verify assets. KHÔNG swap target, KHÔNG viết workflow mới.

### Giải pháp

1. **Matrix spec chốt (Task 1, Chủ nhà 11/06):** `aarch64-apple-darwin` (macos-14) + `x86_64-unknown-linux-gnu` (ubuntu) + `x86_64-pc-windows-msvc` (windows, asset có `.exe`). KHÔNG `x86_64-apple-darwin`. Anchor #2 nói matrix hiện tại đã đúng bộ 3 này → Task 1 kỳ vọng = verify + asset-name check (diff có thể rỗng), KHÔNG swap. Asset naming contract: `inv-gate-<target-triple>` (+`.exe` cho win) — khớp `${bin}-${TARGET}${EXT}` của install.sh:65 → **logic EXT/`.exe` trong workflow PHẢI GIỮ** (V1 từng cho phép bỏ — thu hồi).
2. **Version freeze F13 (Task 2):** `Cargo.toml` ĐÃ là `0.1.0` `[verified]` → KHÔNG sửa Cargo.toml. F13 thỏa bằng phía CHANGELOG: gom 7 sections `[Unreleased]` (P001-P007) vào heading release `## [0.1.0] — 2026-06-11`, thêm entry P008. Sau task: version Cargo.toml = CHANGELOG latest = 0.1.0.
3. **Ship + tag (Task 4, IG-09):** commit branch → Giám sát review (supply-chain) → merge main → push → xóa branch → tag `v0.1.0` từ main đã merge → push tag → theo dõi CI `gh run watch` → verify release 3 assets đúng tên.
4. **CHẠY ĐƯỢC ≠ SHIP ĐƯỢC (slogan Chủ nhà, Task 5):** tải asset `inv-gate-aarch64-apple-darwin` TỪ GitHub Releases về máy này (arm64 darwin), chmod +x, chạy `--help` + `check runtime` + `gate --all --skip-absent` tại repo root — binary từ RELEASE phải hành xử đúng như binary local. Transcript vào Discovery.
5. **Cross-repo OUT:** join sos-kit `install.sh` BINARIES + pre-commit `[4/7]` swap = việc repo `~/sos-kit` (harvest cuối sprint) — phiếu này CHỈ ghi Discovery hook, không đụng. **Intel-Mac gap** (`x86_64-apple-darwin` không có mapping `Darwin-x86_64` trong install.sh) = kit-level feedback → orchestrator log **IG-10**, NGOÀI scope phiếu. Sprint close-out (BACKLOG Recently shipped, retro) = việc orchestrator, không phải Worker.

### Scope

- CHỈ sửa/tạo: `.github/workflows/release.yml` (chỉ nếu lệch spec — kỳ vọng diff rỗng), `CHANGELOG.md`, `docs/ARCHITECTURE.md` (1 mục §Distribution ngắn), `docs/discoveries/P008.md`, `docs/DISCOVERIES.md`. Ngoài tree: tag `v0.1.0` + GitHub Release (authorized-by-sprint-goal).
- KHÔNG sửa: `Cargo.toml`/`Cargo.lock` (version đã 0.1.0 — confirm only; nếu Worker thấy khác → anchor #3 sai, Debate Log), `src/**`, `tests/**`, `golden/**`, `hooks/**`, `scripts/**`, `docs/BACKLOG.md`, `docs/PROJECT.md`, `CLAUDE.md` (release flow không đổi rules — trừ khi xuất hiện deviation thật, khi đó ghi cả CHANGELOG lẫn CLAUDE.md per rule exit-code/deviation).
- KHÔNG implement: `cargo publish` crates.io (ngoài BACKLOG item — nếu muốn là phiếu riêng), code signing/notarization macOS (binary chạy từ terminal OK), **target `x86_64-apple-darwin`** (Intel-Mac gap = kit-level IG-10, orchestrator log — KHÔNG thêm vào matrix), checksum/SBOM file (chưa ai consume — ghi Park nếu thấy cần), thay đổi sos-kit, thêm bước test vào CI (no-test-in-CI = **intentional, giữ nguyên** — đã chốt Turn 1).

---

## Task 0 — Verification Anchors

> **Bước 0 capability (P285 Layer 1):** phiếu này cần **network + GitHub write**. ✅ **VERIFIED (Worker CHALLENGE Turn 1):** `gh` auth OK, quyền **ADMIN**, token scopes `repo`+`workflow`, `gh release list` trống, `gh run list` trống. Trong CI: `GITHUB_TOKEN` mặc định + `permissions: contents: write` (đã có sẵn trong workflow `[verified]`) là đủ — KHÔNG cần secret mới.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Baseline: main `2531343` (P007 merged), `cargo test` = 95 tests xanh, working tree clean | `git log -1` + `git status` + `cargo test` đầu phiếu | `[unverified — Worker confirm; số khác → Discovery, count mới = baseline]` |
| 2 | `.github/workflows/release.yml` tồn tại, trigger `push: tags: ["v*"]`, matrix 3 entry (mac-arm64/linux-x64/win-x64), asset naming `inv-gate-<target-triple>[.exe]` qua `softprops/action-gh-release@v2`, `permissions: contents: write`, `dtolnay/rust-toolchain@stable` | đọc file | ✅ **VERIFIED (Architect Read + Worker CHALLENGE)** — matrix hiện tại = ĐÚNG spec V2; Task 1 = verify/naming-check, không swap |
| 3 | `Cargo.toml` version = `0.1.0` (bootstrap đặt sẵn) → F13 = confirm + CHANGELOG heading, KHÔNG bump | `grep '^version' Cargo.toml` | ✅ **VERIFIED (Architect Read + Worker CHALLENGE)** |
| 4 | CHANGELOG: 7 sections `## [Unreleased] — P00x` (P001..P007) + `## v0.0.0`; CHƯA có heading `[0.1.0]` | `grep '^## ' CHANGELOG.md` | ✅ **VERIFIED (Architect Read + Worker CHALLENGE)** |
| 5 | Kit-family convention 3 target = mac-arm64 + linux-x64 + win-x64 (install.sh:49-52); win asset có `.exe` (install.sh:65 `${bin}-${TARGET}${EXT}`) | Worker đã đọc `~/sos-kit/install.sh` (CHALLENGE) + Chủ nhà xác nhận trực tiếp 11/06 | ✅ **RESOLVED (Turn 1)** — Option B mirror siblings; BACKLOG wording cũ (mac-x64) = lỗi diễn giải, đã sửa BACKLOG |
| 6 | ~~Runner `macos-13` available~~ | — | **MOOT (V2)** — không build `x86_64-apple-darwin`; macos-14 cho arm64 như siblings. Xóa mọi reference macos-13 |
| 7 | `softprops/action-gh-release@v2`: 3 job song song upload cùng tag → cùng 1 release (không tạo 3 release) | hành vi đã là pattern bootstrap/sibling; nếu race/duplicate error → `gh run rerun --failed` | `[unverified — verify bằng chính run thật; lưu ý retry trong Task 4]` |
| 8 | Máy này = arm64 darwin (acceptance (c) tải asset arm64 chạy local được) | `uname -sm` → `Darwin arm64` | `[unverified — Worker confirm 1 lệnh]` |
| 9 | Binary release < 6 MB (PROJECT.md success criterion #5; release profile đã `strip = true`, `lto`, `opt-level 3` `[verified — Cargo.toml]`) | size assets trên release page / `ls -lh` binary tải về | ✅ **PROXY VERIFIED (Worker CHALLENGE)** — local release binary 2.8 MB < 6 MB. Final = size 3 assets thật trên release; nếu > 6 MB: KHÔNG block, Discovery + escalate |
| 10 | Sau Task 1-3: `git diff` chỉ chạm files trong scope (workflow diff có thể RỖNG); `cargo test` vẫn 95 xanh (zero code change) | `git diff --stat` + `cargo test` trước commit | ⏳ TO VERIFY |
| 11 | Asset names đúng contract sau CI: `inv-gate-aarch64-apple-darwin`, `inv-gate-x86_64-unknown-linux-gnu`, `inv-gate-x86_64-pc-windows-msvc.exe` — ĐÚNG 3, không thừa thiếu, win CÓ `.exe` | `gh release view v0.1.0 --json assets` | ⏳ TO VERIFY — acceptance (b) |
| 12 | Binary từ release chạy đúng như local: `--help` exit 0; `check runtime` exit 0; `gate --all --skip-absent` exit 0 tại repo root (P007 đã làm repo này sạch với flag) | Task 5 transcript | ⏳ TO VERIFY — acceptance (c) |

**Anchor mở (Worker verify): #1, #7, #8, #10, #11, #12. ✅ Verified/Resolved: #2, #3, #4, #5, #9 (proxy), Bước 0. MOOT: #6. 0 anchor ❌.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P008
mkdir -p ".backup/${PHIEU_ID}"
cp .github/workflows/release.yml ".backup/${PHIEU_ID}/release.yml.orig"
cp CHANGELOG.md ".backup/${PHIEU_ID}/CHANGELOG.md.orig"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Turn 1 resolved)

### Turn 1 — Worker Challenge (vs V1)

- **[O1.1] BLOCKING — Matrix direction conflict:** V1 lệnh swap win-x64 → `x86_64-apple-darwin` theo BACKLOG wording, nhưng matrix bootstrap + convention sibling (`~/sos-kit/install.sh:49-52`) = mac-arm64 / linux-x64 / win-x64. Anchor #5 gate trigger đúng thiết kế — không drop/swap ratified target im lặng.
- **[O1.2]** Anchor #6: runner `macos-13` khả năng retired trên GitHub Actions.
- **Facts fold (Worker verified):** Bước 0 capability ✅ (`gh` ADMIN, scopes `repo`+`workflow`, release list trống, run list trống); binary local release 2.8 MB < 6 MB ✅ (proxy anchor #9); CHANGELOG 7 sections `[Unreleased]` ✅; Cargo.toml version 0.1.0 ✅; release.yml: softprops@v2 + `contents: write` + dtolnay stable ✅; workflow không có bước test — flag intentional?

**Status:** ✅ RESPONDED (xem Turn 1 — Architect Response)

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → DEFER TO CHỦ NHÀ → RESOLVED:** Chủ nhà xác nhận trực tiếp 11/06: *"scope của sos kit là window, macos, linux"* → **Option B — 3-target MIRROR SIBLINGS** đúng convention install.sh:49-52: `aarch64-apple-darwin` (macos-14) + `x86_64-unknown-linux-gnu` (ubuntu) + `x86_64-pc-windows-msvc` (windows, asset có `.exe`). KHÔNG mac-x86_64. BACKLOG wording cũ = lỗi diễn giải Quản đốc, BACKLOG đã sửa. Asset naming contract: `inv-gate-<target>` (+`.exe` win) khớp `${bin}-${TARGET}${EXT}` install.sh:65 → logic EXT trong workflow PHẢI GIỮ (V1 cho phép bỏ — thu hồi). **Task 1 viết lại = verify matrix hiện tại vs spec** — anchor #2 nói matrix đã đúng → kỳ vọng chỉ verify + asset-name check, diff có thể rỗng.
- **[O1.2] → MOOT:** không build mac-x86_64 nữa; macos-14 cho arm64 như siblings. Xóa mọi reference macos-13 khỏi phiếu.
- **No-test-in-CI:** intentional, giữ nguyên (pre-commit + nghiệm thu local cover; workflow tối giản như bootstrap/siblings).
- **Facts fold** vào Task 0: Bước 0 ✅, anchor #9 proxy ✅, #5 RESOLVED, #6 MOOT; #2/#3/#4 double-confirmed.
- **Discovery hook thêm:** Intel-Mac gap (`x86_64-apple-darwin` không có mapping `Darwin-x86_64` trong install.sh) = kit-level feedback → orchestrator log **IG-10**, NGOÀI scope phiếu này.

**Status:** ✅ RESOLVED — phiếu bumped V2 (oracle cho direction = install.sh:49-52,65 + Chủ nhà ruling 11/06; không còn objection mở)

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

> Thứ tự: Task 1 (workflow verify) → Task 2 (CHANGELOG) → Task 3 (docs) → commit + Giám sát + merge → Task 4 (tag + CI watch + verify assets) → Task 5 (ship-test asset) → Task 6 (Discovery).

### Task 1: Matrix 3-target — VERIFY `.github/workflows/release.yml` vs spec (V2: KHÔNG swap)

**File:** `.github/workflows/release.yml`

**Spec chốt (Chủ nhà 11/06 — contract Tầng 1 = bộ 3 target triple + asset naming + trigger + permissions; runner label / YAML style = Tầng 2):**

| Target triple | Runner (Tầng 2) | Asset name (Tầng 1) |
|---|---|---|
| `aarch64-apple-darwin` | `macos-14` | `inv-gate-aarch64-apple-darwin` |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` hiện tại OK (glibc cũ = compat rộng); `ubuntu-latest` như siblings cũng chấp nhận — Worker chọn, ghi Discovery nếu đổi | `inv-gate-x86_64-unknown-linux-gnu` |
| `x86_64-pc-windows-msvc` | `windows-2022` hiện tại OK; `windows-latest` chấp nhận — Worker chọn | `inv-gate-x86_64-pc-windows-msvc.exe` |

**Việc cần làm:**

1. Đối chiếu matrix hiện tại vs spec từng dòng. Anchor #2 (✅ verified): matrix hiện tại ĐÃ là bộ 3 này → kỳ vọng **KHÔNG sửa gì**, chỉ verify.
2. **Asset-name check:** xác nhận template tạo đúng `inv-gate-<target-triple>` và win CÓ `.exe` (logic EXT **PHẢI GIỮ** — install.sh:65 `${bin}-${TARGET}${EXT}` consume `.exe`). Lệch naming → chỉnh tối thiểu đúng contract.
3. Header comment (mac-arm64 / linux-x64 / win-x64): xác nhận khớp thực tế — đã khớp thì KHÔNG đụng.
4. **KHÔNG đổi:** trigger `tags: ["v*"]`, `permissions: contents: write`, `fail-fast: false`, `dtolnay/rust-toolchain@stable`, `softprops/action-gh-release@v2`, build command `cargo build --release --target ...`.

**Lưu ý:**
1. Verdict verify (khớp nguyên / chỉnh gì) ghi Discovery — kể cả khi diff rỗng.
2. KHÔNG thêm bước test vào workflow — **intentional, đã chốt Turn 1** (pre-commit + nghiệm thu local cover; workflow tối giản). Muốn thêm = Debate Log, không tự quyết (supply-chain file).
3. KHÔNG thêm target `x86_64-apple-darwin` (Intel-Mac gap = IG-10 kit-level, ngoài phiếu).

### Task 2: Version freeze F13 — `CHANGELOG.md`

**File:** `CHANGELOG.md`

1. Thêm heading release `## [0.1.0] — 2026-06-11` ngay trên block P007 hiện tại; **demote** 7 heading `## [Unreleased] — P00x ...` → `### P00x ...` (nội dung từng section GIỮ NGUYÊN VẸN — chỉ đổi heading level + bỏ chữ `[Unreleased]`; thứ tự giữ nguyên P007→P001). `## v0.0.0` giữ nguyên cuối.
2. Thêm subsection P008 (mới nhất, trên P007) trong `[0.1.0]`: release CI 3-target confirm (mac-arm64/linux-x64/win-x64 mirror siblings — Chủ nhà chốt 11/06; BACKLOG wording cũ mac-x64 = lỗi diễn giải, đã sửa BACKLOG), version freeze 0.1.0 (Cargo.toml đã 0.1.0 từ bootstrap — confirm, không bump), tag v0.1.0 + GitHub Release 3 assets (win có `.exe`), ship-test transcript pointer → `docs/discoveries/P008.md`.
3. Contract kiểm chứng: `grep -n '^## ' CHANGELOG.md` → `## [0.1.0] — 2026-06-11` là heading release ĐẦU TIÊN, không còn `[Unreleased]` nào phía trên nó; docs-gate `check_all` (changelog check) pass.

**Lưu ý:** KHÔNG sửa `Cargo.toml` (anchor #3 — đã 0.1.0). Nếu thấy version khác 0.1.0 → anchor sai → Debate Log, không tự bump.

### Task 3: Docs — `docs/ARCHITECTURE.md`

**File:** `docs/ARCHITECTURE.md`

Thêm 1 mục ngắn §Distribution (vị trí cuối file hoặc cạnh mục build, Tầng 2): release CI (`.github/workflows/release.yml`) — trigger tag `v*`, 3 target kit-family (mac-arm64 / linux-x64 / win-x64), asset naming `inv-gate-<target-triple>[.exe]` = contract sos-kit install.sh (`${bin}-${TARGET}${EXT}`), release profile strip/lto, exit-code contract 0/1/2 là API pre-commit (đã ghi CLAUDE.md — cite, không duplicate dài).

### Task 4: Ship + tag + CI watch (IG-09 — thứ tự CỨNG)

1. `cargo test` (95 xanh — anchor #10) + `cargo build --release` + docs-gate `check_all` → commit trên `feat/P008-release-v0.1.0` (hook pre-commit chạy thật, KHÔNG `--no-verify`).
2. **GIÁM SÁT REVIEW DIFF — BẮT BUỘC TRƯỚC MERGE** (orchestrator chạy; supply-chain surface): trọng tâm = workflow diff nếu có (3 target đúng spec? action versions không đổi? không secret mới? không bước lạ chèn vào build? asset naming + `.exe` nguyên?); workflow diff rỗng → review CHANGELOG/docs diff + xác nhận verdict verify Task 1. Verdict ghi Discovery.
3. Merge `main` → push `origin main` → XÓA branch ngay (local + remote). Không stack.
4. **Tag từ main ĐÃ MERGE** (tag phải trỏ commit chứa CHANGELOG release + workflow đã verify): `git tag -a v0.1.0 -m "inv-gate v0.1.0 — CLI 4 checks + gate --skip-absent + MCP serve"` → `git push origin v0.1.0`. (Annotated vs lightweight = Tầng 2, annotated khuyến nghị.)
5. Theo dõi: `gh run list --workflow=release` + `gh run watch <id>` đến khi cả 3 job XANH. Verify: `gh release view v0.1.0 --json assets,tagName` → đúng 3 asset names (anchor #11, win có `.exe`).
6. **Retry protocol (CI đỏ):** fix CHỈ trong workflow file qua commit mới trên main (diff nhỏ, vẫn qua hook) → xóa tag + release hỏng (`gh release delete v0.1.0 --yes` + `git tag -d v0.1.0` + `git push origin :refs/tags/v0.1.0`) → re-tag từ main mới. Xóa tag CHỈ hợp lệ TRƯỚC khi release được verify/announce (chưa ai consume — sos-kit join là harvest sau). **Cap = 2 lần re-tag**; lần 3 → DỪNG, escalate Chủ nhà với log CI.

### Task 5: Ship-test asset — CHẠY ĐƯỢC ≠ SHIP ĐƯỢC (slogan Chủ nhà)

1. Confirm máy: `uname -sm` → `Darwin arm64` (anchor #8).
2. Tải TỪ RELEASE (không dùng binary local): `gh release download v0.1.0 --pattern 'inv-gate-aarch64-apple-darwin' --dir /tmp/p008-shiptest` → `chmod +x`.
3. Chạy tại repo root (`/Users/nguyenhuuanh/inv-gate`), ghi transcript ĐẦY ĐỦ (lệnh + exit code + output):
   - `/tmp/p008-shiptest/inv-gate-aarch64-apple-darwin --help` → exit 0
   - `... check runtime` → exit 0, output khớp `target/release/inv-gate check runtime` (diff 2 output)
   - `... gate --all --skip-absent` → exit 0, output khớp binary local (P007 acceptance: repo này sạch với flag — gồm SKIP lines + `7 passed, 0 failed, 2 warnings`)
4. Size check (anchor #9 — proxy local 2.8 MB đã pass): cả 3 asset < 6 MB (PROJECT.md #5). > 6 MB → ghi Discovery + escalate, KHÔNG tự rollback release.
5. Transcript + size → `docs/discoveries/P008.md`.

### Task 6: Discovery report

**File:** `docs/discoveries/P008.md` + 1-line index `docs/DISCOVERIES.md` — gồm: (i) anchors ĐÚNG/SAI (đặc biệt verdict verify Task 1 — khớp nguyên hay chỉnh gì; #7 race có xảy ra không); (ii) CI run id + link + verdict 3 job; (iii) transcript Task 5 (ship-test) + asset sizes; (iv) Giám sát verdict; (v) **hooks cross-repo cho harvest sos-kit**: asset names + URL pattern cho install.sh BINARIES (win `.exe` khớp `${EXT}` sẵn), pre-commit `[4/7]` swap sang `inv-gate gate --all --skip-absent` (việc ~/sos-kit, ngoài scope); (vi) **IG-10 Intel-Mac gap**: `x86_64-apple-darwin` KHÔNG được install.sh hỗ trợ (Darwin-x86_64 không có mapping) — kit-level feedback, owner = orchestrator, NGOÀI scope phiếu; (vii) tier escalations ("None" nếu không); (viii) note cho orchestrator: sprint close-out (BACKLOG Recently shipped, retro, dogfood W-items) = mục riêng owner=orchestrator, KHÔNG thuộc phiếu.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.github/workflows/release.yml` | Task 1: VERIFY vs spec (matrix đã đúng per anchor #2) — diff có thể RỖNG; chỉ chỉnh nếu lệch asset-naming/`.exe` contract |
| `CHANGELOG.md` | Task 2: heading `## [0.1.0] — 2026-06-11` + demote 7 sections + entry P008 |
| `docs/ARCHITECTURE.md` | Task 3: §Distribution ngắn |
| `docs/discoveries/P008.md` + `docs/DISCOVERIES.md` | Task 6: report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `Cargo.toml`, `Cargo.lock` | Diff RỖNG — version đã 0.1.0 (anchor #3); khác → Debate Log |
| `src/**`, `tests/**` | Diff RỖNG TUYỆT ĐỐI — phiếu zero-code; 95 tests xanh nguyên |
| `golden/**`, `tests/golden/**` | READ-ONLY tuyệt đối |
| `hooks/**`, `scripts/**` | Dogfood nguyên trạng |
| `docs/BACKLOG.md`, `docs/PROJECT.md`, `CLAUDE.md` | Orchestrator/Chủ nhà-only (CLAUDE.md chỉ đụng nếu phát sinh deviation thật — ghi rõ trong Discovery) |

---

## Luật chơi (Constraints)

1. **Zero code change:** `src/` + `tests/` diff rỗng; `cargo test` 95 xanh nguyên trước commit. Phiếu này là CI + docs + actions.
2. **Bước 0 capability đã verify ✅ (Turn 1):** gh ADMIN, scopes repo+workflow. Nếu trạng thái đổi lúc EXECUTE (token hết hạn...) → DỪNG, escalate.
3. **Matrix spec chốt V2 (Chủ nhà 11/06):** mac-arm64 + linux-x64 + win-x64 (`.exe`). KHÔNG mac-x86_64, KHÔNG swap, KHÔNG drop. File thực tế lệch spec → chỉnh VỀ spec + ghi Discovery; muốn lệch KHỎI spec → Debate Log.
4. **Tag trỏ main đã merge** chứa CHANGELOG release + workflow đã verify — tag trước merge = CI build từ commit thiếu nội dung release = sai.
5. **Asset naming contract (Tầng 1):** `inv-gate-<target-triple>`, win có `.exe` (install.sh:65 `${bin}-${TARGET}${EXT}`), đúng 3 asset anchor #11. Logic EXT PHẢI GIỮ. Runner label, YAML style, annotated-vs-lightweight tag = Tầng 2.
6. **Supply-chain kỷ luật:** KHÔNG thêm action mới / bump action version / secret mới / bước build lạ / bước test CI (intentional). Diff workflow tối thiểu (kỳ vọng rỗng). Giám sát review BẮT BUỘC trước merge (verdict vào Discovery).
7. **Ship-test bắt buộc (acceptance (c)):** binary TỪ RELEASE chạy trên máy này — không lấy binary local thế chỗ. Transcript vào Discovery.
8. **Retry cap:** tối đa 2 lần xóa-tag/re-tag; lần 3 → escalate Chủ nhà. Xóa tag chỉ hợp lệ khi release chưa được consume/announce.
9. **F13:** Cargo.toml version = CHANGELOG latest release = 0.1.0; docs-gate `check_all` pass trước commit.
10. Cross-repo (sos-kit BINARIES, pre-commit [4/7], IG-10 Intel-Mac gap) = Discovery hook ONLY — không sửa gì ngoài repo này trừ tag/release của chính repo.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — 95 nguyên (zero code change, anchor #10)
- [ ] `git diff src/ tests/ golden/ hooks/ scripts/ Cargo.toml Cargo.lock` RỖNG
- [ ] docs-gate `check_all` pass (changelog có `[0.1.0]` heading + entry P008)
- [ ] `grep '^version' Cargo.toml` → `0.1.0`; heading release đầu tiên CHANGELOG = `[0.1.0] — 2026-06-11` (F13)

### Manual Testing (acceptance Chủ nhà)
- [ ] (b) Tag `v0.1.0` pushed → `gh run watch`: cả 3 job release CI XANH trên GitHub; `gh release view v0.1.0` → ĐÚNG 3 assets: `inv-gate-aarch64-apple-darwin`, `inv-gate-x86_64-unknown-linux-gnu`, `inv-gate-x86_64-pc-windows-msvc.exe`
- [ ] (c) **CHẠY ĐƯỢC ≠ SHIP ĐƯỢC:** asset arm64 tải từ Release → `--help` exit 0; `check runtime` exit 0 khớp local; `gate --all --skip-absent` exit 0 khớp local (SKIP lines + `7 passed, 0 failed, 2 warnings`) — transcript Discovery
- [ ] Asset sizes < 6 MB (PROJECT.md #5; proxy local 2.8 MB đã pass) — vượt thì Discovery + escalate, không block

### Regression
- [ ] `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree (dogfood nguyên)
- [ ] `cargo run -- check secrets|runtime|port|schema` + `gate --all` behavior nguyên (không build/code change)

### Docs Gate
- [ ] `CHANGELOG.md` — `[0.1.0]` heading + P008 entry (3-target mirror-siblings resolution ghi rõ)
- [ ] `docs/ARCHITECTURE.md` — §Distribution

### Discovery Report
- [ ] `docs/discoveries/P008.md`: anchors verdict (Task 1 verify / #7), CI run link, ship-test transcript + sizes, Giám sát verdict, hooks harvest sos-kit, **IG-10 Intel-Mac gap (owner=orchestrator)**, tier escalations ("None" nếu không), note close-out = orchestrator
- [ ] Append 1-line index `docs/DISCOVERIES.md`
