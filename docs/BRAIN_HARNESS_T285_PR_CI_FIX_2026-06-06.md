# Brain Harness T285 PR CI Fix - 2026-06-06

## Scope

T285 fixes the first failing GitHub Actions run on draft PR
`https://github.com/ymeiri/engram/pull/2`.

T285 does not run native Claude, change Memory OS ranking, mutate lifecycle state, run M6, deprecate
legacy data, or mark the PR ready for review.

## Failing Run

Workflow run `27057416611` ran against PR merge commit
`9d424b3f4d27bad5eb88496fe194d603ac673152` for branch head
`239271d0b095433df984c957d2ad23799d323528`.

| Check | Result | Relevant failure |
| --- | --- | --- |
| Check | Passed | `cargo check --all-targets` completed in 1m35s. |
| Format | Passed | `cargo fmt --all --check` completed in 17s. |
| Docs | Passed | `cargo doc --no-deps` completed in 1m8s. |
| Clippy | Failed | Three `clippy::unnecessary_sort_by` errors in `engram-store/src/repos/memory.rs`. |
| Test | Failed | `cargo test --all-targets` hit `rust-lld` signal 7 bus errors while linking integration-test binaries. |

## Changes

- Replaced three timestamp comparisons with `sort_by_key` in
  `engram-store/src/repos/memory.rs`.
- Changed the CI Test job from `cargo test --all-targets` to
  `cargo test --all-targets --jobs 1` to serialize build/link work and reduce concurrent linker
  pressure on the GitHub runner.

## Local Verification

The following commands passed locally after the fix:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets --jobs 1`

## Non-Claims

T285 does not prove full PR readiness. It fixes the observed CI failures and provides local
validation for the updated commands. The pushed branch still needs a fresh remote GitHub Actions
run on the new commit.
