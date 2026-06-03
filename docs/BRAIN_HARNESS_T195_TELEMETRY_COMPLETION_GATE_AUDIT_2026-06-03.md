# T195 Telemetry Completion-Gate Audit

Date: 2026-06-03
Status: Telemetry feedback and docs-only completion-gate audit.

## Scope

This slice records the current Brain Harness real-session telemetry state after T194. It submitted
feedback only for current assessable retrieval traces, then reran the read-only
`real_session_eval` report.

It did not run T194/T192 document indexing, T193/T191/T187 lifecycle archive, `lint apply_safe`,
T186 process cleanup, native Claude input, Claude Bridge, M6/migration/quarantine actions, harness
install or edits, ranking/`orient`/source changes, public MCP/schema/storage/index/document-index
behavior changes, deletion, rollback, old-binary reinstall, or user-owned-file edits.

## Research Question

After T194, does the current real-session telemetry window meet the numerical confidence gate, and
if so, what still prevents treating Engram as complete enough for the full Brain OS definition of
done?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Current trace scoring makes the rolling telemetry confidence gate pass numerically, but the result remains weak completion evidence because exact approval gates, document visibility gaps, stale active handoff noise, missing external-session labels, and M6/native-Claude work remain unresolved. |
| Null | Passing the telemetry confidence gate is sufficient to claim Brain Harness completion or migration readiness. |
| Simpler alternative | Do not record the telemetry change; rely only on T194 current-plan memory. |
| Failure | Telemetry feedback is submitted for unassessed traces, a sliding-window pass is mistaken for approval to run gated work, or the audit hides remaining blockers behind a green confidence gate. |

## Evidence

Startup and retrieval:

- Lean `orient` trace `019e8e9b-3074-7231-a3e3-8565b6005052` returned current-plan memory
  `019e8e99-f041-7d81-9b29-10f888154711` first and no open obligations.
- Direct current-plan search trace `019e8e9b-539e-7f52-91b6-f981c6ec7b97` returned the active
  T194 current-plan memory first, then stale active handoff noise.
- Completion-blocker search trace `019e8e9b-546f-7872-9b89-0e4ed6b12552` returned the T194
  current-plan memory first and latest handoff second, then older active handoffs.
- Design-philosophy trace `019e8e9b-553c-7871-b03b-cb473c201dff` returned reviewed user preference
  `019e6924-256b-7093-b1c5-286ec4d02461`, but only after stale rolling handoffs. This is a
  retrieval-noise caveat for direct search, not a source-change approval.
- `lint(action="run", write=false, limit=20)` reported existing wrong-scope feedback findings and
  many superseded-active findings; `applied_safe_actions=0`.
- `obligations(action="doctor", project="engram")` returned `open=[]`, `warnings=[]`.
- `git status --short --branch` showed only pre-existing untracked root `AGENTS.md`.

Telemetry feedback submitted in this slice:

| Trace | Feedback ID | Used | Stale/Noise |
| --- | --- | --- | --- |
| `019e8e9b-3074-7231-a3e3-8565b6005052` | `019e8e9b-e428-7671-83ac-93c9a47e1b52` | T194 current-plan `019e8e99...` | none |
| `019e8e9b-539e-7f52-91b6-f981c6ec7b97` | `019e8e9b-e4ce-7381-824f-a0b39bc63a91` | T194 current-plan `019e8e99...` | stale handoff `019e839e...` |
| `019e8e9b-546f-7872-9b89-0e4ed6b12552` | `019e8e9b-e4d9-79d2-871d-45c48d0111f3` | T194 current-plan `019e8e99...`, latest handoff `019e8e9a...` | stale handoff `019e8e97...` |
| `019e8e9b-553c-7871-b03b-cb473c201dff` | `019e8e9b-e4e1-7a81-b2fb-b74e4eee8e93` | design preference `019e6924...` | stale handoffs `019e8475...`, `019e838b...` |

