# Changelog

Format loosely follows Keep a Changelog.

## [Unreleased] — P002 check secrets — 2026-06-11

### Added
- `src/checks/secrets.rs` — INV-009 port of `golden/check-hardcoded-secrets.py` (parity 1:1). All patterns, allowlist (8 entries), skip rules (test-file / path-level / comment-line), masking, and output format preserved verbatim with `golden:line` citations.
- `src/main.rs` — clap derive CLI skeleton: `inv-gate check secrets` subcommand. Exit 2 on unknown subcommand/flag (anchor #12 verified).
- `src/checks/mod.rs` — checks module layout (1 module / INV, per ARCHITECTURE.md).
- `tests/parity_secrets.rs` — 2 parity tests (dirty/clean) byte-exact vs `tests/golden/pins/`; 2 mandatory unit tests (V2): `should_skip_path(src/generated/)` + comment-line skip.

## [Unreleased] — P001 pin golden oracle — 2026-06-11

### Added
- `tests/golden/fixtures/` — fixture set (dirty/clean) for 4 INV checks + orchestrator gate.
- `tests/golden/repin.sh` — hermetic pin harness: temp git repos, fixed dates, path normalization.
- `tests/golden/pins/` — frozen stdout/stderr/exit codes per check × fixture.
- `tests/golden/MANIFEST.md` — fixture provenance, invocation contracts, unit-spec table (F06).
- `docs/DISCOVERIES.md` + `docs/discoveries/P001.md` — discovery index.

### Deviation from exit-code contract (document per CLAUDE.md rule)
The 4 individual check scripts (`check-hardcoded-secrets.py`, `check-port-bind.py`,
`check-runtime-secrets.py`, `check-schema-safety.sh`) have **no exit-2 mode**. They
exit 0 (clean) or 1 (violations) only. Exit 2 (`usage/config error`) applies ONLY to
`security-gate.sh` (unknown flag path `golden/security-gate.sh:14`). This is the
oracle reality — deviation noted per anchor #5/#8.

## v0.0.0 — sos adopt — 2026-06-11

- Spine retrofitted via `sos adopt`.
