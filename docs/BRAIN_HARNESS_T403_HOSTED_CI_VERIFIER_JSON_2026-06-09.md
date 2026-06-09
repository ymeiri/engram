# Brain Harness T403 Hosted CI Verifier JSON

Date: 2026-06-09
Status: completed structured-output hardening for hosted-CI pre-step blocker verification.

## Scope

T403 adds `--json` to `scripts/verify-hosted-ci-prestep-blocker.sh`. The script remains read-only
and evidence-only: it does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag,
publish, mutate release state, launch native Claude, run `/hooks`, signal processes, mutate M6, or
perform lifecycle cleanup.

## Research Question

Can the hosted-CI pre-step blocker verifier emit machine-readable success evidence without
weakening its existing fail-closed checks or changing the default text report?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | `--json` can expose the verified run, expected/current head, workflow, event, expected jobs, per-job status/conclusion/step counts, and non-action flags after all existing checks pass. | Supported. |
| Null | Text output is enough because the beta gate report can wrap the verifier. | Rejected. Direct verifier JSON makes the waiver condition independently automatable. |
| Simpler alternative | Only parse the beta gate report JSON. | Rejected because the lower-level hosted-CI proof remains a separate release-owner evidence artifact. |
| Failure | JSON mode emits success evidence for a wrong head, wrong job set, real workflow-step failure, or release acceptance. | Avoided. Wrong-head JSON validation failed closed and emitted no success object. |

## Structured Output Contract

`scripts/verify-hosted-ci-prestep-blocker.sh --json <run-id>` emits one JSON object on success:

- `condition_verified: true`,
- run ID, URL, expected head, actual head, status, conclusion, expected workflow, actual workflow,
  and event,
- expected job list,
- per-job name, status, conclusion, and `step_count`,
- the verified condition string,
- `hosted_ci_fallback_accepted: false`,
- `release_actions_performed: false`.

Failures remain stderr-only and non-zero. A failed check does not emit a success JSON object.

## Validation

Validation performed for this slice:

- `bash -n scripts/verify-hosted-ci-prestep-blocker.sh`
- `scripts/verify-hosted-ci-prestep-blocker.sh 27190538964`
- `scripts/verify-hosted-ci-prestep-blocker.sh --json 27190538964`
- JSON field assertion with `jq -e`
- `EXPECTED_HEAD_SHA=0000000000000000000000000000000000000000 scripts/verify-hosted-ci-prestep-blocker.sh --json 27190538964` failed closed with a run-head mismatch and emitted no success JSON object.

The JSON assertion verified:

```text
condition_verified == true
run.id == 27190538964
run.head == 0de4f2745ba627266200b8f6e03d1b06edb2dc82
run.workflow == CI
run.event == pull_request
jobs length == 5
all jobs completed with conclusion failure and step_count 0
hosted_ci_fallback_accepted == false
release_actions_performed == false
```

## Gate Impact

T403 improves release-owner evidence automation for the hosted-CI waiver condition. It does not
accept the waiver, close hosted CI, mark PR #3 ready, merge, tag, publish, close native Claude
prompt-bearing proof, close effective-hook visibility, close live host labels, complete multi-host
parity, mutate M6, run lifecycle cleanup, or make Engram production/GA ready.
