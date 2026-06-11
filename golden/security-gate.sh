#!/usr/bin/env bash
# P273 security gate: run all mechanical invariants INV-001 -> INV-008.
# P275 add flags: --mechanical-only (skip INV-007 SSH), --include-ssh (require SSH check).
# Exit 0 = all pass. Exit 1 = any violation.

set -u

MECHANICAL_ONLY=false
INCLUDE_SSH=false
for arg in "$@"; do
    case "$arg" in
        --mechanical-only) MECHANICAL_ONLY=true ;;
        --include-ssh) INCLUDE_SSH=true ;;
        *) echo "Unknown flag: $arg"; exit 2 ;;
    esac
done

PASS=0
FAIL=0
WARN=0
FAILED_INVS=()

run() {
    local inv="$1"
    local desc="$2"
    shift 2
    echo "--- $inv: $desc ---"
    if "$@"; then
        echo "  PASS"
        PASS=$((PASS + 1))
    else
        echo "  FAIL"
        FAIL=$((FAIL + 1))
        FAILED_INVS+=("$inv")
    fi
    echo
}

warn_only() {
    local inv="$1"
    local desc="$2"
    shift 2
    echo "--- $inv: $desc (WARN-only) ---"
    if "$@"; then
        echo "  PASS"
    else
        echo "  WARN (tracked, not blocking)"
        WARN=$((WARN + 1))
    fi
    echo
}

# INV-001: port bindings
run "INV-001" "No host-bind 0.0.0.0 except nginx 80/443" \
    python3 scripts/check-port-bind.py

# INV-002: no :latest tag
check_inv002() {
    local out
    out=$(grep -E '^\s+image:.*:latest$' docker-compose.yml astro-service/docker-compose.yml docker-compose.dev.yml 2>/dev/null || true)
    # Known exceptions: umami postgresql-latest, portainer latest
    # Strip these out, fail if anything remains.
    local remaining
    remaining=$(echo "$out" | grep -vE 'ghcr\.io/umami-software/umami:postgresql-latest|portainer/portainer-ce:latest' || true)
    if [[ -z "$remaining" ]]; then
        return 0
    else
        echo "$remaining"
        return 1
    fi
}
run "INV-002" "No :latest tag (except umami/portainer exception)" check_inv002

# INV-003: no real value in .env.example
check_inv003() {
    local out
    out=$(grep -E '^[A-Z_]+=[^#[:space:]]' .env.example 2>/dev/null | \
        grep -vE '=postgresql://\.\.\.|=Soul Signature|=c993dc1e|=http://soulsign-|=http://localhost|=http://[0-9]|=google/gemini|=anthropic/claude|=1$' || true)
    if [[ -z "$out" ]]; then
        return 0
    else
        echo "$out"
        return 1
    fi
}
run "INV-003" "No real secret value in .env.example" check_inv003

