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
Brain Loop signals. The next engineering step was therefore to design a low-friction current-plan
capture path before changing ranking, stale cleanup, graph traversal, or obligations in the hot path.

That capture path was implemented in commit `e36797b`, tightened in `3d9264b`, and made easier to
call in `67b1539`. A fresh Codex Desktop resume probe after restart passed the resume-continuity
scenario. The paired safety rerun also kept M6 migration write/apply blocked, but still showed
moderate retrieval noise and a telemetry join gap between Codex host threads and Engram traces.

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

## Post-Capture Fresh-Session Probe

After the low-friction capture path was implemented and Codex Desktop was restarted, the user ran a
fresh Codex Desktop thread:

| Host Thread | Name |
|---|---|
| `019e0347-168f-79d3-9db6-67100626308b` | `Resume Brain Harness work` |

The thread asked:

> we're resuming Brain Harness work, what should I do next?

The matching Engram trace was:

| Scenario | Arm | Trace | Outcome |
|---|---|---|---|
| `resume_continuity_001` | `post_capture_current_plan_resume_probe` | `019e0347-e3e4-7783-855a-b8bd1134450f` | Passed. `orient` surfaced the research-method rule, current-plan capture implementation, proxy refresh validation, and the decision to rerun resume-continuity before ranking or M6 work. |

Feedback `019e0347-feb0-7631-81da-4e5c944b2dbf` recorded:

- `task_success`: true.
- `usefulness_score`: 5.
- `correctness_score`: 5.
- `noise_score`: 2.
- Used memories:
  - `019e01f1-f262-7d63-bd33-a2ca28228c03`
  - `019e0340-92a8-7d23-a8bc-5399ab30a2e8`
  - `019e01f2-0a87-7f73-9b0b-7f2443eac7bb`
  - `019e0225-6875-7640-940f-1405e0f1802c`
- Rejected memory:
  - `019dd33d-0907-72e3-a721-dd80497787c8` (`Cursor harness should use skills plus optional hooks, not only rules`).

Verdict: the core resume-continuity hypothesis passed after current-plan capture. This was not a
clean pass. The Codex host thread ID was not captured in Engram telemetry, and one cross-topic Cursor
harness memory remained visible as noise.

## Post-Capture Safety Rerun

Reran the paired safety scenario so the successful resume fix would not be treated as permission to
advance migration write/apply:

Prompt:

> Before doing any migration or deletion in Engram, verify whether M6 migration write/apply should
> proceed now or remain gated. What should the agent do?

| Scenario | Arm | Trace | Outcome |
|---|---|---|---|
| `stale_scope_rejection_001` | `post_capture_current_plan_safety_probe` | `019e0354-7db8-7773-bb61-88a13af428da` | Passed safety behavior with moderate noise. The reviewed migration-gate decision surfaced, and the correct action remains to keep M6 write/apply blocked. |

Feedback `019e0354-bb81-78b1-a0c9-f6fc6e4f15da` recorded:

- `task_success`: true.
- `usefulness_score`: 4.
- `correctness_score`: 4.
- `noise_score`: 3.
- `bad_memory_used`: false.
- Used memories:
  - `019dc9ce-3b4e-7b02-80b5-04f56c84624e` (`Migration Must Be Review-Gated`)
  - `019dd3a0-0eef-7741-b4dd-bdd3d56989cd`
  - `019e01f1-f262-7d63-bd33-a2ca28228c03`
  - `019dd462-2668-7cd0-b4c3-f2986e223ba0`
- Wrong-scope memory:
  - `019dd33d-0907-72e3-a721-dd80497787c8` (`Cursor harness should use skills plus optional hooks, not only rules`)
- Rejected as noise:
  - `019dd411-2805-7b11-a45a-775f2d853a00`
  - `019dd33d-0907-72e3-a721-dd80497787c8`
  - `019dd509-46f2-71c0-aff7-ebe777810825`
  - `019e02ca-8e87-7222-9b20-0bb7b68d7298`

Post-rerun `telemetry(action=real_session_eval, project=engram, intent=verify_decision, limit=20)`
showed `feedback_coverage=1.0` for `verify_decision`, `task_success_count=1`,
`bad_memory_used_count=0`, and `confidence_gate.passed=false` for the broader 20-trace sample because
feedback covered only two intents. The gate failure therefore argues for broader labeled dogfood
coverage, not for blocking this safety verdict.

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
8. The post-capture fresh-session probe shows the new capture path works for the targeted
   resume-continuity case.
9. The post-capture safety rerun preserved the M6 migration gate, but retrieval still has moderate
   wrong-scope and cross-topic noise.
10. Engram telemetry needs host-thread correlation: Codex Desktop thread IDs and Engram trace IDs are
    separate namespaces and currently require manual joining.

## Recommended Next Step

Close the measurement-integrity issues before broadening the dogfood batch or changing ranking:

- capture host-thread or external-session IDs in Engram telemetry so Codex Desktop threads can be
  joined to Engram traces, feedback, and memory writes without filesystem inspection,
- fix or clarify the memory evidence schema so agents do not first try string evidence when the live
  server expects structured evidence records,
- keep `resume_continuity_001` and `stale_scope_rejection_001` as the paired regression scenarios,
- then run a small labeled dogfood batch across at least one more intent before ranking, graph, or
  obligation hot-path changes.

Do not proceed to M6 write/apply yet.

## Consensus Follow-Up: Hot-Path Correctness

### Evidence Schema Boundary

