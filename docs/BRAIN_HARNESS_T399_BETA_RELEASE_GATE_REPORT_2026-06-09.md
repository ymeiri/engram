# T399 Beta Release Gate Report

Date: 2026-06-09

## Research Question

Can the remaining `v0.2.0-beta.1` release-owner decision use one repeatable command to collect
branch, PR, hosted-CI fallback, local CI, and package/install evidence without performing release
actions?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A small release-gate report script can fail closed on branch/PR/head drift and collect the same evidence the release owner must review. |
| Null | The existing individual commands are enough, and a wrapper only duplicates release process text. |
| Simpler alternative | Keep the release notes checklist and ask the release owner to run each command manually. |
| Failure | The script could make waiver acceptance look automatic or pass while checking the wrong head. |

## Measurement

The report must fail unless:

- the current branch has an upstream and is synced with it,
- tracked working-tree and index changes are absent unless explicitly allowed for development,
- PR #3 points at the local `HEAD`,
- hosted CI is either green for the expected release-gate jobs or the hosted pre-step blocker
  verifier passes for the same `HEAD`,
- the local CI and package/install smoke commands pass unless the caller explicitly requests a
  quick or skipped run.

The report must also print that release-owner decision is still required and that release actions
were not performed.

## Implementation

`./scripts/beta-release-gate-report.sh` now collects release-owner evidence. By default it:

1. checks branch sync and clean tracked source state,
2. checks PR #3 head alignment with the local `HEAD`,
3. verifies hosted CI success or delegates to
   `./scripts/verify-hosted-ci-prestep-blocker.sh`,
4. runs `./scripts/local-ci.sh`,
5. runs `./scripts/package-install-smoke.sh`,
6. prints a concise evidence summary.

The script supports `--quick`, `--skip-local-ci`, `--skip-package-smoke`, `--hosted-run <id>`,
`--pr <number>`, and `--allow-tracked-changes` for development and status checks.

## Validation

- `bash -n scripts/beta-release-gate-report.sh`
- `./scripts/beta-release-gate-report.sh --quick --allow-tracked-changes --hosted-run 27182460048`

The final signed-off invocation should be run after this slice is committed and pushed so the PR
head, hosted run, manifest git head, local CI, and package/install smoke all refer to the same
commit.

## Decision

The beta release-owner review now has a single evidence command for the current fallback gate. This
reduces release-process drift without changing the release decision.

## Boundary

T399 does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native
Claude, run `/hooks`, prove effective-hook visibility, prove live host labels, mutate M6, run broad
lifecycle cleanup, or make Engram production/GA ready.
