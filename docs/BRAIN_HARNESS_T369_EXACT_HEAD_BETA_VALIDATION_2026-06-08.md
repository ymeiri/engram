# Brain Harness T369 Exact-Head Beta Validation

Date: 2026-06-08
Status: completed exact-head local CI and package/install validation for the scoped local/Codex
MVP beta path.

## Scope

T369 refreshes beta acceptance evidence after T368. The validation target is the current PR #3
candidate based on head `8dad84e37b6bc033904319c675683345c7b60972`
(`Record T368 native Claude attribution blocker`), plus this docs-only evidence note.

This slice does not change source behavior, mark PR #3 ready, merge, tag, publish, close hosted
CI, run native Claude, execute `/hooks`, signal processes, mutate harness settings or adapters,
run broad lifecycle cleanup, run `lint apply_safe`, delete data, or change the supported beta
scope.

## Research Question

Can the current PR #3 candidate be treated as locally validated for the scoped local/Codex
`v0.2.0-beta.1` beta path while hosted GitHub Actions remains externally blocked before workflow
steps?

## Decision

Yes, for the scoped beta fallback path. The current candidate has fresh local CI-equivalent proof
and fresh package/install proof. The remaining beta gate is still a release-owner decision:
restore hosted exact-head CI and get it green, or explicitly accept the local CI plus package
install fallback while hosted CI is account-blocked.

This is not a production/GA readiness claim.

## Evidence

Current branch state before validation:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
HEAD = 8dad84e37b6bc033904319c675683345c7b60972
HEAD...origin/yuval.meiri/memory-os-phase1 = 0 0
```

The untracked root `AGENTS.md` is user-owned instruction context and was not staged.

PR #3 hosted CI state remains external:

```text
PR #3 head = 8dad84e37b6bc033904319c675683345c7b60972
isDraft = true
mergeable = MERGEABLE
mergeStateStatus = UNSTABLE
hosted run = 27138579667
hosted jobs = Docs, Format, Clippy, Check, Test
hosted job conclusions = failure
hosted job steps = []
```

The hosted jobs still match the billing/spending-limit pre-step failure pattern, not a Rust,
workflow, or package failure.

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

Daemon and Memory OS health checks after the validation slice:

```text
engram daemon status:
Daemon status: running
Port: 8765
PID: 47577
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1

vault status /Users/yuval.meiri/.engram/vault:
generated_file_count = 2677
expected_generated_file_count = 2677
user_file_count = 0

obligations doctor --scope-project engram:
open = []
warnings = []
```

## Gate Impact

T369 removes the stale "current head only has docs-only proof after T366" concern for the scoped
local/Codex beta path. The beta release decision is now narrowed to:

- release-owner acceptance of exact-head local CI plus package/install fallback, or
- restored exact-head hosted CI green.

After that decision, the remaining mechanics are marking PR #3 ready, merging, tagging
`v0.2.0-beta.1`, publishing the artifact, and running a post-publish smoke.

## Non-Claims

T369 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, execute
`/hooks`, prove prompt-bearing behavior, prove effective-hook visibility, prove live host labels,
delete legacy data, run broad lifecycle cleanup, run M6 write-apply, or make Engram production/GA
ready.

Native Claude prompt-bearing proof, effective-hook proof, and live host-label proof remain blocked
by current attribution preflight state unless a fresh preflight proves otherwise and the user gives
exact approval. The 50-trace telemetry confidence gate remains GA/write-apply hardening, not a
scoped beta blocker.
