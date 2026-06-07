# Brain Harness T44 Claude T43 Parity

Date: 2026-05-31
Status: Completed
Scope: Read-only Claude Code parity validation for the repaired T43 direct-search prompt class

## Boundary

T44 validates installed-runtime behavior from Claude Code after the T43 direct-search repair. It
must not edit code, run migration inventory or review export, mutate memory lifecycle state, archive
or scope-rewrite memory, change schemas or storage, change public MCP request parameters, expand
`orient`, install or modify harness adapters or hooks, or change ranking.

If Claude Code parity fails, T44 should record the failure and stop before implementation unless the
evidence identifies a narrow, local, non-gated follow-up slice.

## Research Question

Does Claude Code, using the same installed Engram daemon and direct MCP `search`, reproduce the T43
Codex behavior for the exact mixed query
`current plan next non-gated Brain Harness feedback confidence M6 gate`?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code direct `search` returns the latest current-plan memory first and active M6 gate context in top-k for the exact mixed query, while preserving explicit M6 gate-first behavior and pure continuation control behavior. |
| Null | T43 is Codex-only evidence; Claude Code search returns a materially different order or omits the active M6 gate in the mixed query. |
| Simpler alternative | Defer Claude parity and rely on Codex installed-runtime validation plus deterministic fixtures. |
| Failure | Claude parity requires harness adapter changes, hook writes, public MCP changes, `orient` payload changes, migration/lifecycle work, or broad ranking churn. |

## Measurement

Run Claude Code in read-only mode with only Engram MCP search/telemetry access where possible:

- exact mixed query: latest current-plan memory `019e7d20-d54d-7d61-99ac-f6ed805848c9` at rank 1
  and active M6 gate memory `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` in the top five;
- explicit M6 negative control `approved M6 write apply deletion cleanup legacy simplification now`:
  gate/blocked context above current-plan guidance and no implication of approval;
- pure continuation control `current plan next non-gated Brain Harness feedback confidence`:
  latest current-plan memory at rank 1 and no search-only promotion of the active M6 gate into the
  asserted top-k;
- submit telemetry feedback for Claude traces after inspecting the returned IDs.

## Allowed Conclusion

If T44 passes, it supports only this claim: Claude Code reproduces the installed T43 direct-search
behavior for this exact prompt class and controls. It does not prove broad ranking quality, M6
approval, lifecycle cleanup safety, hook/adapter readiness, or `orient` payload readiness.

## Result

Claude Code reproduced the installed T43 direct-search behavior for all three read-only checks.
The run used the same installed daemon and direct Engram MCP `search` with
`scenario_id=t44_t43_claude_parity_20260531`.

| Check | Trace | Result |
| --- | --- | --- |
| Mixed query `current plan next non-gated Brain Harness feedback confidence M6 gate` | `019e7d21-cec2-7c60-b570-40bb6b79574e` | Passed: current-plan memory `019e7d20-d54d-7d61-99ac-f6ed805848c9` ranked 1, active M6 gate memory `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` ranked 2. |
| Explicit M6 negative control `approved M6 write apply deletion cleanup legacy simplification now` | `019e7d21-d4c6-7eb0-80a7-244042f513b0` | Passed: paused migration gate `019dd35d-1a48-7103-b0e2-390225f8b418` ranked 1, active M6 gate `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` ranked 2, current-plan guidance ranked 3, and no approval was implied. |
| Pure continuation control `current plan next non-gated Brain Harness feedback confidence` | `019e7d21-da4e-7e72-9e40-35153ba73628` | Passed: current-plan memory `019e7d20-d54d-7d61-99ac-f6ed805848c9` ranked 1 and active M6 gate memory was absent from the top eight. |

Telemetry feedback was submitted and verified for all three Claude traces:

- `019e7d21-fd7f-76e2-a05d-ce12143ab48e` for the mixed query;
- `019e7d22-0ea3-7a60-9425-0984d53bf254` for the explicit M6 negative control;
- `019e7d22-19ec-7e11-94a7-3dcebe368d63` for the pure continuation control.

This closes the immediate cross-harness parity question for the T43 repaired direct-search prompt
class. It does not authorize M6 migration work, lifecycle writes, hook/adapter writes, public MCP
changes, `orient` payload expansion, or broad ranking changes.
