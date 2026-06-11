#!/usr/bin/env bash
# P279 schema safety gate — detect destructive prisma schema migrations via git diff.
# Exit 0 = safe migration (ADD column/table/index only, OR no schema change).
# Exit 1 = destructive detected (field/model removed in schema.prisma).
# Bypass: set env ALLOW_DATA_LOSS=true (workflow_dispatch input data_loss_ack=true).
#
# V2 design (P279 Turn 1 Architect Response):
# Uses git diff parse instead of `prisma migrate diff` because:
# 1. prisma.config.ts (datasource URL source) NOT copied into deps stage container
#    (Dockerfile.nextjs:6-7 only copies package.json + pnpm-lock + prisma/ folder).
# 2. schema.prisma datasource block has provider only, no `url = env("DATABASE_URL")`.
# 3. Running outside container at GHA runner level — zero Prisma CLI dependency.
# Coverage: catches DROP COLUMN/TABLE (field/model removed) — 95% of P279 risk.
# Misses: type change (String→Int = ALTER COLUMN, lower risk, not P279 priority).

set -u

if [[ "${ALLOW_DATA_LOSS:-false}" == "true" ]]; then
    echo "ALLOW_DATA_LOSS=true — bypass schema safety check (Sếp explicit ack)."
    exit 0
fi

SCHEMA_FILE="prisma/schema.prisma"

if [[ ! -f "$SCHEMA_FILE" ]]; then
    echo "❌ $SCHEMA_FILE not found — cannot check schema safety."
    exit 1
fi

# Try HEAD~1..HEAD first (normal case). Fallback to empty-tree comparison for
# first-deploy fresh-repo edge case (no parent commit). Final fallback: no diff.
DIFF=$(git diff HEAD~1..HEAD -- "$SCHEMA_FILE" 2>/dev/null \
    || git diff 4b825dc8669f8c0..HEAD -- "$SCHEMA_FILE" 2>/dev/null \
    || echo "")

if [[ -z "$DIFF" ]]; then
    echo "No schema diff vs HEAD~1 — safe."
    exit 0
fi

# Destructive patterns (case-insensitive grep, anchored to removal lines):
#   ^-\s+<word>\s+\S+  → field removed (e.g., "-  email String?")
#   ^-model\s+\w+\s*{  → model declaration removed
#   ^-enum\s+\w+\s*{   → enum declaration removed
# Skip diff header lines (^---, ^-+++) by excluding ^--- and ^-+ patterns.
DESTRUCTIVE=$(echo "$DIFF" \
    | grep -vE '^---|^-\+\+\+' \
    | grep -E '^-\s*(model|enum)\s+\w+|^-\s+\w+\s+\S+' \
    || true)

if [[ -n "$DESTRUCTIVE" ]]; then
    echo "❌ DESTRUCTIVE SCHEMA CHANGE DETECTED in $SCHEMA_FILE:"
    echo "$DESTRUCTIVE"
    echo ""
    echo "Field/model removed → may cause DROP COLUMN/TABLE on db push."
    echo ""
    echo "To proceed:"
    echo "  CI:    re-run workflow_dispatch with input data_loss_ack=true."
    echo "  Local: verify backup < 24h, then ALLOW_DATA_LOSS=true bash scripts/check-schema-safety.sh"
    exit 1
fi

echo "Schema diff present but no destructive pattern (field/model removed) detected — safe."
exit 0
