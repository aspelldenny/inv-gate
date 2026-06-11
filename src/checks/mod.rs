// src/checks/mod.rs — 1 module / INV (per docs/ARCHITECTURE.md)
pub mod secrets;
pub mod runtime;
pub mod port;
pub mod schema;

/// Buffered output from a check core function.
/// stdout/stderr contain the exact bytes that would be emitted to the respective streams.
/// Each buffer already includes any trailing newlines — CLI wrapper uses print!/eprint! (no extra \n).
#[derive(Debug, Default)]
pub struct CheckOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}
