// P001 fixture: dirty secrets — triggers INV-009
// Pattern 1: github-pat (golden/check-hardcoded-secrets.py:69)
const API_TOKEN = "ghp_FAKETOKEN000000000000000000000000000";
// Pattern 2: generic-entropy (golden/check-hardcoded-secrets.py:80-81)
const api_key = "FAKEKEYABCDEFGHIJKLMNOPQRSTUVWXYZ12345678";
