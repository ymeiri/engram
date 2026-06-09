#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

run_step() {
    local name="$1"
    shift
    printf '\n==> %s\n' "$name"
    "$@"
}

run_step "git diff whitespace check" git diff --check
run_step "rustfmt" cargo fmt --all --check
run_step "cargo check" cargo check --locked --all-targets
run_step "clippy" cargo clippy --locked --all-targets -- -D warnings
run_step "tests" env CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --locked --all-targets --jobs 1
run_step "docs" cargo doc --locked --no-deps

printf '\nLocal CI-equivalent validation passed.\n'
