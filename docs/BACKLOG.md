# BACKLOG — inv-gate

> **Mục đích:** Single source of truth cho "Chủ nhà nên làm gì tiếp theo".
> Idea mới → vào đây trước (qua /idea skill). Phiếu → chỉ viết cho item trong Active sprint.
> Wave-based, KHÔNG time-based. Sprint kết thúc khi xong hoặc Chủ nhà đổi hướng.
>
> **Quy tắc Architect (Rule 0):** Architect chỉ viết phiếu cho item nằm trong "Active sprint" hoặc Chủ nhà explicit move từ "Next sprint" lên. Không phiếu cho item ở "Open backlog" / "Park" cho đến khi Chủ nhà pick.

---

## 🔥 Active sprint: Sprint 2 — MCP + distribution (Phase 2-3, chốt dự án)

> **Mục tiêu:** Hoàn tất vision PROJECT.md — dual mode (CLI ✅ Sprint 1 + MCP serve) + ship v0.1.0 ra GitHub Releases. Kỷ luật mới (IG-09, lệnh Chủ nhà): XONG 1 PHIẾU = merge main + push + xóa branch NGAY, không stack.
> **Kết thúc khi:** `inv-gate serve` expose tools check_* + gate qua rmcp stdio (test bằng MCP client thật), `gate --all --profile` chạy sạch trên repo non-webapp, tag v0.1.0 → release CI 3-target xanh trên GitHub.
> **Started:** 11/06/2026 (ngay sau Sprint 1)

- [ ] **[NEW] P006 — `serve` (MCP stdio).** rmcp server expose 5 tools: `check_secrets`/`check_runtime`/`check_port`/`check_schema`/`gate` — mỗi tool wrap check function in-process (cùng code path CLI, KHÔNG duplicate logic), trả findings + exit-code-equivalent. Dep mới `rmcp` (đã ghi trong stack CLAUDE.md). Test: spawn server, gọi tool qua stdio client, assert kết quả khớp CLI trên cùng fixture.
- [ ] **[NEW] P007 — `gate --all` profile mode (từ P005 decision (c)).** Behavior change ĐẦU TIÊN sau parity (method rule 3): cơ chế cho gate chạy trên repo non-webapp (tarot-INV đòi file vắng mặt → skip-with-note thay vì FAIL, hoặc `--profile <type>`). SECURITY SURFACE → Giám sát review bắt buộc trước merge. Golden/pins KHÔNG đổi — parity tests giữ nguyên xanh (profile ≠ default behavior).
- [ ] **[NEW] P008 — Release v0.1.0.** Version sync Cargo.toml + CHANGELOG (F13), release CI workflow (tag → build 3-target: macOS arm64 + x86_64, linux x86_64 → GitHub Releases), tag v0.1.0 push, verify release assets. Note cross-repo: join sos-kit install.sh BINARIES + pre-commit [4/7] swap là việc repo ~/sos-kit (harvest cuối sprint, ngoài scope repo này).

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

- [ ] **[DEBT] Fix bad-SHA fallback trong schema check** — golden `check-schema-safety.sh:33` dùng SHA empty-tree malformed 15-char (`4b825dc8669f8c0`, thiếu 25 hex); P004 port as-is theo parity-first (method rule 3). Post-parity behavior-change phiếu: dùng SHA empty-tree đủ 40-char. Estimate: 1h. Nguồn: P004 O1.2, docs/discoveries/P004.md.

---

## 🅿️ Park / nghĩ thêm

> Idea chưa chín, hoặc đã suy nghĩ nhưng chưa quyết, hoặc bị reject mềm (chưa hẳn no).

- [ ] <Idea cần research, hoặc đã debate chưa kết>

---

## ✅ Recently shipped (3 sprint gần nhất)

> Quick reference. Chi tiết đầy đủ → CHANGELOG.md.

<!-- Khi sprint xong, move tóm tắt 1-line vào đây. Giữ tối đa 3 sprint gần nhất. -->

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
