# Brain Harness Dogfood Run: 2026-05-07

Status: Pilot evidence
Protocol: `docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md`
Branch: `yuval.meiri/memory-os-phase0`
Head before run: `0eede1eeaa3f8218735d084ed47c7f96fa0dca2b`

## Summary

This run validated the new labeled telemetry path and produced the first scenario/arm evidence.
It is not decision-grade comparative evidence yet because the `no_memory` controls were run inside
an active Codex conversation that already contained Engram context and a compacted handoff.

The important finding is product-level: `orient` can preserve a safety-critical reviewed decision
for migration gating, but it did not surface the newest dogfood protocol when asked to resume after
adding that protocol. The next fix should target capture/indexing/ranking for fresh project plan
documents and decisions before broadening the eval batch.

A focused rerun after `dcb72f2` narrowed the failure: recent Git context now surfaces the fresh
Brain Harness documents and commits, but Brain Loop still ranks older harness-adapter memories above
the current research method and dogfood plan. Diagnostic search did not return the new Brain Harness
docs as document results. The next fix should therefore target active MemoryItem capture/promotion
or project-doc indexing for current plan documents, not graph/obligation hot-path expansion.

After Claude Bridge and AI Council review, a two-stage decoupled probe separated project-document
indexing from active MemoryItem capture. Registering and indexing the three Brain Harness docs made
document search succeed, but did not make `orient` succeed. Adding exactly two active project-scoped
MemoryItems for the research method and current probe made `orient` return those items as the top two
Brain Loop signals. The next engineering step should therefore design a low-friction current-plan
capture path before changing ranking, stale cleanup, graph traversal, or obligations in the hot path.

## Preflight

### Corpus Shape

Daemon-backed read-only preflight showed enough live MemoryItem signal to run a pilot:

- Active MemoryItems sampled: 500, capped at the call limit.
- Active MemoryItems with evidence: 420.
- Active MemoryItems without evidence: 80.
- Active reviewed MemoryItems in the capped sample: 259.
- `needs_review` MemoryItems: 21.
- Active corpus is broad enough for dogfood, but the sample contained repeated `Rolling handoff`
  entries that look noisy for resume-continuity scenarios.

Active sample by kind:

| Kind | Count |
|---|---:|
| `decision` | 164 |
| `handoff` | 68 |
| `limitation` | 34 |
| `preference` | 1 |
| `project_fact` | 51 |
| `repository_fact` | 3 |
| `rule` | 27 |
| `session_insight` | 145 |
| `task_fact` | 1 |
| custom kinds | 7 |

### Telemetry Baseline

`telemetry(action=real_session_eval, limit=25)` before the pilot:

| Field | Value |
|---|---:|
| `trace_count` | 25 |
| `feedback_count` | 20 |
| `feedback_coverage` | 0.80 |
| `distinct_scenario_count` | 0 |
| `distinct_arm_count` | 0 |
| `outcome_feedback_count` | 4 |
| `bad_memory_used_count` | 0 |
| `confidence_gate.passed` | true |

The gate passed operationally, but the system had no comparative scenario/arm evidence before this
run.

## Scenarios

### `resume_continuity_001`

Prompt:

> Resume the Engram brain harness work after adding the dogfood protocol. What is the next action
> and why?

Expected helpful context:

- The new dogfood protocol.
- The Council/Claude synthesis that dogfood should precede M6 write/apply.
- The immediate next action: run labeled scenarios through daemon-backed MCP and record feedback.

| Arm | Trace | Outcome |
|---|---|---|
| `no_memory` | `019e0151-b824-7402-9048-4e69f5960375` | Control trace recorded, but contaminated by current-session transcript. |
| `memoryitem_orient` | `019e0151-d3ee-7ec1-854d-6a55b8b2f638` | Failed as retrieval evidence. It surfaced the daemon lock limitation but did not surface the dogfood protocol or current next action. |

Recorded feedback for `memoryitem_orient`:

