# Brain Harness T54 Rolling Telemetry Audit

Status: Completed read-only evidence-quality audit.
Date: 2026-05-31
Scope: Post-T53 rolling telemetry and feedback-evidence calibration

This audit did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

After T53 feedback scoring, does the rolling `real_session_eval(project=engram, limit=50)` report
still support the Evidence and feedback loop row without overstating confidence or hiding known
weaknesses?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The rolling eval remains numerically healthy after T53 feedback and shows no bad-memory-used evidence, but remains weak agent-assessed evidence and does not approve M6, lifecycle, harness, ranking, schema/storage/index, public MCP, or `orient` work. |
| Null | The current window fails the confidence gate or shows task failures/bad memory that reopen evidence-loop risk. |
| Simpler alternative | Rely on older T35/T38 evidence and skip another rolling audit. |
| Failure | The audit treats a numerical gate pass as product completion or migration/lifecycle approval. |

## Measurement

Before editing docs, Codex ran:

- `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work",
  response_shape="lean")`, trace `019e7d65-dc7f-7a81-8014-41581b79bd56`.
- Direct Engram searches for current architecture/gates, user software design philosophy, and
  recent failures/open risks:
  - `019e7d66-39a9-7193-b09d-0983ce38fb20`
  - `019e7d66-4476-79a3-bbad-6c425396deb7`
  - `019e7d66-4f4c-7321-ad17-df460c1330c6`
- `telemetry(action="real_session_eval", project="engram", limit=50)`.
- `telemetry(action="list_feedback", project="engram", limit=10)`.

The rolling eval generated at `2026-05-31T09:38:18Z` returned:

| Metric | Value |
| --- | ---: |
| `trace_count` | `50` |
| `feedback_trace_count` | `31` |
| `feedback_coverage` | `0.6200000047683716` |
| `memory_judgment_coverage` | `1.0` |
| `distinct_intent_count` | `4` |
| `task_success_count` | `30` |
| `task_failure_count` | `1` |
| `bad_memory_used_count` | `0` |
| `stale_memory_count` | `25` |
| `wrong_scope_memory_count` | `0` |
| `missing_context_count` | `6` |
| `repeated_context_question_count` | `0` |
| `external_session_trace_count` | `13` |
| `unspecified_external_session_trace_count` | `37` |
| `confidence_gate.passed` | `true` |

Recent feedback lookup returned T53 records first. The latest feedback row
`019e7d64-6e74-72f3-b833-9184f3e1e95d` scored the final T53 orient trace as successful while
marking the stale repository-scoped current-plan target
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` stale.

## Verdict

Pass as a rolling evidence-quality audit, not as a completion gate.

The current window still meets the telemetry confidence gate and reports no bad-memory-used events,
no wrong-scope memory use, and no repeated context questions. That supports keeping the Evidence
and feedback loop row as partially validated.

The same report also shows why the row cannot be promoted to complete:

- feedback coverage is only `31/50`;
- one task failure remains in the sampled window;
- stale-memory feedback is still active (`stale_memory_count=25`);
- external session joinability is still partial (`13/50` traces labelled);
- the report itself says confidence gates require user approval for gated decisions.

## Next Action

The approval gates remain unchanged. The next non-gated work can continue to improve targeted
validation, evidence quality, cross-harness replication, or documentation synchronization. M6
read-only inventory/review-export still requires explicit user-approved scope, and M6 write apply,
deletion, lifecycle mutation, harness writes, ranking changes, schema/storage/index changes, public
MCP changes, and `orient` expansion remain separately gated.
