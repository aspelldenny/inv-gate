# PHIẾU P006: `serve` — MCP stdio (rmcp), 5 tools wrap check/gate in-process

> **Loại:** Feature (Phase 3 MCP — dual-mode vision PROJECT.md)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) MCP tools = API surface MỚI public (tool names + response schema là contract client depend); (b) dep ngoài ĐẦU TIÊN (`rmcp` + `tokio`) trên binary security-surface → supply-chain, Giám sát review PR bắt buộc (CLAUDE.md); (c) refactor chạm cả 4 check + gate (toàn bộ security surface đã port) — parity byte-exact là invariant; (d) exit-code contract 0/1/2 expose qua MCP (`exit_code` field).
> **Ảnh hưởng:** `Cargo.toml`/`Cargo.lock` (dep mới), `src/main.rs`, `src/serve.rs` (MỚI), `src/checks/{secrets,runtime,port,schema}.rs` + `mod.rs` (refactor buffered-core, behavior-neutral), `src/gate.rs` (refactor buffered-core), `tests/mcp_serve.rs` (MỚI), `CHANGELOG.md`, `docs/ARCHITECTURE.md`
> **Dependency:** Sprint 1 merged main `6828ecc` (P001-P005: 4 check + gate, 79 tests xanh) `[unverified — Worker confirm anchor #9]`. Branch: `feat/P006-serve-mcp` từ `main`.

> *Note scope: BACKLOG Active sprint (Rule 0, single source) đặt P006 = `serve` MCP. PROJECT.md §Roadmap (placeholder) đánh số khác (P009-P010) — BACKLOG thắng (precedent P003-P005). PROJECT.md ghi "8 tools" (roadmap cũ) — BACKLOG item chốt **5 tools**, BACKLOG thắng.*

---

## Context

### Vấn đề hiện tại

CLI hoàn chỉnh (Sprint 1): 4 check (`src/checks/{secrets,runtime,port,schema}.rs`) + `gate --all` (`src/gate.rs`) đều là function in-process, clap dispatch trong `src/main.rs`, 79 tests xanh parity byte-exact. Chưa có MCP mode — vision PROJECT.md là dual mode: pre-commit hook (CLI) + agent gọi tool (MCP), "em call 1 lệnh thay vì 5 path lookup".

**Hazard trung tâm (lý do phiếu này có refactor):** các check hiện print findings THẲNG ra stdout/stderr của process và return exit code `[unverified — per P005 anchor #6 + ARCHITECTURE; Worker confirm anchor #3-4]`. MCP stdio dùng **stdout làm kênh JSON-RPC** — 1 byte ngoài protocol lọt ra stdout khi đang serve là hỏng framing. Vậy refactor mức tối thiểu: mỗi check/gate có **buffered core** (findings vào buffer, không side-effect stream thật); CLI path in buffer ra đúng stream như cũ (byte-exact — cùng string); MCP path đóng gói buffer vào tool response.

### Giải pháp