- `task_success`: false.
- `usefulness_score`: 2.
- `correctness_score`: 3.
- `noise_score`: 4.
- Used memory: `019dd462-2668-7cd0-b4c3-f2986e223ba0` (`Direct CLI store commands conflict with running global daemon`).
- Rejected memories: older harness installation/compaction decisions that were not the current
  resume target.
- Missing context: dogfood protocol, Council/Claude dogfood-first decision, immediate scenario run
  plan.

Interpretation: resume-continuity is the clearest failure mode. The current Brain Loop can be
operationally correct while still missing the freshest project plan.

### `stale_scope_rejection_001`

Prompt:

> Before doing any migration or deletion in Engram, verify whether M6 migration write/apply should
> proceed now or remain gated. What should the agent do?

Expected helpful context:

- `Migration Must Be Review-Gated`.
- Dogfood protocol safety rule: no M6 write/apply from this protocol.
- Direct daemon/MCP path limitation for read operations.

| Arm | Trace | Outcome |
|---|---|---|
| `no_memory` | `019e0152-0d2a-73d2-9731-bf34731f10fa` | Control trace recorded, but contaminated by current-session transcript. |
| `memoryitem_orient` | `019e0152-2745-75d1-bf69-2e698c78aaea` | Passed safety behavior with moderate noise. It surfaced the reviewed migration-gate decision and would keep M6 write/apply blocked. |

Recorded feedback for `memoryitem_orient`:

- `task_success`: true.
- `usefulness_score`: 4.
- `correctness_score`: 4.
- `noise_score`: 3.
- Used memories:
  - `019dc9ce-3b4e-7b02-80b5-04f56c84624e` (`Migration Must Be Review-Gated`).
  - `019dd3a0-0eef-7741-b4dd-bdd3d56989cd` (`Migration inventory now supports durable batch progression`).
  - `019dd462-2668-7cd0-b4c3-f2986e223ba0` (`Direct CLI store commands conflict with running global daemon`).
- Wrong-scope memory:
  - `019dd33d-0907-72e3-a721-dd80497787c8` (`Cursor harness should use skills plus optional hooks, not only rules`).
- Missing context: the newer dogfood protocol gating M6 from this specific run.

Interpretation: `orient` is already useful for high-stakes safety gating, but it still needs better
fresh-plan linkage and less wrong-scope noise.

## Post-Run Telemetry

`telemetry(action=real_session_eval, limit=50)` after the pilot:

| Field | Value |
|---|---:|
| `trace_count` | 50 |
| `feedback_count` | 24 |
| `feedback_coverage` | 0.48 |
| `distinct_scenario_count` | 2 |
| `distinct_arm_count` | 2 |
| `scenario_counts.resume_continuity_001` | 2 |
| `scenario_counts.stale_scope_rejection_001` | 2 |
| `outcome_feedback_count` | 8 |
| `task_success_count` | 7 |
| `task_failure_count` | 1 |
| `wrong_scope_memory_count` | 1 |
| `bad_memory_used_count` | 0 |
| `confidence_gate.passed` | false |

The post-run gate failed only because the larger 50-trace window had 48% feedback coverage. The
pilot successfully created the first labeled scenario/arm coverage.

## Focused Rerun After Research Method

After commit `dcb72f2 Add brain harness research method`, reran `resume_continuity_001` to test the
claim from `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`: recent Git context plus current Brain Loop
ranking should either fix resume continuity or reveal that fresh project-plan docs need active
MemoryItem capture/promotion.

Prompt:

> Resume the Engram brain harness work after adding the Brain Harness research method and recent Git
> context. What is the next action and why?

Expected helpful context:

- `dcb72f2 Add brain harness research method`.
- `68ed6d2 Add recent git context to orient`.
- `4fe2570 Add brain harness dogfood pilot report`.
- `0eede1e Add brain harness dogfood protocol`.
- Explicit next action: use the rerun evidence to choose between broader dogfood and targeted
  capture/ranking/promotion for fresh plan context.

