# Changelog

Format loosely follows Keep a Changelog.

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
