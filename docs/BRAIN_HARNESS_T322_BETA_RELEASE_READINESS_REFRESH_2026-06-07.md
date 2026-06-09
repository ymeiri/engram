# T322 Beta Release Readiness Refresh

Date: 2026-06-07
Status: completed docs-only release-readiness refresh

Supersession note: this packet records the T322 point-in-time state. T330 is the current
release-facing evidence packet after PR #3 advanced to
`fe46d0a73d39e3309b149703dda4c108da91fc02` and hosted run `27096981016` remained externally
billing/spending-limit blocked before workflow steps.

## Question

After T321 and the fresh AI Council MVP assessment, what is the current beta-readiness state for
PR #3, and what remains before shipping the initial `v0.2.0-beta.1` local/Codex beta?

## Current PR State

PR #3 is draft/open at head:

```text
22bd01ccc0276ba41d846ef368ab950869a83da5
```

The branch is synced with its upstream. The only untracked local file is the user-owned root
`AGENTS.md`.

Hosted GitHub Actions run `27092233443` on this head failed before workflow steps ran. All five
check runs completed with billing/spending-limit annotations, so the run is an external account
gate, not source-failure evidence.

## Readiness Assessment

Fresh AI Council review on 2026-06-07 reconfirmed the release boundary:

- Initial MVP beta should be judged as the supported local/Codex Brain Loop path.
- The old `35%` framing is a production/GA-parity maturity frame, not an MVP-readiness metric.
- Current beta readiness is about `90-93%` while exact-head hosted CI remains externally blocked,
  and about `95%` if the release owner explicitly accepts the local validation fallback.
- Production/GA Brain OS readiness remains materially lower, about `40-50%`, because host parity,
  native Claude proof, effective-hook proof, telemetry completeness, and production operations
  remain open.

## Beta Blockers

The remaining beta gates are procedural and evidence-presentation gates:

- record the current-head local validation and hosted-CI billing blocker in release-facing text;
- keep the supported beta scope explicit as local/Codex Brain Loop only;
- decide whether hosted CI must be fixed before release or whether local validation is accepted as
  the fallback for this beta;
- receive explicit release-owner approval before marking PR #3 ready, merging, tagging, or
  publishing.

## Explicit Deferrals

These remain safe deferrals for this beta and should not block the initial local/Codex release:

- native Claude prompt-bearing proof,
- effective-hook proof,
- live host-label proof,
- full multi-host parity,
- direct legacy deletion/deprecation,
- exhaustive lifecycle cleanup or broad `lint apply_safe`,
- telemetry completeness,
- packaging, performance, and cross-platform polish,
- new feature work.

## Boundary

T322 does not run native Claude, run `/hooks`, repair adapters, edit harness settings, change code,
change CI workflow behavior, archive memory, mutate M6/vault/lifecycle state, mark PR #3 ready,
merge, tag, publish, or release.
