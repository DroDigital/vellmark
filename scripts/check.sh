#!/usr/bin/env bash
# The checks a change must pass before it is committed: every test in the workspace,
# the shipping build, and the license rules. Long output goes to logs/.
set -uo pipefail
cd "$(dirname "$0")/.."
mkdir -p logs
fail=0

echo "tests..."
if ! cargo test --workspace > logs/test.log 2>&1; then
  grep -E "FAILED|panicked|^error" -A6 logs/test.log | head -60
  fail=1
fi
grep -E "^warning: unused|^warning: .*never (used|read)" logs/test.log | sort -u | head

echo "shipping build (no development tools)..."
if ! cargo build -p vellmark-app --no-default-features > logs/shipping-build.log 2>&1; then
  grep -E "^error" -A12 logs/shipping-build.log | head -40
  fail=1
fi

echo "licenses..."
if command -v cargo-deny >/dev/null 2>&1; then
  if ! cargo deny check licenses bans sources > logs/deny.log 2>&1; then
    tail -20 logs/deny.log
    fail=1
  fi
else
  echo "  cargo-deny is not installed; skipped"
fi

if [ "$fail" -eq 0 ]; then echo "all checks passed"; else echo "CHECKS FAILED"; fi
exit "$fail"
