# SOS_KIT_FEEDBACK — inv-gate end-to-end dogfood (biểu điểm)

> **Mục đích:** inv-gate Sprint 1 là dogfood 3-trong-1: (1) ra tool thật, (2) lần ĐẦU chạy trọn
> workflow `DRAFT → CHALLENGE → APPROVAL_GATE → EXECUTE` trong repo adopt-born tinh với kit
> bản mới nhất, (3) chấm điểm Principle 7 (adopt-hiểu-repo: fit 70-80%? mài dao gì?).
> **Cách điền:** Quản đốc inv-gate vừa chạy vừa ghi findings `[IG-NN] sev — triệu chứng → root
> cause → đề xuất` (sev: 🔴 block/correctness · 🟡 friction · 🟢 worked-as-intended-ghi-để-giữ).
> Kit-bug → route về `~/sos-kit` (file này được sos-kit harvest cuối sprint). Tool-bug của
> chính inv-gate → BACKLOG repo này, KHÔNG ghi vào đây.
> **Tên file = chuẩn mới** (P066): mọi repo dogfood dùng `docs/SOS_KIT_FEEDBACK.md`.

---

## WATCHLIST — chạy tới đâu, tick tới đó

### A. Spine wiring lúc RUNTIME (adopt CONNECTED rồi, giờ xem nó SỐNG không)
- [ ] W1. SessionStart banner hiện đúng Sprint 1 (P001-P005) — không placeholder
- [ ] W2. Quản đốc load deferred tools đầu session (AskUserQuestion/TaskCreate...) — không lỗi InputValidationError về sau
- [ ] W3. Marker 2 chiều đúng nhịp (architect-active ↔ worker-active) — quên touch → orchestrator-guard chặn Write đầu của Thợ (F-001 class)
- [ ] W4. idea-smell hook: lúc nào Sếp nói "ghi vào backlog"/"anh nghĩ ra" → có thấy dòng nhắc 💡 → Quản đốc có invoke /idea thật không (test sống trigger-fix)
- [ ] W5. B+3 shim: mọi Bash call chạy bình thường (binary có trên PATH); thử nghĩa vụ chặn nếu có dịp merge PR

### B. Workflow state machine (lần đầu full-flow trên repo tinh)
- [ ] W6. Architect DRAFT: docs-only — có tôn trọng `golden/` là READ-ONLY reference không; Write-envelope chỉ cho phiếu `P*-*.md` (P069 guard)
- [ ] W7. Worker CHALLENGE: Task 0 grep anchors vào `golden/` + `tests/` thật — có bịa anchor không
- [ ] W8. Đếm turns DRAFT⇄CHALLENGE (lane budget §1 — quá 3 = ghi)
- [ ] W9. APPROVAL_GATE: đúng 1 AskUserQuestion duy nhất trước EXECUTE — không fake-gate giữa chừng
- [ ] W10. Discovery Report cuối phiếu có dòng "Tầng 1 docs updated: <list>" (Rule #8)

### C. Doctrine TRANSFER (cốt lõi — chữ viết có tự chạy không, hay phải nhắc mồm?)
- [ ] W11. Pin-TRƯỚC-port-SAU: Architect/Worker có tự bám method trong CLAUDE.md không, hay đòi port luôn (doc-rotate phải học bằng máu — inv-gate được CHO SẴN chữ; transfer fail = doctrine-ship ≠ doctrine-survive)
- [ ] W12. F06: mọi field offset trong golden pin có spec UNIT (char vs byte) không
- [ ] W13. F07: Worker có bịa fixture FILE không (synthetic in-code = OK)
- [ ] W14. Exit-code contract (0/1/2) có được giữ như API qua các phiếu không

### D. Gates live-fire (repo Rust ĐẦU TIÊN qua chain 7-phase mới)
- [ ] W15. `[1/7]` cargo check: thời gian trên 8GB (lần đầu compile deps — đo; quá đau → finding)
- [ ] W16. `[4/7]` security gate quét `golden/*.py` (file ĐỊNH NGHĨA pattern secret) — false-positive INV-009? (sos-kit pass nhưng đường dẫn khác — WATCH)
- [ ] W17. no-code-on-default: có ép feature branch đúng lúc không, có chặn nhầm docs không
- [ ] W18. CHANGELOG freshness NGÀY 2+: skeleton đề ngày 11/06 — sáng mai commit đầu sẽ đòi entry mới (JA-09 class: gate-fail có guided không hay fail khô)

### E. Principle 7 calibration (chấm % fit của adopt)
- [ ] W19. Liệt kê phần adopt mang vào mà sprint này KHÔNG đụng (advisory pipeline? AGENT_MAP stub? phần nào = noise cho repo tool nhỏ?)
- [ ] W20. Liệt kê cái repo PHẢI TỰ MÀI (thiếu gì phải chế thêm) → đây chính là input cho survey/classify engine sau này
- [ ] W21. Chấm: adopt cho repo này được ~bao nhiêu % fit? (Sếp ước lượng cuối sprint)

### F. Phiếu lifecycle + tools
- [ ] W22. `phieu <slug>` / .phieu-counter hoạt động? Phiếu được track + move done sạch?
- [ ] W23. doctor/docs-gate/ship gọi trong repo cargo — lỗi gì không (doctor v0.1.1 shim-aware đã verify CONNECTED)

---

## FINDINGS (điền dần — IG-01, IG-02, ...)

_(trống — Quản đốc inv-gate điền trong lúc chạy Sprint 1)_
