// P001 fixture: clean secrets — no INV-009 violation
// Reads secrets from environment variables only (allowlist: process.env.)
const API_TOKEN = process.env.API_TOKEN;
const api_key = process.env.API_KEY;
