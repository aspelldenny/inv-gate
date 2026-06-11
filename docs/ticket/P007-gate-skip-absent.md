# PHIẾU P007: `gate --all --skip-absent` — skip-with-note cho INV đòi file vắng (repo non-webapp)

> **Loại:** Feature (behavior change ĐẦU TIÊN sau parity — method rule 3 CLAUDE.md)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) đổi SEMANTICS của security gate (scan-target/skip policy = SECURITY surface, CLAUDE.md — **Giám sát review diff BẮT BUỘC trước merge**); (b) MCP tool `gate` đổi input schema (API contract client depend); (c) exit-code contract 0/1/2 phải giữ nguyên; (d) allowlist "INV nào được skip" là quyết định fail-closed — sai 1 dòng là gate hổng trên repo thật.
> **Ảnh hưởng:** `src/main.rs`, `src/gate.rs`, `src/serve.rs`, `tests/` (probes + MCP arg test), `.gitignore` (repo hygiene INV-004), `CHANGELOG.md`, `CLAUDE.md`, `docs/ARCHITECTURE.md`
> **Dependency:** P006 merged main `c9a6d76` (gate buffered-core + serve 5 tools, 84 tests xanh) `[unverified — Worker confirm anchor #1]`. Branch: `feat/P007-gate-skip-absent` từ `main`.

> *Note naming: BACKLOG item gọi "profile mode" và mở 2 option ("skip-with-note thay vì FAIL, **hoặc** `--profile <type>`"). Phiếu chốt **bool flag `--skip-absent`** — lý do §Giải pháp 1. Sprint exit-criterion BACKLOG ghi "`gate --all --profile` chạy sạch trên repo non-webapp" — thỏa bằng `gate --all --skip-absent` (cùng intent, tên flag là quyết định phiếu này; orchestrator note khi close-out).*

---

## Context

### Vấn đề hiện tại

`gate --all` là parity port của golden `security-gate.sh --mechanical-only` — viết cho TAROT (webapp Next.js + astro-service + docker). Trên repo non-webapp, một số INV FAIL chỉ vì file prerequisite không tồn tại. Live verdict trên chính repo này (P005 discovery §Informational + MCP `gate` 11/06, exit 1) `[verified — docs/discoveries/P005.md]`:

- **INV-004 FAIL** — `.gitignore` CÓ nhưng thiếu entries: `.env.production .env.staging .env.backup .env.local`
- **INV-005 FAIL** — không có `src/lib/sentry.ts` ("No beforeSend/beforeBreadcrumb")
- **INV-008 FAIL** — không có `docker-compose.yml` ("file not found")

Hệ quả: pre-commit của bất kỳ repo kit-family nào dùng `gate --all` sẽ block vĩnh viễn. Cần cơ chế opt-in cho repo non-webapp — nhưng KHÔNG được đổi default (golden parity, 84 tests + pins xanh nguyên) và KHÔNG được skip im lặng.

### Giải pháp

1. **Flag = bool `--skip-absent`** (Architect chốt, đối chiếu 4 tiêu chí):
   - vs `--profile <type>` enum: profile chế taxonomy (webapp/minimal/rust?) cho đúng 1 use case, định nghĩa profile thành security surface thứ hai phải review/version — vi phạm tiêu chí "đơn giản". REJECT.
   - vs `--missing-file=skip` option-valued: ngụ ý nhiều mode (`=fail`/`=warn`?) không ai cần; bool + default-fail-như-cũ là tối giản. REJECT.
   - `--skip-absent`: default (không flag) = byte-identical hành vi hiện tại (tiêu chí 1); 1 bool (tiêu chí 2); output SKIP LOUD (tiêu chí 3); chỉ skip khi prerequisite VẮNG, có-mà-fail vẫn FAIL (tiêu chí 4).
2. **Allowlist skip HARDCODED — chỉ INV-005 + INV-008** (bảng §Skip-eligibility dưới). KHÔNG generic "check nào kêu missing file thì skip" — INV mới muốn skippable phải opt-in qua phiếu riêng. Fail-closed by construction.
3. **INV-004 KHÔNG skippable** (điểm phải nghĩ kỹ): `.gitignore` thiếu entry ≠ "repo không cần" — repo Rust vẫn có thể chứa `.env` thật (MCP keys, test env); `.gitignore` vắng/thiếu = mọi `.env` tạo sau đều trackable + commit được. Bảo vệ này universal và fix cực rẻ (thêm entries) → **fix repo, không nới check**. Task 5 thêm entries vào `.gitignore` repo này — đó là cách acceptance (b) đạt exit 0.
4. **Cơ chế:** guard clause TRƯỚC logic hiện có của từng INV trong allowlist, gated trên flag: `flag && prerequisite vắng` (INV-005: prerequisite = CẢ 2 nguồn golden quét — **guard kép**, xem bảng + Task 1; INV-008: 1 file) → in dòng `  SKIP (...)` per-INV (stdout), đếm vào WARN (counter sẵn có — P005 discovery §2), ghi vào danh sách SKIPPED; ngược lại chạy logic cũ NGUYÊN BYTE. Flag đi qua buffered-core của gate (P006) → CLI và MCP cùng code path.
5. **MCP:** tool `gate` nhận optional arg `skip_absent: boolean` (default false). 4 tools còn lại không đổi. Response contract 4 fields (P006) không đổi — SKIP lines nằm trong `findings`.

