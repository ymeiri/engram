# Brain Harness T402 Beta Release Gate JSON

Date: 2026-06-09
Status: completed structured-output hardening for the beta release gate report.

## Scope

T402 adds `--json` to `scripts/beta-release-gate-report.sh`. The script remains evidence-only:
it does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, mutate harness
state, change release scope, launch native Claude, run `/hooks`, signal processes, mutate M6, or
perform lifecycle cleanup.

## Research Question

Can the beta release gate report provide machine-readable evidence for release-owner review without
weakening the existing text report, branch/PR/head checks, hosted-CI fallback verification, local CI
execution, or package/install smoke execution?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | `--json` can emit a stable final evidence object while routing step logs to stderr so stdout remains parseable. | Supported. |
| Null | Text-only release evidence is sufficient for release-owner review and automation. | Rejected. T401 showed structured output reduces stale hand-copied evidence risk. |
| Simpler alternative | Keep the report text-only and rely on PR body updates. | Rejected because PR prose is not a machine-readable release gate. |
| Failure | JSON mode hides blockers, skips validation unintentionally, or changes release semantics. | Avoided. JSON mode still runs the same checks unless skip flags are passed, and release actions remain false. |

## Structured Output Contract

`scripts/beta-release-gate-report.sh --json` emits one final JSON object on stdout with:

- branch/upstream/head state,
- tracked source state,
- PR number, URL, draft flag, merge state, and CI check summaries,
- hosted CI state and optional run ID,
- local CI and package/install smoke status,
- `release_owner_decision_required: true`,
- `release_actions_performed: false`.

Step logs and validation command output are written to stderr in JSON mode. This preserves parseable
stdout for automation while keeping the same human-readable progress stream visible in terminals.

## Validation

Validation performed for this slice:

- `bash -n scripts/beta-release-gate-report.sh`
- `scripts/beta-release-gate-report.sh --hosted-run 27189649472 --quick --allow-tracked-changes --json`
- JSON field assertion with `jq -e`
- `scripts/beta-release-gate-report.sh --hosted-run 27189649472 --quick --allow-tracked-changes`

The JSON assertion verified:

```text
head == 20c61727e65378945b4c2fac108a8db54c145c12
tracked_changes_present == true
hosted_ci.state == pre_step_blocker_verified
local_ci == skipped
package_install_smoke == skipped
release_owner_decision_required == true
release_actions_performed == false
pr.checks length == 5
```

Final committed-head validation should omit `--allow-tracked-changes` and can run the full
non-quick gate report.

## Gate Impact

T402 improves release gate evidence quality and automation readiness. It does not accept the
release-owner fallback, close hosted CI, mark PR #3 ready, merge, tag, publish, close native Claude
prompt-bearing proof, close effective-hook visibility, close live host labels, complete multi-host
parity, mutate M6, run lifecycle cleanup, or make Engram production/GA ready.
