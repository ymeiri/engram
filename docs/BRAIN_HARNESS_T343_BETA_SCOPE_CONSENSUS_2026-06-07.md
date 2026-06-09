# Brain Harness T343 Beta Scope Consensus

Date: 2026-06-07
Branch: `yuval.meiri/memory-os-phase1`
Current head: `966dc00d5248ac342b156974b5392700706f3139`
PR: https://github.com/ymeiri/engram/pull/3

## Research Question

For a 24-hour `0.2.0-beta.1` decision, what is still a true blocker for the scoped
local/Codex MVP beta, and what can be explicitly deferred without claiming production/GA readiness?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The scoped local/Codex MVP beta is shippable if the release owner accepts exact-head local validation while hosted CI is externally blocked. |
| Null | Red hosted GitHub checks make the PR unshippable regardless of local validation because they may indicate source failure. |
| Simpler alternative | Use the existing PR body and release notes without another tracked consensus packet. |
| Failure | The beta wording accidentally claims production/GA readiness or treats deferred native-Claude/effective-hook/host-label gates as closed. |

## Measurement

The decision uses:

- current git state: branch synced with upstream at `966dc00d5248ac342b156974b5392700706f3139`;
- PR #3 body, which records T342 exact-head local CI-equivalent validation on `966dc00`;
- hosted GitHub Actions run `27101972733`, whose five jobs all failed with `steps: []`;
- fresh AI Council broadcast on 2026-06-07 with three successful model responses;
- current release notes, architecture, and implementation-plan beta-scope wording.

Evidence level: L1 model critique plus L2 source/doc/PR inspection. This is release-scope
judgment evidence, not behavioral production-readiness proof.

## Evidence

Current PR #3 body records that head `966dc00d5248ac342b156974b5392700706f3139` passed:

- `git diff --check`
- `cargo check --all-targets`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo doc --no-deps`

The latest hosted run on the same head is `27101972733`. `gh run view` reports `steps: []` for
Check, Test, Format, Clippy, and Docs, matching the prior external account/billing/spending-limit
pattern rather than workflow-step source failures.

AI Council consensus:

- 3/3 models treat the scoped local/Codex beta as shippable if the fallback decision is explicit.
- 3/3 models reject production/GA readiness.
- 3/3 models treat native Claude prompt-bearing proof, effective-hook proof, live host-label proof,
  full host parity, broad lifecycle cleanup, direct legacy deletion, exhaustive telemetry, auth
  edges, and new features as beta deferrals.
- The strongest shared blocker is not source readiness; it is release hygiene: visible local
  validation evidence, explicit CI-blocker wording, accurate scope docs, and a release-owner
  decision if hosted CI remains blocked.

One model framed the beta as `100%` complete for the narrowly defined beta scope after explicit
fallback acceptance. The safer synthesized percentage remains `95-98%` before release mechanics,
because PR-ready/merge/tag/publish still require either hosted CI restoration or release-owner
fallback acceptance.

## Decision

The local/Codex MVP beta is effectively ready to ship from a source and scoped-product standpoint,
but PR #3 must remain draft unless one of these release gates closes:

1. hosted GitHub Actions is restored and exact-head hosted CI passes; or
2. the release owner explicitly accepts exact-head local validation as the beta fallback while the
   hosted CI account gate remains external.

The production/GA Brain OS remains incomplete. The current beta decision does not close native
Claude prompt-bearing behavior, effective-hook visibility, live Claude host labels, full multi-host
parity, broad lifecycle cleanup, direct legacy deprecation/deletion, exhaustive telemetry, auth
edge hardening, packaging, performance, or cross-platform polish.

## Falsifier

This decision is false if any scoped local/Codex MVP path has a known P0/P1 bug, if the recorded
local validation cannot be reproduced on the release head, if hosted CI failures begin executing
workflow steps and reveal source failures, or if release-facing docs claim support beyond the
validated local/Codex beta path.

## Boundary

T343 does not mark PR #3 ready, merge, tag, publish, close hosted CI, launch native Claude, run
`/hooks`, signal processes, mutate settings/adapters, prove prompt-bearing behavior, prove
effective-hook visibility, prove live host labels, run `lint apply_safe`, archive memory, or claim
production/GA Brain Harness completion.
