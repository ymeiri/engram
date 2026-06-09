# Brain Harness T371 Exact-Head Validation After T370

Date: 2026-06-08
Status: completed local CI and package/install validation for the current PR #3 candidate after
the T370 telemetry confidence refresh.

## Scope

T371 refreshes exact-head local validation after T370 moved PR #3 to
`4249855bee0fe4b33a9bd343d7750ce7a8da368f`
(`Record T370 telemetry confidence refresh`). The validation target is that T370 candidate plus
this docs-only evidence note and the release-evidence cross-references added with it.

This slice does not change source behavior, mark PR #3 ready, merge, tag, publish, close hosted
CI, run native Claude, execute `/hooks`, signal processes, mutate harness settings or adapters,
run lifecycle cleanup, run `lint apply_safe`, run M6 write-apply, delete data, or change the
supported beta scope.

## Research Question

Does the current PR #3 tree still pass the local CI-equivalent and package/install release checks
after the T370 telemetry evidence commit?

## Evidence

Current branch state before validation:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
HEAD = 4249855bee0fe4b33a9bd343d7750ce7a8da368f
HEAD...origin/yuval.meiri/memory-os-phase1 = 0 0
```

The untracked root `AGENTS.md` is user-owned instruction context and was not staged.

PR #3 state before validation:

```text
headRefOid = 4249855bee0fe4b33a9bd343d7750ce7a8da368f
isDraft = true
mergeable = MERGEABLE
mergeStateStatus = UNSTABLE
hosted run = 27141590404
hosted jobs = Check, Test, Format, Clippy, Docs
hosted job conclusions = failure
hosted job steps = []
```

The hosted jobs still match the account/billing pre-step failure pattern, not a Rust, workflow, or
package failure.

Local CI-equivalent validation passed through `./scripts/local-ci.sh`, including:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`
- `cargo doc --no-deps`

Package/install smoke passed through `./scripts/package-install-smoke.sh`:

- release binary built in `--release` mode;
- `dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz` was created;
- `dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256` verified;
- archive extracted successfully;
- packaged binary installed into a temporary prefix;
- packaged binary reported `engram 0.2.0-beta.1`;
- packaged `engram serve --http --memory` returned
  `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}` from `/health`.

## Gate Impact

T371 refreshes the local fallback proof on top of the T370 telemetry evidence. The scoped local/Codex
MVP beta remains release-logistics-limited:

- release-owner acceptance of the exact-head local CI plus package/install fallback, or
- restored exact-head hosted CI green,
- then ready/merge/tag/publish mechanics.

## Non-Claims

T371 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, execute
`/hooks`, prove prompt-bearing behavior, prove effective-hook visibility, prove live host labels,
delete legacy data, run broad lifecycle cleanup, run M6 write-apply, or make Engram production/GA
ready.

The T370 telemetry gate passage remains sampled retrieval-feedback evidence only. It does not
authorize M6 write-apply or any other write path that requires exact user approval.
