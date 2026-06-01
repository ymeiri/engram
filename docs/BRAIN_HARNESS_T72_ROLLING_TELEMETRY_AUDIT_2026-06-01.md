# Brain Harness T72 Rolling Telemetry Audit

Status: Completed read-only telemetry audit; numerical pass remains partial evidence
Date: 2026-06-01
Scope: Rolling `real_session_eval` after T71 feedback/current-plan capture

This audit did not run M6 inventory, review export, review apply, candidate decisions, deletion,
lifecycle mutation, harness writes, schema/storage/index changes, public MCP changes, ranking
changes, document indexing, or `orient` payload changes.

## Research Question

After T71 feedback scoring and current-plan capture, does the rolling
`real_session_eval(project=engram, limit=50)` report still support the Evidence and feedback loop
row without overstating completion or crossing any approval gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The rolling report still passes numerically with no task failures and no bad-memory-used evidence, but remains partial because coverage, stale/wrong-scope judgments, and external-session joinability are still weak. |
| Null | The current window regresses enough to reopen evidence-loop risk, such as a failed confidence gate, task failures, or bad-memory-used feedback. |
| Simpler alternative | Rely on T56/T63/T71 and do not record another rolling audit. |
| Failure | The numerical pass is treated as product completion or as approval for M6, lifecycle, harness, ranking, schema/storage/index, public MCP, document-index, or `orient` work. |

## Measurement

Before editing docs, Codex used only read-only evidence:

- startup `orient` and direct Engram searches for current plan, architecture, user design
  philosophy, and recent risks;
- governing docs and latest T67/T68/T69/T70/T71 reports;
- `git status --short` and `git log --oneline -12`;
- `telemetry(action="real_session_eval", project="engram", limit=50)`;
- `telemetry(action="list_feedback", project="engram", limit=12)`;
- `telemetry(action="stats_by_intent", project="engram")`.

The rolling eval generated at `2026-06-01T09:03:03.924115Z` returned:

| Metric | Value |
| --- | ---: |
| `trace_count` | `50` |
| `feedback_trace_count` | `32` |
| `feedback_coverage` | `0.6399999856948853` |
| `memory_judgment_coverage` | `1.0` |
| `distinct_intent_count` | `3` |
| `distinct_operation_count` | `2` |
| `task_success_count` | `32` |
| `task_failure_count` | `0` |
| `bad_memory_used_count` | `0` |
| `stale_memory_count` | `28` |
| `wrong_scope_memory_count` | `1` |
| `missing_context_count` | `0` |
| `repeated_context_question_count` | `0` |
| `external_session_trace_count` | `11` |
| `unspecified_external_session_trace_count` | `39` |
| `confidence_gate.passed` | `true` |

Intent distribution in the sampled window was narrow:

| Intent | Traces | Feedback Traces | Coverage | Task Failures | Bad Memory Used |
| --- | ---: | ---: | ---: | ---: | ---: |
| `plan_work` | `47` | `29` | `0.6170212626457214` | `0` | `0` |
| `verify_decision` | `2` | `2` | `1.0` | `0` | `0` |
| `follow_user_preference` | `1` | `1` | `1.0` | `0` | `0` |

Recent feedback lookup returned the latest T71 records first. The newest feedback row
`019e8269-4b70-72c1-89bd-bf976f86cbfb` scored post-T71 orient trace
`019e8269-35cc-71c0-b5d5-d0c744318466` as successful while marking stale repository-scoped
current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` stale. The same recent feedback batch
shows the current thread's `external_session_id` is being supplied for the newest traces, but the
rolling sample still contains many older unlabeled traces.

## Interpretation

T72 improves the evidence-loop picture relative to T54/T56 in one important way: the rolling sample
now has zero task failures and zero bad-memory-used records. Feedback coverage remains above the
conservative threshold, and all feedback records with memory judgments are judged.

The row still cannot be promoted to complete:

- feedback coverage is `32/50`, not full coverage;
- the sampled window is mostly `plan_work` traces, with only three distinct intents;
- stale-memory feedback remains active (`stale_memory_count=28`);
- one wrong-scope judgment appears in the sample;
- external-session joinability is weaker than T56 in this rolling window (`11/50` traces labeled);
- the confidence gate explicitly still requires user approval for gated decisions.

## Completion Matrix Delta

The Evidence and feedback loop row stays partially validated. T72 strengthens it by replacing the
older rolling task-failure residue with a zero-failure sampled window, while preserving the same
weak-evidence caveat: agent feedback is useful operational telemetry, not proof of product
completion, M6 readiness, lifecycle safety, or harness readiness.

No approval gates changed. T69, T70, T52, T47, M6 apply/deletion, schema/storage/index changes,
ranking changes, public MCP changes, document-index writes, harness writes, and `orient` expansion
remain separately gated.

## Next Action

Continue with non-gated validation, evidence-quality work, cross-harness replication, or another
concrete capture/lifecycle gap surfaced by evidence. The next executable M6 step still requires
the exact T69 count-drift inspection approval, and T70 document indexing still requires its exact
approval phrase.
