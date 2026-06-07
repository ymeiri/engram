# Brain Harness T63 Scoped Feedback Drill-Down Fix

Status: Completed
Date: 2026-05-31
Scope: Telemetry evidence-loop drill-down correctness

This slice fixes one internal telemetry report inconsistency. It does not change public MCP
parameters, schema/storage/index state, search ranking, `orient`, migration, lifecycle state, or
harness adapters/hooks.

## Research Question

Does scoped `telemetry(action="list_feedback", project/scenario/arm, limit=N)` apply the scope
before the limit, so drill-down feedback agrees with scoped `real_session_eval` when newer
out-of-scope traces exist?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | `list_feedback_scoped` should find matching traces first, fetch feedback for those trace IDs, then truncate the feedback rows to the requested/default list size. |
| Null | Existing tests already cover the behavior and no code change is needed. |
| Simpler alternative | Document that scoped feedback drill-down is approximate and rely on `real_session_eval` only. |
| Failure | The fix makes scoped feedback unbounded, changes public API behavior, or breaks existing telemetry filtering. |

## Source Finding

`real_session_eval_report_scoped` already applies project/scenario/arm filters before sampling
traces, then fetches feedback for that scoped trace set.

Before T63, `list_feedback_scoped` applied `limit` before scope on both sides:

- `repo.list_traces(limit)` then in-memory trace filtering;
- `repo.list_feedback(limit)` then filtering by the reduced trace ID set.

That meant `list_feedback(limit=1, project="engram", scenario_id=..., arm=...)` could return no
feedback if the newest trace or newest feedback row belonged to another project, even when an
in-scope trace and feedback existed.

## Change

`TelemetryService::list_feedback_scoped` now:

1. calls `repo.list_traces_scoped(Some(max(limit, 100)), project, scenario_id, arm)`;
2. fetches feedback with `repo.list_feedback_for_traces(&trace_ids)`;
3. truncates the newest-first feedback rows to `limit.unwrap_or(100)`.

This keeps the drill-down endpoint bounded while matching the scoped-eval filtering model.

## Consultation

AI Council recall found prior telemetry guidance that `external_session_id` and eval metadata are
secondary joinability signals, not confidence proof. Claude Bridge reviewed the proposed slice and
agreed the semantics should be "newest feedback whose parent trace matches the scope"; it suggested
the exact regression shape used here.

## Validation

Source validation:

```text
cargo test -p engram-tests --test telemetry_tests mcp_list_feedback_applies_scope_before_limit
cargo test -p engram-tests --test telemetry_tests
cargo fmt --all --check
cargo check -p engram-cli
```

All passed.

Installed runtime validation:

- installed binary hash:
  `fd7287ef6186d77532c20486034f95729b89e00c043e6ef94aa870bc873846da`;
- daemon restarted on port `8765`, PID `92869`;
- live smoke scenario `t63_scoped_feedback_drilldown_scan_20260531` wrote two in-scope feedback
  rows and one newer out-of-scope feedback row;
- scoped `list_feedback(project="engram", scenario_id=..., arm="memory_items", limit=1)` returned
  target-new in-scope feedback `019e7f63-654d-72b1-bf10-44ce210fa206` for trace
  `019e7f63-59fe-7d62-b3ce-127867ac5c18`, not the newer out-of-scope feedback.

## Verdict

T63 closes a concrete telemetry drill-down correctness gap. It strengthens evidence inspection for
Brain Harness evals, but it is not product-completion evidence by itself and does not authorize M6,
lifecycle cleanup, ranking changes, `orient` changes, schema/storage/index changes, public MCP
changes, or harness writes.
