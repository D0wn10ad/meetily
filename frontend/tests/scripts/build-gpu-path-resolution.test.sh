#!/usr/bin/env bash
# Tests for frontend/build-gpu.sh:
#   (a) path resolution is independent of the caller's cwd
#   (b) unknown GPU tokens fail loudly with the recognised values
#   (c) the ORT --features parser is whitespace/comma tolerant and empty-safe
#
# shellcheck is not installed on this host, so this is a plain bash test.

set -uo pipefail

TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$TEST_DIR/../../.." && pwd -P)"
BUILD_GPU="$REPO_ROOT/frontend/build-gpu.sh"

failures=0

pass() { printf 'PASS %s\n' "$1"; }
fail() {
  printf 'FAIL %s\n' "$1"
  failures=$((failures + 1))
}

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    pass "$label"
  else
    fail "$label (expected [$expected], got [$actual])"
  fi
}

# Source only the helper functions defined at the top of build-gpu.sh.
parse() (
  set +e
  BUILD_GPU_LIB_ONLY=1 source "$BUILD_GPU"
  parse_ort_features "$@"
)

# ---------------------------------------------------------------------------
# (a) + (b): run from an unrelated cwd with an unknown GPU token.
# Validation happens before any build step, so this is fast and hermetic.
# ---------------------------------------------------------------------------
out="$(cd /tmp && TAURI_GPU_FEATURE=bogus bash "$BUILD_GPU" 2>&1)"
status=$?

if [ "$status" -ne 0 ]; then
  pass "(a/b) unknown GPU token exits non-zero (status=$status)"
else
  fail "(a/b) unknown GPU token must exit non-zero (status=$status)"
fi

if printf '%s\n' "$out" | grep -Fq "Workspace: $REPO_ROOT"; then
  pass "(a) WORKSPACE_ROOT resolves canonically to $REPO_ROOT from /tmp"
else
  fail "(a) WORKSPACE_ROOT did not resolve to $REPO_ROOT from /tmp"
  printf '     output: %s\n' "$out"
fi

if printf '%s\n' "$out" | grep -Fq "Valid values:"; then
  pass "(b) failure message lists Valid values"
else
  fail "(b) failure message does not list Valid values"
  printf '     output: %s\n' "$out"
fi

for token in coreml metal cuda vulkan hipblas openblas; do
  if printf '%s\n' "$out" | grep -Fq "$token"; then
    pass "(b) valid-value list mentions $token"
  else
    fail "(b) valid-value list is missing $token"
  fi
done

if printf '%s\n' "$out" | grep -Fqi "CPU fallback"; then
  fail "(b) unknown-token message must not say 'CPU fallback'"
else
  pass "(b) unknown-token message does not say 'CPU fallback'"
fi

# ---------------------------------------------------------------------------
# (c) ORT --features parsing
# ---------------------------------------------------------------------------
assert_eq "(c) comma list" "a,b" "$(parse --features a,b)"
assert_eq "(c) comma+space list" "a,b" "$(parse --features a, b)"
assert_eq "(c) empty value yields empty list" "" "$(parse "")"
assert_eq "(c) bare --features yields empty list" "" "$(parse --features)"
assert_eq "(c) --features= form" "a,b" "$(parse --features=a,b)"
assert_eq \
  "(c) real ORT value" \
  "ort-cuda,ort-directml" \
  "$(parse --features ort-cuda,ort-directml)"

# ---------------------------------------------------------------------------
if [ "$failures" -eq 0 ]; then
  printf 'All build-gpu script tests passed.\n'
  exit 0
fi

printf '%s test(s) failed.\n' "$failures"
exit 1
