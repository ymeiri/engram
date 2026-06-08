# Brain Harness T388 Exact-Head Release Validation

Date: 2026-06-08
Branch: `yuval.meiri/memory-os-phase1`
Head validated: the commit containing this document.

## Research Question

Does the current PR #3 head still satisfy the scoped local/Codex beta release validation path after
the T384-T387 lifecycle, evidence, session, and coordination cleanup slices?

## Hypotheses

- Preferred: The T388 head remains locally release-valid because the recent commits are docs and
  Memory OS state hygiene, not source behavior changes.
- Falsifier: `./scripts/local-ci.sh` or `./scripts/package-install-smoke.sh` fails on the exact
  T388 head.

## Scope

This slice validates the current head only. It does not mark PR #3 ready, merge, tag, publish,
accept the hosted-CI fallback, launch native Claude, run hooks, mutate M6, or change production/GA
scope.

## Validation

The exact T388 head passed:

```bash
./scripts/local-ci.sh
```

The script covered:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- CI-like `cargo test --all-targets --jobs 1`
- `cargo doc --no-deps`

It completed with:

```text
Local CI-equivalent validation passed.
```

The exact T388 head also passed:

```bash
./scripts/package-install-smoke.sh
```

The package smoke rebuilt the release binary and archive, verified the checksum, extracted the
archive, installed the binary into a temporary prefix, started packaged HTTP service with in-memory
storage, and verified version/health output.

Artifacts verified:

```text
/Users/yuval.meiri/projects/engram/dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz
/Users/yuval.meiri/projects/engram/dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256
engram 0.2.0-beta.1
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Decision

The scoped local/Codex beta validation path is current for the PR #3 head containing this document.

## Remaining Gates

Hosted GitHub Actions remains externally blocked before workflow-step execution. The latest tracked
hosted run before this slice was `27165862509`, where all jobs failed with `steps=[]`.

The release gate is unchanged:

- restore hosted CI and pass exact-head hosted checks; or
- get explicit release-owner acceptance of exact-head local CI plus package/checksum/install smoke
  as the beta fallback, then mark PR #3 ready, merge, tag, and publish `v0.2.0-beta.1`.

Production/GA gates remain outside this beta validation:

- native Claude prompt-bearing proof
- effective-hook visibility
- live host-label proof
- full multi-host parity
- direct legacy deprecation/deletion
- M6 write-apply expansion
- broader lifecycle cleanup
- telemetry/auth/ops/performance/cross-platform hardening
