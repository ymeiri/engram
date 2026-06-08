# Brain Harness T383 Telemetry Confidence Recovery

Date: 2026-06-08
Status: completed evidence-only telemetry feedback catch-up.

## Research Question

Can Engram restore the current project-scoped real-session telemetry confidence gate by scoring only
judgeable recent traces, without changing telemetry formulas, thresholds, source behavior, or the
beta/production boundary?

Preferred hypothesis: the gate can pass by scoring recent traces whose trace payloads and result
docs are directly assessable, improving sampled retrieval evidence while preserving all remaining
release and production gates.

Null hypothesis: the current window will still fail the 50% feedback-coverage threshold after
bounded feedback catch-up.

Failure hypothesis: catch-up scoring hides stale or wrong-scope memory, scores unjudgeable traces,
or is mistaken for M6 write approval, native-Claude proof, hosted-CI acceptance, or production/GA
readiness.

## Preflight

- `git status --branch --short` showed branch `yuval.meiri/memory-os-phase1` tracking
  `origin/yuval.meiri/memory-os-phase1` with only untracked user-owned `AGENTS.md`.
- `git rev-list --left-right --count HEAD...@{u}` returned `0 0`; the divergent-branch pull hint
  was not true for the current branch state.
- Canonical vault status was aligned at `total_file_count=2746`,
  `generated_file_count=2746`, `user_file_count=0`, and
  `expected_generated_file_count=2746`.
- Obligations doctor returned `open=[]` and `warnings=[]`.
- Before catch-up, `telemetry(action="real_session_eval", project="engram", limit=50)` failed the
  confidence gate only on feedback coverage: `21/50` traces had feedback (`42%`), with
  `task_failure_count=0`, `bad_memory_used_count=0`, `stale_memory_count=0`, and
  `wrong_scope_memory_count=0`.

## Feedback Catch-Up

Feedback was submitted only for trace IDs that were judgeable from trace payloads plus current docs:

- `019ea8a5-f60b-76c2-9e53-016980300077` - current planning `orient` trace for the
  continuation. It surfaced the T382 preflight, beta-scope consensus, hosted-CI limitation,
  research method, and commit preference, and led to this safe telemetry-confidence slice.
- `019ea884-8d3b-7f92-9433-2c3b35159208` - T382 planning `orient` trace. It surfaced PR-body
  freshness, native-Claude/production-gate limitations, beta-scope context, hosted-CI blocker, and
  commit preference; T382 stayed read-only and did not launch native Claude.
- `019ea886-d4a6-7890-ac40-b1f3f52baf2d` - `changes_since` freshness check. It returned no newer
  memory after the cursor, which was the expected result for the T382 closeout context.
- `019ea83c-150b-7011-998f-54f61ba618d4` - T380 post-archive verification search. It no longer
  returned the archived stale M6 checkpoint and surfaced current M6 limitation/disposition context.

## Postflight

After the four feedback records:

- `telemetry(action="real_session_eval", project="engram", limit=50)` passed:
  - `trace_count=50`
  - `feedback_count=25`
  - `feedback_coverage=0.5`
  - `distinct_intent_count=7`
  - `distinct_operation_count=3`
  - `task_failure_count=0`
  - `bad_memory_used_count=0`
  - `stale_memory_count=0`
  - `wrong_scope_memory_count=0`
  - `confidence_gate.passed=true`
- `telemetry(action="real_session_eval", project="engram", limit=20)` passed:
  - `trace_count=20`
  - `feedback_count=10`
  - `feedback_coverage=0.5`
  - `distinct_intent_count=4`
  - `distinct_operation_count=3`
  - `task_failure_count=0`
  - `bad_memory_used_count=0`
  - `stale_memory_count=0`
  - `wrong_scope_memory_count=0`
  - `confidence_gate.passed=true`
- Obligations doctor remained clean.

## Gate Impact

T383 restores the current sampled telemetry confidence signal for both the 50-trace and 20-trace
windows. It strengthens retrieval-feedback evidence for the local/Codex Brain Harness path.

It does not change telemetry formulas or thresholds, change source behavior, accept the hosted-CI
fallback, mark PR #3 ready, merge, tag, publish, run M6 write-apply, run lifecycle cleanup, launch
native Claude, run `/hooks`, prove prompt-bearing native-Claude behavior, prove live host labels,
or make Engram production/GA ready.
