# DISCOVERIES — inv-gate

Index of per-phiếu discovery reports. See `docs/discoveries/<P-NNN>.md` for detail.

| Phiếu | Summary |
|-------|---------|
| P001 | Pin golden oracle: fixture set + harness shipped; 5 Tầng-2 self-adapts (src/.ts extension, .gitignore+sentry fixtures, schema alignment, bash 3.2 compat); no Tầng-1 escalation; exit-code deviation documented (4 checks no exit-2 mode). |
| P002 | Port INV-009 check-secrets: parity tests byte-exact (dirty/clean); anchor #12 exit-2 verified; 1 Tầng-2 self-adapt (r#"..."# raw string for backtick in regex); phiếu allowlist count "8 entries" was 9 in golden (port exact, no security impact); order risk for multi-ext dirs noted for P003+. |
| P003 | Port INV-010 check-runtime: parity byte-exact (dirty/clean); O1.1 db-conn `(?!\$)` transcription equivalence-proven + g1-g4 GREEN; errors="ignore" byte-strip ported (not from_utf8_lossy); INFRA_GLOBS read_dir+sort (no glob crate); Sub-mech F dotfile token leak documented; 14 unit tests all pattern classes; dogfood exit 0 on this repo. |
| P004 | Port INV-001 check-port + schema-safety: parity byte-exact all 4 (dirty/clean×2); port stderr pins non-empty (2-line WARN, 108 bytes — byte-exact asserted); schema 6-branch + fallback chain 3 steps; O1.2 bad-SHA `4b825dc8669f8c0` ported as-is (BACKLOG fix); ALLOW_DATA_LOSS bypass exact-match + em-dash; first bash script ported (git-via-Command pattern); 63 tests total all GREEN. |
| P005 | Gate orchestrator port (3 in-process + 6 inline, parity byte-exact dirty/clean); INV-008 Python→Rust-native; dogfood per-check swap in scripts/security-gate.sh (python3 killed, binary live); guard fail-closed + reversibility confirmed; anchor #18: adapted scripts DIFFER from golden (coverage impact documented, exit-code contract identical); 79 tests total all GREEN; build release 26s. |
| P006 | MCP stdio server shipped (rmcp 1.7.0, 5 tools, buffered-core refactor all 5 entry points byte-exact); tokio `time` feature required for rmcp (undocumented, added); 84 tests total (79 old + 5 MCP integration) all GREEN; stdout-poisoning grep 0 println! in core/serve; IG-08 secret scan PASS; dogfood hook exit 0. |
