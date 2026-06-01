# Brain Harness T75 Post-T74 Telemetry Audit

Status: Completed read-only telemetry audit; confidence gate currently fails on intent coverage
Date: 2026-06-01
Scope: Rolling `real_session_eval` after T74 feedback/current-plan capture

This audit did not run M6 inventory, review export, review apply, candidate decisions, deletion,
lifecycle mutation, harness writes, schema/storage/index changes, public MCP changes, ranking
changes, document indexing, or `orient` payload changes.

## Research Question

After T74 feedback scoring and current-plan capture, does the rolling
`real_session_eval(project=engram, limit=50)` report still support the Evidence and feedback loop
row without overstating completion or crossing any approval gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The rolling report remains useful but partial: no task failures and no bad-memory-used evidence, with improved external-session labeling, while coverage or intent diversity still prevents treating the evidence loop as complete. |
| Null | The current window passes the confidence gate across intents and shows no stale/wrong-scope risk, making the evidence loop closer to complete. |
| Simpler alternative | Rely on T72/T74 and skip another rolling telemetry audit. |
| Failure | The report is treated as product completion or as approval for M6, lifecycle, harness, ranking, schema/storage/index, public MCP, document-index, or `orient` work. |

## Measurement

Before editing docs, Codex used only read-only evidence:

- startup `orient` and direct Engram searches for current plan, user design philosophy, and recent
  risks;
- governing docs and latest T73/T74 reports;
- `git status --short` and `git log --oneline -12`;
- `telemetry(action="real_session_eval", project="engram", limit=50)`;
- `telemetry(action="list_feedback", project="engram", limit=12)`.

The rolling eval generated at `2026-06-01T09:21:12.437561Z` returned:

| Metric | Value |
| --- | ---: |
| `trace_count` | `50` |
| `feedback_trace_count` | `27` |
| `feedback_coverage` | `0.5400000214576721` |
| `memory_judgment_coverage` | `1.0` |
| `memory_judgment_trace_coverage` | `0.5400000214576721` |
| `distinct_intent_count` | `3` |
| `distinct_operation_count` | `2` |
| `task_success_count` | `27` |
| `task_failure_count` | `0` |
| `bad_memory_used_count` | `0` |
| `stale_memory_count` | `23` |
| `wrong_scope_memory_count` | `0` |
| `missing_context_count` | `0` |
| `repeated_context_question_count` | `0` |
| `external_session_trace_count` | `36` |
| `unspecified_external_session_trace_count` | `14` |
| `confidence_gate.passed` | `false` |

Intent distribution in the sampled window:

| Intent | Traces | Feedback Traces | Coverage | Task Failures | Bad Memory Used |
| --- | ---: | ---: | ---: | ---: | ---: |
| `plan_work` | `44` | `27` | `0.6136363744735718` | `0` | `0` |
| `follow_user_preference` | `3` | `0` | `0.0` | `0` | `0` |
| `verify_decision` | `3` | `0` | `0.0` | `0` | `0` |

The confidence gate failed with reason:

- `Need feedback across at least 3 intents; found 1.`

Recent feedback lookup returned T74 and T73 records first. The newest rows scored Claude Bridge
traces `019e8278-6bd4-73f3-8973-8ea0d3ec24bc` and
`019e8278-671d-7d02-8a04-fe0a17d31de6` as successful while marking stale repository-scoped
current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` stale. Claude Bridge traces still lack
`external_session_id`, while Codex traces in this thread carry
`019e683b-1560-7361-b535-53b012e04aa5`.

## Interpretation

T75 is mixed evidence, and weaker than a completion signal:

- Positive: the sampled window has zero task failures, zero bad-memory-used records, zero
  wrong-scope judgments, and better external-session labeling than T72 (`36/50` versus `11/50`).
- Negative: feedback coverage fell from T72's `32/50` to `27/50`, and all feedback in the sampled
  window belongs to one intent. The confidence gate therefore fails.
- Persistent caveat: stale repository-scoped current-plan memory remains the main rejected/stale
  item in recent feedback. This supports T52's lifecycle decision boundary, not automatic cleanup.

The Evidence and feedback loop row remains partially validated. T75 strengthens the conclusion
that the loop is useful operational telemetry, but it also proves current evidence is not broad
enough across intents to claim completion.

## Completion Matrix Delta

The Evidence and feedback loop row stays partially validated and should not be promoted. Compared
with T72, T75 improves external-session labeling and keeps the zero-failure/zero-bad-memory-used
state, but it regresses the confidence gate because feedback is concentrated in `plan_work`.

No approval gates changed. T69, T70, T52, T47, M6 apply/deletion, schema/storage/index changes,
ranking changes, public MCP changes, document-index writes, harness writes, and `orient` expansion
remain separately gated.

## Next Action

Continue with non-gated validation, evidence-quality work, cross-harness replication, or another
concrete capture/lifecycle gap surfaced by evidence. The next executable M6 step still requires the
exact T69 count-drift inspection approval, and T52 lifecycle resolution still requires an explicit
option and exact write approval.