### Skip-eligibility — bảng security review (Giám sát đọc bảng này trước tiên)

Cites golden = enumeration đã chốt parity P005 `[verified — docs/discoveries/P005.md §Enumeration]`. Logic chi tiết từng branch: Worker verify lại từ range khi EXECUTE.

| INV | Prerequisite | Golden khi prerequisite vắng | P007 với flag | Lý do an toàn |
|---|---|---|---|---|
| INV-001 port (script, golden `:53-55`) | `docker-compose.yml` | ĐÃ PASS ("no docker-compose.yml") `[verified — live run P005]` | KHÔNG ĐỔI | Golden đã pass-when-absent; không đụng `src/checks/port.rs` |
| INV-002 `:latest` (`:57-72`) | `docker-compose.yml` | PASS (grep không match) `[verified — live run: PASS]` | KHÔNG ĐỔI | Đã tolerant; Worker confirm từ golden range |
| INV-003 `.env.example` (`:74-86`) | `.env.example` | PASS ("no .env.example") `[verified — live run]` | KHÔNG ĐỔI | Đã tolerant |
| INV-004 gitignore+history (`:88-111`) | `.gitignore` + entries | FAIL khi entries thiếu | **KHÔNG SKIP — cố ý** | Universal: repo nào cũng có thể có `.env` thật; vắng/thiếu .gitignore = secrets trackable. Fix repo (Task 5), không nới check |
| INV-005 Sentry scrubber (`:113-123`) | **2 nguồn**: `src/lib/sentry.ts` VÀ glob `sentry.*.config.*` tại root — `inv_005()` quét CẢ HAI, mirror golden `:115-116` `[verified — Worker Turn 1, src/gate.rs:345-364]` | FAIL ("No beforeSend/beforeBreadcrumb") | **SKIP chỉ khi CẢ HAI nguồn vắng (guard kép — O1.1)** | Chỉ khi cả hai nguồn vắng mới chắc không có Sentry init tại các path golden biết → 0 coverage loss. Repo có `sentry.client.config.ts` (Sentry sống, có scrubber cần check) → VẪN CHẠY check, không SKIP oan (fail-closed, probe c). Nguồn nào CÓ mà thiếu `beforeSend` → FAIL như cũ |
| INV-006 CORS (`:125-136`) | `astro-service/app.py` | PASS ("no astro-service/app.py") `[verified — live run]` | KHÔNG ĐỔI | Đã tolerant |
| INV-007 SSH | — | SKIP HẲN trong mechanical-only (P005) | KHÔNG ĐỔI | Không reachable |
| INV-008 internal ports (`:176-193`) | `docker-compose.yml` | FAIL ("file not found") | **SKIP khi file vắng** | Golden chỉ đọc đúng file này `[verified — Worker Turn 1: absent-FAIL hiện đi stderr, stdout body rỗng → SKIP line thay thế sạch, không đụng stderr routing]` — không compose = compose không expose gì → 0 coverage loss. File CÓ mà internal service có `ports:` → FAIL như cũ |
| INV-009 secrets (`:195-197`) | source tree | luôn scan | **KHÔNG BAO GIỜ skip** | Universal — secrets không phụ thuộc loại repo |
| INV-010 runtime (`:199-201`) | `.git/config` + runtime state | luôn scan | **KHÔNG BAO GIỜ skip** | Universal — Sub-mech F |

→ Diff hành vi của flag = ĐÚNG 2 guard clause (INV-005 = guard kép 2 điều kiện, INV-008 = 1 điều kiện), cả hai trong `src/gate.rs`. `src/checks/*.rs` diff RỖNG.

### Scope

