#!/usr/bin/env bash
# env-check.sh — verifiable environment contract for tauricode (TAURICODE-GUIX-LAYER).
#
# Usage: guix shell -m manifest.scm --pure -- bash scripts/env-check.sh
#
# Layers:
#   1. Guix-declared tools (manifest.scm) must resolve INSIDE the shell.
#   2. bun is not in guix: it must exist, and its version MUST equal
#      .bun-version (which mirrors package.json's packageManager field).
#      An unpinned ambient bun is exactly the "arbitrary global install"
#      failure mode this task exists to close.
# Exits non-zero listing every drift found; silent success = clean.
set -u
fail=0

say() { printf '%s\n' "$*"; }
bad() { say "DRIFT: $*"; fail=$((fail + 1)); }

check_guix_tool() {
  local tool="$1"
  if command -v "$tool" >/dev/null 2>&1; then
    say "ok   $tool ($("$tool" --version 2>/dev/null | head -1))"
  else
    bad "$tool not on PATH — run via: guix shell -m manifest.scm --pure -- bash $0"
  fi
}

for t in rustc cargo git; do check_guix_tool "$t"; done

# --- bun layer (host-side by design: guix does not package bun) ---
if [ -f .bun-version ]; then
  want="$(tr -d '[:space:]' < .bun-version)"
else
  bad ".bun-version missing"; want=""
fi
# Under `--pure` the ambient PATH is gone, so probe the canonical
# ~/.bun install location directly instead of pretending it's ambient.
BUN_BIN="$(command -v bun 2>/dev/null || true)"
if [ -z "$BUN_BIN" ] && [ -x "$HOME/.bun/bin/bun" ]; then
  BUN_BIN="$HOME/.bun/bin/bun"
fi
if [ -n "$BUN_BIN" ]; then
  got="$("$BUN_BIN" --version 2>/dev/null)"
  if [ -n "$want" ] && [ "$got" != "$want" ]; then
    bad "bun version drift: have $got, repo pins $want (.bun-version / package.json packageManager)"
  else
    say "ok   bun ($got at $BUN_BIN, matches pin)"
  fi
else
  bad "bun not found (PATH or ~/.bun/bin) — install pinned ${want:-1.3.14}"
fi

if [ "$fail" -eq 0 ]; then
  say "env-check: CLEAN"
else
  say "env-check: $fail drift(s) found"
fi
exit $((fail > 0))
