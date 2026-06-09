# Brain Harness T404 Embedded Hosted Verifier Evidence

Date: 2026-06-09
Status: completed structured release-gate aggregation hardening.

## Scope

T404 embeds the hosted-CI pre-step verifier's structured success object inside the beta release
gate report JSON. The slice is evidence-only: it does not accept the hosted-CI fallback, mark PR #3
ready, merge, tag, publish, mutate release state, launch native Claude, run `/hooks`, signal
processes, mutate M6, or perform lifecycle cleanup.

## Research Question

Can the beta release-gate report produce a single machine-readable artifact that includes exact
hosted-CI pre-step blocker proof without weakening fail-closed checks or changing text output?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | In `--json` mode, the beta report can call the hosted verifier with `--json`, validate `condition_verified=true`, and embed the object under `hosted_ci.verifier`. | Supported. |
| Null | Keeping hosted verifier JSON only as a separate command is sufficient. | Rejected for release-owner automation because it forces callers to correlate two JSON artifacts manually. |
| Simpler alternative | Copy only the hosted run ID and state into the beta report. | Rejected because the release owner also needs head, workflow, job, step-count, and non-action evidence. |
| Failure | Embedding verifier JSON makes a failed or wrong-head hosted run look accepted. | Avoided. The parent script still fails if the verifier fails, and the embedded object preserves `hosted_ci_fallback_accepted=false` and `release_actions_performed=false`. |

## Structured Output Contract

When `./scripts/beta-release-gate-report.sh --json` verifies hosted CI through the pre-step blocker
path, the final JSON object includes:

- `hosted_ci.state: "pre_step_blocker_verified"`,
- `hosted_ci.run_id`,
- `hosted_ci.verifier.condition_verified: true`,
- `hosted_ci.verifier.run` with expected/current head, workflow, event, URL, status, and
  conclusion,
- `hosted_ci.verifier.jobs` with per-job status, conclusion, and `step_count`,
- `hosted_ci.verifier.hosted_ci_fallback_accepted: false`,
- `hosted_ci.verifier.release_actions_performed: false`.

If hosted CI is green, `hosted_ci.verifier` remains `null`.

## Validation

Validation performed for this slice:

- `bash -n scripts/beta-release-gate-report.sh`
- `git diff --check`
- `scripts/beta-release-gate-report.sh --hosted-run 27191318771 --quick --allow-tracked-changes
  --json`
- JSON field assertion with `jq -e`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

The focused assertion verified that the beta report:

```text
hosted_ci.state == "pre_step_blocker_verified"
hosted_ci.run_id == "27191318771"
hosted_ci.verifier.condition_verified == true
hosted_ci.verifier.run.id == 27191318771
hosted_ci.verifier.run.head == ab2a1b439160b70050e6850f33d5f91604bc43cb
all hosted_ci.verifier.jobs have conclusion failure and step_count 0
hosted_ci.verifier.hosted_ci_fallback_accepted == false
hosted_ci.verifier.release_actions_performed == false
release_owner_decision_required == true
release_actions_performed == false
```

## Gate Impact

T404 gives release-owner tooling one parseable artifact for branch/PR/local/package state plus exact
hosted pre-step blocker proof. It does not accept the waiver, close hosted CI, mark PR #3 ready,
merge, tag, publish, close native Claude prompt-bearing proof, close effective-hook visibility,
close live host labels, complete multi-host parity, mutate M6, run lifecycle cleanup, or make
Engram production/GA ready.
