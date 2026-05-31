# Brain Harness T51 T48 Archive Packet Drift

Status: Completed read-only drift report. No lifecycle write is authorized by this
document.
Date: 2026-05-31
Scope: Re-check whether the T48 stale current-plan archive packet is still executable as
written

This report is not a replacement approval packet. It does not authorize
`memory(action="archive")`, supersession, rejection, deletion, migration action, harness write,
schema/storage/index change, public MCP change, ranking change, or `orient` payload change.

## Research Question

Is the T48 stale current-plan lifecycle approval packet still executable as written after T49 and
T50 superseded the active project-scoped current-plan memory?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The T48 packet has drifted because its hard stop conditions and archive reason depend on T47 being the active project current plan, while live state now has T50 as the active project current plan. |
| Null | T48 remains executable as written because the target stale repository-scoped current-plan MemoryItem is unchanged and still lint-flagged. |
| Simpler alternative | Keep rejecting the stale item through telemetry feedback and document the drift without creating a refreshed approval packet. |
| Failure | This report is mistaken for user approval, refreshes the archive payload implicitly, or mutates lifecycle state while investigating drift. |

## Measurement

The drift report is a pass only if read-only evidence can show all of the following:

- the T48 packet's executable preconditions name a specific active T47 project current plan;
- the T48 target MemoryItem still exists as an active repository-scoped `decision` tagged
  `current-plan`;
- the active project-scoped current-plan item is no longer the T47 item named in T48;
- the repository-scoped current-plan list still returns the target item;
- read-only lint still reports the target through `feedback_stale_current_plan` with
  `safe_action=none`;
- no lifecycle write, refreshed approval packet, migration action, harness write, ranking change,
  or `orient` payload change is made.

## Fresh Evidence

- The T48 packet hard-codes project current-plan
  `019e7d3c-afa9-7861-8569-37c2cb68a661` as the active T47 successor and its proposed archive
  reason says the target was superseded by that T47 plan with 129 recent stale-feedback records.
- T51 lean `orient` trace `019e7d50-f519-7c20-9d9a-3f59569a2b2e` returned T50 current-plan
  memory `019e7d4b-f526-7141-809d-035a7003a2ed` first and still included stale repository-scoped
  current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` in the candidate set.
- T51 direct `search` trace `019e7d50-f78f-7352-aecd-485c59259fc4` returned T50 current-plan
  memory first and stale repository-scoped current-plan memory second for the T51 current-plan
  drift query.
- `memory(action="get", id="019e5e0a-86b4-73e3-aa9b-ca350e83e915")` confirms the target remains an
  active repository-scoped `decision` tagged `current-plan`, titled
  `Current plan after Codex document lifecycle follow-through`.
- `memory(action="list", scope_type="project", project_name="engram", tags=["current-plan"],
  status_filter="active")` returned exactly one active project-scoped current-plan item:
  `019e7d4b-f526-7141-809d-035a7003a2ed`, the T50 current plan.
- `memory(action="list", scope_type="repository", local_path="/Users/yuval.meiri/projects/engram",
  tags=["current-plan"], status_filter="active")` returned exactly one active repository-scoped
  current-plan item: `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- Read-only `lint(action="run", limit=20, write=false)` reported
  `feedback_stale_current_plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 139 recent stale-feedback
  records and `safe_action=none`.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  returned `open=[]` and `warnings=[]`.
- `git status --short` showed only untracked root `AGENTS.md`, which remains user-owned and
  unstaged. Recent commits are T50 `a7126d9`, T49 `d31049e`, T48 `d07df30`, T47 `2cc5e17`, and T46
  `d459f29`.

## Consultation Evidence

AI Council T48 consensus said the archive packet was defensible only if it stayed pending,
proposed exactly one archive write, required fresh matching evidence, preserved the active T47
project current plan, and stopped on any state drift.

AI Council T51 broadcast on 2026-05-31 exposed material disagreement: Claude Opus and Gemini
recommended documenting the drift and stopping, while GPT recommended preparing a refreshed
approval packet. Claude Bridge also recommended a refreshed packet, but flagged an important safety
concern: archiving the only repository-scoped current-plan item could leave no repository-scoped
anchor unless the successor scope is handled deliberately.

Because the model consultation disagreed materially and the user confirmed the safer
recommendation, T51 uses the smaller docs-only path: record the drift, do not create a refreshed
packet, and do not mutate memory lifecycle state.

## Verdict

T48 is stale and not executable as written.

The target memory still looks like an archive candidate, but the approval packet no longer matches
live state: its active project-plan precondition names T47, its proposed archive reason names T47,
and its stale-feedback count is now behind the live lint count. Executing the T48 payload now would
violate its own stop conditions.

Do not execute the T48 `memory(action="archive", ...)` payload. T51 does not replace it, approve a
new payload, or imply permission to archive, supersede, reject, delete, scope-correct, or otherwise
mutate any MemoryItem.

## Next Action

The remaining approval gates are unchanged:

- T45 M6 inventory still requires explicit user-approved scope before even another read-only
  inventory run.
- T47 harness repair writes still require explicit approval for the exact dry-run-derived writes.
- The stale repository-scoped current-plan item remains a lifecycle review issue with
  `safe_action=none`; any refreshed archive or scope-correction packet must be a separate explicit
  approval request.
