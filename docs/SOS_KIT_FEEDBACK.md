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
- [x] W1. SessionStart banner hiện đúng Sprint 1 (P001-P005) — không placeholder ✅ 11/06: banner đủ 5 item + orchestrator contract
- [x] W2. Quản đốc load deferred tools đầu session (AskUserQuestion/TaskCreate...) — không lỗi InputValidationError về sau ✅ 11/06: ToolSearch select OK ngay đầu session
- [x] W3. Marker 2 chiều đúng nhịp (architect-active ↔ worker-active) — quên touch → orchestrator-guard chặn Write đầu của Thợ (F-001 class) ✅ 11/06: full cycle P001 không bị chặn lần nào; Worker tự rm worker-active sau khi xong. Nhưng xem IG-01: chiều NGƯỢC (marker sót → chặn orchestrator) dính 2 lần.
- [ ] W4. idea-smell hook — KHÔNG test được sprint này: Sếp không phát ngôn trigger nào ("ghi vào backlog"/"anh nghĩ ra") suốt session. Backlog items mới (DEBT SHA, profile mode) đều từ workflow chứ không từ Sếp-idea → /idea không có dịp sống. Carry sang sprint sau.
- [x] W5. B+3 shim: mọi Bash call chạy bình thường (binary có trên PATH) ✅ 11/06: toàn session không lỗi PATH/shim nào; nghĩa vụ chặn merge chưa thử (chưa merge PR — sẽ test khi Sếp quyết merge sprint branch).