- CHỈ sửa/tạo: `src/main.rs`, `src/gate.rs`, `src/serve.rs`, `tests/` (probes mới — file mới `tests/gate_skip.rs` hoặc append `tests/parity_gate.rs`, Tầng 2; + `tests/mcp_serve.rs` cho arg), `.gitignore` (CHỈ thêm entries INV-004 đòi), `CHANGELOG.md`, `CLAUDE.md` (behavior deviation §Rules), `docs/ARCHITECTURE.md`, `docs/discoveries/P007.md`, `docs/DISCOVERIES.md`.
- KHÔNG sửa: `golden/**` (read-only tuyệt đối), `tests/golden/**` (pins/fixtures/repin.sh/MANIFEST — **profile ≠ default, KHÔNG pin mới, KHÔNG re-pin**), `src/checks/**` (diff rỗng — flag chỉ sống trong gate orchestrator), `hooks/pre-commit` + `scripts/**` (dogfood P005 nguyên trạng — hook KHÔNG đổi sang flag mới trong phiếu này), `Cargo.toml`/`Cargo.lock` (KHÔNG dep mới — schema arg handcraft, KHÔNG schemars; KHÔNG bump version — P008), `docs/BACKLOG.md`, `docs/PROJECT.md`.
- KHÔNG implement: profile enum/config file, skip cho INV ngoài allowlist (kể cả INV-004 — cố ý), đổi default behavior bất kỳ, wire flag vào hook/sos-kit (cross-repo — P008+/sos-kit side, ghi Discovery hook).

---

## Task 0 — Verification Anchors

> **Bước 0 capability:** không network. Cần: cargo + filesystem + sửa Rust/test/docs + raw JSON-RPC qua pipe (pattern `tests/mcp_serve.rs` P006 sẵn có). rmcp API tra từ crate source local (`~/.cargo/registry/src/`) — precedent P006, KHÔNG bịa API.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Baseline: main `c9a6d76` (P006 merged), `cargo test` = 84 tests xanh | `git log -1` + `cargo test` đầu phiếu | `[unverified — Worker confirm; số khác thì ghi Discovery, count mới = baseline regression]` |
| 2 | Golden INV-005 KHÔNG phân biệt file-vắng vs thiếu-scrubber → guard skip phải là check tồn-tại TƯỜNG MINH thêm vào | đọc `golden/security-gate.sh:113-123` + `src/gate.rs` fn INV-005 | ✅ **VERIFIED Turn 1** — `inv_005()` tại `src/gate.rs:345-364` quét **2 nguồn**: `src/lib/sentry.ts` VÀ glob `sentry.*.config.*` từ `read_dir(".")` (mirror golden `:115-116`) → guard phải là **GUARD KÉP** (O1.1, V2 Task 1) |
| 3 | Golden INV-008 `:176-193` CÓ branch file-not-found tường minh ("file not found" → FAIL) | đọc golden range + `src/gate.rs` fn INV-008 | ✅ **VERIFIED Turn 1** — chỉ đọc `docker-compose.yml`; absent-FAIL hiện đi **stderr**, stdout body rỗng → SKIP line (stdout) thay thế sạch, KHÔNG đụng stderr routing; zero coverage loss |
| 4 | INV-004 `:88-111`: danh sách entries `.gitignore` bắt buộc + phần history check cơ chế gì | đọc golden range | ✅ **VERIFIED Turn 1** — golden `:91-99`: ĐÚNG 4 entries (`.env.production .env.staging .env.backup .env.local`) → Task 5 thêm đúng 4 dòng, list live run = list đầy đủ |
| 5 | INV-002/003/006 đã pass-when-absent trong golden (không cần guard) | đọc golden `:57-72`, `:74-86`, `:125-136` | `[unverified — live run PASS cả 3; Worker confirm từng range]` `[oracle: live run + grep golden — SOUND]` |
| 6 | Gate buffered-core (P006): core fn trả `{stdout, stderr, code}`, CLI `run()` = wrapper mỏng; thread thêm 1 param bool là đủ cho cả CLI + MCP | grep `src/gate.rs` signature + call-sites (`src/main.rs`, `src/serve.rs`) | ✅ **VERIFIED Turn 1** — `run_core(skip_absent: bool)`, ĐÚNG 2 call sites: `gate::run()` pass `false`, `serve.rs` handler pass arg đã parse. Feasible, trong scope |
| 7 | `src/serve.rs` đăng ký tools qua `ToolRoute::new_dyn()`, input schema = JsonObject handcraft (không schemars) → thêm optional property `skip_absent` không cần dep mới | đọc `src/serve.rs` + rmcp 1.7.0 source (`Tool.input_schema` type) | ✅ **VERIFIED Turn 1** — `ToolCallContext.arguments: Option<JsonObject>`; đổi `_ctx`→`ctx`, parse `ctx.arguments.as_ref().and_then(|m| m.get("skip_absent"))`; non-bool → `isError` fail-closed. KHÔNG dep mới |
| 8 | `tests/mcp_serve.rs` harness raw JSON-RPC cho phép truyền `arguments` trong `tools/call` | đọc test harness P006 | ✅ **VERIFIED Turn 1** — harness extend bằng helper mới `call_tool_with_args` (Tầng 2) |
| 9 | WARN counter + summary format `Security gate: N passed, M failed, K warnings` đã có trong gate.rs (WARN hiện luôn 0 ở mechanical-only) | grep `src/gate.rs` summary | `[verified — P005 discovery §2 ghi rõ port tracks warn counter]` — Worker re-confirm vị trí. **Math summary verified Turn 1:** sau Task 5 + flag → `7 passed, 0 failed, 2 warnings` (7+0+2=9 sections, INV-007 vắng trong mechanical-only) — giữ làm acceptance string |
| 10 | `.gitignore` repo này hiện thiếu đúng 4 entries live run kêu; git history repo KHÔNG chứa `.env` file thật (phần history INV-004 pass) | `cat .gitignore` + chạy INV-004 sau khi thêm entries | ✅ **VERIFIED Turn 1** — history CLEAN (`git log --all --diff-filter=A -- '.env*'` rỗng) → Task 5 an toàn, không có finding bị che. Worker re-run xác nhận khi EXECUTE (rẻ) |
| 11 | Pins/parity không đổi: default path không flag = byte-identical | `cargo test` (84 cũ xanh nguyên) + `git diff tests/golden/` rỗng | ⏳ TO VERIFY khi EXECUTE `[oracle: cargo test — SOUND]` |
| 12 | Sau Task 5 + flag: live repo `gate --all --skip-absent` exit 0 (chỉ còn INV-005/008 skip) | chạy `target/release/inv-gate gate --all --skip-absent` tại repo root | ⏳ TO VERIFY — acceptance (b) |

