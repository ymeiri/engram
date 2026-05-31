# Brain Harness T46 Harness Readiness Re-Audit

Date: 2026-05-31
Status: Pre-registered
Scope: Read-only harness readiness evidence refresh

## Boundary

T46 may run read-only `harness(action="doctor")` and, if useful, read-only
`harness(action="status")` checks. It must not run `harness(action="install")` with `write=true`,
modify settings, install adapters, re-enable hooks, mutate memory lifecycle state, run M6 inventory
or review export, change schemas/storage/indexes, change public MCP parameters, change ranking, or
expand `orient`.

## Research Question

Do the supported harnesses still report `ready=false` after the T43/T44/T45 work, and if so, what
exact read-only drift remains before harness readiness can be considered complete?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The read-only doctor still reports at least one supported harness as not ready, so harness readiness remains an explicit approval-gated follow-up. |
| Null | All supported harnesses now report ready without any adapter or hook writes. |
| Simpler alternative | Defer harness readiness re-audit and rely on the previous T29/T39 evidence. |
| Failure | The check requires writes, settings mutation, hook installation, or broad configuration changes to answer the question. |

## Measurement

Run read-only `harness(action="doctor", project="engram")` and record:

- per-harness `ready` status;
- missing required files, drifted generated files, missing settings, or user-owned file caveats;
- whether any result changes the completion matrix;
- whether adapter or hook writes remain gated.

## Allowed Conclusion

If T46 passes, it supports only a current readiness statement for the read-only doctor surface. It
does not authorize adapter writes, hook writes, settings changes, M6 work, lifecycle mutation, or
hot-path changes.