| Arm | Trace | Outcome |
|---|---|---|
| `memoryitem_orient` | `019e01d6-de7b-7052-844f-5584a24d5e35` | Partial failure. `repository_context.recent_commits` surfaced all expected recent commits, but active decisions and Brain Loop top items were dominated by older harness-adapter decisions and did not state the current research/dogfood next action. |
| `diagnostic_search` | `019e01d7-32b5-7142-af6b-e7cd0378c883` | Confirmed the likely failure mode. Memory search found adjacent older Brain Harness facts, but not an active MemoryItem for the research-method checkpoint or immediate next action. Document search did not return the new Brain Harness docs. |

Recorded feedback for `memoryitem_orient`:

- `task_success`: false.
- `usefulness_score`: 3.
- `correctness_score`: 3.
- `noise_score`: 4.
- `bad_memory_used`: false.
- Rejected memories:
  - `019dd509-46f2-71c0-aff7-ebe777810825`
  - `019dd334-3b8e-7471-afba-1b1aaeecfe45`
  - `019dd321-4584-7970-ae87-b81eacea7a3f`
  - `019dd320-02fb-7420-8d57-bf255f27a0a9`
  - `019dd313-eb1f-7741-9854-dcf00e3f2229`
- Missing context: active Brain Harness research method, dogfood pilot/protocol as current plan
  guidance, and the explicit next action.

Diagnostic search result:

- Useful adjacent MemoryItems existed:
  - `019dfed3-519d-7f01-8c46-c9245ba0045b`
  - `019dfe2f-7950-7a63-9416-6e059e7af34c`
- Search did not retrieve:
  - `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`
  - `docs/BRAIN_HARNESS_DOGFOOD_RUN_2026-05-07.md`
  - `docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md`

Post-rerun `telemetry(action=real_session_eval, limit=50)`:

| Field | Value |
|---|---:|
| `trace_count` | 50 |
| `feedback_count` | 30 |
| `feedback_coverage` | 0.60 |
| `distinct_scenario_count` | 3 |
| `distinct_arm_count` | 4 |
| `scenario_counts.resume_continuity_001` | 5 |
| `outcome_feedback_count` | 14 |
| `task_success_count` | 12 |
| `task_failure_count` | 2 |
| `bad_memory_used_count` | 0 |
| `confidence_gate.passed` | true |

Interpretation: recent Git context is useful and should stay, but it is not sufficient for
resume-continuity. Fresh plan documents need an active cognitive representation or reliable project
document indexing. The evidence does not justify adding graph traversal, obligation detection, lint,
or raw observation lookup to the `orient` hot path.

## Two-Stage Missing-Signal Probe

Claude Bridge and AI Council agreed on a decoupled probe: first make the missing Brain Harness docs
available as documents, then add only one or two active MemoryItems if `orient` still misses the
current plan. This tested whether the problem was missing document evidence, missing active memory,
or ranking/noise.

### Stage 1: Register and Index Docs

Registered the three missing Brain Harness documents in the knowledge registry:

| Document | Registry ID |
|---|---|
| `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` | `019e01ef-420b-7733-b39d-6048ade20598` |
| `docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md` | `019e01ef-4eb1-7211-88e1-f667d8e5990f` |
| `docs/BRAIN_HARNESS_DOGFOOD_RUN_2026-05-07.md` | `019e01ef-5aa7-7eb0-8781-eb7a1714a852` |

Indexed the same files into the document layer:

| Document | Chunks |
|---|---:|
| `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` | 29 |
| `docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md` | 18 |
| `docs/BRAIN_HARNESS_DOGFOOD_RUN_2026-05-07.md` | 13 |

Document search trace `019e01ef-b16c-79a0-b3a2-c379598ae47f` then returned the dogfood run,
research method, and dogfood protocol as top document results. Feedback
`019e01ef-ea72-7660-bd14-17934b2981df` marked this as successful document retrieval.

However, the Stage 1 `orient` rerun still failed:

| Arm | Trace | Outcome |
|---|---|---|
| `memoryitem_orient_docs_indexed` | `019e01ef-c496-7e62-896c-4de5ce02d5f6` | Failed. Recent Git context remained useful, but active decisions and Brain Loop still missed the research method, dogfood docs, and immediate next action. |

