# Brain Harness T370 Telemetry Confidence Refresh

Date: 2026-06-08
Status: completed point-in-time telemetry feedback refresh for the scoped Brain Harness evidence
trail.

## Scope

T370 refreshes real-session telemetry confidence after T369 exact-head beta validation. The slice
records feedback for recent judgeable `orient` and `search` traces where this session had concrete
evidence of which returned MemoryItems shaped the answer, plan, or validation path.

This slice does not change source behavior, alter telemetry formulas, change ranking or `orient`,
run M6 write-apply, run lifecycle cleanup, mark PR #3 ready, merge, tag, publish, close hosted CI,
run native Claude, execute `/hooks`, or change the supported beta scope.

## Research Question

Does the recent real-session telemetry window still fail the confidence gate because too few
retrieval traces have explicit feedback?

## Baseline

Before the T370 catch-up, project-scoped telemetry showed:

```text
telemetry(real_session_eval, project=engram, limit=20):
trace_count = 20
feedback_count = 10
feedback_coverage = 0.50
confidence_gate.passed = true

telemetry(real_session_eval, project=engram, limit=50):
trace_count = 50
feedback_count = 13
feedback_trace_count = 13
feedback_coverage = 0.26
confidence_gate.passed = false
reason = Need feedback coverage of at least 50%; found 26%.
```

After six initial feedback submissions, the 50-trace report improved but still failed:

```text
telemetry(real_session_eval, project=engram, limit=50):
trace_count = 50
feedback_count = 19
feedback_trace_count = 19
feedback_coverage = 0.38
confidence_gate.passed = false
reason = Need feedback coverage of at least 50%; found 38%.
```

## Feedback Added

T370 added feedback only for traces that were judgeable from this session's evidence. The accepted
feedback records were:

```text
019ea770-e363-7433-bd19-bdc040b46f9b -> 019ea76f-79a2-71c3-9c00-afceca720ae2
019ea770-e37d-7350-bdc8-3d45e5bb259e -> 019ea74f-14ca-7450-a4eb-427723889a84
019ea770-e3e0-72e3-aefb-95eaec1fe949 -> 019ea74a-0826-79b0-8db1-9abe950b3b6b
019ea770-e3f1-7503-bb0f-32f7ab76ed0f -> 019ea74a-0618-7d10-b5f5-4f7ded2dc8b7
019ea770-e3fb-7160-a57b-ea624f03403d -> 019ea749-f4f4-7452-89ca-4591db1b8c5c
019ea770-e406-7a33-ac14-65b54f537221 -> 019ea747-90e1-7323-8cd9-20f18d6f61e6
019ea773-31d3-7441-9ee7-74cdca408005 -> 019ea772-5ea2-7863-95c3-039d5901ca23
019ea773-3238-7c40-be34-5cca2c22d3b3 -> 019ea74a-096c-7c72-83fb-cea903753dd5
019ea773-3246-7011-b8f0-415e4c996ec6 -> 019ea750-5d5f-74a1-9707-8f7d818ae8bf
019ea773-3251-7683-8db3-42918599be0c -> 019ea688-bf66-7310-aa64-57ce8f4d7af5
019ea773-325c-78f0-8850-15e805e81dae -> 019ea682-89f4-7880-a22f-7ed9a806d742
019ea773-3266-77b3-bea1-38eb5753cd20 -> 019ea680-c1be-7303-88b8-47bfe6771813
019ea773-3271-76d3-93a4-d9c5ac19b897 -> 019ea664-48cc-7193-ae1b-039037505ce4
```

One attempted feedback submission used a mistyped trace id and failed with `Trace not found`; it
wrote no feedback.

## Result

After T370 feedback, both sampled windows passed:

```text
telemetry(real_session_eval, project=engram, limit=20):
trace_count = 20
feedback_count = 18
feedback_trace_count = 18
feedback_coverage = 0.90
confidence_gate.passed = true
task_failure_count = 0
bad_memory_used_count = 0

telemetry(real_session_eval, project=engram, limit=50):
trace_count = 50
feedback_count = 26
feedback_trace_count = 26
feedback_coverage = 0.52
confidence_gate.passed = true
task_failure_count = 0
bad_memory_used_count = 0
```

## Gate Impact

T370 removes the immediate 50-trace feedback-coverage failure from the telemetry evidence trail.
The gate is still explicitly sampled, rolling, agent-assessed, and window-sensitive.

T370 does not authorize M6 write-apply. The telemetry report continues to state that confidence
gate passage requires user approval for any write path, and M6 write-apply remains blocked until a
separate exact approval names the intended operation and scope.

## Beta Readiness Note

The telemetry refresh supports the current release evidence quality, but it is not the main scoped
MVP beta blocker. The local/Codex MVP beta remains release-logistics-limited: release-owner
acceptance of the exact-head local CI plus package/install fallback, or restored exact-head hosted
CI green, followed by ready/merge/tag/publish mechanics.

Production/GA readiness remains lower because native Claude prompt-bearing proof, effective-hook
visibility, live host labels, full multi-host parity, lifecycle cleanup, M6 write-apply, and
operational hardening remain open or explicitly deferred.
