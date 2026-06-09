# Brain Harness T366 Beta Acceptance Refresh

Date: 2026-06-08
Status: completed local beta acceptance refresh. No source code behavior, harness files, native
Claude sessions, hosted CI configuration, broad `lint apply_safe`, release tags, or user-owned
files were changed.

## Research Question

Can the current PR #3 candidate be treated as locally accepted for the scoped local/Codex
`v0.2.0-beta.1` path while hosted GitHub Actions remains externally blocked before workflow steps?

## Decision

Yes, for the scoped beta release fallback. The current candidate has fresh local CI-equivalent
evidence and fresh package/install evidence. The remaining beta decision is release-owner
acceptance of that fallback, or restored hosted CI, followed by PR-ready, merge, tag, and publish
mechanics.

This is not a production/GA readiness claim.

## Evidence

PR state before the acceptance refresh:

- PR #3 was draft, `mergeable=MERGEABLE`, and `mergeStateStatus=UNSTABLE`.
- The branch was synchronized with `origin/yuval.meiri/memory-os-phase1`.
- Hosted CI check runs on head `872f2606b363421473dff2881623a661d2710278` still failed before
  workflow execution. Check, Test, Format, Docs, and Clippy reported failed check runs with no
  runner execution evidence.

Local CI-equivalent commands passed:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`
- `cargo clippy --all-targets -- -D warnings`
- `cargo doc --no-deps`

Package/install smoke passed:

- `./scripts/package-install-smoke.sh` built the release package.
- The script verified
  `dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`.
- The packaged binary installed into a temporary prefix.
- `engram --version` returned `engram 0.2.0-beta.1`.
- Packaged `engram serve --http --memory` passed `/health` with
  `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

## Remaining Beta Gate

One of these must happen before tagging:

- hosted GitHub Actions is restored and passes on the release head, or
- the release owner explicitly accepts the local CI-equivalent plus package/install smoke fallback
  for this beta while hosted CI is externally account-blocked.

After that decision, the remaining beta mechanics are:

- mark PR #3 ready,
- merge,
- tag `v0.2.0-beta.1`,
- publish the release artifact.

## Non-Claims

T366 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, execute
`/hooks`, prove prompt-bearing behavior, prove effective-hook visibility, prove live host labels,
delete legacy data, run broad lifecycle cleanup, or make Engram production/GA ready.