**Anchor mở (Worker verify khi EXECUTE): #1, #5, #9 (vị trí), #11, #12. ✅ Verified Turn 1: #2, #3, #4, #6, #7, #8, #10. 0 anchor ❌.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P007
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp .gitignore ".backup/${PHIEU_ID}/.gitignore.orig"   # đường lui Task 5
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Turn 1 — O1.1 ACCEPT, guard kép INV-005)

### Turn 1 — Worker Challenge (relayed qua orchestrator)
- **[O1.1] BLOCKING — guard INV-005 V1 fail-open trên security surface:** `inv_005()` (`src/gate.rs:345-364`) quét 2 nguồn: `src/lib/sentry.ts` VÀ glob `sentry.*.config.*` từ `read_dir(".")` (mirror golden `:115-116`). Guard V1 chỉ check `!sentry.ts.exists()` → repo có `sentry.client.config.ts` (Sentry sống, có scrubber cần check) bị SKIP oan thay vì chạy check — vi phạm Constraint 4 (fail-closed). Đề xuất **option A: guard kép** (SKIP chỉ khi cả 2 nguồn vắng).
- Self-closed (facts verified, không cần Architect respond): (i) INV-008 zero coverage loss — absent-FAIL đi stderr, stdout body rỗng, SKIP line thay sạch; (ii) INV-004 golden list = đúng 4 entries (golden `:91-99`) + git history CLEAN; (iii) math summary `7 passed, 0 failed, 2 warnings` (7+0+2=9, INV-007 vắng); (iv) `run_core(skip_absent: bool)` — 2 call sites, feasible; (v) MCP `ctx.arguments.as_ref().and_then(|m| m.get("skip_absent"))` + helper test mới `call_tool_with_args` (Tầng 2).

**Status:** ✅ RESPONDED

### Turn 1 — Architect Response (phiếu V2)
- **[O1.1] → ACCEPT (option A của Worker).** Fail-open edge đúng loại lỗi mà allowlist fail-closed tồn tại để chặn. Fix V2 (scoped, không redesign): (1) Task 1 guard kép INV-005 — SKIP chỉ khi `src/lib/sentry.ts` vắng **VÀ** `read_dir(".")` không có file match `sentry.*.config.*`; (2) SKIP line INV-005 đổi thành `  SKIP (no sentry.ts / sentry.*.config.* present)`; (3) bảng Skip-eligibility cập nhật prerequisite = 2 nguồn; (4) probe (c) thêm case fail-closed: repo có `sentry.client.config.ts` nhưng không `sentry.ts` + flag bật → INV-005 VẪN CHẠY. INV-008 không đổi.
- Facts self-closed → fold vào anchors #2, #3, #4, #6, #7, #8, #10 (✅ VERIFIED Turn 1) + Task 1/3/4/5 + Nghiệm thu manual (giữ acceptance string `7 passed, 0 failed, 2 warnings`).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Next: orchestrator spawn Worker CHALLENGE verify consensus hoặc tiến APPROVAL_GATE.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

> Thứ tự: Task 1 (gate.rs core) → Task 2 (CLI flag) → Task 3 (MCP arg) → Task 4 (tests) → Task 5 (.gitignore hygiene) → Task 6 (docs) → Ship (Giám sát review).

### Task 1: Skip logic trong `src/gate.rs` (core — cả CLI lẫn MCP đi qua đây)

**File:** `src/gate.rs`

