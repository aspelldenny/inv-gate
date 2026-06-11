# PHIẾU P002: `check secrets` (INV-009) — port `check-hardcoded-secrets.py` + parity test chống pin

> **Loại:** Feature (Rust port — phiếu Rust ĐẦU TIÊN của sprint)
> **Ưu tiên:** P1
> **Tầng:** 1 — (a) scan-target patterns là SECURITY surface (CLAUDE.md: AUTO Tầng 1, Giám sát review PR); (b) CLI skeleton clap là shared contract cho P003-P005 (sai cấu trúc subcommand → LAN vào mọi phiếu port sau); (c) exit-code contract 0/1/2 là API pre-commit hook depend.
> **Ảnh hưởng:** `src/main.rs`, `src/checks/` (MỚI), `tests/parity_secrets.rs` (MỚI), `CHANGELOG.md`, `docs/ARCHITECTURE.md`
> **Dependency:** P001 (shipped — branch `feat/P001-pin-golden-oracle`). Phiếu này CHẠY TRÊN pins của P001; anchor #1 verify pins có trong base của branch P002 (nếu P001 chưa merge main → branch off `feat/P001-pin-golden-oracle` hoặc escalate orchestrator merge trước).

---

## Context

### Vấn đề hiện tại

Oracle đã pin (P001): `tests/golden/pins/` chứa stdout + exit codes của `golden/check-hardcoded-secrets.py` chạy trên `fixtures/dirty/` (exit 1) và `fixtures/clean/` (exit 0), kèm `MANIFEST.md` ghi pattern citation, invocation contract, unit-spec. Chưa có dòng Rust nào: `src/` là bootstrap skeleton, `src/checks/` mới ở trạng thái "planned" trong `docs/ARCHITECTURE.md` `[unverified — Worker verify anchor #7]`.

### Giải pháp

Ba phần, đúng method CLAUDE.md (parity trước, cải tiến sau):

1. **CLI skeleton** — clap derive nested subcommand `inv-gate check secrets`. Đặt nền cấu trúc cho `check port/runtime/schema` (P003/P004) nhưng CHỈ implement variant `secrets` trong phiếu này — KHÔNG stub chết các variant chưa port.
2. **Port logic INV-009** — `src/checks/secrets.rs` giữ NGUYÊN behavior `golden/check-hardcoded-secrets.py` (192 LOC): cùng scan targets, cùng pattern + pattern_name, cùng allowlist, cùng test-file skip, cùng path-level skip, cùng comment-line skip, cùng masking, cùng format output, cùng exit code. Mọi cải tiến (pattern tốt hơn, INV mới) = phiếu SAU parity, không phải phiếu này.
3. **Parity test** — chạy binary trên copy của fixtures dirty/clean → stdout + exit code phải **BYTE-EXACT** với pins (chi tiết normalize: Task 3).

### Scope

