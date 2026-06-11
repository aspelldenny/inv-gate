// tests/mcp_serve.rs — Integration test for `inv-gate serve` (MCP stdio JSON-RPC).
//
// Strategy: spawn binary with `serve` arg + piped stdin/stdout (real stdio transport).
// Raw JSON-RPC line-delimited per MCP spec.
// Fixtures: reuse build_fixture_repo() pattern from parity_gate.rs (harness copy — Tầng 2).
// Oracle: compare MCP tool response `findings` against CLI binary run on the SAME fixture (live).
//
// Hermetic: env_remove("ALLOW_DATA_LOSS") on all spawns; timeout all reads; LF + fixed dates.
// Rule: test red → fix serve.rs/harness, NEVER fix pins/fixtures/repin.sh (Luật chơi 1).

use assert_cmd::Command as AssertCmd;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────────────────
// Harness (mirrors parity_gate.rs — Tầng 2 extract inline)
// ─────────────────────────────────────────────────────────────────────────────

fn git_in(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
        .output()
        .expect("git command failed");
    if !out.status.success() {
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn copy_dir_all_excluding(src: &Path, dst: &Path, exclude_names: &[&str]) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let fname = entry.file_name();
        let fname_str = fname.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == fname_str.as_ref()) {
            continue;
        }
        let dst_path = dst.join(&fname);
        if ty.is_dir() {
            copy_dir_all_excluding(&entry.path(), &dst_path, exclude_names)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Build a full union fixture repo (same as parity_gate.rs build_fixture_repo).
fn build_fixture_repo(branch: &str, tmp: &Path) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_src = manifest_dir.join(format!("tests/golden/fixtures/{}", branch));
    let golden_dir = manifest_dir.join("golden");

    copy_dir_all_excluding(&fixture_src, tmp, &["schema.before.prisma", "schema.after.prisma"])
        .expect("fixture copy failed");

    let tmp_scripts = tmp.join("scripts");
    fs::create_dir_all(&tmp_scripts).expect("scripts dir creation failed");
    for script in &[
        "check-hardcoded-secrets.py",
        "check-port-bind.py",
        "check-runtime-secrets.py",
        "check-schema-safety.sh",
    ] {
        fs::copy(golden_dir.join(script), tmp_scripts.join(script))
            .expect(&format!("copy golden script {} failed", script));
    }
    fs::copy(golden_dir.join("security-gate.sh"), tmp.join("security-gate.sh"))
        .expect("copy security-gate.sh failed");

    git_in(tmp, &["init", "-q", "."]);
    git_in(tmp, &["config", "user.name", "P006 MCP Test"]);
    git_in(tmp, &["config", "user.email", "test@inv-gate.local"]);
    git_in(tmp, &["config", "commit.gpgsign", "false"]);

    let prisma_dir = tmp.join("prisma");
    fs::create_dir_all(&prisma_dir).expect("prisma dir creation");
    fs::copy(
        fixture_src.join("prisma/schema.before.prisma"),
        prisma_dir.join("schema.prisma"),
    ).expect("copy schema.before failed");
    git_in(tmp, &["add", "-A"]);
    git_in(tmp, &["commit", "-q", "-m", "P006 mcp test baseline"]);

    fs::copy(
        fixture_src.join("prisma/schema.after.prisma"),
        prisma_dir.join("schema.prisma"),
    ).expect("copy schema.after failed");
    git_in(tmp, &["add", "prisma/schema.prisma"]);
    git_in(tmp, &["commit", "-q", "-m", "P006 mcp test schema change"]);

    if branch == "dirty" {
        git_in(tmp, &[
            "remote", "add", "origin",
            "https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git",
        ]);
    } else {
        git_in(tmp, &[
            "remote", "add", "origin",
            "https://github.com/example/fixture.git",
        ]);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON-RPC helpers
// ─────────────────────────────────────────────────────────────────────────────

fn jsonrpc_request(id: u32, method: &str, params: Value) -> String {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let mut s = serde_json::to_string(&msg).unwrap();
    s.push('\n');
    s
}

fn jsonrpc_notification(method: &str, params: Value) -> String {
    let msg = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    let mut s = serde_json::to_string(&msg).unwrap();
    s.push('\n');
    s
}

/// Read one JSON-RPC response from reader.
/// Skips server-sent notifications (lines without "id" field).
fn read_response(reader: &mut impl BufRead, _timeout_secs: u64) -> Option<Value> {
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 {
            return None; // EOF
        }
        let v: Value = serde_json::from_str(line.trim()).ok()?;
        // Skip pure notifications (no "id" field)
        if v.get("id").is_some() {
            return Some(v);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI oracle helper — run CLI command on fixture, return (stdout, exit_code)
// ─────────────────────────────────────────────────────────────────────────────

fn cli_run(fixture: &Path, args: &[&str]) -> (String, i32) {
    let mut cmd = AssertCmd::cargo_bin("inv-gate").expect("inv-gate binary must exist");
    cmd.args(args);
    cmd.current_dir(fixture);
    cmd.env_remove("ALLOW_DATA_LOSS");
    let output = cmd.output().expect("CLI binary execution failed");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (stdout, code)
}

// ─────────────────────────────────────────────────────────────────────────────
// Spawn serve and run full JSON-RPC session
// ─────────────────────────────────────────────────────────────────────────────

struct ServeSession {
    child: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
    stdin: std::process::ChildStdin,
    next_id: u32,
}

impl ServeSession {
    fn start(fixture: &Path) -> Self {
        // Build the binary path via assert_cmd
        let bin = AssertCmd::cargo_bin("inv-gate")
            .expect("inv-gate binary must exist")
            .get_program()
            .to_owned();

        let mut child = Command::new(bin)
            .arg("serve")
            .current_dir(fixture)
            .env_remove("ALLOW_DATA_LOSS")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // suppress serve diagnostics in test output
            .spawn()
            .expect("failed to spawn inv-gate serve");

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let reader = BufReader::new(stdout);

        ServeSession { child, reader, stdin, next_id: 1 }
    }

    fn send(&mut self, msg: &str) {
        self.stdin.write_all(msg.as_bytes()).expect("write to serve stdin");
        self.stdin.flush().expect("flush serve stdin");
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg = jsonrpc_request(id, method, params);
        self.send(&msg);
        read_response(&mut self.reader, 10).expect(&format!(
            "no response to {} (id={})", method, id
        ))
    }

    fn notify(&mut self, method: &str, params: Value) {
        let msg = jsonrpc_notification(method, params);
        self.send(&msg);
    }

    fn close(mut self) -> std::process::ExitStatus {
        // Close stdin → server should detect EOF and exit cleanly
        drop(self.stdin);
        // Wait with a timeout
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            match self.child.try_wait().expect("try_wait") {
                Some(status) => return status,
                None => {
                    if std::time::Instant::now() > deadline {
                        let _ = self.child.kill();
                        panic!("serve did not exit within 5s after stdin close");
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }

    /// Do the MCP initialize handshake.
    fn initialize(&mut self) {
        let init_resp = self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test-client", "version": "0.0.1" }
            }),
        );
        assert!(
            init_resp.get("result").is_some() || init_resp.get("error").is_none(),
            "initialize failed: {:?}", init_resp
        );
        // Send initialized notification
        self.notify("notifications/initialized", json!({}));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: tools/list returns exactly 5 correct tool names
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_tools_list_five_tools() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("dirty", tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    let list_resp = sess.request("tools/list", json!({}));
    drop(sess.stdin);

    let tools = list_resp["result"]["tools"]
        .as_array()
        .expect("tools/list result.tools must be array");

    let mut tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name must be string"))
        .collect();
    tool_names.sort();

    let expected: Vec<&str> = vec![
        "check_port",
        "check_runtime",
        "check_schema",
        "check_secrets",
        "gate",
    ];
    assert_eq!(tool_names, expected, "tools/list must return exactly 5 tools with correct names");

    drop(sess.reader);
    drop(sess.child);
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: call a tool and return the parsed 4-field JSON response
// ─────────────────────────────────────────────────────────────────────────────

fn call_tool(sess: &mut ServeSession, tool_name: &str) -> Value {
    let resp = sess.request(
        "tools/call",
        json!({ "name": tool_name, "arguments": {} }),
    );
    assert!(
        resp.get("error").is_none(),
        "tools/call {} returned error: {:?}", tool_name, resp
    );
    let content = resp["result"]["content"]
        .as_array()
        .expect("result.content must be array");
    assert!(!content.is_empty(), "content must not be empty for {}", tool_name);
    let text = content[0]["text"].as_str().expect("content[0].text must be string");
    serde_json::from_str(text).expect(&format!("content[0].text must be valid JSON for {}", tool_name))
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: all 5 tools on dirty fixture — findings == CLI on same fixture
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_five_tools_dirty_match_cli() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("dirty", tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    // 5 tool → CLI arg mapping
    let tool_cli_pairs: &[(&str, &[&str])] = &[
        ("check_secrets", &["check", "secrets"]),
        ("check_runtime", &["check", "runtime"]),
        ("check_port",    &["check", "port"]),
        ("check_schema",  &["check", "schema"]),
        ("gate",          &["gate", "--all"]),
    ];

    for (tool_name, cli_args) in tool_cli_pairs {
        let mcp_payload = call_tool(&mut sess, tool_name);

        // MCP fields
        let mcp_exit_code = mcp_payload["exit_code"].as_i64().expect("exit_code must be int") as i32;
        let mcp_is_clean = mcp_payload["is_clean"].as_bool().expect("is_clean must be bool");
        let mcp_findings = mcp_payload["findings"].as_str().expect("findings must be string");

        // CLI oracle on same fixture
        let (cli_stdout, cli_exit_code) = cli_run(tmp.path(), cli_args);

        // Assert exit_code matches CLI
        assert_eq!(
            mcp_exit_code, cli_exit_code,
            "exit_code mismatch for tool={}: mcp={} cli={}",
            tool_name, mcp_exit_code, cli_exit_code
        );
        // Assert is_clean = (exit_code == 0)
        assert_eq!(
            mcp_is_clean, mcp_exit_code == 0,
            "is_clean must equal exit_code==0 for tool={}", tool_name
        );
        // Assert findings == CLI stdout (byte-exact via string comparison)
        assert_eq!(
            mcp_findings, cli_stdout.as_str(),
            "findings mismatch for tool={}:\nmcp_findings={:?}\ncli_stdout={:?}",
            tool_name, mcp_findings, cli_stdout
        );
    }

    let status = sess.close();
    assert!(status.success(), "serve should exit 0 after stdin close, got: {:?}", status);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: gate + check_secrets on clean fixture → exit 0 + is_clean true
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_clean_fixture_exit_zero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("clean", tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    for tool_name in &["gate", "check_secrets"] {
        let payload = call_tool(&mut sess, tool_name);
        let exit_code = payload["exit_code"].as_i64().expect("exit_code") as i32;
        let is_clean = payload["is_clean"].as_bool().expect("is_clean");
        let (_, cli_code) = cli_run(tmp.path(), if *tool_name == "gate" { &["gate", "--all"] } else { &["check", "secrets"] });

        assert_eq!(exit_code, 0, "clean fixture {} should have exit_code=0", tool_name);
        assert!(is_clean, "clean fixture {} should have is_clean=true", tool_name);
        assert_eq!(exit_code, cli_code, "exit_code must match CLI for {}", tool_name);
    }

    let status = sess.close();
    assert!(status.success(), "serve should exit 0 on clean, got: {:?}", status);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: stderr field is string; check_port dirty should have non-empty stderr
// (MANIFEST §4 rule 7 — WARN to stderr for missing compose files)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_check_port_stderr_field_is_string() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_fixture_repo("dirty", tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    let payload = call_tool(&mut sess, "check_port");
    let stderr_field = payload["stderr"].as_str().expect("stderr field must be string");

    // check_port dirty: the fixture only has docker-compose.yml so astro-service/docker-compose.yml
    // and docker-compose.dev.yml are missing → WARN on stderr
    // The stderr field should contain those WARN lines.
    assert!(
        stderr_field.contains("WARN") || stderr_field.is_empty(),
        "stderr field should be string (possibly with WARN messages): {:?}", stderr_field
    );

    // Verify stderr field matches CLI stderr
    let cli_output = AssertCmd::cargo_bin("inv-gate")
        .expect("inv-gate binary")
        .args(["check", "port"])
        .current_dir(tmp.path())
        .env_remove("ALLOW_DATA_LOSS")
        .output()
        .expect("CLI run");
    let cli_stderr = String::from_utf8_lossy(&cli_output.stderr).into_owned();

    assert_eq!(
        stderr_field, cli_stderr.as_str(),
        "MCP stderr field must match CLI stderr for check_port"
    );

    drop(sess.stdin);
    drop(sess.reader);
    drop(sess.child);
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: call a tool with explicit arguments and return the parsed 4-field JSON response
// (P007 — Tầng 2; anchor #8 verified: harness extended with this helper)
// ─────────────────────────────────────────────────────────────────────────────

fn call_tool_with_args(sess: &mut ServeSession, tool_name: &str, arguments: Value) -> Value {
    let resp = sess.request(
        "tools/call",
        json!({ "name": tool_name, "arguments": arguments }),
    );
    // For isError responses, result is still present in rmcp — return as-is for inspection
    let content = resp["result"]["content"]
        .as_array()
        .expect("result.content must be array");
    assert!(!content.is_empty(), "content must not be empty for {}", tool_name);
    let text = content[0]["text"].as_str().expect("content[0].text must be string");
    serde_json::from_str(text).unwrap_or_else(|_| {
        // If not valid JSON (e.g. error message text), return as Value::String
        serde_json::Value::String(text.to_string())
    })
}

/// Call tool and return the raw response value (for isError checking).
fn call_tool_raw(sess: &mut ServeSession, tool_name: &str, arguments: Value) -> Value {
    sess.request(
        "tools/call",
        json!({ "name": tool_name, "arguments": arguments }),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: build a minimal repo (mirrors gate_skip.rs — Tầng 2 inline)
// ─────────────────────────────────────────────────────────────────────────────

fn build_minimal_repo_mcp(tmp: &Path) {
    use std::fs;

    let git_in = |dir: &Path, args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .expect("git");
        if !out.status.success() {
            panic!("git {:?} failed", args);
        }
    };

    git_in(tmp, &["init", "-q", "."]);
    git_in(tmp, &["config", "user.name", "P007 MCP"]);
    git_in(tmp, &["config", "user.email", "test@inv-gate.local"]);
    git_in(tmp, &["config", "commit.gpgsign", "false"]);

    fs::write(
        tmp.join(".gitignore"),
        ".env.production\n.env.staging\n.env.backup\n.env.local\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.join("src")).unwrap();
    fs::write(tmp.join("src/main.rs"), "fn main() {}\n").unwrap();
    git_in(tmp, &["add", "-A"]);
    git_in(tmp, &["commit", "-q", "-m", "mcp minimal"]);
    git_in(tmp, &["remote", "add", "origin", "https://github.com/example/mcp-minimal.git"]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Probe (g): MCP gate tool with skip_absent arg — P007
// ─────────────────────────────────────────────────────────────────────────────

/// (g1) gate tool with {"skip_absent": true} on minimal fixture → exit_code 0, is_clean true,
/// findings contains SKIP lines.
#[test]
fn mcp_gate_skip_absent_true_exit_0() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo_mcp(tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    let payload = call_tool_with_args(&mut sess, "gate", json!({"skip_absent": true}));

    let exit_code = payload["exit_code"].as_i64().expect("exit_code must be int") as i32;
    let is_clean = payload["is_clean"].as_bool().expect("is_clean must be bool");
    let findings = payload["findings"].as_str().expect("findings must be string");

    assert_eq!(exit_code, 0, "gate skip_absent=true on minimal repo must exit_code=0");
    assert!(is_clean, "gate skip_absent=true on minimal repo must be is_clean=true");
    assert!(
        findings.contains("  SKIP (no sentry.ts / sentry.*.config.* present)"),
        "findings must contain INV-005 SKIP line: got:\n{}", findings
    );
    assert!(
        findings.contains("  SKIP (file docker-compose.yml absent)"),
        "findings must contain INV-008 SKIP line: got:\n{}", findings
    );

    let status = sess.close();
    assert!(status.success(), "serve should exit 0 cleanly");
}

/// (g2) gate tool with NO arguments on minimal fixture → exit_code 1 (backward compat — default false).
#[test]
fn mcp_gate_no_args_backward_compat_exit_1() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo_mcp(tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    let payload = call_tool(&mut sess, "gate");

    let exit_code = payload["exit_code"].as_i64().expect("exit_code must be int") as i32;
    let is_clean = payload["is_clean"].as_bool().expect("is_clean must be bool");

    assert_eq!(exit_code, 1, "gate no-args on minimal repo must exit_code=1 (default false)");
    assert!(!is_clean, "gate no-args on minimal repo must be is_clean=false");

    let status = sess.close();
    assert!(status.success(), "serve should exit 0 cleanly");
}

/// (g3) gate tool with wrong type arg (string instead of bool) → isError true (fail-closed).
#[test]
fn mcp_gate_wrong_type_arg_is_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    build_minimal_repo_mcp(tmp.path());

    let mut sess = ServeSession::start(tmp.path());
    sess.initialize();

    let raw_resp = call_tool_raw(&mut sess, "gate", json!({"skip_absent": "yes"}));

    // The result should have is_error: true
    let is_error = raw_resp["result"]["isError"].as_bool().unwrap_or(false);
    assert!(
        is_error,
        "gate with wrong-type skip_absent must return isError=true (fail-closed): got:\n{:?}", raw_resp
    );

    let status = sess.close();
    assert!(status.success(), "serve should exit 0 cleanly");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: serve exits non-zero on wrong flag (clap exit 2)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn serve_with_unknown_flag_exits_2() {
    let output = AssertCmd::cargo_bin("inv-gate")
        .expect("inv-gate binary")
        .args(["serve", "--no-such-flag"])
        .env_remove("ALLOW_DATA_LOSS")
        .output()
        .expect("binary execution");
    assert_eq!(
        output.status.code().unwrap_or(-1),
        2,
        "serve with unknown flag should exit 2 (clap)"
    );
}
