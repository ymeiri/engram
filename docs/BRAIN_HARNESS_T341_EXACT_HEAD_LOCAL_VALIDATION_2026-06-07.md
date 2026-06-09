# Brain Harness T341 Exact-Head Local Validation

Date: 2026-06-07
Branch: `yuval.meiri/memory-os-phase1`
Head: `2fa5b577bda8ab6141e0f7272736044d441a7e88`
PR: https://github.com/ymeiri/engram/pull/3

## Purpose

Record exact-head local CI-equivalent validation for PR #3 after hosted GitHub Actions remained
externally blocked before workflow-step execution.

## Current State

At validation time, PR #3 head `2fa5b577bda8ab6141e0f7272736044d441a7e88` was synced with its
upstream branch. The only untracked worktree item was root `AGENTS.md`, which is user-owned and
intentionally unstaged.

Hosted GitHub Actions run `27101388242` failed all five jobs before executing workflow steps. The
job records for Check, Test, Format, Clippy, and Docs all report `steps: []`, matching the earlier
account/billing/spending-limit failure pattern. This is not source-failure evidence.

## Local CI-Equivalent Validation

That head passed the workflow-equivalent local gate:

- `git diff --check`
- `cargo check --all-targets`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo doc --no-deps`

## Council Consensus

Fresh AI Council review after this validation agreed that the scoped local/Codex MVP beta is
shippable if the release owner accepts local validation as the fallback while hosted CI is
externally blocked. Hosted CI restoration and native-Claude/effective-hook/live-host-label proof
remain ops or production/GA follow-up gates unless the release owner explicitly requires hosted CI
for beta.

## Boundary

T341 does not mark PR #3 ready, merge, tag, publish a release, fix hosted GitHub Actions, launch
native Claude, run `/hooks`, signal processes, mutate settings or adapters, prove prompt-bearing
behavior, prove effective-hook visibility, prove live host labels, run `lint apply_safe`, or close
production/GA Brain Harness parity.
