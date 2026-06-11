# golden/ — FROZEN oracle (do NOT edit)

Snapshot of tarot's 5-file mechanical security gate (797 LOC Python+Bash), copied
2026-06-11 from `~/tarot/scripts/`. This is the **golden oracle** for the Rust port
(same method as the doc-rotate port): pin these scripts' outputs on fixtures →
the Rust subcommand must reproduce them byte-meaningfully (same findings, same
exit codes) before any behavior change is allowed.

| File | INV | Port target |
|---|---|---|
| `security-gate.sh` | orchestrator | `inv-gate gate --all` |
| `check-hardcoded-secrets.py` | INV-009 | `inv-gate check secrets` |
| `check-port-bind.py` | INV-001 | `inv-gate check port` |
| `check-runtime-secrets.py` | INV-010 | `inv-gate check runtime` |
| `check-schema-safety.sh` | Prisma | `inv-gate check schema` |

Rules (doc-rotate port lessons — F06/F07):
- Golden snapshot fields must spec UNIT (char vs byte) for cross-language parity.
- No fixture FILES invented by the Worker — synthetic in-code instances OK for parity probes.
- Oracle outputs get pinned in `tests/golden/` by P001 BEFORE any porting starts.
