# T397 Hosted CI Pre-Step Verifier

Date: 2026-06-09

## Research Question

Can the beta release-owner waiver condition for externally blocked hosted CI be verified with a
repeatable, fail-closed command instead of manual GitHub inspection?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A read-only script can prove that a named GitHub Actions run targets the exact local head, is the expected `CI` pull-request workflow, and has only the expected release-gate jobs completed as failures with `steps=[]`. |
| Null | The available GitHub CLI data is too weak or inconsistent, so manual inspection remains required. |
| Simpler alternative | Keep the signoff checklist manual and only record the current run in release notes. |
| Failure | The script could accidentally treat a real source/test failure as an external pre-step blocker, or silently validate a run for the wrong head. |

## Measurement

The verifier must fail unless all of the following are true:

- the run head SHA equals the expected current head,
- the workflow name is `CI`,
- the event is `pull_request`,
- the run is `completed` with conclusion `failure`,
- the job set is exactly `Check`, `Test`, `Format`, `Clippy`, and `Docs`,
- each job is `completed`, has conclusion `failure`, and has zero recorded workflow steps.

The command is evidence for the waiver condition only. It does not accept the waiver, mark the PR
ready, merge, tag, publish, or close hosted CI.

## Implementation

Added:

```text
scripts/verify-hosted-ci-prestep-blocker.sh
```

The script uses `gh run view` and `jq` to inspect a supplied run ID, or discovers the latest
pull-request `CI` run for the current branch when no ID is supplied. It defaults the expected head
to `git rev-parse HEAD`, and supports `EXPECTED_HEAD_SHA` / `EXPECTED_WORKFLOW_NAME` overrides for
explicit release checks.

T403 later added machine-readable success output:

```bash
scripts/verify-hosted-ci-prestep-blocker.sh --json <run-id>
```

## Evidence

The current PR #3 hosted run was verified with:

```bash
scripts/verify-hosted-ci-prestep-blocker.sh 27180509992
```

Result:

```text
Hosted CI pre-step blocker verified:
  run: 27180509992
  url: https://github.com/ymeiri/engram/actions/runs/27180509992
  head: 2368919745cea3050217da9bdc8bd1d6a8435636
  workflow: CI
  jobs: Check Test Format Clippy Docs
  condition: all expected jobs completed with conclusion=failure and steps=[]
```

## Validation

- `scripts/verify-hosted-ci-prestep-blocker.sh 27180509992`
- `EXPECTED_HEAD_SHA=0000000000000000000000000000000000000000 scripts/verify-hosted-ci-prestep-blocker.sh 27180509992` failed closed with a run-head mismatch.
- `bash -n scripts/verify-hosted-ci-prestep-blocker.sh`
- `git diff --check`
- `cargo fmt --all --check`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

## Decision

The hosted-CI waiver condition is now repeatably checkable for the current beta head. The remaining
beta gate is still the release-owner decision to accept that condition together with exact-head
local CI and package/install smoke evidence, or a restored hosted CI run that executes normally and
passes.

## Boundary

T397 does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native
Claude, run `/hooks`, prove effective-hook visibility, prove live host labels, mutate M6, run broad
lifecycle cleanup, or make Engram production/GA ready.