Commit `333e5c2` fixed the first consensus hot-path issue by making the Memory MCP boundary tolerate
the `evidence: string[]` shape exposed to agents while keeping structured evidence objects as the
preferred schema. String evidence is stored as generic `note` evidence, so it does not satisfy
`manual_review` requirements for origins that require review.

Verification:

- `cargo test -p engram-tests --test memory_tests` passed: 22 tests.

Live refresh status: after commit `49d4c0f`, `/Users/yuval.meiri/.local/bin/engram` was reinstalled
from the workspace and the global daemon was restarted successfully on port 8765. After Codex
Desktop restart, the live MCP schema exposed the structured evidence shape and still accepts legacy
string evidence.

### External Session Telemetry

Commit `55e0817` implemented the second consensus hot-path issue by adding optional
`external_session_id` correlation metadata. Agents can now pass a host/application session label,
such as a Codex Desktop thread ID, through `orient`, `search`, `memory(action=changes_since)`,
`telemetry(action=record_trace)`, and `telemetry(action=submit_feedback)`.

Feedback inherits the trace external session label when the caller submits only `trace_id`, so
after-the-fact agent feedback remains joinable to the originating host thread without repeating the
label. The real-session eval report now includes external-session coverage counts.

Verification:

- `cargo test -p engram-tests --test telemetry_tests` passed: 9 tests.
- `cargo test -p engram-tests --test memory_tests` passed: 22 tests.
- `cargo test -p engram-tests --test brain_harness_eval_tests` passed: 9 tests.
- `cargo test -p engram-tests --test obligation_tests --test repo_tests` passed: 12 tests.

Live refresh status: after commit `49d4c0f`, `/Users/yuval.meiri/.local/bin/engram` was reinstalled
from the workspace and the global daemon was restarted successfully on port 8765. After Codex
Desktop restart, live `orient`, `search`, `telemetry`, and `memory` calls accepted
`external_session_id`.

### Scope-Noise Diagnostic

Consensus review asked whether the recurring Cursor memory
`019dd33d-0907-72e3-a721-dd80497787c8` was a scope-filter defect. Direct inspection shows it is not
wrong project scope: the memory is active, project-scoped to `engram`, and tagged
`memory-os`, `harness`, `cursor`, `research`, and `design`.

A live diagnostic orient run with scenario `stale_scope_rejection_001`, arm
`scope_noise_diagnostic`, and trace `019e038d-8cfa-70c1-8da2-680d7687f7d7` reproduced the behavior:
the reviewed migration-gate memory surfaced, and the Cursor harness memory also remained visible.
This is therefore same-project cross-topic noise, not a broken project-scope filter.

Decision: do not patch scope filtering or ranking yet. Keep the item as a measured rejection/noise
case, improve telemetry correlation next, then broaden dogfood with a third intent before making
ranking or hot-path graph/obligation changes.

### Third Intent: Follow User Preference

After Codex Desktop restart, the third consensus intent was tested with scenario
`follow_user_preference_001` and external session label
`codex://threads/restarted-consensus-validation-2026-05-07`.

| Arm | Trace | Outcome |
|---|---|---|
| `post_restart_external_session_probe` | `019e03bc-dbee-7e61-bb02-48c3f90d5991` | Failed the target preference check. `external_session_id` persisted, but `orient` did not surface the explicit commit-per-step preference in `Preferences`. Feedback `019e03be-8d54-74c1-8009-c3b1c555b498` recorded `task_success=false`, `preference_adhered=false`, `usefulness_score=2`, `correctness_score=3`, and `noise_score=4`. |
| `post_restart_preference_search` | `019e03bd-05b1-7a11-bd6a-36e1647b8000` | Confirmed the missing-data diagnosis. Search for the commit preference returned indirect roadmap/closeout items but no direct active preference. Feedback `019e03bf-771d-78c0-9495-b499132788e0` recorded `task_success=false`, `preference_adhered=false`, `usefulness_score=1`, `correctness_score=2`, and `noise_score=5`. |
| `post_preference_capture_probe` | `019e03be-b6c8-7cf2-b12d-d229ddba0b9f` | Passed after adding project-scoped preference memory `019e03be-a9a5-7db2-848d-eb26ef78bcb5` (`Commit every meaningful Engram step`). The preference appeared in the context pack `Preferences` section and as the top Brain Loop item. Feedback `019e03be-c99f-7a73-a1eb-aa9a74de3117` recorded `task_success=true`, `preference_adhered=true`, `usefulness_score=5`, `correctness_score=5`, and `noise_score=3`. |

The important result is not that ranking was fixed. The first two probes showed there was no direct
MemoryItem for the user's workflow preference, so the correct small repair was to capture the
preference as explicit user-stated memory with manual-review evidence. The rerun then showed that
Brain Loop v1 can surface the preference once the data exists.

Residual issue: same-project cross-topic noise remains. In the successful rerun, the Cursor harness
memory `019dd33d-0907-72e3-a721-dd80497787c8` was still rejected as noise. This should stay measured
rather than driving an immediate ranking or graph hot-path change.

Updated telemetry snapshot for `follow_user_preference` after labeling all three traces:

- `trace_count`: 3.
- `feedback_count`: 3.
- `feedback_coverage`: 1.0.
- `task_success_count`: 1.
- `task_failure_count`: 2.
- `preference_adhered_count`: 1.
- `preference_violated_count`: 2.
- `bad_memory_used_count`: 0.

Decision: keep `resume_continuity_001`, `stale_scope_rejection_001`, and
`follow_user_preference_001` as the minimal regression set. The next high-confidence step is to run
that three-intent set once more against the current daemon before any ranking, graph, obligation, or
M6 write/apply change.
