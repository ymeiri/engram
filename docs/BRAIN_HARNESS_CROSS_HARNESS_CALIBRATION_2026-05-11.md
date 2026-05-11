# Brain Harness Cross-Harness Calibration

Status: Pre-registered, not yet run
Date: 2026-05-11
Scope: Phase 1 measurement calibration for Codex Desktop and Claude Code
Related:

- `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`
- `docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md`
- `docs/BRAIN_HARNESS_ARCHITECTURE.md`

---

## 1. Research Question

Can Engram's `memoryitem_orient` path produce behavior-linked improvement in Codex Desktop and
Claude Code, beyond operational telemetry success, without increasing bad-memory use?

This run is not a broad cross-harness benchmark. It is a calibration gate before any larger
benchmark, migration simplification, ranking change, or hot-path expansion.

## 2. Current Evidence Baseline

Current evidence does not yet support broad claims:

- prior same-harness Codex evidence supports durable preference recall in this repository,
- BAF002 and BAF003 did not show a material `memoryitem_orient` advantage,
- BAF004 and BAF005 exposed scoring, target-visibility, and feedback-attribution confounds,
- Claude Code was only just refreshed to project-root readiness and has not been behaviorally
  benchmarked,
- recent Engram telemetry for project `engram` is operational but not comparative: it has feedback
  coverage, but almost all recent traces lack explicit `scenario_id` and `arm`.

Therefore the next step is staged calibration:

1. Claude Code rescue/failure-injection scenarios.
2. Codex Desktop redemption scenarios against prior weak/confounded evidence.
3. Only after both pass, a small cross-harness batch.

## 3. Decision From Model Consultation

AI Council and Claude Bridge agreed on the main constraint:

- do not run full `/research` yet,
- do not run a broad cross-harness benchmark yet,
- do use the research method's pre-registration discipline now,
- measure behavior, not just telemetry,
- handle historical Codex failures directly,
- qualify Claude Code before comparing it to Codex Desktop.

The full research skill may be useful after Phase 1, when the experimental instrument and oracle
are stable enough for a ratchet loop. At this stage, the full workflow would add process before the
measurement surface is proven.

## 4. Working Tree Disposition

The root checkout currently has an untracked `AGENTS.md`.

For this pre-registration commit:

- `AGENTS.md` is treated as pre-existing user-owned state,
- it must not be staged or committed by this run,
- experiment arms must not run in the dirty root checkout,
- each arm must run in a fresh isolated worktree created from a clean committed base.

Before launching any arm, the evaluator must record:

- base commit SHA,
- worktree path,
- `git status --short` output,
- whether any untracked sentinel files are intentionally present.

If an arm starts from a dirty worktree that was not pre-registered, mark the arm invalid with
failure tag `dirty_start`.

## 5. Global Anti-Confound Rules

- Freeze prompts, expected outcomes, and rubrics before launching an arm.
- Do not let an arm read or edit this calibration document.
- Do not let one arm inspect another arm's worktree, commit, transcript, or notes.
- Do not tune memory, ranking, prompts, or docs between arms of the same scenario.
- Count retries as retries, not fresh successes.
- Treat telemetry presence as necessary but not sufficient.
- Prefer within-harness deltas over cross-agent leaderboards.
- Record failures and ambiguous outcomes instead of converting them into passes.

## 6. Required Telemetry

Every arm must record:

- `project=engram`,
- `scenario_id`,
- `arm`,
- `agent`,
- `intent`,
- `external_session_id` when available,
- returned `trace_id`,
- returned memory IDs for orientation/search arms,
- used memory IDs,
- rejected memory IDs,
- stale or wrong-scope memory IDs when applicable,
- outcome feedback before final scoring.

For `no_memory` arms, the agent must not call `orient`, `search`, `memory`, `graph`,
`obligations`, `handoff`, AI Council, Claude Bridge, or Gemini Bridge before completing the task.
After the task, the evaluator records a baseline trace with:

```text
telemetry(action=record_trace, operation=feedback, scenario_id=..., arm=...)
```

Then the evaluator submits feedback to that trace.

For `memoryitem_orient` arms, the agent must call `orient` once at the start with the exact
scenario metadata, then work naturally from the returned context.

## 7. Scoring Rubric

Primary scores:

| Metric | Pass Condition |
|---|---|
| `task_success` | The pre-registered task is completed without avoidable correction. |
| `behavioral_delta` | `memoryitem_orient` changes a relevant decision or action compared with `no_memory`. |
| `preference_adhered` | Known project/user preference is followed without being restated by the user. |
| `bad_memory_used` | Must be false. Any true value blocks escalation. |
| `repeated_context_questions` | Must be lower in `memoryitem_orient` when the scenario tests continuity. |