1. **Task 1 — refactor buffered-core (behavior-neutral):** 5 entry (4 check + gate) tách thành core fn trả `{stdout buffer, stderr buffer, exit code}`; `run()` CLI hiện tại thành wrapper mỏng `print!`/`eprint!` buffer + return code. KHÔNG duplicate format string — đổi chỗ ghi, không đổi nội dung. Parity P002-P005 phải xanh NGUYÊN (pins assert per-stream nên buffer-rồi-in giữ byte-exact).
2. **Task 2 — dep mới:** `rmcp` + `tokio` (PROJECT.md stack), **pin exact `=`**, lý do trong phiếu + CHANGELOG. Worker KHÔNG có web — API tra từ crate source local sau `cargo add` (`~/.cargo/registry/src/`) hoặc `cargo doc -p rmcp --no-deps`. Phiếu cố ý KHÔNG prescribe tên API rmcp (anchor #2 mở, escape hatch Task 4).
3. **Task 3 — clap:** subcommand `serve` (không flag). Chạy rmcp stdio server trên tokio runtime, block đến khi client đóng stdin → exit 0.
4. **Task 4 — `src/serve.rs`:** 5 tools `check_secrets` / `check_runtime` / `check_port` / `check_schema` / `gate` — mỗi tool gọi ĐÚNG core fn CLI dùng (in-process, KHÔNG spawn binary, KHÔNG duplicate logic). Response contract (Tầng 1, chốt tại phiếu — xem Task 4). Tools không nhận argument; scan theo **cwd của process** — client phải launch server với cwd = repo cần quét (đúng contract CLI, MANIFEST §2 "CWD = repo root").
5. **Task 5 — integration test stdio thật:** spawn binary `serve`, nói JSON-RPC qua stdin/stdout (initialize → tools/list → tools/call), fixture dirty/clean dựng từ harness parity hiện có, assert kết quả tool **khớp CLI** chạy trên cùng fixture.
6. **Ship (IG-09 — lệnh Chủ nhà):** xong nghiệm thu = commit → merge `main` → push → **XÓA branch NGAY** (local + remote nếu có). Không stack.

### Scope

- CHỈ sửa/tạo: `Cargo.toml` + `Cargo.lock` (dep mới pinned), `src/main.rs`, `src/serve.rs` (MỚI), `src/checks/{secrets,runtime,port,schema}.rs` + `src/checks/mod.rs` (buffered-core refactor ONLY), `src/gate.rs` (buffered-core refactor ONLY), `tests/mcp_serve.rs` (MỚI), `tests/common/` hoặc helper extraction nếu cần (precedent P004/P005 — Tầng 2), `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/discoveries/P006.md`, `docs/DISCOVERIES.md`.
- Sửa `tests/parity_*.rs` CHỈ khi signature đổi buộc sửa import/call-site — **assertions parity (pin, byte-exact) KHÔNG được đổi**.
- KHÔNG sửa: `golden/**`, `tests/golden/**` (pins/fixtures/repin.sh/MANIFEST — không có pin mới, MCP không có golden oracle), `hooks/pre-commit`, `scripts/**` (dogfood P005 nguyên trạng), `docs/BACKLOG.md`.
- KHÔNG implement: HTTP/SSE transport (stdio only), tool arguments (chọn path/target — cwd contract là đủ), JSON output cho CLI (roadmap cũ PROJECT.md — không thuộc BACKLOG item này), profile mode (P007), version bump + release (P008), pattern/behavior change bất kỳ trong check logic.

---

## Task 0 — Verification Anchors

> **Bước 0 capability:** Worker KHÔNG web access. Cần: cargo network tới crates.io (cho `cargo add` — verify ngay anchor #1), crate source local sau add, `std::process` spawn + pipe stdin/stdout trong test (std đủ, không cần dep test mới). Nếu `cargo add rmcp` fail vì network → DỪNG, escalate (phiếu không thực thi được offline-hoàn-toàn).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `rmcp` resolve được từ crates.io. PROJECT.md ghi "rmcp 1.7.0" — version có thể STALE (viết 2026-05-28) | `cargo add rmcp` → đọc version thật trong `Cargo.toml`/`Cargo.lock`; ghi Discovery | ✅ rmcp 1.7.0 — PROJECT.md ĐÚNG. NHƯNG: dep đã có sẵn trong Cargo.toml (version `"1.7.0"`, chưa pin exact `"=1.7.0"`). Xem O1.1. |
| 2 | rmcp API surface: stdio transport + cách declare server/tools (ServerHandler trait? tool macro? router?) — **phiếu cố ý KHÔNG assume tên API** | đọc crate source `~/.cargo/registry/src/*/rmcp-<ver>/` (examples/ nếu có) + `cargo doc -p rmcp --no-deps` | ✅ Xác định: `ServerHandler` trait + `#[tool_router]`/`#[tool_handler]` macros (feature `macros` = default) + `ServiceExt::serve(transport)` + `.waiting()` block đến shutdown + `transport::io::stdio()` → `(tokio::io::Stdin, tokio::io::Stdout)`. |
| 3 | 4 check `run()` print thẳng stdout/stderr (`println!`/`eprintln!`/`print!`) và RETURN code, KHÔNG `process::exit` bên trong | `grep -n "println!\|eprintln!\|print!\|process::exit" src/checks/*.rs` | ✅ ĐÚNG: secrets.rs, runtime.rs, schema.rs dùng `println!` trả về i32. port.rs dùng `println!` + `eprintln!` (WARN + error), không có `process::exit`. Không có `process::exit` trong bất kỳ check nào. |
| 4 | `gate::run()` tương tự: print thẳng (section headers + summary), gọi `checks::{port,secrets,runtime}::run()` in-process, return code | grep `src/gate.rs` | ✅ ĐÚNG: gate.rs dùng `println!` cho headers/PASS/FAIL/summary + `writeln!(err, ...)` trong inv_008 stderr path. Gọi port::run()/secrets::run()/runtime::run() in-process. Return i32. Không có `process::exit`. |
| 5 | `src/main.rs` dispatch pattern: `run()` → `std::process::exit(code)`; chưa có variant `Serve` | grep `src/main.rs` | ✅ ĐÚNG: chỉ có `Check`/`Gate`, kết thúc `std::process::exit(exit_code)`. Không có `Serve`. |
| 6 | `Cargo.toml` hiện CHƯA có `rmcp`/`tokio`; `serde_json` đang là dev-dep (P005 dùng cho exit_codes.json); serde derive chưa chắc có ở deps thường | đọc `Cargo.toml` | ❌ GIẢ ĐỊNH SAI: `rmcp = "1.7.0"` + `tokio` + `serde` + `serde_json` + `anyhow` + `thiserror` đã có trong `[dependencies]` (KHÔNG phải dev-dep). Task 2 phần "add deps" đã hoàn thành. Chỉ còn pin exact. Xem O1.1. |
| 7 | rmcp stdio cần tokio runtime (PROJECT.md stack: "tokio — only for MCP serve"); feature set tối thiểu nào | đọc `rmcp` Cargo.toml + examples trong crate source | ✅ Cargo.toml đã có: `tokio = { version = "1", features = ["rt", "macros", "io-std"] }` + `rmcp = { version = "1.7.0", features = ["server", "transport-io"] }`. `transport-io` kéo `tokio/io-std` (stdio) + `transport-async-rw` (codec). Đủ cho stdio server. |
| 8 | Harness reuse: `build_fixture_repo()` trong `tests/parity_runtime.rs` (hoặc đã extract `tests/common/` ở P005); harness union gate trong `tests/parity_gate.rs`; harness schema trong `tests/parity_schema.rs` | grep `tests/` | ✅ Không có `tests/common/` — mỗi parity_*.rs tự chứa harness helpers. `build_fixture_repo()` nằm trong `parity_runtime.rs`; gate harness trong `parity_gate.rs`; schema build helpers trong `parity_schema.rs`. Reuse = copy hoặc extract (Tầng 2). |
| 9 | Baseline: main `6828ecc`, `cargo test` = 79 tests xanh TRƯỚC khi sửa | `git log -1` + `cargo test` đầu phiếu | ⚠️ HEAD thật = `8e649dc` (không phải `6828ecc` như phiếu ghi — 2 commit sau). `cargo test` = 79 tests xanh (55+16+2+2+2+2). Số test ĐÚNG, SHA sai — phiếu lỗi thời. |
| 10 | rmcp có client util / test helper dùng được cho integration test | đọc crate source/examples | ✅ rmcp tests dùng `tokio::io::duplex()` (in-process pair transport). Cho binary spawn test: fallback = `std::process::Command` + piped stdin/stdout + raw JSON-RPC line — đủ, không cần dev-dep mới. |
| 11 | Refactor không đổi CLI surface → dogfood hook P005 (`scripts/security-gate.sh` gọi `target/release/inv-gate check ...`) vẫn chạy | sau refactor: `cargo build --release` + `bash scripts/security-gate.sh --mechanical-only; echo $?` trên clean tree → exit 0 | ⏳ TO VERIFY — trước proof-commit (pre-execution, không verify trong CHALLENGE mode) |

**Anchor ⚠️ mở (Worker verify khi EXECUTE): #1-#11 — phiếu này nhiều anchor mở hơn P002-P005 vì dep mới + API crate Worker phải tự đọc. 0 anchor ❌.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=P006
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp Cargo.toml ".backup/${PHIEU_ID}/Cargo.toml.orig"   # đường lui dep
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- #1 ✅ rmcp 1.7.0 đúng, but ⚠️ dep đã sẵn — see O1.1
- #2 ✅ API surface xác định: ServerHandler + tool_router/tool_handler macros + ServiceExt::serve + transport::io::stdio()
- #3 ✅ 4 checks: println!/eprintln!, return i32, no process::exit
- #4 ✅ gate.rs: println! direct, in-process calls, return i32, no process::exit
- #5 ✅ main.rs: Check/Gate only, std::process::exit at bottom
- #6 ❌ ASSUMPTION WRONG — rmcp + tokio + serde + serde_json already in [dependencies]
- #7 ✅ Features confirmed: tokio ["rt","macros","io-std"] + rmcp ["server","transport-io"] already present
- #8 ✅ No tests/common/ — helpers inline in each parity_*.rs
- #9 ⚠️ HEAD = 8e649dc (not 6828ecc), 79 tests green (count correct, SHA stale)
- #10 ✅ std::process + raw JSON-RPC pipe sufficient; no new dev-dep needed
- #11 ⏳ pre-execution verify (not applicable in CHALLENGE mode)

**Objections:**

- [O1.1] **Task 2 assumption về Cargo.toml là SAI** — `rmcp = "1.7.0"`, `tokio = { version = "1", features = ["rt", "macros", "io-std"] }`, `serde`, `serde_json`, `anyhow`, `thiserror` đã có trong `[dependencies]` của `Cargo.toml` (commit 8e649dc, `Cargo.toml:14-23`). Task 2 mô tả "dep mới, `cargo add rmcp`, pin exact" như thể chưa có gì — nhưng thực ra deps đã được thêm vào repo trước khi P006 CHALLENGE. Cái còn thiếu DUY NHẤT là pin exact (`"1.7.0"` → `"=1.7.0"` và `"1"` → `"=1.52.3"` cho tokio, tương tự serde/serde_json).
  - Claim: Cargo.toml có rmcp + tokio sẵn trong [dependencies] rồi không?
  - Oracle: `grep -E "rmcp|tokio|serde" Cargo.toml` — SOUND (grep kết quả dứt khoát)
  - Soundness: SOUND
  - Verdict: self-closed via oracle ✅ — rmcp=1.7.0, tokio với features ["rt","macros","io-std"], serde, serde_json đều có sẵn trong [dependencies] tại `Cargo.toml:14-23`. Task 2 cần điều chỉnh: KHÔNG "cargo add" (sẽ fail vì đã có), chỉ cần sửa version string → pin exact.

- [O1.2] **Task 2 ordering constraint có thể bỏ** — phiếu nói "Task 1 (refactor) TRƯỚC khi thêm dep (Task 2)" để test xanh trước. Nhưng vì deps đã có sẵn, thứ tự vẫn đúng (Task 1 trước Task 3-4), tuy nhiên "pin exact" của Task 2 có thể làm song song với Task 1 vì chỉ là edit string trong Cargo.toml, không ảnh hưởng compilation.
  - Claim: pin exact có break compilation không?
  - Oracle: pin `"=1.7.0"` cho rmcp sẽ resolve thành version đúng đã có trong Cargo.lock (1.7.0) → không break.
  - Soundness: SOUND (Cargo.lock đã lock 1.7.0)
  - Verdict: self-closed ✅ — không cần Architect respond, Worker ghi Discovery khi EXECUTE.

- [O1.3] **Anchor #9 SHA stale** — phiếu ghi baseline commit `6828ecc`, thực tế HEAD = `8e649dc` (2 commit sau, theo git log: sos-kit docs + feedback commits). 79 tests count ĐÚNG; SHA chỉ là informational trong phiếu (không block execution).
  - Claim: 79 tests xanh? SHA mismatch có block không?
  - Oracle: `cargo test 2>&1 | grep "^test result"` → 6 suites, tổng 79 passed. SHA ≠ baseline nhưng test count đúng.
  - Soundness: SOUND (test runner output dứt khoát)
  - Verdict: self-closed ✅ — ghi Discovery correction: "baseline commit thật = 8e649dc, tests = 79 xanh".

**Summary — tất cả objections self-closed via oracle. Không cần Architect respond.**

Những điều Worker cần điều chỉnh khi EXECUTE (Tầng 2, không cần escalate):
1. Task 2: KHÔNG chạy `cargo add rmcp` (đã có). Thay vào đó: sửa `Cargo.toml` trực tiếp → pin exact `=1.7.0` (rmcp), `=1.52.3` (tokio resolved version), `=1.x.y` (serde/serde_json theo Cargo.lock). Commit Cargo.lock unchanged (đã lock đúng).
2. Task 2 CHANGELOG: vẫn ghi lý do pin exact, nhưng ghi "updated existing deps to pin exact" thay vì "added deps".
3. Task 5 integration test: dùng `std::process::Command` + piped raw JSON-RPC — no new dev-dep (assert_cmd đã có trong dev-deps, có thể dùng).
4. Discovery: ghi correction anchor #9 SHA, anchor #6 assumption wrong, rmcp API surface (ServerHandler + macros).

**Status:** ✅ Worker challenge hoàn thành — ready for Chủ nhà approval gate (không cần Architect respond vì tất cả objections self-closed).

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

> Thứ tự thực thi: Task 1 (refactor, tests xanh lại TRƯỚC khi thêm dep) → Task 2 (dep + đọc API) → Task 3-4 (serve) → Task 5 (test) → Task 6 (docs) → Ship.

### Task 1: Refactor buffered-core — 4 check + gate (behavior-neutral)

**File:** `src/checks/{secrets,runtime,port,schema}.rs`, `src/checks/mod.rs`, `src/gate.rs`

**Contract (Tầng 1 — cái GÌ; tên fn/struct/cách truyền buffer = Tầng 2, Worker quyết):**

1. Mỗi check expose 1 **core fn KHÔNG side-effect lên stdout/stderr thật** — mọi output đi vào buffer (vd struct `{ stdout: String, stderr: String, code: i32 }` hoặc Write-sink param — Worker chọn 1 pattern dùng NHẤT QUÁN cả 5 entry).
2. `run()` CLI hiện tại → wrapper mỏng: gọi core, `print!("{}", out.stdout)` + `eprint!("{}", out.stderr)` (KHÔNG `println!` — buffer đã chứa newline của từng dòng), return code. main.rs dispatch giữ nguyên.
3. **KHÔNG duplicate format string** — đổi `println!(...)` → `writeln!(buf, ...)` tại chỗ (hoặc tương đương). Nội dung từng byte không đổi.
4. `gate.rs`: core gate ghi section headers/summary vào buffer của gate, chèn buffer stdout/stderr của từng check con đúng VỊ TRÍ + đúng STREAM như hiện tại. Inline fns (INV-002..006/008) cũng route qua buffer.
5. `schema.rs`: git qua `Command::output` đã capture sẵn — chỉ cần các dòng echo đi vào buffer. Env `ALLOW_DATA_LOSS` đọc từ process env như cũ (MCP serve kế thừa env process — không đổi).
6. **Parity guard tuyệt đối:** `cargo test` sau Task 1 = 79 tests xanh NGUYÊN, parity byte-exact không đổi. Pins assert per-stream nên buffer-rồi-in giữ byte-exact; thay đổi duy nhất quan sát được = findings in ra cuối run thay vì streaming (checks chạy <2s — chấp nhận, ghi Discovery 1 dòng).

**Lưu ý:**
1. Phát hiện check nào `process::exit` bên trong (trái anchor #3) → sửa thành return code (bắt buộc cho serve — process phải sống), behavior CLI không đổi (main vẫn exit với code đó), ghi Discovery.
2. Nếu refactor buộc đổi signature `pub fn run()` mà `tests/parity_*.rs`/`scripts` gọi trực tiếp → chỉ sửa call-site, KHÔNG sửa assertion (precedent P004).

### Task 2: Dep mới — `rmcp` + `tokio`, pin exact

**File:** `Cargo.toml`, `Cargo.lock`

1. `cargo add rmcp` → đọc version resolve thật (anchor #1) → sửa thành **pin exact** `rmcp = "=<ver>"`. Tương tự `tokio = "=<ver>"` với features TỐI THIỂU rmcp stdio cần (anchor #7 — kỳ vọng cỡ `rt`/`macros`/io; Worker chốt từ examples crate, ghi Discovery).
2. **Lý do pin exact (ghi vào CHANGELOG):** dep ngoài đầu tiên trên binary security-surface — pin exact = build reproducible + Giám sát review đúng 1 version cụ thể; upgrade là phiếu riêng có chủ đích, không trôi theo semver.
3. `serde`/`serde_json` lên deps thường CHỈ nếu rmcp/tool schema đòi (anchor #6) — cũng pin exact, ghi CHANGELOG.
4. Commit `Cargo.lock`.
5. KHÔNG bump version `Cargo.toml` (v0.1.0 = P008, F13).

### Task 3: CLI — subcommand `serve`

**File:** `src/main.rs`

**Thêm:**
- Variant `Serve` (ngang hàng `Check`/`Gate`), không flag. Doc-comment: `/// MCP stdio server (rmcp) — exposes check_*/gate tools; scans process cwd`.
- `mod serve;` + dispatch → `serve::run()` (block đến khi client đóng stdin / shutdown sạch → exit 0; lỗi runtime → exit khác 0, message ra **stderr**).
- Tokio runtime: khởi tạo trong `serve::run()` (vd `Runtime::new().block_on(...)`) hoặc pattern rmcp examples khuyên — Worker chọn theo crate (Tầng 2). main.rs KHÔNG cần `#[tokio::main]` toàn cục (CLI paths còn lại sync, đừng kéo async vào check/gate).

**Lưu ý:** `serve` với flag lạ → clap error exit 2 (mặc định, không cần code thêm). KHÔNG đụng variants `Check`/`Gate`.

### Task 4: MCP server — `src/serve.rs` (MỚI)

**File:** `src/serve.rs`

1. **Server identity:** name `inv-gate`, version = `env!("CARGO_PKG_VERSION")`.
2. **5 tools — TÊN là API contract (Tầng 1, không đổi):** `check_secrets` (INV-009) · `check_runtime` (INV-010) · `check_port` (INV-001) · `check_schema` (Prisma) · `gate` (≡ CLI `gate --all` ≡ golden `--mechanical-only` — KHÔNG gọi schema, đúng P005). Mỗi tool = gọi đúng core fn Task 1 in-process. CẤM spawn binary, CẤM duplicate logic.
3. **Input schema:** không argument (object rỗng). Description mỗi tool GHI RÕ contract (Tầng 1): *"Scans the server process's current working directory — launch the server with cwd set to the repo to scan (same contract as the CLI). exit_code: 0 = clean, 1 = findings."*
4. **Response contract (Tầng 1 — chốt tại phiếu, đơn giản, không over-engineer):** 1 text content item = JSON object serialize:
   ```json
   {
     "exit_code": 1,
     "is_clean": false,
     "findings": "<stdout buffer verbatim — byte như CLI in ra>",
     "stderr": "<stderr buffer verbatim, \"\" nếu rỗng — vd WARN của check_port>"
   }
   ```
   - `exit_code` ∈ {0, 1} từ core (2 không reachable — tools không có args). `is_clean` = `exit_code == 0`.
   - Findings (exit 1) = tool chạy THÀNH CÔNG, `isError` của MCP = false. `isError` = true CHỈ khi internal error (io/panic-catch tùy rmcp pattern) — message lỗi trong content.
5. **Escape hatch (anchor #2):** nếu API rmcp version resolve được KHÔNG khớp mô hình trên (vd bắt buộc structured-content schema khác, hay tool definition không cho mô tả tự do) → Worker giữ NGUYÊN 4 field semantic trên trong bất kỳ wrapper nào rmcp đòi, ghi Debate Log objection nếu phải lệch contract field-name. Field names = contract — lệch là Tầng 1, không tự quyết.
6. **Protocol hygiene (Luật chơi 3):** trong serve, KHÔNG bất kỳ `print!`/`println!` nào ra stdout thật ngoài rmcp; diagnostics (nếu cần) → stderr, tối thiểu.

### Task 5: Integration test — `tests/mcp_serve.rs` (MỚI)

**File:** `tests/mcp_serve.rs` (+ helper extraction `tests/common/` nếu gọn — Tầng 2)

1. **Stdio THẬT:** spawn binary (assert_cmd `cargo_bin` hoặc `std::process::Command` + pipe), arg `serve`, cwd = fixture dir, stdin/stdout piped. Nói JSON-RPC: `initialize` → `notifications/initialized` → `tools/list` → `tools/call`. Nếu rmcp có client util (anchor #10) → dùng được; fallback raw JSON-RPC line-delimited theo spec MCP — cả 2 đều đạt acceptance "stdio thật". KHÔNG gọi core fn trực tiếp trong test này (đó không phải MCP test).
2. **Fixtures = reuse harness parity (anchor #8), KHÔNG invent fixture trong `tests/golden/fixtures/` (F07):**
   - Union harness gate (mirror `tests/parity_gate.rs`) dirty + clean → phục vụ `gate`, `check_secrets`, `check_runtime`, `check_port`.
   - Harness schema (mirror `tests/parity_schema.rs`) → phục vụ `check_schema` (cần git repo 2-commit; serve spawn riêng với cwd = fixture đó).
3. **Asserts tối thiểu:**
   - `tools/list` → ĐÚNG 5 tools, đúng tên (so khít set, không subset).
   - **Cả 5 tools** call trên fixture dirty: parse JSON content → `exit_code`/`is_clean` đúng; `findings` == stdout CLI chạy trên CÙNG fixture (so sánh trực tiếp output 2 đường — chạy binary CLI tương ứng trong test, không hardcode expected); `stderr` field tương tự (check_port có WARN — MANIFEST §4 rule 7).
   - `gate` + ít nhất 1 check trên fixture clean: `exit_code` 0, `is_clean` true, khớp CLI.
   - Sau call cuối: đóng stdin → process thoát sạch (exit 0) trong timeout.
4. **Hermetic:** mọi spawn (serve + CLI đối chiếu) `env_remove("ALLOW_DATA_LOSS")`; timeout cứng cho mỗi đoạn đọc response (test treo = đỏ, không hang CI); fixed dates harness như parity (LF, `2026-01-01T00:00:00 +0000`).

**Lưu ý:**
1. So sánh findings vs CLI = chạy CLI trên cùng fixture trong test (oracle sống) — KHÔNG chép pin gate vào expected (pin là của parity tests, đừng couple).
2. Test đỏ → sửa `serve.rs`/harness — KHÔNG sửa pins/fixtures/repin.sh (Luật chơi 1).

### Task 6: Docs gate

**File:** `CHANGELOG.md` — entry P006 (Unreleased): `serve` MCP stdio (rmcp), 5 tools wrap core in-process; buffered-core refactor (CLI byte-exact không đổi, parity xanh nguyên); dep mới `rmcp =<ver>` + `tokio =<ver>` pinned exact + lý do (supply-chain, security surface); response contract 4 fields. KHÔNG bump version (F13 — P008).
**File:** `docs/ARCHITECTURE.md` — Components: thêm `src/serve.rs` shipped (P006): 5 tools, in-process core reuse, response schema 4 fields, cwd contract ("client launch server với cwd = repo cần quét"), stdout = JSON-RPC only. Components các check/gate: note 1 dòng buffered-core (CLI wrapper print buffer). Data flow: thêm nhánh MCP: agent → `inv-gate serve` (stdio) → tool call → core fn → JSON response.
**File:** `docs/discoveries/P006.md` + index 1-line `docs/DISCOVERIES.md` — gồm: (i) rmcp version thật + API surface đã dùng (PROJECT.md "1.7.0" đúng/sai); (ii) tokio features chốt; (iii) anchors #3-#5 confirm/diff; (iv) refactor diff đáng kể nào (signature đổi, call-site sửa); (v) transcript 1 phiên JSON-RPC mẫu (initialize→list→call) làm evidence; (vi) hooks cho P007/P008.

---

## Ship (IG-09 — KỶ LUẬT MỚI, lệnh Chủ nhà)

> Phiếu này ship = **commit → merge `main` → push → XÓA branch NGAY**. Không stack branch chờ.

1. Commit trên `feat/P006-serve-mcp` — hook pre-commit chạy thật (KHÔNG `--no-verify`); nhớ `cargo build --release` TRƯỚC commit (dogfood guard P005 đòi binary release — anchor #11).
2. Sau nghiệm thu Chủ nhà/Quản đốc: merge vào `main` + push `origin main`.
3. Xóa branch ngay: `git branch -d feat/P006-serve-mcp` (+ `git push origin --delete ...` nếu đã push branch). Worktree cleanup qua `phieu-done` nếu dùng.
4. BACKLOG checkbox move = việc ORCHESTRATOR (Worker/Architect không edit BACKLOG).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `Cargo.toml` + `Cargo.lock` | Task 2: `rmcp` + `tokio` pin exact (+ serde nếu đòi); KHÔNG bump version |
| `src/checks/{secrets,runtime,port,schema}.rs`, `src/checks/mod.rs` | Task 1: buffered-core, CLI wrapper print buffer — behavior-neutral |
| `src/gate.rs` | Task 1: buffered-core (headers/summary/inline fns + chèn buffer check con đúng vị trí/stream) |
| `src/main.rs` | Task 3: variant `Serve` + `mod serve;` + dispatch |
| `src/serve.rs` | Task 4: MỚI — rmcp stdio server, 5 tools, response contract 4 fields |
| `tests/mcp_serve.rs` | Task 5: MỚI — JSON-RPC qua stdio thật, khớp CLI trên cùng fixture |
| `tests/parity_*.rs`, `tests/common/` | CHỈ call-site/import nếu signature đổi — assertions pin KHÔNG đổi |
| `CHANGELOG.md`, `docs/ARCHITECTURE.md` | Task 6 |
| `docs/discoveries/P006.md` + `docs/DISCOVERIES.md` | Discovery report + index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng |
| `tests/golden/**` (pins, fixtures, repin.sh, MANIFEST.md) | Diff rỗng — không có pin/oracle mới cho MCP; parity đỏ → sửa Rust |
| `hooks/pre-commit`, `scripts/**` | Dogfood P005 nguyên trạng — diff rỗng; anchor #11 verify hook vẫn exit 0 |
| `docs/BACKLOG.md` | Orchestrator-only |
| `docs/PROJECT.md`, `CLAUDE.md` | Không đụng — stack đã ghi rmcp/dual-mode sẵn. PROJECT.md "rmcp 1.7.0" sai thì ghi Discovery, KHÔNG tự sửa PROJECT.md (vision doc = Chủ nhà) |

---

## Luật chơi (Constraints)

1. **Parity bất khả xâm phạm:** pins/fixtures/repin.sh/MANIFEST untouched; parity assertions byte-exact giữ NGUYÊN; output CLI từng byte không đổi sau refactor. `cargo test` xanh sau Task 1 TRƯỚC khi thêm dep.
2. **KHÔNG duplicate logic:** tools gọi đúng core fn CLI dùng, in-process. CẤM spawn chính binary trong serve, CẤM async hóa check/gate logic (core giữ sync; async chỉ ở lớp rmcp/serve).
3. **Protocol hygiene:** trong serve mode, stdout = JSON-RPC ONLY. Mọi output check qua buffer. Diagnostics → stderr, tối thiểu. (Integration test gián tiếp enforce — byte lạ trên stdout phá framing = test đỏ.)
4. **Dep discipline (security surface):** chỉ `rmcp` + `tokio` (+ serde nếu rmcp đòi), pin exact `=`, lý do CHANGELOG, `Cargo.lock` commit. Dep nào khác thấy "tiện" → DỪNG, Debate Log. PR cần Giám sát review (dep mới đầu tiên — CLAUDE.md).
5. **API contract (Tầng 1):** tool names + 4 response field names + cwd contract + exit-code semantics 0/1 — đúng như phiếu, lệch = Debate Log, không tự quyết. Tên struct/fn nội bộ, tổ chức serve.rs, pattern buffer, cấu trúc test helper = Tầng 2.
6. **KHÔNG `process::exit` trong code path serve** — server sống đến khi stdin đóng; check nào đang exit bên trong (trái anchor #3) phải sửa thành return.
7. **Test hermetic:** `env_remove("ALLOW_DATA_LOSS")` mọi spawn; timeout cứng mọi read; harness LF + fixed dates (precedent P002/P003).
8. **Worker KHÔNG web:** API rmcp tra từ crate source local + `cargo doc`. KHÔNG bịa API theo trí nhớ — không chắc thì mở source crate.
9. `cargo test` xanh TOÀN BỘ trước commit — gồm 79 regression. `cargo build --release` sạch trước proof-commit (anchor #11).
10. **IG-09:** merge main + push + xóa branch NGAY sau nghiệm thu — ghi transcript bước này vào Discovery.

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — 79 tests cũ NGUYÊN (parity byte-exact 4 check + gate) + tests mới `mcp_serve`
- [ ] `tools/list` qua stdio thật = đúng 5 tools: `check_secrets`/`check_runtime`/`check_port`/`check_schema`/`gate`
- [ ] Cả 5 tools call qua stdio thật trên fixture dirty: `exit_code`/`is_clean`/`findings`/`stderr` khớp CLI chạy trên CÙNG fixture; `gate` + ≥1 check trên clean: exit 0 khớp CLI
- [ ] Serve thoát sạch (exit 0) khi client đóng stdin, trong timeout
- [ ] `git diff golden/ tests/golden/ hooks/ scripts/` rỗng; `Cargo.toml` diff CHỈ dep mới pinned (không bump version)
- [ ] `cargo build --release` sạch

### Manual Testing
- [ ] Tay: `printf` initialize + tools/list vào `target/release/inv-gate serve` → JSON-RPC response hợp lệ, KHÔNG byte lạ trên stdout
- [ ] `cargo run -- serve --no-such-flag` → exit 2 (clap)
- [ ] Chạy serve với cwd = repo này, gọi `check_secrets` → kết quả khớp `cargo run -- check secrets` tại repo root (informational, ghi Discovery)

### Regression
- [ ] `cargo run -- check secrets|runtime|port|schema` + `cargo run -- gate --all` behavior/output nguyên (đối chiếu parity tests + 1 lần chạy tay)
- [ ] Dogfood: `bash scripts/security-gate.sh --mechanical-only` exit 0 trên clean tree sau `cargo build --release` (anchor #11); proof-commit chạy hook thật KHÔNG `--no-verify`
- [ ] `bash tests/golden/repin.sh` vẫn chạy + `git diff tests/golden/pins/` rỗng sau khi chạy

### Docs Gate
- [ ] `CHANGELOG.md` — entry P006 (dep pinned + lý do, response contract, refactor note)
- [ ] `docs/ARCHITECTURE.md` — serve.rs shipped + buffered-core note + MCP data flow + cwd contract

### Discovery Report
- [ ] `docs/discoveries/P006.md`: anchors ĐÚNG/SAI (đặc biệt #1 version thật vs PROJECT.md "1.7.0", #2 API surface, #3-#5), tokio features, refactor diff notes, transcript JSON-RPC mẫu, tier escalations ("None" nếu không), IG-09 ship transcript
- [ ] Append 1-line index `docs/DISCOVERIES.md`
