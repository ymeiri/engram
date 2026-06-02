# Brain Harness T139 Stale Current-Plan Lifecycle Approval Packet

Status: Pending user approval. No memory lifecycle write is authorized by this document.
Date: 2026-06-02
Scope: Proposal for one exact archive action on stale repository-scoped current-plan guidance

This packet is a request for approval, not approval itself. No `memory(action="archive")`,
supersede, rejection, deletion, migration action, harness write, schema/storage/index change,
document-index behavior change, public MCP change, ranking change, or `orient` change has been run
for T139.

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely ask for explicit approval to archive one stale repository-scoped current-plan
MemoryItem, using fresh read-only evidence and a default-deny single-item lifecycle scope, without
changing ranking or the `orient` hot path?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single-item archive approval packet can remove known stale current-plan guidance from normal active-memory retrieval after exact user approval, while preserving the active T138 project-scoped current plan and avoiding broad lifecycle cleanup. |
| Null | The stale repository-scoped item is harmless lower-ranked noise, so continuing to reject it via telemetry is enough. |
| Simpler alternative | Leave lifecycle state unchanged and keep documenting the stale/wrong-scope feedback signal. |
| Failure | The packet implies approval, archives the wrong item, treats lint feedback as proof, hides needed context, or bundles ranking, `orient`, M6, harness, schema/storage/index, document-index, or other lifecycle work. |

## Current Evidence

- Lean `orient` trace `019e8860-97cb-7782-b2db-ec171f0e2a37` returned the current T138
  project-scoped plan `019e885c-abde-7811-9314-7654bb6667a9` first and the stale repository-scoped
  item `019e5e0a-86b4-73e3-aa9b-ca350e83e915` in the top five.
- Direct current-plan search trace `019e8860-b70a-73d3-8210-4781a80cca19` returned the T138 plan
  first and the stale item fourth.
- Direct stale/current-plan lifecycle search trace `019e8860-b73f-7ff2-9978-00371411c574`
  returned the T138 plan first and older handoff/current-plan noise behind it.
- `memory(action="get", id="019e5e0a-86b4-73e3-aa9b-ca350e83e915")` confirms the target is an
  active repository-scoped `decision` tagged `current-plan`, titled
  `Current plan after Codex document lifecycle follow-through`, scoped to
  `/Users/yuval.meiri/projects/engram`, created on `2026-05-25`, and last updated on
  `2026-05-25T07:30:08.716259Z`.
- `memory(action="list", tags=["current-plan"], status_filter="active", project_name="engram")`
  returned three active current-plan items: the current T138 Engram project plan
  `019e885c-abde-7811-9314-7654bb6667a9`, the stale repository-scoped target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, and an unrelated `voice-layer` project item returned by
  the broad list behavior.
- Read-only `lint(action="run", limit=40)` reported
  `feedback_stale_current_plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 207 recent stale
  feedback records and `safe_action=none`.
- The same read-only lint run reported
  `feedback_wrong_scope_active_memory:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 14 recent
  wrong-scope feedback records and `safe_action=none`.
- Source inspection of `engram-index/src/lint.rs` confirms the stale-current-plan lint message is
  intentionally a review signal: recent feedback is not proof and no automatic lifecycle action is
  safe.
- Direct `graph(action="around", node="019e5e0a-86b4-73e3-aa9b-ca350e83e915", depth=1)` showed
  the target's evidence links, repository scope, the older item it supersedes
  `019e59f2-524d-76f0-929a-7d2be0cea901`, and the capture commit. It did not show another
  MemoryItem directly depending on the target.
- AI Council recall found the prior T48 decision boundary: a stale-current-plan archive packet is
  valid only as pending/default-deny, with exactly one archive write, fresh pre-write evidence,
  current-plan preservation, and stop conditions for drift or bundling.
- Fresh AI Council broadcast agreed the packet is narrow if it pins UUID/title/scope, re-checks
  immediately before execution, and stops on drift. One model requested the direct graph/dependency
  check above because lint uses `safe_action=none`.
- Claude Bridge read-only critique agreed the direction is narrow, and requested explicit drift
  definitions, approval expiry, an active-state precondition, and clear wording that this is a
  human-authorized exception to lint's default-deny signal.

## Proposed Approval

If the user explicitly approves this packet, the authorized lifecycle write is exactly:

