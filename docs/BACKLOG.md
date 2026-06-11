# BACKLOG — inv-gate

> **Mục đích:** Single source of truth cho "Chủ nhà nên làm gì tiếp theo".
> Idea mới → vào đây trước (qua /idea skill). Phiếu → chỉ viết cho item trong Active sprint.
> Wave-based, KHÔNG time-based. Sprint kết thúc khi xong hoặc Chủ nhà đổi hướng.
>
> **Quy tắc Architect (Rule 0):** Architect chỉ viết phiếu cho item nằm trong "Active sprint" hoặc Chủ nhà explicit move từ "Next sprint" lên. Không phiếu cho item ở "Open backlog" / "Park" cho đến khi Chủ nhà pick.

---

## 🔥 Active sprint: Sprint 3 — Maintenance mini (Sếp pick 11/06 tối)

> 2 item DEBT Sếp pick trực tiếp từ Open backlog ("em có thể làm cái 1. và 3. luôn").
> Per-phiếu merge discipline + Giám sát trước merge (cả 2 đụng surface nhạy).

- [x] **P009 — CI actions bump node24** ✅ 11/06 merged `d62ffc3` — checkout@v5 + gh-release@v3 + conditional prerelease cho rc tag; rc1 verify 3/3 xanh 0 annotation, latest pointer an toàn, cleanup sạch. Deadline 16/06 đã hóa giải.
- [x] **P010 — empty-tree SHA fix** ✅ 11/06 merged `f1c96b8` — deviation #1 có chủ đích khỏi golden (red→green proof, fail-open thu hẹp, oracle-guard test), pins byte-identical, 100 tests, Giám sát APPROVE.

- [x] **P006 — `serve` (MCP stdio)** ✅ 11/06 merged `c9a6d76` — 5 tools rmcp 1.7.0, buffered-core refactor ~43 sites, 84 tests, live JSON-RPC session verified.
- [x] **P007 — `gate --skip-absent`** ✅ 11/06 merged `e0150c7` — behavior-change đầu tiên sau parity, guard kép INV-005 + INV-008, fail-closed, Giám sát APPROVE 5 focus, 95 tests.
- [x] **P008 — Release v0.1.0** ✅ 11/06 tag `v0.1.0` (merge `aa262e4`) — CI 3-target xanh attempt 1, assets 2.8-3.6MB (<6MB), release binary byte-identical local (acceptance "chạy được ≠ ship được" PASS).

## 🎯 Next sprint: (trống — Sprint 2 là sprint chốt dự án theo PROJECT.md)

## 🎯 Next sprint (template): <Sprint name / theme>

> **Trigger:** <Khi nào move lên Active — vd "khi sprint hiện tại xong + Chủ nhà dùng feedback X".>
> **Theme:** <Một câu mô tả chủ đề sprint này.>

<!-- Idea cluster đã thành hình nhưng chưa active. Có thể thay đổi. -->

- [ ] <Item planned for next sprint>

---

## 🌊 Future waves (cam kết level low)

> Idea cluster lớn hơn — Phase / Sprint xa. Có thể thay đổi nhiều.

- [ ] **<Future Sprint name>** — <high-level description>
  - <sub-bullet>
  - <sub-bullet>

---

## 💡 Open backlog (chưa thuộc sprint)

> Idea rời, chưa cluster thành sprint. Khi đủ 2-3 cái cùng chủ đề → cluster.

<!-- Sếp dump idea ở đây qua /idea skill, hoặc trực tiếp edit tay. -->

- [ ] **[KIT-HARVEST] sos-kit: join BINARIES + pre-commit [4/7] swap** (cross-repo ~/sos-kit): thêm `inv-gate` vào install.sh BINARIES (asset contract khớp sẵn), swap python3 call trong pre-commit template → binary. Kèm 2 kit-feedback: IG-10 (Intel-Mac/ARM-Linux gap), quarantine UX note (xattr guidance). Nguồn: SOS_KIT_FEEDBACK.md harvest.

---

## 🅿️ Park / nghĩ thêm

> Idea chưa chín, hoặc đã suy nghĩ nhưng chưa quyết, hoặc bị reject mềm (chưa hẳn no).

- [ ] <Idea cần research, hoặc đã debate chưa kết>

---

## ✅ Recently shipped (3 sprint gần nhất)

> Quick reference. Chi tiết đầy đủ → CHANGELOG.md.

<!-- Khi sprint xong, move tóm tắt 1-line vào đây. Giữ tối đa 3 sprint gần nhất. -->

- ✅ **Sprint 2 — MCP + distribution** (11/06/2026, v0.1.0 released) — 3/3 phiếu cùng ngày: serve MCP 5 tools + gate --skip-absent (Giám sát-gated) + release CI 3-target kit-family. 95 tests. Kỷ luật per-phiếu merge+clean (IG-09) chạy trọn từ P006. DỰ ÁN ĐẠT VISION PROJECT.md.
- ✅ **Sprint 1 — Golden-oracle port (Phase 1 CLI)** (11/06/2026, merged `6828ecc`) — 5/5 phiếu trong 1 ngày: oracle pinned (16 pins) + 4 check + gate 9-INV port sang Rust, 79 tests xanh parity byte-exact, dogfood per-check swap live (python3 killed khỏi [4/7]). History rewritten pre-push (IG-08): P001 `164c2d0` P002 `9c0fa86` P003 `18dab08` P004 `df202ee` P005 `bf002ca`.

---

## ❌ Đã reject (lưu để khỏi nghĩ lại)

> Idea đã suy nghĩ và quyết KHÔNG làm. Lưu lý do rõ ràng để 6 tháng sau khỏi reconsider.

- **<Idea name>** — reject DD/MM/YYYY, lý do: <ngắn gọn>

---

## 📌 Quy tắc maintenance

1. **Idea mới** → `/idea` skill → tự append vào "Open backlog" hoặc "Active sprint" tùy phân loại.
2. **Phiếu xong** → move item từ Active sprint xuống "Recently shipped".
3. **Sprint xong** → tổng kết trong CHANGELOG.md, BACKLOG chỉ giữ 3 sprint gần nhất ở "Recently shipped".
4. **Discovery debt** mới → từ DISCOVERIES.md → append vào "Open backlog" với prefix `[DEBT]`.
5. **Architect rule** (cứng): không viết phiếu cho item nằm ngoài "Active sprint". Chủ nhà move item lên trước → Architect mới viết.
6. **Review monthly** — Chủ nhà đọc Park, quyết: promote lên Open backlog, hay ship to Reject với lý do.

---

*File này là LIVE. Chủ nhà chỉnh trực tiếp được. Architect/Worker chỉ ĐỌC, không tự edit khi đang viết phiếu.*
