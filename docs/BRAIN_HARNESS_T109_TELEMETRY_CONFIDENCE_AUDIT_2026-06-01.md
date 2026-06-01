# Brain Harness T109 Telemetry Confidence Audit

Status: Completed docs-only telemetry confidence audit; no calibration traces
Date: 2026-06-01
Scope: Current `real_session_eval(project=engram, limit=50)` confidence state after T108

This audit did not create calibration traces, run M6 inventory, run review export or apply,
inspect migration candidate files, mutate lifecycle state, run `lint(action="apply_safe")`, index
documents, change ranking, expand `orient`, change public MCP/schema/storage/index behavior,
change document-index behavior, or write harness adapters/hooks.

## Research Question

After T108, should Engram improve the telemetry evidence row by generating labeled calibration
traces, or should it preserve the current confidence-gate failure as read-only evidence until
real task-boundary traces provide stronger signal?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The safe T109 slice is a docs-only audit: record current report behavior, model disagreement, and the gate failure without generating calibration traces or implying product proof. |
| Null | The current report already provides enough evidence to treat the feedback loop as complete or to proceed toward M6. |
| Simpler alternative | Pause with no artifact after the model disagreement. |
| Failure | Synthetic or evaluator-shaped traces make the gate look healthier without proving real agent continuity, preference adherence, or bad-memory containment. |

## Measurement

Codex used only read-only evidence and ordinary current-session feedback scoring:

- lean `orient` trace `019e845d-a10b-74d0-8fb0-57c83dd9e29c`;
- direct search traces `019e845d-dcf1-7af1-a2f3-174fc94d15f3`,
  `019e845d-de9e-71c2-b472-856f449cd4eb`, `019e845d-e046-7051-80b6-4e9d4c364aa8`,
  `019e845d-e1ec-7c81-ac8e-341d91f4601a`, and
  `019e845d-e392-7592-a91a-044793fa8aca`;
- `telemetry(action="real_session_eval", project="engram", limit=50)` before and after ordinary
  feedback scoring for startup traces;
- `lint(action="run", limit=10)`;
- source reads in `engram-index/src/telemetry.rs` and `engram-core/src/telemetry.rs`;
- AI Council broadcast and Claude Bridge critique for the calibration-trace decision.

Source inspection confirms:

- unscoped `real_session_eval` reads a bounded trace sample, then fetches feedback only for those
  sampled trace IDs;
- scoped reports filter by project, `scenario_id`, and `arm` before applying the trace limit;
- `feedback_coverage` is trace coverage, while `feedback_records_per_trace` is feedback density;
- `intent` is caller-provided workflow metadata and is not a substitute for scenario, arm, or
  outcome evidence;
- the confidence gate requires trace count, feedback count, feedback coverage, feedback across at
  least three intents, at least one memory judgment, and at least one outcome-feedback record; it
  still requires explicit user approval for M6 write paths even when passing.

The report generated at `2026-06-01T18:03:17.359444Z`, before scoring the latest startup traces,
returned:

| Metric | Value |
| --- | ---: |
| `trace_count` | `50` |
| `feedback_trace_count` | `15` |
| `feedback_coverage` | `0.30000001192092896` |
| `memory_judgment_coverage` | `0.8823529481887817` |
| `memory_judgment_trace_coverage` | `0.3333333432674408` |
| `distinct_intent_count` | `4` |
| `external_session_trace_count` | `0` |
| `task_success_count` | `16` |
| `task_failure_count` | `1` |
| `bad_memory_used_count` | `0` |
| `confidence_gate.passed` | `false` |

The confidence gate failed because feedback coverage was `30%` and feedback appeared across only
two intents.

After scoring ordinary current-session startup retrievals, the report generated at
`2026-06-01T18:06:57.424972Z` returned:

| Metric | Value |
| --- | ---: |
| `trace_count` | `50` |
| `feedback_trace_count` | `18` |
| `feedback_coverage` | `0.36000001430511475` |
| `memory_judgment_coverage` | `0.8999999761581421` |
| `memory_judgment_trace_coverage` | `0.3913043439388275` |
| `distinct_intent_count` | `4` |
| `external_session_trace_count` | `0` |
| `task_success_count` | `19` |
| `task_failure_count` | `1` |
| `bad_memory_used_count` | `0` |
| `confidence_gate.passed` | `false` |

The later gate failed only on feedback coverage:

- `Need feedback coverage of at least 50%; found 36%.`

The same lint run reported stale current-plan target
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` first, with 239 recent stale-feedback records and
`safe_action=none`.

## Model Critique

AI Council and Claude Bridge agreed that generic `i approve` is not authorization for M6,
lifecycle, indexing, harness, ranking, `orient`, public MCP, schema, storage, or document-index
changes. They also agreed that passive telemetry coverage and caller-supplied intent labels are
weak evidence.

They disagreed on calibration traces:

- AI Council accepted a tiny labeled calibration set only if it was permanently separated from gate
  evidence and could block rather than unlock decisions.
- Claude Bridge argued that generating the calibration set would still risk teaching the system to
  satisfy the metric rather than measuring real behavior.

T109 takes the conservative resolution: do not generate calibration traces. The audit records the
real rolling-window state and keeps the confidence gate failed.

## Interpretation

T109 improves evidence hygiene, not product capability:

- The telemetry report is functioning as a useful operational signal: it exposes feedback coverage,
  outcome feedback, memory judgments, external-session label sparsity, intent concentration, and
  stale-memory pressure.
- The sampled window remains too weak for completion claims: `external_session_trace_count=0`,
  one task failure remains in the window, and feedback coverage is only `36%`.
- Ordinary startup feedback made the intent-coverage reason disappear, but that does not prove
  cross-intent behavioral quality. It only shows that real current-session traces can be scored.
- Stale repository-scoped current-plan guidance remains a lifecycle review pressure signal, not an
  automatic archive permission.

## Completion Matrix Delta

The Evidence and feedback loop row remains partially validated. The confidence gate still fails,
and the audit reinforces that passing the numerical gate would not by itself authorize migration,
lifecycle cleanup, harness writes, ranking changes, `orient` expansion, public MCP changes,
schema/storage/index changes, or document-index writes.

The Migration row remains gated at T69 count-drift inspection and later explicit apply/deletion
approval. The Memory quality / lifecycle row remains gated/default-deny for
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`. Cross-harness readiness remains risky until T47 or a
separate exact harness-write gate is approved.

## Next Action

Continue with non-gated validation or evidence-quality work, or wait for an exact approval gate:

- T69: `Approve T69: inspect index.md and 0012-skip-plan.md.`
- T70: `Approve T70: index exact files T59, T68, and T69.`
- T47 exact harness-write repair remains pending.
- T108 future lifecycle gate remains exact-target gated.

Do not generate calibration traces for the confidence gate unless Engram first has a structural
way to exclude them from decision-grade gate calculations and the user explicitly approves that
eval-design change.