- CHỈ sửa/tạo: `src/main.rs`, `src/checks/mod.rs`, `src/checks/secrets.rs`, `tests/parity_secrets.rs`, `CHANGELOG.md`, `docs/ARCHITECTURE.md` (+ `tests/golden/MANIFEST.md` §4 một note ordering NẾU phát hiện — Task 4).
- KHÔNG sửa: `golden/**` (read-only tuyệt đối), `tests/golden/pins/**` + `tests/golden/fixtures/**` (oracle — parity fail thì sửa Rust, không sửa pin), `Cargo.toml` (deps đã đủ — anchor #8; không bump version, sprint chưa release).
- KHÔNG implement: `check port/runtime/schema`, `gate`, `serve`, JSON output (P006), flag nào ngoài subcommand path.

---

## Task 0 — Verification Anchors

> Bước 0 capability: phiếu này không cần network/POST — toàn bộ là filesystem scan + cargo. Capability duy nhất phải verify thật = pins tồn tại (anchor #1) và regex crate compile được pattern golden (anchor #3).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Pins secrets tồn tại trong base branch: `tests/golden/pins/secrets--dirty.stdout.txt`, `secrets--clean.stdout.txt` (tên file exact `[needs Worker verify]` — convention `<check>--<fixture>.stdout.txt` per P001 Task 3), `exit_codes.json` có key `secrets--dirty: 1`, `secrets--clean: 0` `[unverified — MANIFEST §6]` `[oracle: ls + cat — SOUND]` | `ls tests/golden/pins/ && cat tests/golden/pins/exit_codes.json` | ⏳ TO VERIFY — nếu thiếu → P001 chưa trong base → escalate orchestrator (merge/branch), KHÔNG tự re-pin |
| 2 | Exact regex các pattern class tại `golden/check-hardcoded-secrets.py`: github-pat `:69`, generic-entropy `:80-81`, allowlist `:85-95`, test-file skip `:51-58`, scan targets+ext `:33-35`, output format string `:182` `[unverified — cite qua MANIFEST §1/§5; bảng MANIFEST ghi "abridged" — KHÔNG được dùng bản abridged làm implementation source]` | Worker đọc golden source (được phép, read-only), chép exact pattern vào Rust kèm comment cite `golden/<file>:<line>` | ⏳ TO VERIFY |
| 3 | Golden regex KHÔNG dùng lookaround/backreference (regex crate compile được) `[needs Worker verify]` `[oracle: grep `(?=`/`(?!`/`(?<`/`\1` trong pattern strings + `cargo test` compile — SOUND]` | grep pattern strings trong golden; `Regex::new` trong unit test | ⏳ TO VERIFY — nếu CÓ lookaround/backref → DỪNG, ghi Debate Log objection + DEFER (xem Luật chơi 2), KHÔNG tự thêm fancy-regex |
| 4 | Masking algorithm của `<masked>` trong output (Python mask secret thế nào: bao nhiêu char đầu/cuối, ký tự mask) `[needs Worker verify]` | đọc golden quanh `:182` + đối chiếu literal trong `pins/secrets--dirty.stdout.txt` | ✅ RESOLVED (Turn 1) — `:121-125`: `len <= 12 → "***" + last4`, else `first4 + "..." + last4` `[verified — Worker Turn 1]` |
| 5 | Stdout của golden có banner/summary lines ngoài finding lines không (vd "Scanning...", "N violations", success message ở clean run) `[needs Worker verify]` | đọc cả 2 pin stdout files | ⏳ TO VERIFY — byte-exact nghĩa là replicate MỌI dòng stdout, kể cả banner/summary/clean-message + trailing newline |
| 6 | Finding paths trong pin là relative (`src/config.ts:3: ...`) — script chạy cwd=repo-root với target dir relative; normalize §4.1 của repin.sh có thể đã no-op cho check này `[unverified — MANIFEST §2 + §4.1]` | đọc pin stdout | ⏳ TO VERIFY — quyết định parity test có cần normalize temp-path hay không (Task 3 Lưu ý 2) |
| 7 | `src/main.rs` tồn tại (bootstrap stub); `src/checks/` CHƯA tồn tại `[unverified — ARCHITECTURE.md ghi "planned"]` | `ls src/ src/checks/ 2>/dev/null` | ⏳ TO VERIFY — nếu checks/ đã có file → DISCOVERY_REPORT trước khi ghi đè |
| 8 | `Cargo.toml` đã có đủ deps: clap 4 derive, regex, walkdir, serde_json; dev: assert_cmd, predicates, tempfile `[verified — Architect đọc Cargo.toml]` | — | ✅ Không thêm dependency nào trong phiếu này |
| 9 | Double-firing: dòng chứa `ghp_` token fire CẢ `github-pat` LẪN `generic-entropy` (2 finding lines cho 1 dòng source) `[verified — docs/discoveries/P001.md §Edge cases]` | đối chiếu pin dirty: `src/config.ts:3` xuất hiện 2 lần? `[needs Worker verify chi tiết dòng nào]` | ✅ behavior / ⏳ chi tiết — Rust PHẢI replicate cả 2 firing, không dedupe |
| 10 | Ordering: fixture P001 chỉ có 1 file match scan ext trong `src/` (`config.ts`; `lib/sentry.ts` cũng `.ts` — có bị scan và clean không?) → order findings = thứ tự duyệt file × lineno tăng dần trong file `[unverified — MANIFEST §4.3 + discovery P001: rglob order OS-dependent]` | `ls -R tests/golden/fixtures/dirty/src/` + đọc pin order | ⏳ TO VERIFY — xem Task 3 Lưu ý 3 (spec order) |
| 11 | Behavior khi target dir vắng mặt: fixture KHÔNG có `astro-service/` (MANIFEST §1 note) mà golden vẫn exit 0/1 sạch → script guard missing dir (không crash, không stderr?) `[needs Worker verify]` | đọc golden + pin stderr (file stderr tồn tại không?) | ⏳ TO VERIFY — Rust phải cùng behavior với missing dir (parity test fixture không có `astro-service/`) |
| 12 | clap 4 parse error (unknown subcommand/flag) exit code mặc định = 2 — khớp contract usage error CLAUDE.md `[unverified — clap default]` `[oracle: chạy `inv-gate check bogus; echo $?` — SOUND]` | probe sau khi build | ⏳ TO VERIFY — giữ ⏳ qua Turn 1, probe post-build (Nghiệm thu Manual) |
| 13 | Python đọc file scan thế nào khi gặp file không decode được UTF-8 / unreadable (errors=ignore? skip?) `[needs Worker verify — không trigger trong fixture nhưng quyết định port hành vi đọc file]` | đọc golden file-read code | ✅ RESOLVED (Turn 1) — `:132-134`: non-UTF-8/unreadable → skip file SILENT `[verified — Worker Turn 1]` — Rust skip y hệt, không stderr |
| 14 | Line numbering: 1-indexed, line-only, không col/offset (F06) `[verified — MANIFEST §5 row 1]` | — | ✅ Rust đếm line 1-indexed; Python `splitlines()` ≡ Rust `lines()` với fixtures LF `[verified — Worker Turn 1]` |

**Anchor #1 hoặc #3 ra ❌ → DỪNG trước khi viết Rust logic, escalate theo cột Result. Các anchor khác ❌ → Worker fold vào Debate Log Turn 1.**

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

**Phiếu version:** V2 (Turn 1 — Architect accepted O1.1 + O1.2)

### Turn 1 — Worker Challenge (vs V1)

**Verdict:** PASS-with-notes. Worker đã đọc golden source + pins, verify line cites bằng grep.

**Anchor verification (recap):**
- #4 ✅ resolved — masking `:121-125`: `len <= 12 → "***" + last4`, else `first4 + "..." + last4`
- #13 ✅ resolved — non-UTF-8/unreadable → skip file silent `:132-134`
- #14 ✅ — `splitlines()` ≡ `lines()` (fixtures LF)
- #12 ⏳ giữ — clap exit 2 chỉ probe được post-build

**Objections (Tầng 1 — phiếu cần sửa; cả 2 cùng lý do: parity test BLIND với cơ chế này, fixture không exercise → thiếu là binary xanh test nhưng false-positive trên repo thật):**
- [O1.1] Task 2 V1 THIẾU path-level skip `SKIP_PATH_SUBSTR` (`golden/check-hardcoded-secrets.py:39-48`, áp qua `should_skip_path()` `:106-107`) — gồm `node_modules/`, `.next/`, `src/generated/` (Prisma WASM blob — AKIA coincidental collision, security surface), `target/`, ...
- [O1.2] Task 2 V1 THIẾU comment-line skip (`:98-114`: `JS_COMMENT_PREFIX = ("//",)`, `PY_COMMENT_PREFIX = ("#",)`, `is_comment_line()`, áp tại `:137`).

**Tầng 2 notes (không yêu cầu sửa thiết kế, fold vào phiếu):**
- Allowlist `:85-95` đủ **8 entries** — V1 cite thiếu `google/gemini` + `anthropic/claude`
- GENERIC_PATTERN bắt secret qua `group(2)` → Rust dùng `captures[2]`
- Order risk multi-ext (nhiều ext cùng target dir) chưa trigger ở P002 — ghi sẵn Discovery hook cho P003+

**Status:** ✅ RESPONDED (xem Architect Response bên dưới)

### Turn 1 — Architect Response (phiếu V2)
- [O1.1] → **ACCEPT** — thêm Task 2 mục 5 (path-level skip `SKIP_PATH_SUBSTR`, cite `:39-48` + `should_skip_path` `:106-107`, port nguyên danh sách) + unit test bắt buộc (a) ở Task 3 Lưu ý 4 + Nghiệm thu.
- [O1.2] → **ACCEPT** — thêm Task 2 mục 6 (comment-line skip, cite `:98-114`, áp `:137`, port nguyên cơ chế) + unit test bắt buộc (b).
- Tầng 2 notes → fold: allowlist 8 entries (Task 2 mục 3), `captures[2]` (mục 2), masking rule (mục 7 + anchor #4), non-UTF-8 silent skip (Lưu ý 2 + anchor #13), `lines()` (anchor #14), order-risk multi-ext → Discovery hook P003+ (Nghiệm thu Discovery Report). Anchor #12 giữ ⏳ post-build.

**Status:** ✅ RESPONDED — phiếu bumped to V2. Fix nhỏ, không redesign. Next: Worker CHALLENGE verify consensus hoặc approval gate.

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: CLI skeleton — clap derive nested subcommand

**File:** `src/main.rs` (sửa — bootstrap stub hiện tại thay bằng dispatch; nội dung stub cũ `[needs Worker verify]` anchor #7, nếu có logic thật thì DISCOVERY trước khi xóa)

**Thêm / Thay bằng:**

```rust
// Cấu trúc bắt buộc (tên type/field nội bộ = Tầng 2, Worker chọn):
// inv-gate check secrets
#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Mechanical INV checks
    Check {
        #[command(subcommand)]
        check: CheckCommand,
    },
}

#[derive(clap::Subcommand)]
enum CheckCommand {
    /// INV-009 — hardcoded secrets scan (parity port of golden/check-hardcoded-secrets.py)
    Secrets,
}
```

- `main()` dispatch `Check { Secrets }` → `checks::secrets::run()` → trả exit code → `std::process::exit(code)`.
- `src/checks/mod.rs` (MỚI): `pub mod secrets;` — layout per `docs/ARCHITECTURE.md` Components (1 module / INV).

**Lưu ý:**
1. CHỈ variant `Secrets`. P003/P004 thêm variant sau (<100 LOC/INV — success criteria 8 PROJECT.md). KHÔNG thêm `Port`/`Runtime`/`Schema` stub trả "not implemented" — dead surface.
2. Usage error (unknown subcommand/flag) → clap tự exit 2 (anchor #12) — khớp contract `0 clean / 1 findings / 2 usage` CLAUDE.md. KHÔNG viết exit-2 path thủ công trong secrets logic (golden không có exit-2 mode — deviation đã document P001).
3. Không tokio/async ở đây — `serve` mới cần (Phase 3).

### Task 2: Port logic INV-009 — `src/checks/secrets.rs`

**File:** `src/checks/secrets.rs` (MỚI)

**Thêm:** Port 1:1 từ `golden/check-hardcoded-secrets.py` (Worker đọc source — Architect không đọc được). Các surface bắt buộc GIỮ NGUYÊN, mỗi cái kèm comment cite `golden/check-hardcoded-secrets.py:<line>`:

1. **Scan targets + extensions** (`:33-35` per MANIFEST §1): `["src"]` cho `.ts/.tsx/.js/.jsx`, `["astro-service"]` cho `.py`. Target dir vắng mặt → cùng behavior golden (anchor #11), không panic.
2. **Pattern classes + pattern_name** (`:69`, `:80-81`): exact regex từ golden (KHÔNG dùng bản abridged MANIFEST — anchor #2). `pattern_name` xuất hiện trong output → phải giống hệt string golden dùng. GENERIC_PATTERN lấy secret qua `group(2)` → Rust dùng `captures[2]` `[verified — Worker Turn 1]`.
3. **Allowlist skip** (`:85-95`): đủ **8 entries** `[verified — Worker Turn 1]` — gồm cả `google/gemini` + `anthropic/claude` (V1 cite thiếu 2 entry này). Worker chép exact 8 entries + exact matching semantics (substring? case-sensitive?) TỪ GOLDEN, không dùng list trong phiếu làm source.
4. **Test-file skip** (`:51-58`): `/tests/`, `*.test.*`, `*.spec.*`, `/__tests__/`, `/__mocks__/`, `prisma/seed-*.ts` — exact semantics (path-substring vs glob) từ golden.
5. **Path-level skip — `SKIP_PATH_SUBSTR`** (`:39-48`, áp qua `should_skip_path()` `:106-107`) `[verified — Worker Turn 1]`: gồm `node_modules/`, `.next/`, `src/generated/` (Prisma WASM blob — AKIA coincidental collision, security surface), `target/`, ... — port NGUYÊN danh sách exact từ golden. *(V2 — O1.1)*
6. **Comment-line skip** (`:98-114`): `JS_COMMENT_PREFIX = ("//",)`, `PY_COMMENT_PREFIX = ("#",)`, `is_comment_line()`, áp tại `:137` `[verified — Worker Turn 1]` — port nguyên cơ chế. *(V2 — O1.2)*
7. **Masking** (`:121-125`, anchor #4 resolved): `len <= 12 → "***" + last4`, else `first4 + "..." + last4` — byte-exact với pin.
8. **Output**: stdout, format `path:lineno: INV-009 violated -- <masked> (<pattern_name>)` (`:182`, MANIFEST §5 — line-only, 1-indexed). MỌI dòng stdout khác của golden (banner/summary/clean-message — anchor #5) replicate verbatim.
9. **Double-firing** (anchor #9): 1 dòng source match nhiều pattern → in đủ từng finding, KHÔNG dedupe.
10. **Exit**: ≥1 finding → 1; 0 finding → 0. Không có exit path nào khác trong module này.

**Lưu ý:**
1. Duyệt file: `walkdir` (dep sẵn) hay `std::fs` đệ quy = Tầng 2, Worker chọn — miễn tập hợp file scan + skip rules đúng golden. Order duyệt: xem Task 3 Lưu ý 3.
2. Đọc file: non-UTF-8/unreadable → skip file SILENT (`:132-134`, anchor #13 resolved) — không stderr, không panic. Fixtures là LF — `lines()` ≡ Python `splitlines()` ở đây, đếm 1-indexed (anchor #14, F06).
3. Đây là security surface: từng regex/allowlist/skip entry lệch 1 ký tự so golden = vi phạm Luật chơi 1. Khi nghi golden "sai" (pattern dở) → pin nguyên trạng behavior, ghi DISCOVERY, cải tiến để phiếu sau.

### Task 3: Parity test chống pin

**File:** `tests/parity_secrets.rs` (MỚI — integration test, assert_cmd + tempfile + serde_json, deps sẵn)

**Thêm:** 2 test case (dirty / clean), mỗi case:

1. `tempfile::tempdir()` → copy nguyên cây `tests/golden/fixtures/<fixture>/` vào temp (copy hết — file ngoài scan target, vd `prisma/`, `docker-compose.yml`, inert với secrets check).
2. Chạy binary (`assert_cmd::Command::cargo_bin("inv-gate")`) args `["check", "secrets"]`, **cwd = temp dir** (invocation contract MANIFEST §2: cwd = repo root, no args).
3. So sánh:
   - **exit code** == giá trị key `secrets--<fixture>` trong `tests/golden/pins/exit_codes.json` (parse bằng serde_json — KHÔNG hardcode 0/1 trong test; pin là nguồn).
   - **stdout** == nội dung `tests/golden/pins/secrets--<fixture>.stdout.txt` **BYTE-EXACT** (kể cả trailing newline).
   - **stderr** == pin stderr file nếu tồn tại, ngược lại == rỗng (anchor #11).

**Lưu ý:**
1. Pin là oracle: test ĐỎ → sửa `src/checks/secrets.rs`, tuyệt đối không sửa pin/fixture cho khớp (Luật chơi 3).
2. **Normalize:** kỳ vọng pin chứa path relative (anchor #6) → Rust in path relative từ cwd → so raw bytes, KHÔNG normalize gì. Nếu anchor #6 ra ❌ (pin chứa artifact normalize §4.1) → apply ĐÚNG rule MANIFEST §4.1 (temp-prefix→relative) lên stdout của Rust trước khi compare, và ghi rule đã dùng vào discovery. Không phát minh normalize khác.
3. **Order spec:** so byte-exact nghĩa là order findings = EXACT như pin. Pin P001 capture trên chính platform này; fixture nhỏ (anchor #10) nên order khả thi. Nếu Rust traversal order ≠ pin order → KHÔNG chuyển sang so order-insensitive một cách âm thầm: trước hết thử ép order duyệt khớp Python (anchor #10 — Worker xác định order thật từ pin: per-target-dir theo thứ tự `["src"]` rồi `["astro-service"]`, trong dir theo order rglob); nếu order rglob không tái tạo được deterministically → ghi Debate Log objection + đề xuất (đây là quyết định parity-contract cho cả P003-P005, Tầng 1).
4. **Unit probes — 2 test BẮT BUỘC (V2, từ O1.1/O1.2; parity test BLIND với 2 cơ chế này):** synthetic in-code instances (F07-compliant, KHÔNG invent fixture file mới trong `tests/golden/fixtures/`): (a) `should_skip_path` với path chứa `src/generated/` → skipped; (b) dòng comment chứa secret-like string → skipped. Probe thêm cho allowlist/double-firing/masking = optional, Tầng 2.

### Task 4: Docs gate

**File:** `CHANGELOG.md` — entry P002 (mục Unreleased): port INV-009, CLI skeleton `check secrets`, parity vs pin. KHÔNG bump version `Cargo.toml` (chưa release — F13 chỉ áp khi bump; precedent P001).
**File:** `docs/ARCHITECTURE.md` — Components: `src/checks/secrets.rs` từ "(planned)" → shipped (P002); giữ port/runtime/schema là planned. Sửa tối thiểu cho khớp reality.
**File:** `tests/golden/MANIFEST.md` §4 — CHỈ KHI Task 3 Lưu ý 3 chốt được order rule cụ thể (vd "rglob order = ... , Rust khớp bằng ..."): append 1-3 dòng kèm citation, để P003-P005 dùng lại. Không phát hiện gì mới → không đụng.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Task 1: clap derive dispatch `check secrets` |
| `src/checks/mod.rs` | Task 1: MỚI — module decl |
| `src/checks/secrets.rs` | Task 2: MỚI — port INV-009, mọi pattern cite golden:line |
| `tests/parity_secrets.rs` | Task 3: MỚI — 2 parity test chống pin + 2 unit test bắt buộc (V2) |
| `CHANGELOG.md` | Task 4: entry P002 |
| `docs/ARCHITECTURE.md` | Task 4: checks/secrets.rs shipped |
| `tests/golden/MANIFEST.md` | Task 4: §4 ordering note — CONDITIONAL, chỉ khi có rule mới + citation |
| `docs/discoveries/P002.md` + `docs/DISCOVERIES.md` | Discovery report + 1-line index |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `golden/**` | READ-ONLY tuyệt đối — `git diff golden/` rỗng khi nghiệm thu |
| `tests/golden/pins/**`, `tests/golden/fixtures/**`, `tests/golden/repin.sh` | Oracle — `git diff` rỗng. Parity fail → sửa Rust |
| `Cargo.toml` | Deps đã đủ (anchor #8 ✅) — không thêm dep, không bump version. Diff rỗng |
| `CLAUDE.md` | Chỉ đụng nếu phát hiện deviation exit-code MỚI (rule "document any deviation") — kỳ vọng: không |

---

## Luật chơi (Constraints)

1. **Parity-first, security surface:** KHÔNG đổi/thêm/nới bất kỳ pattern, pattern_name, allowlist entry, skip rule (test-file / path-level / comment-line), masking, output wording nào so với oracle. Mỗi surface trong Task 2 mang comment cite `golden/check-hardcoded-secrets.py:<line>`. PR cần Giám sát review trước merge (CLAUDE.md).
2. **regex crate only.** Nếu golden pattern đòi lookaround/backreference (anchor #3 ❌) → DỪNG, ghi objection Debate Log + DEFER quyết định dep mới (fancy-regex) cho Architect/Chủ nhà — Worker KHÔNG tự thêm dependency (security + supply-chain surface).
3. **Pin bất khả xâm phạm:** parity test đỏ → sửa Rust. Sửa pin/fixture/MANIFEST-để-test-xanh = vi phạm scope, dừng + escalate. (`MANIFEST.md` ngoại lệ duy nhất: Task 4 conditional note, không bao giờ để khớp test.)
4. **Byte-exact** bao gồm banner/summary/clean-message lines + trailing newline (anchor #5). Không thêm log/print nào ra stdout ngoài những gì golden in; diagnostic riêng của Rust (nếu cần) → stderr, nhưng stderr cũng phải khớp pin/rỗng trong parity run.
5. **Exit contract:** secrets logic chỉ exit 0/1. Exit 2 = clap usage error duy nhất (deviation 4-script-không-exit-2 đã document P001 — không che, không "sửa giúp" golden).
6. **CLI tối thiểu:** chỉ `check secrets`. Không flag thêm, không JSON output (P006), không dead stub.
7. **PROJECT.md constraints:** không mutate file scan, không network, cross-platform (không hardcode path separator — dùng path API; lưu ý output dùng `/` như golden trên macOS/Linux).
8. `cargo test` xanh toàn bộ trước commit (CLAUDE.md).

---

## Nghiệm thu

### Automated
- [ ] `cargo test` xanh — gồm 2 parity test: `secrets--dirty` exit 1 + stdout byte-exact pin; `secrets--clean` exit 0 + stdout byte-exact pin
- [ ] 2 unit test synthetic bắt buộc xanh (V2 — O1.1/O1.2): (a) path chứa `src/generated/` bị `should_skip_path` skip; (b) comment line chứa secret-like string bị skip — parity test BLIND với 2 cơ chế này nên unit test là lưới duy nhất
- [ ] `cargo check` / build sạch
- [ ] `git diff golden/ tests/golden/fixtures/ tests/golden/pins/ tests/golden/repin.sh` rỗng
- [ ] `git diff Cargo.toml` rỗng

### Manual Testing
- [ ] Dogfood: `cargo run -- check secrets` tại repo root inv-gate → exit 0, không panic (src/ không có file `.ts/.js` → 0 finding)
- [ ] `cargo run -- check bogus` → clap usage error, exit 2 (anchor #12)
- [ ] Copy `fixtures/dirty/` ra temp, chạy binary cwd=temp → mắt thường đối chiếu findings (gồm double-firing dòng `ghp_` — anchor #9) khớp `pins/secrets--dirty.stdout.txt`
- [ ] Mỗi pattern/allowlist/skip constant trong `secrets.rs` có comment cite `golden/check-hardcoded-secrets.py:<line>`

### Regression
- [ ] `bash tests/golden/repin.sh` vẫn chạy được + `git diff tests/golden/pins/` rỗng sau khi chạy (oracle không bị xáo trộn bởi phiếu này)

### Docs Gate
- [ ] `CHANGELOG.md` — entry P002
- [ ] `docs/ARCHITECTURE.md` — `src/checks/secrets.rs` shipped, hết "planned"
- [ ] `tests/golden/MANIFEST.md` §4 — ordering note nếu có (kèm citation), không thì ghi "None" trong discovery
- [ ] Discovery hook P003+ (V2): ghi mục riêng trong `docs/discoveries/P002.md` về order risk multi-ext (nhiều extension trong cùng target dir — rglob order chưa bị exercise ở P002)

### Discovery Report
- [ ] `docs/discoveries/P002.md`: assumptions ĐÚNG/SAI (cite file:line — đặc biệt anchor #3/#5/#10/#11), masking rule thật (#4 resolved :121-125), order rule thật, scope expansion, edge cases, docs updated, tier escalation ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