Diagnostic scores:

| Metric | Purpose |
|---|---|
| target visibility | Expected helpful memory appeared in the pre-arm smoke orientation. |
| memory use correctness | Used memory IDs match the expected helpful set. |
| rejection correctness | Stale/wrong-scope memory is ignored and reported. |
| latency-adjusted utility | Orientation overhead is justified by behavior improvement. |
| contamination | Arm stayed isolated from other arms and evaluator notes. |

Failure tags:

- `dirty_start`
- `target_absent`
- `telemetry_gap`
- `cross_arm_contamination`
- `prompt_drift`
- `missing_context`
- `repeated_context_question`
- `preference_violation`
- `bad_memory_used`
- `stale_memory_used`
- `wrong_scope_memory_used`
- `no_behavioral_delta`
- `self_referential_slice`

## 8. Phase 1A: Claude Code Rescue

Purpose: prove whether Claude Code has a continuity problem that Engram can rescue.

Harness:

- `claude_code`

Arms:

- `claude_no_memory`
- `claude_memoryitem_orient`

Pre-arm smoke:

- For each `claude_memoryitem_orient` scenario, evaluator runs `orient` with
  `arm=prearm_smoke` before launching the Claude arm.
- The expected target-bearing memory must appear.
- If absent, do not launch the treatment arm.

### Scenario `claude_rescue_current_plan_001`

Intent: `resume_session`

Question:

Can Claude identify the current Engram measurement direction without the user restating the
consultation outcome?

Prompt packet:

```text
We are continuing Engram brain-harness work. Determine the correct next measurement step and
explain what should not be run yet. Keep the answer actionable and evidence-bound.
```

Expected helpful memory:

- `019e17e9-b6d4-76b2-9463-dbeeaf376398`
  `Consensus: staged Claude rescue and Codex redemption benchmark before cross-harness claims`
- `019e01f1-f262-7d63-bd33-a2ca28228c03`
  `Brain Harness work follows research method`

Expected behavior:

- says not to run full `/research` yet,
- says not to run broad cross-harness benchmark yet,
- proposes staged Claude rescue plus Codex redemption,
- mentions pre-registration and behavior-linked scoring,
- does not propose M6 write apply, deletion, ranking churn, or hot-path expansion.

No-memory expected failure mode:

- asks the user to restate recent consultation context, or
- proposes generic benchmarking without the staged rescue/redemption structure.

### Scenario `claude_rescue_commit_hygiene_001`

Intent: `follow_user_preference`

Question:

Can Claude preserve Engram's commit hygiene and user-owned file constraint inside a small work
slice?

Prompt packet:

```text
Prepare a small Engram doc-only calibration update plan. Include how you will handle unrelated
files and when you will commit. Do not implement yet.
```

Expected helpful memory:

- `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
  `Commit every meaningful Engram step`
- `019e17e9-b6d4-76b2-9463-dbeeaf376398`
  current benchmark plan memory

Expected behavior:

- explicitly keeps unrelated user-owned files out of commits,
- identifies untracked `AGENTS.md` as out of scope unless user approves otherwise,
- says a future implementation step should commit only meaningful calibration/doc changes after
  verification,
- does not ask whether to commit every meaningful Engram step.

No-memory expected failure mode:

- omits commit hygiene,
- treats `AGENTS.md` as a cleanup target,
- asks the user to repeat the known commit preference.

### Scenario `claude_rescue_bad_memory_guard_001`

Intent: `verify_decision`

Question:

Can Claude reject broad or premature next steps even if they sound aligned with the long-term
vision?

Prompt packet:

```text
Evaluate whether the next Engram step should be M6 inventory, ranking changes, graph/obligation
hot-path expansion, or benchmark calibration. Give a verdict and the reason.
```

Expected helpful memory:

- `019e17e9-b6d4-76b2-9463-dbeeaf376398`
- `019e01f1-f262-7d63-bd33-a2ca28228c03`

Expected behavior:

- chooses benchmark calibration,
- rejects M6 write apply, deletion, broad ranking changes, and hot-path expansion,
- treats read-only M6 inventory as deferred unless explicitly approved and justified by evidence.

No-memory expected failure mode:

- over-eagerly recommends architecture changes or inventory as the next step.

## 9. Phase 1B: Codex Desktop Redemption

Purpose: address the hard fact that prior Codex dogfood did not show a material
`memoryitem_orient` advantage in BAF002/BAF003 and later runs exposed confounds.

Harness:

- `codex`

Arms:

- `codex_no_memory`
- `codex_memoryitem_orient`

The `codex_no_memory` arm must not use Engram retrieval tools before task completion.

### Scenario `codex_redemption_current_plan_001`

Intent: `implement_change`

Question:

Can Codex preserve a non-obvious current plan and safety gate in a small doc-only work slice?

Work slice:

Add a short note to a new arm-local run report explaining that recent telemetry is operational but
not benchmark-grade unless `scenario_id` and `arm` are populated. Do not edit this
pre-registration document during the arm.

Expected helpful memory:

- `019e17e9-b6d4-76b2-9463-dbeeaf376398`
- `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
- `019e01f1-f262-7d63-bd33-a2ca28228c03`

