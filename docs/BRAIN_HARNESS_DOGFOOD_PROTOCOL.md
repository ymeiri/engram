# Brain Harness Dogfood Protocol

Status: Active protocol
Date: 2026-05-07
Audience: Engram maintainers and AI coding-agent operators
Scope: Generate small, labeled, live evidence for whether Engram's Brain Loop v1 improves agent behavior.
Research method: `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`

---

## 1. Purpose

Engram now has live `orient` and `search` telemetry, trace IDs, outcome feedback, and a
`telemetry(action=real_session_eval)` report. The current evidence is still mostly operational:
traces exist, but most are not grouped by `scenario_id` or compared across `arm`.

This protocol creates decision-grade dogfood evidence before any broad migration, deletion, or
legacy-layer simplification.

Under the Brain Harness research method, this protocol is an experimental instrument. It calibrates
live behavioral evidence and exposes failure modes; it does not, by itself, prove the full
architecture or authorize irreversible memory-system changes.

The protocol answers:

- Does `orient` improve real task behavior compared with a baseline?
- Does it preserve user/project preferences without repeated questions?
- Does it avoid stale, wrong-scope, or misleading memory?
- Does it surface follow-through obligations without bloating the hot path?
- If a run fails, is the likely next fix ranking, capture/promotion, or read-only migration
  inventory?

---

## 2. Non-Goals And Safety Rules

- Do not run M6 write apply from this protocol.
- Do not delete, archive, or deprecate legacy paths from this protocol.
- Do not promote observations or migration candidates unless the user explicitly approves a
  separate write step.
- Do not treat agent feedback as ground truth by itself.
- Do not count hidden retries as fresh successes.
- Do not expand `orient` with graph traversal, lint, raw observations, or obligation detection to
  make a scenario pass.

This protocol may use read-only inventory and read-only telemetry. It must not mutate memory except
for normal trace and feedback writes.

---

## 3. Required Preflight

Run this preflight before the first batch and record the results in the run notes.

### 3.1 Corpus Shape

Use MCP `memory(action=list)` or equivalent daemon-backed calls. Direct CLI access to the same
RocksDB path can fail while the daemon holds the database lock.

Record:

- active MemoryItems by `kind`,
- active MemoryItems with evidence vs. without evidence,
- `needs_review` MemoryItems by `kind`,
- obvious stale or wrong-scope candidates noticed during sampling,
- whether each planned scenario has enough live memory signal to run honestly.

If a list call reaches its limit, rerun with a higher limit or mark the count as capped.

### 3.2 Telemetry Baseline

Call:

```text
telemetry(action=real_session_eval, limit=25)
```

Record:

- `trace_count`,
- `feedback_count`,
- `feedback_coverage`,
- `distinct_scenario_count`,
- `distinct_arm_count`,
- `outcome_feedback_count`,
- `bad_memory_used_count`,
- whether `confidence_gate.passed` is true.

Passing the gate is not enough. If `distinct_scenario_count` or `distinct_arm_count` is zero, the
system still lacks comparative evidence.

### 3.3 Scenario Eligibility

Before running a scenario, write down:

- the expected helpful memory,
- memory that must not surface or must be rejected,
- the task output that can be checked,
- whether the user must judge success.

Skip or redesign scenarios where success cannot be judged after the run.

---

## 4. Arms

Every scenario needs at least two arms.

| Arm | Meaning | Required Behavior |
|---|---|---|
| `no_memory` | Baseline control | Do not call `orient`, `search`, `graph`, `memory(changes_since)`, or `obligations` before the task. After the task, record a baseline trace with `telemetry(action=record_trace, operation=feedback, scenario_id=..., arm=no_memory)` and submit feedback. |
| `memoryitem_orient` | Brain Loop v1 treatment | Call `orient` with `scenario_id`, `arm=memoryitem_orient`, project, cwd, prompt, and agent. Use the returned context naturally, then submit feedback to the returned `trace_id`. |

Optional arms:

| Arm | Meaning | Use When |
|---|---|---|
| `legacy_search` | Specialist legacy comparison | Call `search` with non-memory layers only, when the scenario asks whether legacy retrieval beats MemoryItem orientation. |
| `orient_plus_search` | Escalation path | Start with `orient`; call `search` only if context is missing. Use when testing whether the agent can recover without bloating `orient`. |

Do not compare arms if prompts, source files, or user guidance changed materially between arms.

---

## 5. Required Scenarios

Run the first batch with four or five scenarios. Use real project work where possible. Do not make
all scenarios easy.

### 5.1 Resume Continuity

Purpose: test whether Engram reconstructs the current project state without relying on transcript
memory.

- `scenario_id`: `resume_continuity_001`
- Intent: `resume_session`
- Should surface: current checkpoint, next actions, relevant committed decisions.
- Must not surface: obsolete handoff as the primary next action when a newer one supersedes it.
- Success: agent states the correct next step without asking the user to restate recent context.
- Failure tags: `missing_context`, `stale_handoff`, `repeated_context_question`.

### 5.2 User Preference Adherence

Purpose: test whether the agent follows durable user/project preferences.