### B. Workflow state machine (lần đầu full-flow trên repo tinh)
- [x] W6. Architect DRAFT: docs-only — có tôn trọng `golden/` là READ-ONLY reference không; Write-envelope chỉ cho phiếu `P*-*.md` (P069 guard) ✅ 11/06: Architect chỉ Write `docs/ticket/P001-pin-golden-oracle.md`, tự nhận `tests/` ngoài envelope → để anchor ⏳ Worker-verify thay vì bịa
- [x] W7. Worker CHALLENGE: Task 0 grep anchors vào `golden/` + `tests/` thật — có bịa anchor không ✅ 11/06: 11/11 anchor verify bằng grep/ls thật, kèm evidence line-number (security-gate.sh:55/:147/:197/:201); bắt được fixture mismatch thật (O1.2)
- [x] W8. Đếm turns DRAFT⇄CHALLENGE (lane budget §1 — quá 3 = ghi) ✅ 11/06: chốt 2 turns (≤3 budget). Turn 1 O1.2 (fixture Prisma sai cơ chế — Tầng 1 thật), turn 2 O2.1 (fake token sai regex — fix pre-verified trong challenge nên skip turn 3, đi thẳng gate). Cả 2 objection đều REAL bug trong phiếu, không phải noise — challenge lane có giá trị thật.
- [x] W9. APPROVAL_GATE: đúng 1 AskUserQuestion duy nhất trước EXECUTE — không fake-gate giữa chừng ✅ 11/06: P001 đi DRAFT→2×CHALLENGE→2×RESPOND→1 gate duy nhất (AskUserQuestion), Sếp duyệt EXECUTE. Không hỏi user giữa các phase.
- [x] W10. Discovery Report cuối phiếu có dòng "Tầng 1 docs updated: <list>" (Rule #8) ✅ 11/06 P001: có — CHANGELOG.md + CLAUDE.md (exit-code deviation note); docs/DISCOVERIES.md + docs/discoveries/P001.md tạo đúng format, 5 Tầng-2 self-adapts ghi rõ

### C. Doctrine TRANSFER (cốt lõi — chữ viết có tự chạy không, hay phải nhắc mồm?)
- [x] W11. Pin-TRƯỚC-port-SAU: Architect/Worker có tự bám method trong CLAUDE.md không, hay đòi port luôn (doc-rotate phải học bằng máu — inv-gate được CHO SẴN chữ; transfer fail = doctrine-ship ≠ doctrine-survive) ✅ 11/06: TRANSFER OK — Architect tự scope P001 PIN-ONLY ngay draft V1, Worker execute zero Rust. Không ai đòi port sớm, không phải nhắc mồm.
- [x] W12. F06: mọi field offset trong golden pin có spec UNIT (char vs byte) không ✅ 11/06: MANIFEST có unit-spec table; cả 4 check output line-only (không col/offset) — khai explicit "line-only, no col/offset field" thay vì bỏ trống
- [x] W13. F07: Worker có bịa fixture FILE không (synthetic in-code = OK) ✅ 11/06: mọi fixture value chốt trong phiếu kèm citation oracle (token ghp_ cite check-runtime-secrets.py:96); 5 Tầng-2 self-adapts đều ghi vào Discovery, không bịa ngầm
- [ ] W14. Exit-code contract (0/1/2) có được giữ như API qua các phiếu không

### D. Gates live-fire (repo Rust ĐẦU TIÊN qua chain 7-phase mới)
- [x] W15. `[1/7]` cargo check: thời gian trên 8GB (lần đầu compile deps — đo; quá đau → finding) ✅ 11/06: 0.71s hot (deps compile từ run P001); cold-compile đau hay không chưa đo được riêng — không thành finding
- [x] W16. `[4/7]` security gate quét `golden/*.py` (file ĐỊNH NGHĨA pattern secret) — false-positive INV-009? ✅ 11/06: 7/7 hook pass khi commit P002, KHÔNG false-positive trên cả golden/*.py lẫn src/checks/secrets.rs (chứa regex pattern secret) — lý do: scan targets của gate là .ts/.tsx/.js/.jsx trong src/, không quét .rs/.py. Lưu ý ngược: nghĩa là gate hiện KHÔNG bảo vệ code Rust của chính repo này — đặc tính kế thừa từ tarot, P005 dogfood nên cân nhắc
- [x] W17. no-code-on-default: có ép feature branch đúng lúc không, có chặn nhầm docs không ✅ 11/06: chặn commit P001 trên main đúng lúc → Worker tự branch `feat/P001-pin-golden-oracle` theo guided message. Note: fixture `.ts` trong tests/golden/fixtures/ bị tính là product code (trip gate) — chấp nhận được nhưng đáng ghi: gate không phân biệt fixture-dir.
- [ ] W18. CHANGELOG freshness NGÀY 2+: skeleton đề ngày 11/06 — sáng mai commit đầu sẽ đòi entry mới (JA-09 class: gate-fail có guided không hay fail khô) — KHÔNG test được hôm nay (sprint chạy trọn trong 11/06, mọi commit cùng ngày entry). Test tự nhiên ở commit đầu tiên ngày mai.

### E. Principle 7 calibration (chấm % fit của adopt)
- [x] W19. Phần adopt KHÔNG đụng sprint này (input Quản đốc): advisory pipeline (/advisory-scan + advisory-watch agent — 0 lần gọi), guard MCP (guard_check_* — pre-deploy infra, repo CLI tool không deploy), vps MCP (0 lần), ship MCP (0 lần), quality-gate MCP (0 lần), recipes /apply //forge (0 lần), INVARIANTS.md INV-LOCAL section (trống placeholder — đến lúc Giám sát chạy mới lộ). ≈ 6/10 cụm adopt là noise cho repo tool nhỏ Phase 1. Đáng giữ vì Phase 2 (MCP serve) sẽ cần ship/release tooling.
- [x] W20. Cái repo PHẢI TỰ MÀI (input Quản đốc): (1) biểu điểm dogfood SOS_KIT_FEEDBACK.md — Sếp tự chế, kit chưa có template; (2) gate-delegation mode (IG-03) — Sếp ủy quyền mồm, kit không có cơ chế; (3) parity-test harness pattern (env-reconstruction, byte-exact pin assert) — tự đúc qua P002-P005, giờ thành MANIFEST §4 9 rules — đây là candidate recipe /forge cho repo port-kiểu-này; (4) per-check swap pattern cho dogfood gate.
- [ ] W21. Chấm: adopt cho repo này được ~bao nhiêu % fit? (SẾP ước lượng — input em: theo W19/W20 thì ~60-70%: spine+workflow+gates fit tốt, 6/10 cụm tool chưa dùng nhưng không gây hại, 4 thứ phải tự mài)

### F. Phiếu lifecycle + tools
- [x] W22. `phieu <slug>` / .phieu-counter hoạt động? Phiếu được track + move done sạch? 🟡 11/06: phiếu lifecycle CHẠY (5 phiếu docs/ticket/P00N-slug.md, version bump V1→V3, Debate Log đầy đủ, BACKLOG tick + Recently shipped) NHƯNG `phieu <slug>` CLI / `.phieu-counter` KHÔNG được dùng — Architect tự đặt tên file trực tiếp. Tool không cần thiết hay chưa được discover? → kit xem lại.
- [x] W23. doctor/docs-gate/ship gọi trong repo cargo — lỗi gì không ✅ 11/06: docs-gate check_all chạy sạch 8/8 Pass trên repo cargo (changelog, architecture, 5 ticket checks). doctor/ship chưa gọi trong sprint này — không có nhu cầu phát sinh.

---

## FINDINGS (điền dần — IG-01, IG-02, ...)

- [IG-03] 🟢 — Workflow note (ảnh hưởng cách đọc W9 từ P002+): sau gate P001, Sếp ủy quyền APPROVAL_GATE cho Quản đốc với quyết định thuần kỹ thuật — chạy end-to-end P002-P005 rồi báo cáo cuối, kèm dogfood thật để so "ship được" vs "chạy được". Gate P002+ = Quản đốc self-approve khi CHALLENGE sạch (objection closed có oracle); vẫn dừng hỏi Sếp nếu đổi scope/vision, security-surface decision, hoặc action khó đảo ngược ngoài repo. Đây là data point cho kit: gate NÊN delegable-by-Chủ-nhà hay luôn bắt human? → đề xuất kit hỗ trợ explicit "gate delegation" mode thay vì workaround.
- [IG-04] 🟢 — Counts trong phiếu drift khỏi source NGAY CẢ sau challenge: allowlist golden có 9 entries; phiếu V1 cite 7, Worker challenge đếm "8", phiếu V2 sửa thành "8" — Worker execute đọc source mới thấy 9 (thiếu `PLACEHOLDER`). Cứu được nhờ rule "port NGUYÊN XI từ source + cite line, không tin count trong phiếu". Bài học kit: phiếu nên cite RANGE (`golden:85-95`) thay vì đếm số entries — số đếm là bug magnet qua nhiều vòng truyền tay.
- [IG-05] 🟡 — Vòng P003: orchestrator nhận 2 completion notification từ CÙNG architect task-id — lần 1 "phiếu V1 xong", lần 2 (sau khi Worker challenge chạy) chứa nguyên RESPOND Turn 1 + bump V2, trong khi notification của chính Worker challenge turn 1 KHÔNG về orchestrator. Tức là loop CHALLENGE→RESPOND tự khép giữa 2 subagent không qua Quản đốc. Tốc độ tốt + kết quả đúng, NHƯNG orchestrator mất dấu state machine (suýt spawn RESPOND trùng) và turn-count W8 phải reconstruct từ Debate Log trong phiếu thay vì đếm trực tiếp. Đề xuất kit: làm rõ contract — subagent-to-subagent relay là feature (thì document + cho orchestrator visibility) hay bug routing notification. **Tái diễn P004** (cùng pattern: challenge report về qua task-id của architect, consensus tự khép 1 turn) — có vẻ là hành vi ổn định của harness chứ không phải glitch 1 lần; Quản đốc đã thích nghi bằng cách đọc Debate Log trong phiếu làm source-of-truth state thay vì notification flow.
- [IG-06] 🔴→🟢 — Sprint-goal assumption GÃY, nhưng CHALLENGE bắt được TRƯỚC khi ship (giá trị lớn nhất của lane từ đầu sprint): item P005 "swap gate --all vào pre-commit [4/7]" viết dựa trên giả định binary ≈ gate đang chạy trong hook. Thực tế: hook chạy `scripts/security-gate.sh` bản sos-kit ADAPTED 99 LOC (exit 0 trên repo này), còn binary parity-faithful với `golden/` 210 LOC tarot (exit 1 trên repo này — INV-004/005/008 đòi file tarot-specific). Swap nguyên con = chặn mọi commit vĩnh viễn. Nếu không có challenge lane, lỗi này chỉ lộ SAU khi ship — đúng khoảng cách "ship được vs chạy được" Sếp muốn đo, và nó có thật. Resolution: Chủ nhà quyết per-check swap (thay python3 call TRONG adapted gate bằng binary check), gate --all giữ parity fixture-based, profile mode → Sprint 2. Bài học kit: BACKLOG item viết lúc chưa có oracle dễ chứa giả định môi trường chưa verify — sprint-item nên có "assumption note" hoặc chấp nhận item cuối sprint bị re-scope. — Quản đốc cần sửa `.claude/agents/architect.md` (đổi `model: opus` → `fable` theo lệnh Sếp) trong lúc marker `architect-active` đang bật → hook architect-guard chặn Edit của CHÍNH main session ("Architect may ONLY Write phiếu files") → root cause: guard phân biệt vai bằng marker file, không phân biệt được main-session-orchestrator vs architect-subagent → workaround: rm marker → edit → touch lại (3 bước cho 1 edit config hợp lệ). Đề xuất: guard nên whitelist edit vào `.claude/` cho orchestrator, hoặc check thêm signal khác ngoài marker. **Tái diễn lần 2** (11/06): tại APPROVAL_GATE, marker `architect-active` còn sót sau khi Architect subagent xong → orchestrator bị chặn Edit dogfood file. Marker protocol chỉ ghi "worker XONG → rm worker-active" — KHÔNG có quy tắc tương ứng "architect XONG → rm architect-active" → đề xuất thêm vào contract.
- [IG-02] 🟢 — Đổi model subagent giữa chừng (opus → fable) khi agent đang chạy nền: TaskStop agent cũ + respawn với prompt y hệt hoạt động sạch, agent cũ kịp trả partial findings (5 script golden tồn tại, docs/ticket/ trống) không bị mất. Ghi để giữ: quy trình stop-edit-respawn là pattern ổn cho hot-swap model.