**Thêm (contract Tầng 1 — output bytes + semantics; tên param/fn/cách thread = Tầng 2):**

1. Core gate fn nhận thêm 1 bool (`run_core(skip_absent: bool)` — signature verified Turn 1, anchor #6). Default path (`false`) = code path hiện tại, **KHÔNG một byte output nào đổi** — mọi code mới nằm sau `if skip_absent`.
2. **Guard clause cho ĐÚNG 2 INV (allowlist — KHÔNG thêm INV nào khác):**
   - INV-005 — **GUARD KÉP (V2, O1.1):** nếu `skip_absent` VÀ `!Path("src/lib/sentry.ts").exists()` VÀ `read_dir(".")` không có entry nào match `sentry.*.config.*` → SKIP. Ngược lại logic cũ nguyên byte. Guard phải mirror đúng 2 nguồn `inv_005()` quét tại `src/gate.rs:345-364` / golden `:115-116` `[verified — Worker Turn 1]` — guard 1 nguồn = fail-open trên repo có `sentry.client.config.ts`, vi phạm Constraint 4.
   - INV-008: nếu `skip_absent && !Path("docker-compose.yml").exists()` → SKIP. Ngược lại logic cũ nguyên byte (anchor #3 verified — absent-FAIL hiện đi stderr, stdout body rỗng → SKIP line thay sạch, không đụng stderr routing).
3. **Output SKIP (LOUD — verbatim contract, per-INV):** section header in NGUYÊN như default path; thay body+`  PASS`/`  FAIL` bằng đúng 1 dòng (stdout, 2-space indent — cùng rhythm `  PASS`/`  FAIL`; blank line sau section như wrapper hiện tại):
   - INV-005: `  SKIP (no sentry.ts / sentry.*.config.* present)`
   - INV-008: `  SKIP (file docker-compose.yml absent)`
4. **Counters:** SKIP → `warn += 1`; KHÔNG vào pass, KHÔNG vào fail, KHÔNG vào `FAILED_INVS`. Push tên INV vào danh sách skipped mới.
5. **Summary:** dòng `Security gate: N passed, M failed, K warnings` giữ NGUYÊN format (K nay đếm skips). `Failed invariants: ...` giữ nguyên vị trí/điều kiện. Khi skipped > 0, in THÊM 1 dòng ngay sau (sau `Failed invariants` nếu có):
   ```
   Skipped invariants: INV-005 INV-008
   ```
   Dòng này chỉ reachable khi flag bật → pins không thể đụng.
6. **Exit code:** không đổi — `FAIL > 0` → 1, ngược lại 0. Skip KHÔNG bao giờ tự gây exit 1. Contract 0/1/2 nguyên (CLAUDE.md).
7. **Invariant mạnh:** flag bật nhưng mọi prerequisite ĐỀU tồn tại → output byte-identical với không-flag (probe d Task 4 enforce).

**Lưu ý:**
1. Cite comment: mỗi guard ghi `// P007 --skip-absent: golden/security-gate.sh:<range> demands <file(s)>; absent => SKIP (allowlist — see docs/ticket/P007)`. Guard INV-005 cite cả 2 nguồn (golden `:115-116`).
2. Muốn skip thêm INV nào khác "tiện thể" → DỪNG, Debate Log. Allowlist là quyết định security, không mở rộng im lặng.

### Task 2: CLI flag — `src/main.rs`

**File:** `src/main.rs`

**Thêm:** arg bool `--skip-absent` trên variant `Gate` (cạnh `--all`), default false. Doc-comment: `/// Skip allowlisted INVs (INV-005, INV-008) whose prerequisite file is absent — prints SKIP note, counts as warning. Default: off (golden parity).` Dispatch thread bool xuống `run_core` (anchor #6 verified — `gate::run()` là call site pass `false` cho path cũ).

**Lưu ý:** `gate --skip-absent` thiếu `--all` → vẫn usage error exit 2 (clap, `--all` required như cũ). Unknown flag → exit 2 như cũ. KHÔNG đụng `Check`/`Serve` variants.

### Task 3: MCP arg — `src/serve.rs`

**File:** `src/serve.rs`

1. Tool `gate`: input schema từ object rỗng → object có ĐÚNG 1 optional property (handcraft JsonObject — KHÔNG schemars, KHÔNG dep mới):
   ```json
   {"type":"object","properties":{"skip_absent":{"type":"boolean","description":"Skip allowlisted INVs (INV-005, INV-008) whose prerequisite file is absent. Default false (golden parity)."}},"required":[]}
   ```
2. Handler: đọc `arguments.skip_absent` — vắng/null → `false`; là bool → dùng; **sai type → MCP tool error (`isError: true` + message), KHÔNG silently default** (fail-closed). Pattern verified Turn 1 (anchor #7): `ToolCallContext.arguments: Option<JsonObject>` — đổi `_ctx`→`ctx`, parse `ctx.arguments.as_ref().and_then(|m| m.get("skip_absent"))`.
3. Tool description của `gate` append 1 câu document arg. 4 tools còn lại: KHÔNG đổi schema/description/handler.
4. Response contract 4 fields (`exit_code`/`is_clean`/`findings`/`stderr`) KHÔNG đổi — SKIP lines + `Skipped invariants` nằm trong `findings` verbatim.

**Lưu ý:** field name `skip_absent` = API contract (Tầng 1) — khớp CLI flag. Cách parse/serde nội bộ = Tầng 2 (anchor #7).

### Task 4: Tests — probes skip + MCP arg

**File:** `tests/gate_skip.rs` (MỚI) hoặc append `tests/parity_gate.rs` (Tầng 2 — Worker chọn; assertions parity cũ KHÔNG đổi); `tests/mcp_serve.rs` (append — extend bằng helper mới `call_tool_with_args`, Tầng 2, anchor #8 verified).

Fixtures = synthetic in-code (F07 — KHÔNG invent file trong `tests/golden/fixtures/`). Dựng "minimal repo" hermetic: git repo + `.gitignore` đủ entries INV-004 + KHÔNG sentry.ts + KHÔNG `sentry.*.config.*` + KHÔNG docker-compose.yml + source sạch (reuse/adapt harness `tests/parity_gate.rs`, cấu trúc helper Tầng 2; `env_remove("ALLOW_DATA_LOSS")`, LF, fixed dates — precedent P002/P005).

**Probes BẮT BUỘC:**
- (a) minimal repo + flag → INV-005 có dòng `  SKIP (no sentry.ts / sentry.*.config.* present)`, INV-008 có dòng `  SKIP (file docker-compose.yml absent)`; summary `... 2 warnings`; có dòng `Skipped invariants: INV-005 INV-008`; **exit 0**.
- (b) minimal repo KHÔNG flag → INV-005/INV-008 FAIL, exit 1 (default không đổi trên repo thiếu file).
- (c) **fail-closed per-INV:**
  - sentry.ts TỒN TẠI nhưng thiếu `beforeSend` + flag → INV-005 vẫn FAIL;
  - **(c2 — guard kép, O1.1):** repo có `sentry.client.config.ts` nhưng KHÔNG có `src/lib/sentry.ts` + flag bật → INV-005 **VẪN CHẠY** (không SKIP; FAIL vì thiếu scrubber);
  - docker-compose.yml TỒN TẠI có internal-service `ports:` + flag → INV-008 vẫn FAIL ("file có mà fail thì VẪN fail dù có flag").
- (d) **flag no-op khi đủ file:** fixture union webapp (harness parity gate, đủ mọi prerequisite) chạy flag vs không-flag → stdout VÀ stderr **byte-identical** (so sánh 2 run trực tiếp).
- (e) **INV-004 không skippable:** `.gitignore` thiếu entry + flag → INV-004 vẫn FAIL, exit 1.
- (f) skipped > 0 nhưng có INV khác FAIL → exit vẫn 1, có CẢ `Failed invariants` lẫn `Skipped invariants` đúng thứ tự.
- (g) MCP (`tests/mcp_serve.rs`): `tools/call gate` với `arguments {"skip_absent": true}` trên minimal fixture → `exit_code` 0, `is_clean` true, `findings` chứa SKIP lines; cùng tool KHÔNG arguments trên cùng fixture → `exit_code` 1 (backward compat); arg sai type (vd `"yes"`) → `isError` true.
- Regression: 84 tests cũ xanh NGUYÊN (pins byte-exact — flag không reachable trong parity runs).

**Lưu ý:** test đỏ → sửa `gate.rs`/`serve.rs`/harness — KHÔNG sửa pins/fixtures/repin.sh (Luật chơi 2). Mapping probe→cơ chế ghi Discovery (precedent P003-P005).

### Task 5: Repo hygiene — `.gitignore` (INV-004, đường đến acceptance (b))

**File:** `.gitignore` (repo này — KHÔNG liên quan fixtures)

1. Danh sách entries INV-004 đòi = ĐÚNG 4 entries (golden `:91-99`, anchor #4 ✅ verified Turn 1): `.env.production .env.staging .env.backup .env.local` — Task 5 thêm đúng 4 dòng thiếu vào `.gitignore` (đối chiếu hiện trạng trước khi thêm, không duplicate).
2. History INV-004: ✅ verified Turn 1 CLEAN (`git log --all --diff-filter=A -- '.env*'` rỗng, anchor #10) — Worker re-run xác nhận khi EXECUTE (rẻ). **Nếu re-run ra khác (history chứa `.env` thật) → DỪNG, Debate Log/escalate — đó là finding thật, không che.**
3. Sau task: `target/release/inv-gate gate --all` (KHÔNG flag) trên repo này → INV-004 PASS, chỉ còn INV-005/INV-008 FAIL (exit 1 — default vẫn đúng nguyên tắc); thêm flag → exit 0. Cả 2 transcript vào Discovery.

### Task 6: Docs gate

**File:** `CHANGELOG.md` — entry P007 (Unreleased): behavior change đầu tiên sau parity (method rule 3): flag opt-in `gate --all --skip-absent`, allowlist INV-005/INV-008 skip-when-absent (SKIP note + warning, fail-closed: file-present-but-failing vẫn FAIL; INV-005 guard kép 2 nguồn sentry; INV-004/009/010 không bao giờ skip), MCP tool `gate` optional arg `skip_absent`; default = golden parity nguyên (pins untouched). KHÔNG bump version (F13 — P008).
**File:** `CLAUDE.md` §Rules — 1-2 dòng deviation: `gate --all --skip-absent` = opt-in deviation khỏi golden (skip allowlist INV-005/INV-008 khi prerequisite vắng); default KHÔNG flag = golden `--mechanical-only` parity như cũ.
**File:** `docs/ARCHITECTURE.md` — gate.rs: flag `--skip-absent` + allowlist (INV-005 guard kép) + WARN/`Skipped invariants` semantics; serve.rs: tool `gate` input schema 1 optional property. Data flow note: non-webapp repo dùng flag.
**File:** `docs/discoveries/P007.md` + 1-line index `docs/DISCOVERIES.md` — gồm: (i) anchors ĐÚNG/SAI (đặc biệt #2 guard kép O1.1, #4 list entries, #10 history verdict re-run); (ii) transcript live repo: default exit 1 (INV-005/008) + flag exit 0 với SKIP notes + MCP call với `skip_absent: true`; (iii) probe→cơ chế mapping (gồm c2 guard kép); (iv) hooks cho P008/sos-kit: pre-commit `[4/7]` cross-repo sẽ dùng `gate --all --skip-absent` (việc sos-kit, ngoài scope); (v) tier escalations ("None" nếu không).

---

## Ship (IG-09 + Giám sát review — SECURITY SURFACE)

1. Commit trên `feat/P007-gate-skip-absent` — hook pre-commit chạy thật (KHÔNG `--no-verify`); `cargo build --release` trước commit (dogfood guard P005).
2. **GIÁM SÁT REVIEW DIFF — BẮT BUỘC TRƯỚC MERGE** (orchestrator chạy): trọng tâm review = bảng Skip-eligibility (allowlist đúng 2 INV? guard INV-005 đủ KÉP — cả 2 nguồn? guard chỉ fire khi prerequisite VẮNG? INV-004/009/010 không đụng? default path zero-diff? `src/checks/` diff rỗng?). Verdict ghi vào Discovery.
3. Sau review pass + nghiệm thu: merge `main` → push `origin main` → XÓA branch ngay (local + remote). Không stack.
4. BACKLOG checkbox + sprint note ("--profile" wording exit-criterion → thỏa bằng `--skip-absent`) = việc ORCHESTRATOR.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/gate.rs` | Task 1: param bool + 2 guard clause (INV-005 guard kép / INV-008) + SKIP lines per-INV + warn count + `Skipped invariants` |
| `src/main.rs` | Task 2: flag `--skip-absent` trên `Gate` + thread xuống `run_core` |
| `src/serve.rs` | Task 3: tool `gate` optional arg `skip_absent` (handcraft schema, fail-closed type check, `ctx.arguments` pattern) |
| `tests/gate_skip.rs` (hoặc append `parity_gate.rs`) | Task 4: probes (a)-(f) gồm (c2) guard kép |
| `tests/mcp_serve.rs` | Task 4 (g): helper `call_tool_with_args` + arg test + backward-compat + isError |
| `.gitignore` | Task 5: CHỈ thêm đúng 4 entries INV-004 đòi |
| `CHANGELOG.md`, `CLAUDE.md`, `docs/ARCHITECTURE.md` | Task 6 |
| `docs/discoveries/P007.md` + `docs/DISCOVERIES.md` | Discovery report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng |
| `tests/golden/**` (pins, fixtures, repin.sh, MANIFEST) | Diff rỗng — profile ≠ default, KHÔNG pin mới/re-pin; parity đỏ → sửa Rust |
| `src/checks/*.rs`, `src/checks/mod.rs` | Diff RỖNG TUYỆT ĐỐI — flag chỉ sống trong gate orchestrator |
| `hooks/pre-commit`, `scripts/**` | Dogfood P005 nguyên trạng — diff rỗng; hook KHÔNG đổi sang flag (cross-repo = sos-kit side) |
| `Cargo.toml`, `Cargo.lock` | Diff rỗng — KHÔNG dep mới (schema handcraft), KHÔNG bump version (P008) |
| `docs/BACKLOG.md`, `docs/PROJECT.md` | Orchestrator/Chủ nhà-only |

---

## Luật chơi (Constraints)

1. **Default bất khả xâm phạm:** không flag = byte-identical hiện tại — 84 tests + pins xanh NGUYÊN, mọi code mới sau `if skip_absent`. Probe (d) enforce no-op khi đủ file.
2. **Pin/oracle untouched:** parity đỏ → sửa Rust/harness; sửa pins/fixtures/repin.sh = vi phạm scope.
3. **Allowlist đóng:** chỉ INV-005 + INV-008 skippable; INV-004/009/010 KHÔNG BAO GIỜ; thêm INV vào allowlist = Debate Log/phiếu mới, không tự quyết.
4. **Fail-closed:** skip CHỈ khi prerequisite VẮNG — INV-005 = CẢ 2 nguồn vắng (guard kép, V2/O1.1); nguồn/file có mà check fail → FAIL nguyên. Skip LOUD: SKIP line per-INV + WARN count + `Skipped invariants` line — KHÔNG skip im lặng. MCP arg sai type → isError, không default ngầm.
5. **Exit contract:** 0/1 từ gate logic (skip không đổi exit), 2 chỉ từ clap. Đây là API pre-commit depend (CLAUDE.md).
6. **KHÔNG dep mới, KHÔNG version bump** (F13 — P008). Schema MCP handcraft theo pattern P006.
7. **API contract (Tầng 1):** flag name `--skip-absent`, MCP arg `skip_absent`, SKIP line format per-INV (Task 1.3 verbatim), `Skipped invariants` line, response 4 fields — đúng phiếu, lệch = Debate Log. Tên fn/param nội bộ, vị trí guard trong wrapper, file test, helper test = Tầng 2.
8. **Giám sát review bắt buộc trước merge** (security surface — CLAUDE.md). Không merge khi chưa có verdict.
9. Test hermetic: `env_remove("ALLOW_DATA_LOSS")`, LF, fixed dates; `cargo test` xanh TOÀN BỘ trước commit.
10. Cite range, không count (IG-04); guard comments cite golden range tương ứng.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — 84 cũ NGUYÊN (parity byte-exact, pins untouched) + probes (a)-(g) gồm (c2) guard kép
- [ ] Probe fail-closed (c) + (c2 — sentry.client.config.ts hiện diện → INV-005 vẫn chạy) + INV-004-not-skippable (e) + flag-no-op-byte-identical (d) pass
- [ ] MCP: `gate` với `{"skip_absent": true}` → exit_code 0 + SKIP trong findings; không args → behavior cũ; sai type → isError
- [ ] `git diff golden/ tests/golden/ src/checks/ hooks/ scripts/ Cargo.toml Cargo.lock` RỖNG
- [ ] `cargo build --release` sạch

### Manual Testing
- [ ] Repo này: `target/release/inv-gate gate --all` → exit 1, fail = INV-005 INV-008 (INV-004 PASS sau Task 5) — transcript Discovery
- [ ] Repo này: `target/release/inv-gate gate --all --skip-absent` → **exit 0**, dòng `  SKIP (no sentry.ts / sentry.*.config.* present)` (INV-005) + `  SKIP (file docker-compose.yml absent)` (INV-008) + `Security gate: 7 passed, 0 failed, 2 warnings` (math verified Turn 1: 7+0+2=9 sections, INV-007 vắng) + `Skipped invariants: INV-005 INV-008` — transcript Discovery
- [ ] MCP live: serve với cwd = repo này, `tools/call gate` `{"skip_absent": true}` → `exit_code` 0 — transcript Discovery
- [ ] `cargo run -- gate --skip-absent` (thiếu `--all`) → exit 2; `gate --all --no-such-flag` → exit 2

### Regression
- [ ] `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree (dogfood P005 nguyên — hook không đổi)
- [ ] `bash tests/golden/repin.sh` chạy + `git diff tests/golden/pins/` rỗng sau khi chạy
- [ ] `cargo run -- check secrets|runtime|port|schema` behavior nguyên

### Docs Gate
- [ ] `CHANGELOG.md` — entry P007 (behavior change đầu tiên sau parity + allowlist guard kép + MCP arg)
- [ ] `CLAUDE.md` §Rules — deviation note flag opt-in
- [ ] `docs/ARCHITECTURE.md` — gate flag + serve schema

### Discovery Report
- [ ] `docs/discoveries/P007.md`: anchors ĐÚNG/SAI (#2 guard kép O1.1, #4, #10 re-run), transcripts (default exit 1 / flag exit 0 / MCP), probe mapping (gồm c2), Giám sát verdict, hooks sos-kit, tier escalations ("None" nếu không)
- [ ] Append 1-line index `docs/DISCOVERIES.md`
