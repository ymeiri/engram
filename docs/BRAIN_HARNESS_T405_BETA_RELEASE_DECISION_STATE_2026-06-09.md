# Brain Harness T405 Beta Release Decision State

Date: 2026-06-09
Status: completed release-owner automation hardening.

## Scope

T405 adds explicit release-decision state to `scripts/beta-release-gate-report.sh`. The slice is
evidence-only: it does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish,
mutate release state, launch native Claude, run `/hooks`, signal processes, mutate M6, or perform
lifecycle cleanup.

## Research Question

Can the beta release-gate report distinguish full exact-head evidence ready for release-owner
review from quick or incomplete evidence without performing any release action?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The report can expose `release_gate_state`, `ready_for_release_owner_review`, `hosted_ci_fallback_decision_required`, and JSON `remaining_release_actions` after existing checks pass. | Supported. |
| Null | `release_owner_decision_required=true` is enough. | Rejected because it does not distinguish quick evidence from full local/package evidence. |
| Simpler alternative | Document the distinction only in release notes. | Rejected because release-owner automation needs machine-readable state. |
| Failure | The new fields make the hosted-CI fallback look accepted. | Avoided. The fallback path reports `fallback_release_owner_decision_required`, leaves `release_owner_decision_required=true`, and keeps `release_actions_performed=false`. |

## Structured Output Contract

The report now emits:

- `release_gate_state`,
- `ready_for_release_owner_review`,
- `hosted_ci_fallback_decision_required`,
- `remaining_release_actions` in JSON mode.

The current supported states are:

- `evidence_incomplete`: local CI or package/install smoke was skipped, usually by `--quick` or a
  skip flag.
- `hosted_ci_passing_release_owner_review_required`: hosted CI is green and local/package evidence
  passed.
- `fallback_release_owner_decision_required`: hosted CI is verified as a pre-step blocker and
  local/package evidence passed.

## Validation

Validation performed for this slice:

- `bash -n scripts/beta-release-gate-report.sh`
- `git diff --check`
- `scripts/beta-release-gate-report.sh --hosted-run 27192319428 --quick --allow-tracked-changes
  --json`
- JSON field assertion that quick mode reports `evidence_incomplete` and
  `ready_for_release_owner_review=false`
- `scripts/beta-release-gate-report.sh --hosted-run 27192319428 --quick --allow-tracked-changes`
- text assertion that quick mode prints the release-decision fields
- `./scripts/beta-release-gate-report.sh --hosted-run 27192319428 --allow-tracked-changes
  --json`
- JSON field assertion that full evidence reports `fallback_release_owner_decision_required`,
  `ready_for_release_owner_review=true`, `hosted_ci_fallback_decision_required=true`, local CI
  passed, package/install smoke passed, and no release actions performed

## Gate Impact

T405 reduces release-owner ambiguity by making the beta report name the current decision state and
the remaining release actions explicitly. It does not accept the waiver, close hosted CI, mark PR
#3 ready, merge, tag, publish, close native Claude prompt-bearing proof, close effective-hook
visibility, close live host labels, complete multi-host parity, mutate M6, run lifecycle cleanup,
or make Engram production/GA ready.
