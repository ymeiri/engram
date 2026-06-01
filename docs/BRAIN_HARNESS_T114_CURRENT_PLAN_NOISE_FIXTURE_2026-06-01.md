# Brain Harness T114 Current-Plan Noise Fixture

Status: Completed test-only regression fixture; no behavior change
Date: 2026-06-01
Scope: Lock in the observed post-T113 current-plan ordering against stale and wrong-scope noise.

This slice did not change ranking, `orient`, lifecycle state, telemetry behavior, public MCP
request/response shape, schema/storage/index behavior, document-index behavior, M6 state, document
indexing, harness adapters, hooks, or migration behavior.

## Research Question

Can the existing direct `search` behavior be captured in a deterministic fixture where the latest
project-scoped current-plan MemoryItem outranks a stale repository-scoped current-plan item and a
Claude-Code-authored rule that live telemetry has flagged as wrong-scope noise?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A narrow fixture can reproduce the live T113/T114 noise shape and pass without any ranking behavior change. |
| Null | The live ordering is not captured by deterministic tests; adding the fixture fails and would require a gated ranking decision. |
| Simpler alternative | Keep T113 as docs-only evidence and avoid adding executable coverage. |
| Failure | The fixture accidentally encodes a broad ranking contract or implies lifecycle/ranking cleanup approval. |

## Measurement

- Startup `orient` trace `019e8485-462b-7a80-8d41-66324c22809f` returned the T113 current-plan
  memory first, while still surfacing older stale current-plan guidance lower in candidate context.
- Direct current-plan/noise search trace `019e8485-7acf-79b0-abc7-7d00a7fc64e7` returned T113
  current-plan memory `019e8480-a1ea-7413-ac5e-403771aa8d6f` first and stale repository-scoped
  current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower.
- Direct risk/noise search trace `019e8485-7bcb-7c42-bc66-7682aaa4eb65` returned the T113
  current-plan memory first and the Claude-Code-authored rule
  `019e7f52-4fc2-7f61-93b4-9a741aba966e` lower.
- AI Council agreed the smallest safe slice is a deterministic fixture with relative-order
  assertions only: target first, known noisy distractors present but below it, no exact ordering
  between noisy items, and no score thresholds.
- Claude Bridge read-only critique timed out after 120 seconds, so no Claude critique evidence was
  obtained for this slice.
- Added
  `test_memory_search_t114_current_plan_outranks_stale_and_wrong_scope_noise` in
  `engram-tests/tests/search_tests.rs`.
- Targeted validation passed:
  `cargo test -p engram-tests test_memory_search_t114_current_plan_outranks_stale_and_wrong_scope_noise`.

## Completion Matrix

| Area | T114 state | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Direct current-plan search | Validated for one noise shape | New deterministic fixture passes with target first and stale/wrong-scope noise below it | Does not prove other query phrasings, larger corpora, or embedding-model behavior. |
| Ranking behavior | Unchanged | No production ranker code changed | Any ranking behavior change remains gated. |
| `orient` hot path | Unchanged | T114 touched only direct `search` fixture/docs | Do not expand payload or hot-path responsibilities without explicit approval. |
| Lifecycle cleanup | Unchanged/gated | Stale item and wrong-scope rule remain active live memory; lint still has `safe_action=none` for stale current-plan | No archive, scope correction, or `apply_safe` is authorized. |
| M6 migration | Stopped/gated | No M6 command or export inspection ran | T69/T70/M6 apply gates remain exact-approval only. |
| Cross-harness evidence | Inconclusive for T114 | Claude Bridge critique timed out | Native Claude parity still requires a separate safe run; bridge timeouts are not Engram failures. |
| Git/worktree hygiene | Preserved so far | Root `AGENTS.md` remains untracked/user-owned | Keep it unstaged and untouched. |

## Interpretation

T114 converts a live retrieval observation into executable regression coverage without changing
product behavior. The fixture is intentionally narrow: it asserts only that the active project
current-plan item is first and that the two known noisy distractors do not outrank it. It does not
assert exact noisy-item order, score values, broad ranking quality, lifecycle remediation, or
cross-harness parity.
