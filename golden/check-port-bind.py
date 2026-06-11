#!/usr/bin/env python3
"""
INV-001 enforcer: scan docker-compose files for port bindings,
allow only 127.0.0.1:* (loopback) and "80:80" + "443:443" (nginx public).

Exit 0 = clean. Exit 1 = violation(s) printed.
"""
import re
import sys
from pathlib import Path

COMPOSE_FILES = [
    "docker-compose.yml",
    "docker-compose.dev.yml",
    "astro-service/docker-compose.yml",
]

# Allowlist: loopback IP bind + nginx public ports
ALLOWED_PUBLIC = {"80:80", "443:443"}
# Matches: optional indent + `- ` + optional quote + port spec + optional quote
PORT_LINE_RE = re.compile(r'^\s*-\s*"?([^"]+?)"?\s*$')

def is_in_ports_block(lines, idx):
    """Walk back to find if this line is under a `ports:` key."""
    for i in range(idx - 1, -1, -1):
        line = lines[i]
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        # If we hit another `key:` at same/less indent -> not in ports:
        if stripped.endswith(":") and not stripped.startswith("-"):
            return stripped == "ports:"
        # Still in list items
        if stripped.startswith("-"):
            continue
        return False
    return False

def classify(spec):
    """Return 'ok' or violation reason for a port spec like '127.0.0.1:8001:8001' or '8001:8001'."""
    parts = spec.split(":")
    if len(parts) == 3:
        # IP:HOST:CONTAINER
        ip, host, container = parts
        if ip == "127.0.0.1":
            return "ok"
        return f"public IP bind: {spec}"
    if len(parts) == 2:
        # HOST:CONTAINER (no IP = 0.0.0.0 default)
        if spec in ALLOWED_PUBLIC:
            return "ok"
        return f"implicit 0.0.0.0 bind: {spec}"
    return f"unrecognized format: {spec}"

def main():
    violations = []
    for fname in COMPOSE_FILES:
        path = Path(fname)
        if not path.exists():
            print(f"WARN: {fname} not found, skipping", file=sys.stderr)
            continue
        lines = path.read_text().splitlines()
        for idx, line in enumerate(lines):
            m = PORT_LINE_RE.match(line)
            if not m:
                continue
            spec = m.group(1).strip()
            # Skip if not numeric/IP-ish (could be other list items)
            if not re.match(r'^[\d.:]+$', spec):
                continue
            if not is_in_ports_block(lines, idx):
                continue
            result = classify(spec)
            if result != "ok":
                violations.append(f"{fname}:{idx + 1}: INV-001 violated -- {result}")
    if violations:
        print("\n".join(violations))
        sys.exit(1)
    print("INV-001: PASS (port bindings clean)")

if __name__ == "__main__":
    main()
