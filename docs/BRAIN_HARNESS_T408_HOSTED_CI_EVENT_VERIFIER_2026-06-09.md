# Brain Harness T408 Hosted CI Event Verifier

Date: 2026-06-09
Status: completed hosted-CI evidence hardening.

## Scope

T408 updates `scripts/verify-hosted-ci-prestep-blocker.sh` so the same fail-closed verifier can
check both pull-request CI runs and post-merge main push CI runs. The default remains
`pull_request`; callers can pass `--event <event>` or set `EXPECTED_EVENT`.

This slice does not accept a hosted-CI fallback, mark a PR ready, merge, tag, publish, mutate
harness state, launch native Claude, run `/hooks`, signal processes, mutate M6, or run lifecycle
cleanup.

## Research Question

Can the hosted pre-step blocker verifier cover post-merge `push` runs without weakening the
existing pull-request verifier contract?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Add an explicit expected-event parameter that defaults to `pull_request` and is included in JSON evidence. | Supported. |
| Null | Manual `gh run view` inspection is enough for main-push evidence. | Rejected because production/release evidence should be repeatable and script-verifiable. |
| Simpler alternative | Stop checking the event. | Rejected because wrong-event evidence can accidentally prove the wrong gate. |
| Failure | The verifier starts accepting any failed run. | Avoided. It still checks exact head, workflow, event, expected jobs, completed failure status, and zero workflow steps. |

## Validation

Validation performed for this slice:

- `bash -n scripts/verify-hosted-ci-prestep-blocker.sh`
- default pull-request verification:
  `EXPECTED_HEAD_SHA=6e1d1789cc0584ee023c726986b38a8fb4194e5b
  ./scripts/verify-hosted-ci-prestep-blocker.sh --json 27196834196`
- default event mismatch remains fail-closed for the main push run:
  `EXPECTED_HEAD_SHA=21cb1f4948557510f448fffcd98f3ac775bb161d
  ./scripts/verify-hosted-ci-prestep-blocker.sh --json 27197574826`
- explicit push verification:
  `EXPECTED_HEAD_SHA=21cb1f4948557510f448fffcd98f3ac775bb161d
  ./scripts/verify-hosted-ci-prestep-blocker.sh --event push --json 27197574826`
- `git diff --check`

## Gate Impact

T408 makes post-merge hosted-CI blocker evidence as repeatable as PR hosted-CI blocker evidence.
It does not make hosted CI pass. Native Claude prompt-bearing proof, effective-hook visibility,
live host-label proof, and broader production/GA hardening remain open.
