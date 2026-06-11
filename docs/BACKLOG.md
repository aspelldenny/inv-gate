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

- [x] **P001 — Pin golden oracle.** ✅ 11/06 shipped `d014e44` — fixture set + repin.sh + 16 pins + exit_codes.json, MANIFEST unit-spec (F06: line-only).
- [x] **P002 — `check secrets` (INV-009)** ✅ 11/06 shipped `65b1140` — CLI skeleton + port 1:1 + parity byte-exact.
- [x] **P003 — `check runtime` (INV-010)** ✅ 11/06 shipped `8c7bda3` — 18 pattern classes, db-conn lookahead transcription (equivalence proof 15/15), parity byte-exact.
- [x] **P004 — `check port` (INV-001) + `check schema`** ✅ 11/06 shipped `683098c` — 2 check + parity stdout+stderr byte-exact, schema git-harness 2-commit.
- [x] **P005 — `gate --all`** ✅ 11/06 shipped `5a82c00` — orchestrator 9-INV port (6 inline check + INV-008 Python→Rust thuần), parity stdout+stderr byte-exact, dogfood swap PER-CHECK (Chủ nhà decision: thay python3 call trong adapted gate bằng binary; gate --all nguyên con cần profile mode → Sprint 2). Proof-commit hook thật + reversibility test pass.

## 🎯 Next sprint: Sprint 2 — MCP + distribution

- `serve` (rmcp stdio — tools check_* + gate) · release tag v0.1.0 → 3-target CI → join sos-kit `install.sh` BINARIES · sos-kit pre-commit [4/7] swap python→binary (B+3-style cho gate class nếu fail-closed cần)
- **Profile/flag mode cho `gate --all` đa-repo** (từ P005 decision (c)): golden gate là tarot-specific (INV-004/005/008 đòi file webapp) → exit 1 trên repo non-webapp. Cần `--profile <type>` hoặc skip-INV-khi-file-absent — behavior change, phiếu riêng theo method rule 3, đụng security surface → Giám sát review.

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

- ✅ **Sprint 1 — Golden-oracle port (Phase 1 CLI)** (11/06/2026) — 5/5 phiếu shipped trong 1 ngày: oracle pinned (16 pins) + 4 check + gate orchestrator port sang Rust, 79 tests xanh, parity byte-exact, dogfood per-check swap live trong pre-commit (python3 killed khỏi [4/7]).

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
