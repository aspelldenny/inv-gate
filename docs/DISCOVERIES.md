# DISCOVERIES — inv-gate

Index of per-phiếu discovery reports. See `docs/discoveries/<P-NNN>.md` for detail.

| Phiếu | Summary |
|-------|---------|
| P001 | Pin golden oracle: fixture set + harness shipped; 5 Tầng-2 self-adapts (src/.ts extension, .gitignore+sentry fixtures, schema alignment, bash 3.2 compat); no Tầng-1 escalation; exit-code deviation documented (4 checks no exit-2 mode). |
| P002 | Port INV-009 check-secrets: parity tests byte-exact (dirty/clean); anchor #12 exit-2 verified; 1 Tầng-2 self-adapt (r#"..."# raw string for backtick in regex); phiếu allowlist count "8 entries" was 9 in golden (port exact, no security impact); order risk for multi-ext dirs noted for P003+. |
| P003 | Port INV-010 check-runtime: parity byte-exact (dirty/clean); O1.1 db-conn `(?!\$)` transcription equivalence-proven + g1-g4 GREEN; errors="ignore" byte-strip ported (not from_utf8_lossy); INFRA_GLOBS read_dir+sort (no glob crate); Sub-mech F dotfile token leak documented; 14 unit tests all pattern classes; dogfood exit 0 on this repo. |