Expected behavior:

- makes only the pre-selected narrow doc update,
- does not implement runtime code,
- does not rerun or rescore prior arms,
- runs `git diff --check`,
- commits only intended files,
- preserves unrelated `AGENTS.md`.

No-memory expected failure mode:

- makes a generic doc update without tying telemetry to benchmark-grade criteria,
- fails to preserve unrelated untracked files,
- does not commit the meaningful step.

### Scenario `codex_redemption_preference_in_work_slice_001`

Intent: `follow_user_preference`

Question:

Can Codex use durable preference memory to constrain a real work slice without the prompt fully
restating the preference?

Work slice:

Prepare a pre-run checklist in a new arm-local run report for the next Claude rescue arm. The
checklist must be operationally useful but must not launch an arm or edit this pre-registration
document.

Expected helpful memory:

- `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
- `019e17e9-b6d4-76b2-9463-dbeeaf376398`

Expected behavior:

- checklist includes clean worktree, base commit, target-visibility smoke, telemetry metadata,
  scoring rubric, and no-memory restrictions,
- commits only intended checklist/doc files after `git diff --check`,
- does not stage unrelated user-owned files.

No-memory expected failure mode:

- checklist omits target-visibility smoke or telemetry feedback,
- fails commit hygiene,
- asks for already-known Engram workflow preferences.

## 10. Optional Diagnostic Arm

Claude Bridge suggested an optional diagnostic arm:

- `memoryitem_orient_preference_stripped`

Use only if the evaluator can reliably sanitize the orientation packet before the agent sees it.
The purpose is to test whether improvement remains after direct preference/rule memories are
removed. If all lift disappears under this arm, record the honest claim as preference-recall-only.

Do not run this arm unless the sanitizer is defined in advance. Manual, inconsistent redaction
would add more confounding than signal.

## 11. Stop And Go Gates

Phase 1A Claude gates:

- Stop if Claude `no_memory` does not show any continuity-sensitive weakness across the selected
  scenarios. The scenario may be too easy or Claude may already solve it from repo context.
- Stop if `claude_memoryitem_orient` retrieves target memory but does not change behavior.
- Stop if any treatment uses stale, wrong-scope, or harmful memory.

Phase 1B Codex gates:

- Stop if Codex again shows no material behavioral delta under corrected rubrics.
- Stop if the only observed advantage is telemetry compliance rather than task behavior.
- Stop if the treatment requires prompt changes that the no-memory arm did not receive.

Phase 2 cross-harness gate:

- Proceed only when both harnesses have at least one behavior-linked `memoryitem_orient` lift and
  neither has elevated bad-memory use.
- If only one harness passes, do a harness-specific implementation or protocol fix before any
  cross-harness comparison.
- If both fail, revise orientation content, capture policy, or scenario design before scaling.

Implementation gate:

- A runtime implementation change is allowed only when the calibration result identifies one
  specific failure class and a small reversible fix.

## 12. Phase 2 Placeholder

If Phase 1 passes, run a small cross-harness batch:

- 6-8 scenarios,
- same scenario packets where feasible,
- arms: `no_memory`, `memoryitem_orient`,
- optional `legacy_search` only if Phase 1 shows orientation lacks necessary context,
- primary metric: within-harness delta,
- secondary metric: cross-harness comparability.

Do not score cross-harness results as an agent leaderboard.

## 13. What Counts As Evidence

Valid evidence:

- pre-registered scenario packets,
- target-visibility smoke trace IDs,
- isolated worktree paths and base SHAs,
- committed arm outputs,
- test or verification output,
- telemetry traces and feedback,
- independent or user scoring notes where judgment is required.

Invalid evidence:

- telemetry without scenario/arm labels,
- agent self-report without artifact or evaluator scoring,
- hidden retries,
- arms launched after prompt/rubric changes,
- treatment arms that can inspect no-memory outputs,
- success caused only by a fully specified prompt.

## 14. Next Action

Before launching any arm:

1. Commit this pre-registration document.
2. Choose the exact base commit for Phase 1.
3. Create isolated worktrees for each arm.
4. Run evaluator-side target-visibility smoke checks for treatment arms.
5. Record the smoke trace IDs in a run report.
6. Launch Claude rescue first, then Codex redemption.
