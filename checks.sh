#!/usr/bin/env bash
set -euo pipefail

passed=0
failed=0
failures=()

run_check() {
  local label="$1"
  shift
  local output
  if output=$("$@" 2>&1); then
    printf "  \033[32m✓\033[0m %s\n" "$label"
    passed=$((passed + 1))
  else
    printf "  \033[31m✗\033[0m %s\n" "$label"
    failed=$((failed + 1))
    failures+=("$(printf "\033[31m── %s ──\033[0m\n%s" "$label" "$output")")
  fi
}

printf "\n\033[1m Running checks…\033[0m\n\n"

run_check "svelte-check + TypeScript"  npm run check
run_check "ESLint"                     npm run lint
run_check "Prettier"                   npm run format:check
run_check "Cargo clippy"               cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml
run_check "Cargo test"                 cargo test --manifest-path src-tauri/Cargo.toml

printf "\n"

if [ "$failed" -gt 0 ]; then
  printf "\033[1;31m Failure output\033[0m\n\n"
  for f in "${failures[@]}"; do
    printf "%s\n\n" "$f"
  done
fi

printf "\033[1m Results: \033[32m%d passed\033[0m" "$passed"
if [ "$failed" -gt 0 ]; then
  printf ", \033[31m%d failed\033[0m" "$failed"
fi
printf "\n\n"

exit "$failed"
