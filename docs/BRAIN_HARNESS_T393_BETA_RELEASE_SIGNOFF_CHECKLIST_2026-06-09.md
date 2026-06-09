# Brain Harness T393 Beta Release Signoff Checklist

Date: 2026-06-09
Branch: `yuval.meiri/memory-os-phase1`
Head reviewed: `4182a47d70c7a2af0786cf1f3cc04d9d2d90e8be`

## Research Question

Can the remaining `v0.2.0-beta.1` release-management gap be reduced to an explicit,
auditable release-owner decision without changing source behavior or widening the beta scope?

## Hypotheses

- Preferred: a single checklist that names the exact accepted evidence, hosted-CI exception,
  known beta limitations, and manual release actions makes the beta decision clear enough to ship.
- Null: existing release notes already state the same facts, so a checklist does not improve the
  release gate.
- Failure: the checklist accidentally accepts hosted-CI fallback, marks PR #3 ready, or implies
  production/GA readiness without release-owner action.

## Current Evidence

The current PR #3 candidate head is:

```text
4182a47d70c7a2af0786cf1f3cc04d9d2d90e8be
```

Local validation passed on that exact worktree:

```bash
./scripts/local-ci.sh
./scripts/package-install-smoke.sh
```

The package smoke rebuilt the `0.2.0-beta.1` archive, verified its checksum, installed the
packaged binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`, and verified packaged
HTTP `/health`.

Hosted GitHub Actions run `27176297515` targeted the same T392 head but failed before
workflow-step execution with `steps=[]` for all jobs. This is treated as an external
account/billing/spending-limit blocker, not as a Rust, docs, or packaging failure.

Fresh AI Council review on 2026-06-09 estimated scoped MVP-beta technical completeness at 95%,
96%, and 100%. The conservative synthesized call is 96-98% beta-complete if exact-head local CI
plus package/install smoke are accepted as the beta release gate. Release-management completion
remains lower until explicit signoff, ready/merge/tag/publish, and release artifact publication
are performed.

## Release-Owner Signoff Checklist

The release owner should explicitly confirm all of the following before the beta is tagged:

1. Accept `./scripts/local-ci.sh` on head `4182a47d70c7a2af0786cf1f3cc04d9d2d90e8be` as the
   local CI-equivalent beta gate while hosted GitHub Actions is externally blocked.
2. Accept `./scripts/package-install-smoke.sh` on the same head as package/checksum/install/health
   proof for `0.2.0-beta.1`.
3. Waive hosted run `27176297515` for this beta only because it failed before workflow-step
   execution with `steps=[]`.
4. Confirm that the beta scope is the supported local/Codex Brain Loop path described in
   `docs/RELEASE_NOTES_V0_2_0_BETA_1.md`.
5. Confirm that the known beta limitations below are acceptable for the initial beta.
6. After signoff, mark PR #3 ready, merge the signed-off head, tag `v0.2.0-beta.1`, publish the
   release archive and checksum, and verify the published install commands.

## Known Beta Limitations

These remain known limitations for `v0.2.0-beta.1` and must not be interpreted as closed by this
checklist:

- hosted GitHub Actions is not green on the current head because of the external pre-step blocker;
- native Claude prompt-bearing behavior is not proved;
- effective-hook visibility is not proved;
- live Claude host-label proof is not closed;
- full multi-host parity is not claimed;
- direct legacy deprecation/deletion remains deferred;
- broad lifecycle cleanup and broad `lint apply_safe` remain deferred;
- M6 write-apply expansion remains deferred beyond the reviewed current-data batch;
- exhaustive telemetry, auth, ops, performance, and cross-platform hardening remain GA work.

## Boundary

T393 is documentation and release-gate clarification only. It does not accept the hosted-CI
fallback, mark PR #3 ready, merge, tag, publish, change source behavior, mutate Memory OS state,
launch native Claude, run `/hooks`, prove live host labels, run lifecycle cleanup, or make Engram
production/GA complete.