```text
memory(
  action="archive",
  id="019e5e0a-86b4-73e3-aa9b-ca350e83e915",
  archive_reason="Stale repository-scoped current-plan guidance superseded by active T138 project-scoped current plan 019e885c-abde-7811-9314-7654bb6667a9; read-only lint reported feedback_stale_current_plan with 207 recent stale-feedback records and feedback_wrong_scope_active_memory with 14 recent wrong-scope records, both with safe_action=none.",
  archived_by="codex"
)
```

Any missing, conditional, partial, or ambiguous approval remains default-deny.

This is a human approval exception to lint's `safe_action=none`, not a lint-approved safe action.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `decision`, scope is repository `/Users/yuval.meiri/projects/engram`, tags still include `current-plan`, and `updated_at` is not later than this packet's target state unless the user re-approves after seeing the drift. |
| `memory(action="list", tags=["current-plan"], status_filter="active", project_name="engram")` | The T138 project-scoped current plan `019e885c-abde-7811-9314-7654bb6667a9` remains active and present. Any extra item that changes the single-target analysis stops execution. |
| Lean `orient` or direct current-plan search | Current T138 guidance remains recoverable before the stale item is archived. Ranking is evidence only, not proof. |
| `lint(action="run", write=false)` | The target is still reported as stale or wrong-scope active memory with `safe_action=none`. |
| `graph(action="around", node=..., depth=1)` | No new direct MemoryItem dependency on the target appears. Existing evidence, scope, and supersedes edges are acceptable. |
| `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## In Scope After Explicit Approval

| Item | Allowed after explicit approval? | Notes |
| --- | --- | --- |
| Fresh read-only get/list/orient/search/lint/graph checks for this target | Yes | Required preconditions above must match. |
| The single `memory(action="archive", ...)` call above | Yes | Only if the fresh pre-write checks match and no intervening writes occur. |
| Post-write read-only validation | Yes | Confirm target is archived, T138 remains active, and the stale-current-plan lint finding for the target no longer appears as active memory. |
| Markdown report, telemetry feedback, obligation cleanup, handoff update, and documentation commit | Yes | Evidence annotation only; no extra lifecycle mutation. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| Archiving, superseding, rejecting, editing, reviewing, or deleting any other MemoryItem | No |
| Creating a replacement MemoryItem | No |
| Running `lint(action="apply_safe", write=true)` or any automatic lifecycle cleanup | No |
| Changing `handoff(update)` semantics or bulk-archiving old handoffs | No |
| M6 migration inventory, review export, status, prioritize, apply, deletion, cleanup, or legacy simplification | No |
| Inspecting M6 quarantine candidates | No |
| Harness install, adapter/settings/hook writes, `adopt_user_owned=true`, or installed user hook/settings edits | No |
| Schema, storage, index, document-index behavior, public MCP, ranking, `orient`, graph, lint rule, telemetry formula, or search changes | No |

## Stop Conditions

Stop and ask the user again before writing anything if:

- approval is missing, conditional, ambiguous, or changes the allowed scope;
- the target UUID, title, kind, scope, status, `current-plan` tag, or archive payload differs from
  this packet;
- `updated_at` is later than `2026-05-25T07:30:08.716259Z` and the user has not re-approved after
  seeing the fresh item contents;
- the target is already archived, superseded, rejected, deleted, or missing;
- the T138 project-scoped current plan `019e885c-abde-7811-9314-7654bb6667a9` is missing or
  inactive;
- fresh lint no longer reports the target as stale or wrong-scope active memory with
  `safe_action=none`;
- fresh direct graph depth 1 shows a new MemoryItem directly depending on the target;
- any write occurs after the final fresh pre-write read and before the archive;
- the user approval sounds like deletion rather than archive;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, or executing harness writes;
- post-write validation cannot prove the target was archived and the T138 current plan remains
  active.

## Approval Wording

Exact approval should name T139 and the target UUID. A safe approval phrase is:

```text
Approve T139: after fresh matching read-only get/list/orient-or-search/lint/graph evidence and no
intervening writes, archive exactly MemoryItem 019e5e0a-86b4-73e3-aa9b-ca350e83e915 with the
archive payload in docs/BRAIN_HARNESS_T139_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-02.md.
Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient,
public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, harness
installs/settings/hooks/adapters, or user-owned files.
```
