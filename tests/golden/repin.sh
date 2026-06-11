#!/usr/bin/env bash
# P001 repin.sh — golden oracle pin harness
# Run from repo root: bash tests/golden/repin.sh
#
# Builds temp git repos per fixture, runs 5 golden scripts, captures
# stdout/stderr/exit codes into tests/golden/pins/.
#
# Requirements: python3, bash, git on PATH.
# Must be run from repo root (cwd contract per invocation contract in MANIFEST).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
GOLDEN_DIR="${REPO_ROOT}/golden"
FIXTURE_DIR="${SCRIPT_DIR}/fixtures"
PINS_DIR="${SCRIPT_DIR}/pins"

mkdir -p "${PINS_DIR}"

# Temp file to accumulate exit code pairs (key=value lines) — bash 3 compatible
EXIT_CODES_TMP="${PINS_DIR}/.exit_codes_tmp"
> "${EXIT_CODES_TMP}"

record_exit() {
    echo "$1=$2" >> "${EXIT_CODES_TMP}"
}

# ─────────────────────────────────────────────────────────────────────────────
# build_fixture_repo <fixture_branch> <tmp>
#   Copies fixture files into $tmp, inits a real git repo (hermetic, fixed
#   dates), creates 2 commits (before→after schema), sets INV-010 remote URL.
# ─────────────────────────────────────────────────────────────────────────────
build_fixture_repo() {
    local branch="$1"   # "dirty" or "clean"
    local tmp="$2"
    local fixture_src="${FIXTURE_DIR}/${branch}"

    # Env hygiene: check-schema-safety.sh:18 exits 0 early if ALLOW_DATA_LOSS=true
    unset ALLOW_DATA_LOSS

    # Copy fixture files (EXCEPT schema.before/after.prisma — handled via 2-commit)
    rsync -a --exclude='prisma/schema.before.prisma' --exclude='prisma/schema.after.prisma' \
        "${fixture_src}/" "${tmp}/"

    # Copy golden scripts into $tmp/scripts/ layout
    # (security-gate.sh hardcodes "python3 scripts/check-*.py" — anchor #6 verified
    #  golden/security-gate.sh lines 55/197/201)
    mkdir -p "${tmp}/scripts"
    cp "${GOLDEN_DIR}/check-hardcoded-secrets.py" "${tmp}/scripts/"
    cp "${GOLDEN_DIR}/check-port-bind.py"         "${tmp}/scripts/"
    cp "${GOLDEN_DIR}/check-runtime-secrets.py"   "${tmp}/scripts/"
    cp "${GOLDEN_DIR}/check-schema-safety.sh"     "${tmp}/scripts/"
    cp "${GOLDEN_DIR}/security-gate.sh"           "${tmp}/"

    # Init hermetic git repo
    git init -q "${tmp}"
    git -C "${tmp}" config user.name  "P001 Pin Harness"
    git -C "${tmp}" config user.email "pin@inv-gate.local"
    git -C "${tmp}" config commit.gpgsign false

    # Fixed deterministic dates — ensures commit SHAs are stable across runs
    # (idempotency requirement: repin.sh run twice → git diff pins/ empty)
    export GIT_AUTHOR_DATE="2026-01-01T00:00:00 +0000"
    export GIT_COMMITTER_DATE="2026-01-01T00:00:00 +0000"

    # Commit 1: schema.before.prisma → baseline
    mkdir -p "${tmp}/prisma"
    cp "${fixture_src}/prisma/schema.before.prisma" "${tmp}/prisma/schema.prisma"
    git -C "${tmp}" add -A
    git -C "${tmp}" commit -q -m "P001 fixture baseline"

    # Commit 2: schema.after.prisma → change (dirty=deletion, clean=additive)
    cp "${fixture_src}/prisma/schema.after.prisma" "${tmp}/prisma/schema.prisma"
    git -C "${tmp}" add prisma/schema.prisma
    git -C "${tmp}" commit -q -m "P001 fixture schema change"

    # INV-010 remote inject (nhánh A — anchor #13 verified; check-runtime-secrets.py:40-44)
    # Token format: ghp_ + 36 alphanum (golden/check-runtime-secrets.py:96 O2.1)
    # self-check: echo -n 'ghp_FAKETOKEN000000000000000000000000000' | wc -c = 40
    if [[ "${branch}" == "dirty" ]]; then
        git -C "${tmp}" remote add origin \
            "https://x-access-token:ghp_FAKETOKEN000000000000000000000000000@github.com/example/fixture.git"
    else
        git -C "${tmp}" remote add origin \
            "https://github.com/example/fixture.git"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# run_check <key> <tmp> <cmd...>
#   Runs a check command from cwd=$tmp, captures stdout+stderr+exit code.
#   Normalizes absolute temp path → fixture-relative in output.
#   Writes to pins/<key>.stdout.txt (and .stderr.txt if non-empty).
# ─────────────────────────────────────────────────────────────────────────────
run_check() {
    local key="$1"
    local tmp="$2"
    shift 2
    local cmd=("$@")

    local stdout_file="${PINS_DIR}/${key}.stdout.txt"
    local stderr_file="${PINS_DIR}/${key}.stderr.txt"
    local stderr_tmp="/tmp/repin_stderr_${$}_capture"

    local exit_code=0
    local stdout_raw
    stdout_raw=$(cd "${tmp}" && "${cmd[@]}" 2>"${stderr_tmp}") || exit_code=$?
    local stderr_raw
    stderr_raw=$(cat "${stderr_tmp}")
    rm -f "${stderr_tmp}"

    # Normalize: replace absolute temp path with empty (fixture-relative)
    # Normalize rule §4 in MANIFEST: sed "s|$tmp/||g"
    local stdout_norm stderr_norm
    stdout_norm=$(printf '%s' "${stdout_raw}" | sed "s|${tmp}/||g")
    stderr_norm=$(printf '%s' "${stderr_raw}" | sed "s|${tmp}/||g")

    printf '%s\n' "${stdout_norm}" > "${stdout_file}"
    if [[ -n "${stderr_norm}" ]]; then
        printf '%s\n' "${stderr_norm}" > "${stderr_file}"
    else
        rm -f "${stderr_file}"
    fi

    record_exit "${key}" "${exit_code}"
    echo "  [${key}] exit=${exit_code}"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

for branch in dirty clean; do
    echo "=== Building fixture: ${branch} ==="
    tmp=$(mktemp -d)

    build_fixture_repo "${branch}" "${tmp}"

    echo "--- Running checks on ${branch} ---"

    # check-hardcoded-secrets.py (INV-009)
    run_check "secrets--${branch}" "${tmp}" \
        python3 scripts/check-hardcoded-secrets.py

    # check-port-bind.py (INV-001)
    run_check "port--${branch}" "${tmp}" \
        python3 scripts/check-port-bind.py

    # check-runtime-secrets.py (INV-010)
    run_check "runtime--${branch}" "${tmp}" \
        python3 scripts/check-runtime-secrets.py

    # check-schema-safety.sh (Prisma)
    run_check "schema--${branch}" "${tmp}" \
        bash scripts/check-schema-safety.sh

    # security-gate.sh --mechanical-only (anchor #7: skip SSH INV-007)
    run_check "gate--${branch}" "${tmp}" \
        bash security-gate.sh --mechanical-only

    rm -rf "${tmp}"
done

# Usage-error run: security-gate.sh only (anchor #5/#8: only gate has exit-2 mode)
# 4 individual check scripts have no usage-error mode — "no usage-error mode" in MANIFEST §1
echo "=== Usage-error run: security-gate.sh --no-such-flag ==="
tmp=$(mktemp -d)

# Minimal setup for usage-error run — just needs the script present
cp "${GOLDEN_DIR}/security-gate.sh" "${tmp}/"
mkdir -p "${tmp}/scripts"
cp "${GOLDEN_DIR}/check-hardcoded-secrets.py" "${tmp}/scripts/"
cp "${GOLDEN_DIR}/check-port-bind.py"         "${tmp}/scripts/"
cp "${GOLDEN_DIR}/check-runtime-secrets.py"   "${tmp}/scripts/"
cp "${GOLDEN_DIR}/check-schema-safety.sh"     "${tmp}/scripts/"

run_check "gate--usage-error" "${tmp}" \
    bash security-gate.sh --no-such-flag

rm -rf "${tmp}"

# ─────────────────────────────────────────────────────────────────────────────
# Write exit_codes.json from accumulated key=value pairs
# ─────────────────────────────────────────────────────────────────────────────
{
    printf '{\n'
    first=true
    # Ordered key list
    for key in \
        "secrets--dirty" "secrets--clean" \
        "port--dirty"    "port--clean" \
        "runtime--dirty" "runtime--clean" \
        "schema--dirty"  "schema--clean" \
        "gate--dirty"    "gate--clean" \
        "gate--usage-error"; do
        val=$(grep "^${key}=" "${EXIT_CODES_TMP}" | tail -1 | cut -d= -f2)
        if [[ "${first}" == "true" ]]; then
            first=false
        else
            printf ',\n'
        fi
        printf '  "%s": %s' "${key}" "${val}"
    done
    printf '\n}\n'
} > "${PINS_DIR}/exit_codes.json"

rm -f "${EXIT_CODES_TMP}"

echo ""
echo "=== exit_codes.json ==="
cat "${PINS_DIR}/exit_codes.json"
echo ""
echo "=== Acceptance check ==="
PASS=true

check_exit_json() {
    local key="$1" expected="$2"
    local actual
    actual=$(python3 -c "import json,sys; d=json.load(open('${PINS_DIR}/exit_codes.json')); print(d.get('${key}','MISSING'))")
    if [[ "${actual}" == "${expected}" ]]; then
        echo "  OK  ${key}: exit ${actual}"
    else
        echo "  FAIL ${key}: expected exit ${expected}, got ${actual}"
        PASS=false
    fi
}

for check in secrets port runtime schema gate; do
    check_exit_json "${check}--dirty" 1
    check_exit_json "${check}--clean" 0
done
check_exit_json "gate--usage-error" 2

if [[ "${PASS}" == "true" ]]; then
    echo ""
    echo "All acceptance checks passed. Pins written to tests/golden/pins/"
else
    echo ""
    echo "FAIL: one or more acceptance checks failed. See above."
    exit 1
fi
