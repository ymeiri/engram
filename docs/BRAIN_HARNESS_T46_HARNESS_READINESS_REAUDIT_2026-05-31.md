# Brain Harness T46 Harness Readiness Re-Audit

Date: 2026-05-31
Status: Completed
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

## Result

T46 ran only read-only harness doctor/status checks. No adapter install, settings mutation, hook
registration, lifecycle mutation, migration action, ranking change, public MCP change, schema
change, or `orient` payload change was performed.

The default `harness(action="doctor", project="engram")` checks the generic harness policy rather
than aggregating all supported harnesses. It returned `ready=false` because the required generic
policy document is missing at `/Users/yuval.meiri/.engram/harness-policy.md`.

| Harness | Ready | Read-only finding |
| --- | --- | --- |
| `generic` | false | Required generic harness policy document is missing. |
| `claude_code` | false | Required generated adapter files are installed, but the optional settings snippet is user-owned and Claude settings still lack required `SessionStart:startup|resume|compact` and `SessionEnd` registrations. Settings also contain extra legacy Engram permission entries outside the current contract. |
| `codex` | false | Required `codex-memory-session-skill` and `codex-resume-session-skill` are drifted; `project-agents-snippet` is installed. |
| `gemini_cli` | false | Required `gemini-memory-session-command`, `gemini-resume-session-command`, and `gemini-global-context` are drifted; `gemini-end-session-command` is installed. |
| `cursor` | false | Required `cursor-memory-session-skill` and `cursor-resume-session-skill` are drifted; `cursor-end-session-skill` is installed. |

## Allowed Conclusion

If T46 passes, it supports only a current readiness statement for the read-only doctor surface. It
does not authorize adapter writes, hook writes, settings changes, M6 work, lifecycle mutation, or
hot-path changes.

## Conclusion

The preferred hypothesis holds: supported harness readiness still reports `ready=false`. The result
updates the completion matrix as a current read-only evidence refresh only. Harness adapter and hook
repair remains an explicit approval gate, separate from the M6 migration gate.