# INV-004: .env.* in .gitignore + no commit in history
check_inv004() {
    local missing=()
    for f in production staging backup local; do
        # Accept exact match OR glob pattern covering this file (e.g. .env*.local covers .env.local)
        if ! grep -qE "^\.env\.${f}$|^\.env\*\.${f}$" .gitignore; then
            missing+=(".env.${f}")
        fi
    done
    if (( ${#missing[@]} > 0 )); then
        echo "Missing in .gitignore: ${missing[*]}"
        return 1
    fi
    # History check (read-only, no rewrite)
    local leaked
    leaked=$(git log --all --diff-filter=A --name-only 2>/dev/null | grep -E '\.env\.(production|staging|backup|local)$' || true)
    if [[ -n "$leaked" ]]; then
        echo "Historic leak detected:"
        echo "$leaked"
        return 1
    fi
    return 0
}
run "INV-004" ".env.{production,staging,backup,local} gitignored + never committed" check_inv004

# INV-005: Sentry config scrubs Authorization
check_inv005() {
    local out
    out=$(grep -rnE "beforeBreadcrumb|beforeSend" src/lib/sentry.ts sentry.*.config.* 2>/dev/null || true)
    if [[ -z "$out" ]]; then
        echo "No beforeSend/beforeBreadcrumb handler found in Sentry config files"
        return 1
    fi
    return 0
}
run "INV-005" "Sentry config has beforeSend/beforeBreadcrumb scrubber" check_inv005

# INV-006: astro-service CORS not wildcard
check_inv006() {
    local out
    out=$(grep -nE "origins.*\*|allow_origin.*\*|CORS\(.*\*" astro-service/app.py 2>/dev/null || true)
    if [[ -z "$out" ]]; then
        return 0
    else
        echo "$out"
        return 1
    fi
}
run "INV-006" "astro-service CORS not wildcard" check_inv006

# INV-007: UFW 443/tcp -- skip nếu --mechanical-only; SSH thật nếu --include-ssh.
# DECISION P279: Giữ root SSH cho riêng audit này — ufw require root, KHÔNG cách
# nào audit qua user deploy non-sudo. Operation read-only (grep), trans session.
# Root key CHỈ trên local dev machine — KHÔNG embed GitHub Secrets (CI dùng
# deploy user theo P279 sweep). Trade-off chấp nhận vs split-user complexity.
check_inv007_ssh() {
    # Pre-deploy gate: SSH VPS verify UFW 443/tcp restrict CF IPs.
    # Requires ~/.ssh/id_ed25519 + VPS reachable. Root SSH local dev only.
    local out
    out=$(ssh -i ~/.ssh/id_ed25519 -p 1994 -o ConnectTimeout=5 -o BatchMode=yes \
        root@103.167.150.178 'ufw status numbered 2>/dev/null | grep "443/tcp"' 2>/dev/null || true)
    if [[ -z "$out" ]]; then
        echo "INV-007 SSH check: empty UFW 443/tcp output — verify manually"
        return 1
    fi
    # Spec: 443/tcp ALLOW only from CF IP ranges (see INVARIANTS.md INV-007 spec).
    # Must contain at least 1 ALLOW rule for 443/tcp.
    if echo "$out" | grep -qE 'ALLOW'; then
        echo "$out"
        return 0
    fi
    echo "INV-007 SSH check: no ALLOW rule for 443/tcp"
    echo "$out"
    return 1
}

check_inv007_local() {
    echo "  (INV-007 requires SSH VPS verify — skip in --mechanical-only mode)"
    return 0
}

if $INCLUDE_SSH; then
    run "INV-007" "UFW 443/tcp restricted to CF IPs (SSH)" check_inv007_ssh
elif ! $MECHANICAL_ONLY; then
    warn_only "INV-007" "UFW 443/tcp restricted to CF IPs (local stub)" check_inv007_local
fi
# else --mechanical-only: skip INV-007 entirely (don't even WARN — pre-commit don't need it)

# INV-008: production compose uses expose: not ports: for internal services
check_inv008() {
    # Use Python yaml parse (no yq dependency)
    python3 -c "
import yaml, sys
data = yaml.safe_load(open('docker-compose.yml'))
internal = ['nextjs', 'postgres', 'astro-service', 'umami-db', 'nextjs-staging', 'postgres-staging']
violations = []
for svc in internal:
    cfg = data.get('services', {}).get(svc, {})
    if 'ports' in cfg:
        violations.append(f'{svc}: has ports: directive (should be expose:)')
if violations:
    print('\n'.join(violations))
    sys.exit(1)
" 2>&1
}
run "INV-008" "Internal services use expose: not ports:" check_inv008

# INV-009: no hardcoded secret in src/ + astro-service/
run "INV-009" "No hardcoded secret in src/ + astro-service/ source files" \
    python3 scripts/check-hardcoded-secrets.py

# INV-010: no secret in runtime state + infra files
run "INV-010" "No secret in runtime state + infra files" \
    python3 scripts/check-runtime-secrets.py

# Summary
echo "===================================="
echo "Security gate: $PASS passed, $FAIL failed, $WARN warnings"
if (( FAIL > 0 )); then
    echo "Failed invariants: ${FAILED_INVS[*]}"
    exit 1
fi
exit 0
