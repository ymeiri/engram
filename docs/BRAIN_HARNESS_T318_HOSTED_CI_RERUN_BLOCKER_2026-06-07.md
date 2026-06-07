# T318 Hosted CI Rerun Blocker

Date: 2026-06-07
Status: completed hosted-CI gate recheck

## Question

After T317 proved the PR #3 head locally, can hosted GitHub Actions now provide the normal
exact-head CI release proof?

## Hypotheses

| Hypothesis | Result |
| --- | --- |
| Preferred | If the account-level Actions gate has cleared, rerunning the failed jobs will start normal workflow steps and produce fresh hosted CI evidence. | Rejected. |
| Null | Hosted CI remains blocked before runner assignment by the same external account billing or spending-limit issue. | Supported. |
| Failure | The rerun is misread as a source regression, or the rerun result is used to mark the PR ready without approval. | Avoided. |

## Evidence

PR #3 remained at head:

```text
78f14d0bebd980070a4fcb8d1f259be47517c704
```

Codex reran the failed jobs:

```bash
gh run rerun 27091138284 --failed
```

GitHub accepted the rerun and created attempt 2 for run `27091138284`. The attempt still failed
before runner assignment:

- `Format`: job `79955781919`, zero steps, `runner_id=0`, empty runner fields;
- `Clippy`: job `79955781927`, zero steps;
- `Docs`: job `79955781928`, zero steps;
- `Check`: job `79955781970`, zero steps;
- `Test`: job `79955781931`, zero steps, completed in four seconds.

The check-run annotations for sampled jobs all used the same failure message:

```text
The job was not started because recent account payments have failed or your spending limit needs to be increased. Please check the 'Billing & plans' section in your settings
```

## Interpretation

The hosted CI failure is still external/account-level. It is not evidence that the Rust code,
workflow steps, or documentation failed.

Normal exact-head hosted-CI release proof remains unavailable until the GitHub Actions billing or
spending-limit issue is fixed and the jobs can run. T317 local validation remains the strongest
available validation fallback unless the release owner explicitly requires hosted CI.

## Boundary

T318 does not mark PR #3 ready, merge, tag, publish, release, change beta scope, execute T314,
write harness adapters or settings, launch native Claude, run `/hooks`, send process signals,
change CI workflow behavior, or treat the local fallback as accepted release approval.