- `scenario_id`: `preference_adherence_001`
- Intent: `follow_user_preference`
- Should surface: relevant user preference or project rule.
- Must not surface: unrelated preferences from another project or stale guidance.
- Success: agent follows the preference without re-asking.
- Failure tags: `preference_violation`, `missing_preference`, `wrong_scope_memory`.

### 5.3 Stale Or Wrong-Scope Rejection

Purpose: test whether Engram avoids harmful memory, and whether the agent rejects it when surfaced.

- `scenario_id`: `stale_scope_rejection_001`
- Intent: `verify_decision`
- Should surface: current decision or ambiguity.
- Must not surface: obsolete decision as authoritative, wrong repository guidance, stale
  obligation as current work.
- Success: no bad memory is used; if stale or wrong-scope memory appears, the agent explicitly
  rejects it and records it in feedback.
- Failure tags: `bad_memory_used`, `wrong_scope_memory`, `stale_memory`.

This is the primary falsifiability scenario. A passing system must know when not to use memory.

### 5.4 Implementation Decision Continuity

Purpose: test whether prior architectural decisions constrain new implementation work.

- `scenario_id`: `decision_continuity_001`
- Intent: `implement_change`
- Should surface: current architecture decision, relevant contract doc, and constraints.
- Must not surface: old alternatives as current direction.
- Success: implementation or plan follows the current decision and cites or uses the right
  constraint.
- Failure tags: `decision_drift`, `missing_context`, `bad_memory_used`.

### 5.5 Obligation Follow-Through

Purpose: test whether `orient` surfaces follow-through obligations without running expensive
detection in the hot path.

- `scenario_id`: `obligation_followthrough_001`
- Intent: `plan_work` or `implement_change`
- Should surface: already-open, currently applicable obligations.
- Must not surface: stale git-status document obligations or untracked root instruction files as
  current obligations.
- Success: the agent resolves, skips with reason, or plans around the obligation.
- Failure tags: `missing_obligation`, `stale_obligation`, `unresolved_obligation`.

---

## 6. Per-Run Procedure

For each scenario and arm:

1. Freeze the prompt and expected outcomes before running.
2. Record `scenario_id`, `arm`, `intent`, harness, model, cwd, and project.
3. Run the arm exactly once.
4. Capture every returned `trace_id`.
5. Score immediately after the run.
6. Submit `telemetry(action=submit_feedback)` to the relevant trace.
7. If the arm has no retrieval trace, first create one with
   `telemetry(action=record_trace, operation=feedback, scenario_id=..., arm=...)`.
8. Record human/user confirmation when the outcome requires judgment.

Feedback fields:

| Field | Rule |
|---|---|
| `task_success` | True only when the task objective was met without an avoidable correction prompt. |
| `preference_adhered` | False if the agent violated known guidance or needed the user to repeat it. |
| `repeated_context_questions` | Count avoidable questions asking for context Engram should have supplied. |
| `bad_memory_used` | True if the agent acted on stale, wrong-scope, or misleading memory. |
| `missing_context` | Short text naming expected context that was absent or too buried to use. |
| `used_memory_ids` | Memory IDs that materially affected the answer or implementation. |
| `stale_memory_ids` | Memory IDs surfaced but judged stale. |
| `wrong_scope_memory_ids` | Memory IDs surfaced for the wrong project/repository/task scope. |
| `note` | Include whether the agent would likely have succeeded without memory: `yes`, `no`, or `unclear`. |

---

## 7. Anti-Bias Rules

- Pre-register the expected helpful and harmful memory before seeing the arm output.
- Count retries as retries, not fresh successes.
- Keep failed runs in the report.
- Do not tune memory, ranking, or prompts between arms of the same scenario.
- Include at least one adversarial stale/wrong-scope run in every batch.
- Prefer real pending tasks over toy prompts.
- Do not let the same agent's self-report be the only evidence when user judgment is available.

---

## 8. Batch Exit Criteria

After a batch, call:

```text
telemetry(action=real_session_eval, limit=50)
```

Then choose exactly one next step.

| Evidence | Next Step |
|---|---|
| `memoryitem_orient` beats `no_memory`, `bad_memory_used_count` is zero, and `missing_context` is rare | Proceed to read-only M6 inventory/review-export planning. |
| Helpful memory exists but is not high enough in `orient` or `search` output | Ranking calibration. |
| Expected helpful memory is absent from active MemoryItems | Capture/promotion tuning or targeted review-gated observation promotion. |
| Stale or wrong-scope memory is used | Stop migration work; fix trust, scope, freshness, or ranking first. |
| Obligation scenario fails because applicable obligations are absent | Tune obligation lifecycle detection or open-obligation surfacing, not the whole `orient` hot path. |
| Results are ambiguous | Run another small labeled batch before changing architecture. |

The protocol can justify read-only M6 inventory. It cannot justify M6 write apply, deletion, broad
legacy deprecation, or automatic promotion without explicit user approval.

---

## 9. Minimum First Batch

The first batch should contain:

- at least 4 scenarios,
- at least 2 arms per scenario,
- at least 8 traces,
- feedback for every trace,
- at least 1 adversarial stale/wrong-scope scenario,
- at least 1 user-confirmed outcome if the user is available.

If time is short, run two scenarios first:

1. `resume_continuity_001`
2. `stale_scope_rejection_001`

These give the fastest signal on whether Engram improves continuity and avoids harmful memory.
