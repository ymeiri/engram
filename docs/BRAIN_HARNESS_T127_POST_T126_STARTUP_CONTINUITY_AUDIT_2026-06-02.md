# Brain Harness T127 Post-T126 Startup Continuity Audit

Date: 2026-06-02
Status: Completed
Scope: Read-only startup continuity and retrieval-quality audit

T127 checked whether a fresh Codex startup path after T126 can recover the active plan, handoff,
and approval gates without relying on hidden session context. It did not change ranking, `orient`,
document indexing, lifecycle state, migration state, public MCP parameters, schema/storage/index
behavior, harness adapters, hooks, or user-owned files.

## Research Question

After T126, does read-only startup retrieval still provide enough current-plan continuity to choose
the next action safely, and what retrieval or documentation gaps remain before gated work resumes?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Lean `orient`, direct search, `memory(list)`, and `handoff(get)` recover the T126 current plan and handoff, while stale lower-ranked memory and unindexed fresh docs remain evidence-quality gaps. |
| Null | Current-plan or handoff retrieval fails, making the next action ambiguous or unsafe without user input. |
| Simpler alternative | Rely on the T126 current-plan write and skip another startup audit. |
| Failure | The audit crosses into T125 quarantine inspection, M6 status/prioritize/apply, lifecycle mutation, document indexing, ranking changes, `orient` expansion, public MCP/schema/storage/index changes, or harness writes. |

## Measurement

Read-only evidence collected on 2026-06-02:

- Lean `orient` for the post-T126 continuation prompt returned current-plan memory
  `019e877e-dc11-7b90-b5ec-7bca7720a9f4` first in trace
  `019e8782-4431-7f90-908e-7ec458d7e863`.
- Direct search for `current plan after T126 harness readiness recheck next gate T125` returned
  current-plan memory `019e877e-dc11-7b90-b5ec-7bca7720a9f4` first and the T126 rolling handoff
  `019e877f-039f-71b3-b891-7f21de8f6ca6` second in trace
  `019e8782-45d3-7642-9a90-f74641ddf7db`.
- Direct search for `what should happen next after T126 Engram Brain Harness` returned the T126
  current plan first in trace `019e8783-7454-7ba1-a67e-bd77515a018b`, but also returned older
  active handoffs and stale repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` in lower-ranked results.
- Direct search for the T125 quarantine-inspection phrasing returned the T126 handoff first and
  the T126 current plan fourth in trace `019e8783-762d-7460-91d4-3eff63d4a892`; older handoffs
  ranked above the current plan, so exact approval-gate lookup remains noisy.
- `memory(action="list", project_name="engram", scope_type="project", tags=["current-plan"],
  status_filter="active")` returned exactly one active project current-plan item:
  `019e877e-dc11-7b90-b5ec-7bca7720a9f4`.
- `handoff(action="get", project="engram")` returned the T126 handoff
  `019e877f-039f-71b3-b891-7f21de8f6ca6`.
- `docs(action="search", query="Brain Harness T126 Harness Readiness Recheck", limit=5)` did not
  return the T126 report in the top five results, so fresh report visibility remains dependent on
  explicit document indexing or reading repo docs directly.
- `lint(action="run", write=false, limit=20)` still reported stale/wrong-scope feedback on
  repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, including 232
  stale-feedback records and 10 wrong-scope records, with `safe_action=none`.
- `telemetry(action="real_session_eval", project="engram", limit=50)` returned
  `feedback_coverage=0.38`, `confidence_gate.passed=false`, `bad_memory_used_count=0`, and
  `external_session_trace_count=0`.
- `obligations(action="doctor", project="engram")` returned no open obligations, and
  `git status --short` showed only the user-owned untracked root `AGENTS.md` before writing this
  report.

## Result

Startup continuity is sufficient for a careful agent to continue without guessing: the active
project current plan is unique, `orient` and direct continuation search recover it first, and the
latest handoff preserves both T125 and T47 gates.

The result is not a broad retrieval-quality pass. Older active handoffs still appear prominently
for exact T125 wording, and the stale repository-scoped current-plan item remains lower-ranked
noise in broad next-step searches. The standalone M6 gate did not need to rank in the top results
because the current plan and handoff both carry the exact T125 gate, but this should not be treated
as approval-audit coverage for every prompt class.

The T126 report is not visible in top-five document search for its title. That is a document
visibility gap, not implicit approval to index documents. Repo docs remain authoritative until an
exact indexing gate is approved.

## Completion Matrix Delta

| Area | T127 state | Evidence |
| --- | --- | --- |
| Current-plan continuity | Healthy for the tested startup prompts | Lean `orient` and direct continuation search returned current-plan memory `019e877e-dc11-7b90-b5ec-7bca7720a9f4` first. |
| Approval-gate recovery | Recoverable but noisy | T125 is present through the current plan and T126 handoff; exact T125 search ranked older handoffs above the current plan. |
| Document visibility | Partial | T126 report was not found in top-five document search by title. |
| Evidence loop | Still partial | Rolling `real_session_eval` failed the confidence gate at 38% feedback coverage. |
| Lifecycle quality | Still gated | Lint reports stale/wrong-scope feedback for `019e5e0a-86b4-73e3-aa9b-ca350e83e915`; no automatic lifecycle action is safe. |

## Validation

This is a docs-only evidence slice. Validation is limited to:

- read-only Engram MCP evidence from `orient`, `search`, `memory(list)`, `handoff(get)`,
  `docs(search)`, `lint(run)`, `telemetry(real_session_eval)`, and `obligations(doctor)`;
- exact-source documentation updates in the Brain Harness architecture, research method, and Memory
  OS implementation plan;
- `git diff --check` before commit.

## Next Gate

The next M6 gate remains exact approval for T125:

`Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.`

Harness repair remains a separate T47 approval packet. Generic continuation language does not
authorize document indexing, lifecycle mutation, M6 decisions, migration apply, ranking changes,
`orient` expansion, public MCP/schema/storage/index changes, or harness writes.