Feedback `019e01f0-025c-7c62-920c-2b9cb1c2af2d` recorded `task_success=false`,
`noise_score=4`, and rejected the older harness-adapter memories that dominated the top results.

Interpretation: document indexing fixed document search but not Brain Loop v1. Current `orient`
does not pull document results into the hot path, so document availability alone cannot fix
resume-continuity.

### Stage 2: Add Minimal Active MemoryItems

Added exactly two active project-scoped MemoryItems:

| Kind | ID | Title |
|---|---|---|
| `rule` | `019e01f1-f262-7d63-bd33-a2ca28228c03` | Brain Harness work follows research method |
| `decision` | `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` | Resume continuity probe uses active MemoryItems before ranking changes |

The rule has manual-review, file, and git-commit evidence. The decision has tool-call evidence from
the successful document search and failed Stage 1 `orient` trace, plus file evidence pointing at this
dogfood report.

Recorded Memory OS knowledge commit `019e01f3-085b-7031-9124-d0860267d16c` for the two added
MemoryItems.

The Stage 2 `orient` rerun then passed:

| Arm | Trace | Outcome |
|---|---|---|
| `memoryitem_orient_docs_indexed_memoryitems` | `019e01f2-24f0-72e3-ac95-67f1dfb5ef3b` | Passed. Brain Loop top item 1 was the research-method rule. Top item 2 was the current resume-continuity probe decision. Active Rules and Active Decisions also surfaced the two items. |

Feedback `019e01f2-4de3-72d0-a377-5dbcbe9e4896` recorded:

- `task_success`: true.
- `usefulness_score`: 5.
- `correctness_score`: 5.
- `noise_score`: 2.
- Used memories:
  - `019e01f1-f262-7d63-bd33-a2ca28228c03`
  - `019e01f2-0a87-7f73-9b0b-7f2443eac7bb`

Post-probe `telemetry(action=real_session_eval, scenario_id=resume_continuity_001, limit=20)`
showed the three-arm result:

| Arm | Task Success | Task Failure | Used Memory Count | Rejected Memory Count |
|---|---:|---:|---:|---:|
| `memoryitem_orient` | 0 | 1 | 0 | 5 |
| `memoryitem_orient_docs_indexed` | 0 | 1 | 0 | 5 |
| `memoryitem_orient_docs_indexed_memoryitems` | 1 | 0 | 2 | 0 |

Interpretation: the failure was primarily missing active current-plan MemoryItems, not stale-memory
cleanup, document indexing alone, graph traversal, or obligation hot-path absence. Document indexing
is still valuable as evidence and search substrate, but Brain Loop v1 needs an active cognitive
representation for current plan/method guidance.

## Conclusions

1. Labeled telemetry works end to end for `scenario_id`, `arm`, outcome fields, missing context,
   used/rejected memory, and wrong-scope memory.
2. Clean no-memory comparison still needs a fresh or isolated agent session; this pilot cannot
   claim treatment superiority.
3. `orient` currently misses fresh docs/current-plan context unless that context is also represented
   as active MemoryItems.
4. `orient` can still preserve important reviewed safety decisions, as shown by the M6 migration
   gate scenario.
5. Wrong-scope noise is visible and measurable now, which is useful.
6. The focused rerun shows recent Git context improves artifact visibility, but does not replace
   active MemoryItem guidance or project-doc indexing for current plan continuity.
7. The two-stage probe shows that active current-plan MemoryItem capture is the smallest evidenced
   fix for the resume-continuity failure class.

## Recommended Next Step

Implement the smallest low-friction current-plan capture path that helps `resume_session`
orientation retrieve the latest Brain Harness plan:

- after substantial Brain Harness docs, research decisions, or dogfood reports are created or
  updated, capture one compact active MemoryItem for the current method/plan/next action,
- require file/tool-call/manual-review evidence for durable decisions and rules,
- keep project-document indexing as evidence/search substrate, but do not assume it feeds Brain
  Loop v1,
- rerun `resume_continuity_001` after the change and keep `stale_scope_rejection_001` as the safety
  regression scenario.

Do not proceed to M6 write/apply yet.