Read-only telemetry eval before this slice's feedback:

| Field | Value |
| --- | ---: |
| `trace_count` | 50 |
| `feedback_count` | 35 |
| `feedback_trace_count` | 35 |
| `feedback_coverage` | 70% |
| `memory_judgment_trace_coverage` | 73.33% |
| `distinct_intent_count` | 6 |
| `outcome_trace_count` | 35 |
| `task_failure_count` | 0 |
| `bad_memory_used_count` | 0 |
| `external_session_trace_count` | 0 |
| Confidence gate | passed |

Read-only telemetry eval after this slice's feedback:

| Field | Value |
| --- | ---: |
| `trace_count` | 50 |
| `feedback_count` | 39 |
| `feedback_trace_count` | 39 |
| `feedback_coverage` | 78% |
| `memory_judgment_trace_coverage` | 82.22% |
| `distinct_intent_count` | 6 |
| `outcome_trace_count` | 39 |
| `task_success_count` | 39 |
| `task_failure_count` | 0 |
| `preference_adhered_count` | 39 |
| `preference_violated_count` | 0 |
| `bad_memory_used_count` | 0 |
| `stale_memory_count` | 93 |
| `wrong_scope_memory_count` | 0 |
| `external_session_trace_count` | 0 |
| Confidence gate | passed |

## Completion Matrix Delta

| Area | State After T195 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Current-plan retrieval | Healthy for current continuation | `orient` and direct search return T194 current plan first | Does not prove broad ranking quality |
| Telemetry confidence gate | Numerically passing | `feedback_trace_count=39/50`, six intents, zero task failures, zero bad-memory-used reports | Sliding-window, agent-assessed evidence; still requires user approval for migration decisions |
| Design preference retrieval | Partially healthy | Reviewed preference returned and used | Direct search ranked stale handoffs above it in one trace |
| Stale active handoff noise | Still high | `stale_memory_count=93`; lint lists many superseded-active findings | Lifecycle archive packets remain exact-gated; no `apply_safe` run |
| Document visibility | Still gated | T194/T192 packets exist; indexing not executed | Requires exact T194/T192 approval |
| Native Claude cleanup / visibility | Still gated | T190/T186/T172 remain separate state | Requires exact T186 or future approved visibility work |
| M6/migration completion | Still high-risk | Telemetry pass is not migration approval | Candidate decisions, dry-run/apply evidence, rollback plan, and exact approval still required |
| External-session joinability | Missing in current sample | `external_session_trace_count=0` | Harness/host adoption gap remains |

## Decision

T195 removes the specific T189 blocker that the rolling telemetry confidence gate was failing. The
current 50-trace project sample now passes the numerical gate with broad intent coverage and no
reported task failures or bad-memory use.

This is not completion proof. It does not authorize migration write-apply, lifecycle archive,
document indexing, native Claude process actions, hook/harness writes, ranking or `orient` changes,
schema/storage/index changes, public MCP changes, deletion, rollback, or user-owned-file edits.

The full Brain Harness definition of done remains incomplete because exact-gated work is still
pending and some evidence remains weak or missing:

- T194 and T192 document indexing are pending exact approval.
- T193, T191, and T187 lifecycle archive packets are pending exact approval.
- T186 native Claude process cleanup and T172 effective-hook visibility remain unresolved.
- M6 candidate decisions, dry-run/apply evidence, rollback plan, and explicit migration approval
  remain incomplete.
- Stale active handoff noise remains visible in search/lint.
- `external_session_id` coverage is still zero in the current project sample.

## Negative Scope

T195 is not approval for any future gated work. Generic continuation remains non-authorization for
process signals, native Claude input, document indexing, lifecycle archive, M6/migration, harness
writes, source changes, ranking/`orient` changes, schema/storage/index changes, document-index
behavior changes, deletion, rollback, or user-owned-file edits.
