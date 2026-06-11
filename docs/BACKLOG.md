# BACKLOG — inv-gate

> **Mục đích:** Single source of truth cho "Chủ nhà nên làm gì tiếp theo".
> Idea mới → vào đây trước (qua /idea skill). Phiếu → chỉ viết cho item trong Active sprint.
> Wave-based, KHÔNG time-based. Sprint kết thúc khi xong hoặc Chủ nhà đổi hướng.
>
> **Quy tắc Architect (Rule 0):** Architect chỉ viết phiếu cho item nằm trong "Active sprint" hoặc Chủ nhà explicit move từ "Next sprint" lên. Không phiếu cho item ở "Open backlog" / "Park" cho đến khi Chủ nhà pick.

---

## 🔥 Active sprint: Sprint 1 — Golden-oracle port (Phase 1 CLI)

> **Mục tiêu:** Port 4 check + orchestrator từ `golden/` (797 LOC Python+Bash, frozen từ tarot) sang Rust, parity-tested chống oracle đã pin. Method = doc-rotate precedent: PIN TRƯỚC, PORT SAU.
> **Kết thúc khi:** `inv-gate gate --all` parity với `golden/security-gate.sh` trên fixture set (same findings + same exit codes), `cargo test` xanh.
> **Started:** 11/06/2026

- [ ] **[NEW] P001 — Pin golden oracle.** Dựng fixture set (repo mẫu có secret/port-bind/token-in-url/DROP-migration + bản sạch) → chạy 5 script `golden/` → freeze findings + exit codes vào `tests/golden/`. KHÔNG port gì trước khi pin xong. Spec unit (char vs byte) cho mọi offset field (F06).
- [ ] **[NEW] P002 — `check secrets` (INV-009)** port `check-hardcoded-secrets.py` (192 LOC) + parity test chống pin.
- [ ] **[NEW] P003 — `check runtime` (INV-010)** port `check-runtime-secrets.py` (246 LOC — token-in-url, .git/config, Sub-mech F) + parity.
- [ ] **[NEW] P004 — `check port` (INV-001) + `check schema`** port `check-port-bind.py` (82) + `check-schema-safety.sh` (64) + parity.
- [ ] **[NEW] P005 — `gate --all`** orchestrator (aggregate exit codes, parity với `security-gate.sh` 210 LOC) + swap thử vào pre-commit `[4/7]` của chính repo này (dogfood).

## 🎯 Next sprint: Sprint 2 — MCP + distribution

- `serve` (rmcp stdio — tools check_* + gate) · release tag v0.1.0 → 3-target CI → join sos-kit `install.sh` BINARIES · sos-kit pre-commit [4/7] swap python→binary (B+3-style cho gate class nếu fail-closed cần)

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

- [ ] <Idea 1 — gì + tại sao + estimate sơ bộ>

---

## 🅿️ Park / nghĩ thêm

> Idea chưa chín, hoặc đã suy nghĩ nhưng chưa quyết, hoặc bị reject mềm (chưa hẳn no).

- [ ] <Idea cần research, hoặc đã debate chưa kết>

---

## ✅ Recently shipped (3 sprint gần nhất)

> Quick reference. Chi tiết đầy đủ → CHANGELOG.md.

<!-- Khi sprint xong, move tóm tắt 1-line vào đây. Giữ tối đa 3 sprint gần nhất. -->

- ✅ **<Sprint name>** (DD/MM/YYYY) — <one-line summary>

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
