// src/serve.rs — MCP stdio server (rmcp), 5 tools wrap core check/gate fns in-process.
//
// PROTOCOL HYGIENE (Luật chơi 3 — P006):
//   stdout = JSON-RPC ONLY (no print!/println! anywhere in this file).
//   All check output goes through buffered core fns into JSON response fields.
//   Diagnostics (if any) → stderr only.
//
// TOOL CONTRACT (Tầng 1 — P006 §Task 4):
//   Tool names: check_secrets / check_runtime / check_port / check_schema / gate
//   Response: 1 text content item = JSON { exit_code, is_clean, findings, stderr }
//   Tools take no arguments; scan cwd of this process.
//   isError = false for check findings (tool ran successfully); true only for internal error.
//   CWD contract: client launches server with cwd = repo to scan (same as CLI).
//
// ASYNC BOUNDARY:
//   Tokio runtime started in run() via Builder::new_current_thread().enable_io().build().
//   Core check fns (run_core) remain synchronous — no async in check/gate logic.

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::{ToolCallContext, ToolRouter, ToolRoute},
    model::{
        CallToolResult, Content, Implementation, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::{MaybeSendFuture, RequestContext},
    RoleServer,
    transport,
};
use serde_json::json;

/// 4-field response schema (Tầng 1 — do not rename fields without a new phiếu).
fn make_response(out: &crate::checks::CheckOutput) -> CallToolResult {
    let payload = json!({
        "exit_code": out.code,
        "is_clean": out.code == 0,
        "findings": out.stdout,
        "stderr": out.stderr,
    });
    CallToolResult::success(vec![Content::text(payload.to_string())])
}

/// MCP server struct.
struct InvGateServer {
    tool_router: ToolRouter<Self>,
}

impl InvGateServer {
    fn new() -> Self {
        let desc_suffix = "Scans the server process's current working directory — launch the server \
                           with cwd set to the repo to scan (same contract as the CLI). \
                           exit_code: 0 = clean, 1 = findings.";

        let router = ToolRouter::new()
            .with_route(ToolRoute::new_dyn(
                Tool::new(
                    "check_secrets",
                    format!("INV-009: hardcoded secrets scan. {}", desc_suffix),
                    serde_json::Map::new(),
                ),
                |_ctx: ToolCallContext<'_, InvGateServer>| {
                    let out = crate::checks::secrets::run_core();
                    let result = make_response(&out);
                    Box::pin(async move { Ok(result) })
                },
            ))
            .with_route(ToolRoute::new_dyn(
                Tool::new(
                    "check_runtime",
                    format!("INV-010: runtime secrets scan. {}", desc_suffix),
                    serde_json::Map::new(),
                ),
                |_ctx: ToolCallContext<'_, InvGateServer>| {
                    let out = crate::checks::runtime::run_core();
                    let result = make_response(&out);
                    Box::pin(async move { Ok(result) })
                },
            ))
            .with_route(ToolRoute::new_dyn(
                Tool::new(
                    "check_port",
                    format!("INV-001: docker-compose host-bind check. {}", desc_suffix),
                    serde_json::Map::new(),
                ),
                |_ctx: ToolCallContext<'_, InvGateServer>| {
                    let out = crate::checks::port::run_core();
                    let result = make_response(&out);
                    Box::pin(async move { Ok(result) })
                },
            ))
            .with_route(ToolRoute::new_dyn(
                Tool::new(
                    "check_schema",
                    format!("Prisma schema-safety: destructive migration guard. {}", desc_suffix),
                    serde_json::Map::new(),
                ),
                |_ctx: ToolCallContext<'_, InvGateServer>| {
                    let out = crate::checks::schema::run_core();
                    let result = make_response(&out);
                    Box::pin(async move { Ok(result) })
                },
            ))
            .with_route(ToolRoute::new_dyn(
                Tool::new(
                    "gate",
                    format!("Runs all mechanical invariants (equivalent to `inv-gate gate --all`). {}", desc_suffix),
                    serde_json::Map::new(),
                ),
                |_ctx: ToolCallContext<'_, InvGateServer>| {
                    let out = crate::gate::run_core();
                    let result = make_response(&out);
                    Box::pin(async move { Ok(result) })
                },
            ));

        InvGateServer { tool_router: router }
    }
}

impl ServerHandler for InvGateServer {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .build();
        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new("inv-gate", env!("CARGO_PKG_VERSION")))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<ListToolsResult, rmcp::ErrorData>,
    > + MaybeSendFuture + '_ {
        let tools = self.tool_router.list_all();
        async move {
            Ok(ListToolsResult {
                tools,
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<CallToolResult, rmcp::ErrorData>,
    > + MaybeSendFuture + '_ {
        let ctx = ToolCallContext::new(self, request, context);
        async move {
            self.tool_router.call(ctx).await
        }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
    }
}

/// Run the MCP stdio server. Blocks until stdin closes (client disconnects).
/// Returns 0 on clean shutdown, non-0 on error (message to stderr).
pub fn run() -> i32 {
    let rt: tokio::runtime::Runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("inv-gate serve: failed to create tokio runtime: {}", e);
            return 1;
        }
    };

    rt.block_on(async {
        let server = InvGateServer::new();
        let transport = transport::io::stdio();
        match server.serve(transport).await {
            Ok(running) => {
                // Block until client closes stdin / server shuts down
                match running.waiting().await {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("inv-gate serve: task join error: {}", e);
                        1
                    }
                }
            }
            Err(e) => {
                eprintln!("inv-gate serve: initialization error: {}", e);
                1
            }
        }
    })
}
