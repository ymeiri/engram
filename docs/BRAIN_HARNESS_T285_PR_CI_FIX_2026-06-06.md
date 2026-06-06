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

The first pushed T285 fix reached run `27058068496`. That run used the serialized Test command,
but Clippy on Rust `1.96.0` surfaced one additional `clippy::collapsible_match` warning in
`engram-index/src/harness.rs` that was not emitted by the local Rust `1.93.0` toolchain.

The second pushed T285 fix reached run `27058785227`. That run passed Check, Format, Docs, and
Clippy, but Test still failed while linking `engram-mcp`'s test binary after the runner reported
only 87 MB of free disk space. The failure was again `rust-lld` signal 7 / bus error and not a Rust
test assertion failure.

## Changes

- Replaced three timestamp comparisons with `sort_by_key` in
  `engram-store/src/repos/memory.rs`.
- Collapsed the durable `sessionend` branch in `engram-index/src/harness.rs` into a match guard so
  Rust 1.96 Clippy accepts it.
- Changed the CI Test job from `cargo test --all-targets` to
  `cargo test --all-targets --jobs 1` to serialize build/link work and reduce concurrent linker
  pressure on the GitHub runner.
- Added Test-job runner cleanup, disabled incremental/debug-info-heavy dev builds, and stopped
  restoring cached target artifacts so the GitHub runner has more disk headroom for the heavy test
  link step.

## Local Verification

The following commands passed locally after the fix:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets --jobs 1`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`

The CI-specific disk cleanup is validated by a fresh pushed GitHub Actions run rather than by local
filesystem behavior.

## Remote Recheck

T286 records the fresh pushed GitHub Actions run after the disk-headroom fix. Run `27059846266`
completed successfully on head `54c12eb20eefe1f69f162d9151b66868c120a70d`; Check, Format, Docs,
Clippy, and Test all passed. The Test job completed in `42m54s`.

## Non-Claims

T285 does not prove full PR readiness. It fixes the observed CI failures and provides local
validation for the updated commands. T286 closes the remote CI recheck for the T285 fix head, but
PR readiness/review follow-up remains separate.
