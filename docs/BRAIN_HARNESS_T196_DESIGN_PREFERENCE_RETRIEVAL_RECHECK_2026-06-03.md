# T196 Design Preference Retrieval Recheck

Date: 2026-06-03
Status: Docs-only retrieval caveat recheck.

## Scope

This slice rechecks the T195 design-preference retrieval caveat using read-only Engram telemetry
and search evidence. It does not change ranking, `orient`, public MCP parameters, schema/storage/
index behavior, document-index behavior, lifecycle state, M6/migration/quarantine state, native
Claude, Claude Bridge, harness files/settings/hooks/adapters, process state, deletion, rollback,
old-binary install state, or user-owned files.

## Research Question

Is the T195 design-preference direct-search caveat a persistent ranking failure that justifies a
source/ranking change, or a narrower prompt-shape and stale-handoff-noise problem that should be
handled by evidence tracking and existing lifecycle gates?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The caveat is prompt-shape sensitive: the exact T195 query still ranks stale handoffs above the reviewed preference, while close focused queries rank the preference first. |
| Null | Reviewed user design preference retrieval is broadly broken and consistently buried under stale project handoffs. |
| Simpler alternative | Do not change source ranking; record the mixed evidence and keep lifecycle cleanup gated to exact archive packets. |
| Failure | Treat one noisy trace as proof of broad ranking failure, or treat a few passing focused traces as proof that stale active handoff noise is solved. |

## Measurement

Before implementation, the slice defined the useful evidence as:

- T195 trace order for the exact design-philosophy query.
- Current trace order for close focused design-philosophy queries.
- One rerun of the exact T195 query with the same `follow_user_preference` intent.
- Current telemetry and completion-matrix state only as context, not as approval for gated work.

## Evidence

| Probe | Trace | Query Shape | Result |
| --- | --- | --- | --- |
| T195 original | `019e8e9b-553c-7871-b03b-cb473c201dff` | `follow_user_preference`; exact T195 wording: `user software design philosophy Ousterhout deep modules no unrequested features small end-to-end slices evidence over confidence` | Reviewed preference `019e6924-256b-7093-b1c5-286ec4d02461` was rank 3 behind stale handoffs `019e8475-3fa6-7080-9d80-ae81f24c9781` and `019e838b-6b25-7011-8b4b-b4cc61dc450f`. |
| Close focused query | `019e8e9d-ea6d-74c0-bfa7-1b0c0a493247` | `follow_user_preference`; same concepts without `small end-to-end slices` | Reviewed preference `019e6924-256b-7093-b1c5-286ec4d02461` was rank 1. |
| Startup required search | `019e8ea0-533c-7882-8001-26317fef0e3f` | `plan_work`; same concepts with `small slices` | Reviewed preference `019e6924-256b-7093-b1c5-286ec4d02461` was rank 1. |
| Exact rerun | `019e8ea1-159d-7000-9671-ff10928f45fc` | `follow_user_preference`; exact T195 wording | Reviewed preference `019e6924-256b-7093-b1c5-286ec4d02461` was rank 3 behind the same two stale handoffs as T195. |

Additional context:

- Lean `orient` trace `019e8ea0-3633-7ef1-84b4-a896940d4422` returned the active T195 current-plan
  memory first and no open obligations.
- Fresh direct current-plan search trace `019e8ea0-50cf-7330-a698-bab79392071d` returned active
  T195 current-plan memory first, followed by stale active handoffs.
- The current read-only `real_session_eval(project=engram, limit=50)` still passes the numerical
  confidence gate, but the sliding window now reports `feedback_trace_count=27`,
  `feedback_coverage=54%`, `memory_judgment_trace_coverage=52.08%`,
  `task_failure_count=0`, `bad_memory_used_count=0`, and `external_session_trace_count=0`.
- `docs/ORIENT_CONTRACT.md` still keeps graph traversal, obligation detection, lint, migration,
  and raw entity observation lookup out of the `orient` hot path.
- `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` requires preserving mixed evidence and avoiding broad
  implementation moves without adequate evidence.
- Git remains clean except the pre-existing untracked root `AGENTS.md`.

## Decision

The T195 caveat is real and reproducible for the exact query, but it is not broad proof that user
design-preference retrieval is broken. The reviewed user preference can rank first for close focused
queries, while the exact wording still lets stale rolling handoff MemoryItems outrank it.

The appropriate next action is not a ranking or `orient` change. The evidence supports keeping the
design-preference retrieval area marked partially validated and continuing to reduce stale active
handoff noise through the already exact-gated lifecycle archive packets.

## Completion Matrix Delta

| Area | State After T196 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Design preference retrieval | Partially validated, prompt-sensitive | Preference ranks first for two close queries and rank 3 for exact T195 wording | No broad ranking change justified; stale handoff noise remains. |
| Current-plan retrieval | Healthy for current continuation | Lean `orient` and direct current-plan search returned T195 current plan first | Does not prove broad ranking quality. |
| Stale active handoff noise | Still high-impact | Exact preference query still has two stale handoffs above reviewed preference | T193/T191/T187 lifecycle archive packets remain exact-gated; no `lint apply_safe` run. |
| Telemetry confidence gate | Still numerically passing in the current window | Current eval has 27/50 traces with feedback, four intents, zero failures, and zero bad-memory-used reports | Sliding-window, agent-scored evidence; `external_session_trace_count=0` remains. |
| Native Claude cleanup / visibility | Still gated | T190/T186/T172 remain unresolved | Requires exact T186 or another approved visibility/cleanup packet. |
| M6/migration completion | Still high-risk and gated | No migration action in this slice | Candidate decisions, dry-run/apply evidence, rollback plan, and exact approval still required. |

## Negative Scope

T196 is not approval for ranking changes, lifecycle archive, `lint apply_safe`, document indexing,
native Claude input or signals, Claude Bridge, harness writes, M6/migration/quarantine actions,
public MCP/schema/storage/index/document-index behavior changes, deletion, rollback, old-binary
reinstall, or user-owned-file edits.
