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

## Conclusions

1. Labeled telemetry works end to end for `scenario_id`, `arm`, outcome fields, missing context,
   used/rejected memory, and wrong-scope memory.
2. Clean no-memory comparison still needs a fresh or isolated agent session; this pilot cannot
   claim treatment superiority.
3. `orient` currently misses fresh docs/current-plan context in resume prompts.
4. `orient` can still preserve important reviewed safety decisions, as shown by the M6 migration
   gate scenario.
5. Wrong-scope noise is visible and measurable now, which is useful.

## Recommended Next Step

Implement the smallest ranking/capture improvement that helps `resume_session` orientation retrieve
the latest project plan:

- ensure the dogfood protocol and latest plan decision are available as MemoryItem or indexed
  document evidence,
- prefer recent current-branch docs/commits for resume-continuity prompts,
- link the dogfood protocol to the M6 migration-gate decision,
- rerun `resume_continuity_001` and `stale_scope_rejection_001` after the change.

Do not proceed to M6 write/apply yet.
