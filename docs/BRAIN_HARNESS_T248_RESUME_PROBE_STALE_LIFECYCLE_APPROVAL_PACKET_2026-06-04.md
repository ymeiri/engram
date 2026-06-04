# Brain Harness T248 Resume-Probe Stale Lifecycle Approval Packet

Date: 2026-06-04
Status: Pending exact user approval. No memory lifecycle write is authorized by this document.
Scope: Proposal for one exact archive action on stale project-scoped resume-continuity probe
decision `019e01f2-0a87-7f73-9b0b-7f2443eac7bb`.

This packet is a request for explicit approval, not approval itself. No `memory(action="archive")`,
`lint apply_safe`, supersession, rejection, deletion, migration action, M6/quarantine action,
harness write, native Claude action, schema/storage/index change, document-index behavior change,
public MCP change, ranking change, `orient` change, rollback, force-kill, legacy simplification,
or user-owned-file change has been run for T248.

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely ask for exact approval to archive one active project-scoped resume-continuity
probe decision whose next-action guidance was valid on 2026-05-07 but is now stale relative to
later current-plan retrieval fixes and current T247 plan state, without treating age or feedback
alone as proof and without mutating lifecycle state?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A default-deny, single-target packet is the smallest safe lifecycle follow-up for the next unpacketized non-M6 stale-feedback candidate in the bounded sample. |
| Null | The old resume-continuity probe decision remains harmless historical context and should stay active while agents reject it when noisy. |
| Simpler alternative | Record only an inventory note that no new lifecycle packet is needed beyond T234 and T247. |
| Failure | The packet treats the item as stale solely because it is old, duplicates an existing packet, archives without exact approval, or bundles lifecycle cleanup, `lint apply_safe`, M6, harness, ranking, `orient`, schema/storage/index, document-index, native Claude, or user-owned-file work. |

## Measurement

This packet used read-only evidence only:

- Lean post-T247 `orient` trace `019e9277-31d8-72c0-9c47-371aad4cfcad` returned current-plan
  MemoryItem `019e9277-0ddd-7da1-b89c-9a2cd04b066f` first, along with the research-method rule,
  M6 approval gate, and commit preference.
- Bounded T248 `memory(action="list", project_name="engram", status_filter="active", limit=100)`
  showed a truncated active project sample. This is not a complete inventory. It surfaced the
  already-packeted T247 target `019e8291-40aa-71a0-b16b-9ba7b6446cc6` and other historical
  project memories, so T248 explicitly excludes already-packeted targets from new-candidate
  selection.
- Fresh `lint(action="run", write=false, limit=120)` reported generic stale-feedback findings
  with `safe_action=none`, including:
  `feedback-stale-active-memory:019e01f2-0a87-7f73-9b0b-7f2443eac7bb`, with the message that
  `Resume continuity probe uses active MemoryItems before ranking changes` had three recent stale
  feedback records. This is review signal only, not proof and not automatic approval.
- The same lint run also reported already-packeted stale active memory
  `019dd3fe-ec94-7122-af04-1f35b839387f` and
  `019e8291-40aa-71a0-b16b-9ba7b6446cc6`. T234 and T247 already cover those exact targets, so
  T248 does not create duplicate packets for them.
- `memory(action="get", id="019e01f2-0a87-7f73-9b0b-7f2443eac7bb")` confirmed the target is
  `status=active`, kind `decision`, scope `project:engram`, title
  `Resume continuity probe uses active MemoryItems before ranking changes`, tags
  `brain-harness`, `resume-continuity`, `memoryitem-probe`, and `orient`, and
  `updated_at=2026-05-07T10:18:20.167098Z`.
- The target content records a valid 2026-05-07 next action: after document indexing, `orient`
  still missed current research/dogfood plan context, so the slice added minimal active
  MemoryItems and deferred stale cleanup, ranking changes, graph traversal, and obligation
  hot-path work until the probe result was known.
- `docs/BRAIN_HARNESS_DOGFOOD_RUN_2026-05-07.md` confirms the item was useful at creation time:
  Stage 2 `orient` trace `019e01f2-24f0-72e3-ac95-67f1dfb5ef3b` passed after this decision and
  the research-method rule were added.
- The same dogfood report later records `019e01f2...` as stale/noisy context in a bounded
  autonomous follow-through treatment arm; it was surfaced but not acted on.
- `docs/BRAIN_HARNESS_CROSS_HARNESS_RUN_2026-05-11.md` recorded the item as directionally
  consistent historical guard context for one Claude smoke, not as current next-plan authority.
- Later current-plan retrieval evidence supersedes the target's next-action guidance. MemoryItem
  `019e6858-c2e8-7590-a13b-01f45cbe04db` records that the mission-class PlanWork current-plan gap
  was fixed by commit `0b4e35bc34de01073e7f7930dc6102f22db3d337`; MemoryItem
  `019e68e9-2842-76f2-8367-cf159246ce3c` records current-plan lifecycle predicate parity; and the
  current active plan is T247, not the May 7 probe.
- Direct memory search trace `019e927d-54a8-7182-83b5-650309981258` returned the active T247
  current plan first and the target second for an exact stale resume-continuity query, showing the
  target remains visible as active historical guidance.
- `graph(action="around", node="019e01f2-0a87-7f73-9b0b-7f2443eac7bb", depth=1)` showed the target
  is scoped to `project:engram`, was added by commit `019e01f3-085b-7031-9124-d0860267d16c`, has
  file/tool-call evidence edges, and has no direct supersedes or dependent MemoryItem edge at
  depth 1.
- Source inspection confirmed `MemoryItem::with_archive` sets status to `Archived`, records archive
  metadata, and updates `updated_at`; `MemoryService::archive_memory` loads exactly one requested
  item and saves that archived item; `lint apply_safe` can archive every `ArchiveMemoryItem`
  safe-action finding in the report; and feedback-stale active-memory findings have
  `safe_action=none`.
- AI Council recall found prior lifecycle/default-deny guidance. A T248 broadcast to
  `claude-sonnet-4.6`, `gpt-5.4`, and `gemini-3.1-pro` warned to treat zero-candidate as valid,
  exclude already-packeted T247/T234 targets, avoid M6 contamination, use lint only as supporting
  evidence, document bounded-sample limits, and avoid disposition language.
- Claude Bridge read-only critique attempts timed out twice. The second timed-out job was
  `ccb_20260604115306_3fb1a192` and had no result file. T248 therefore treats Claude Bridge as
  attempted but unavailable evidence, not consensus.

## Completion Matrix Delta

| Area | State After T248 Packet | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| Target item `019e01f2...` | Exact future archive candidate documented | Fresh get/search/graph/lint, dogfood/cross-harness docs, later current-plan fixes | Requires exact user approval and fresh pre-write checks |
| Target technical state | Active project-scoped decision, not graph-superseded | `memory(get)`, graph depth 1 | Archive only by direct exact `memory.archive`, not `apply_safe` |
| Bounded lifecycle sample | Found already-packeted T234/T247 targets plus one new unpacketized non-M6 candidate | Active list, lint, searches | Not an exhaustive lifecycle inventory |
| Lifecycle cleanup | Still incomplete | No archive and no `apply_safe` ran | Exact lifecycle write approval remains required |
| M6/cross-harness/hot path | Unchanged | No migration, harness, ranking, or `orient` change | Separate gates remain |

## Proposed Approved Archive

If and only if the user approves with the exact T248 approval wording below, Codex may run one
Memory OS archive write for this single ID:

```text
memory(
  action="archive",
  id="019e01f2-0a87-7f73-9b0b-7f2443eac7bb",
  archive_reason="Content-stale project-scoped resume-continuity probe decision: MemoryItem 019e01f2-0a87-7f73-9b0b-7f2443eac7bb accurately recorded a 2026-05-07 next action to add active MemoryItems and test whether they fixed orient resume-continuity, but that probe subsequently passed, later Brain Harness work fixed the relevant current-plan retrieval classes, and current active plan guidance is T247. Fresh lint reported three recent stale-feedback records with safe_action=none, direct search still returns the item as active historical guidance, and graph depth 1 shows no direct dependent MemoryItem. This is a direct exact-target archive, not lint apply_safe and not migration, ranking, or orient work.",
  archived_by="codex"
)
```

This is a human approval exception to feedback-stale lint's `safe_action=none`, not a lint-approved
safe action. `lint apply_safe` is out of scope.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening Engram memory writes between the final read-only check and archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `decision`, scope is `project:engram`, tags still include `brain-harness`, `resume-continuity`, `memoryitem-probe`, and `orient`, and `updated_at` is not later than `2026-05-07T10:18:20.167098Z` unless the user re-approves after seeing the drift. |
| Lean `orient` or direct current-plan search scoped to `project=engram` | Current Engram plan remains recoverable and outranks the May 7 probe as current next-action guidance. |
| Target visibility check | Exact or related search still shows the target as active resume-continuity probe guidance, or `memory(get)` plus current docs still establish it as active stale historical guidance. |
| Current docs check | Later current-plan/retrieval docs still show the May 7 probe is no longer the current action path, and no newer doc revalidates it as current next-plan authority. |
| `graph(action="around", node=..., depth=1)` | Target remains project-scoped; no new direct MemoryItem dependency or replacement relation appears. |
| `lint(action="run", write=false)` | If checked, any feedback-stale finding remains review evidence only with `safe_action=none`; sampled global lint absence is not a failure. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T248 does not authorize:

- archiving, superseding, rejecting, editing, reviewing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- creating replacement resume-continuity memory;
- changing handoff semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine,
  candidate decisions, human-disposition recording, or legacy simplification;
- native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive Claude
  commands;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, runtime refresh,
  rollback, force-kill, deletion, or old-binary reinstall;
- editing root `AGENTS.md`.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T248 wording and target
  ID;
- the target UUID, title, kind, scope, status, tags, `updated_at`, or archive payload differs from
  this packet;
- the target is already archived, superseded, rejected, deleted, or missing;
- fresh evidence suggests the May 7 probe remains useful current next-action guidance rather than
  historical probe context;
- fresh graph depth 1 shows a new direct MemoryItem dependency or replacement relation for the
  target;
- fresh feedback/lint/docs evidence is inconclusive about the target's current staleness;
- any Engram memory write occurs after the final fresh pre-write read and before the archive;
- any step appears to require applying automatic lint cleanup, mutating other memories, changing
  ranking, changing `orient`, running M6, inspecting quarantine candidates, executing Claude, or
  executing harness writes.

## Approval Wording

To authorize only the single archive action above, reply exactly:

```text
Approve T248: after fresh matching read-only get/orient-or-search/target-visibility/current-docs/graph/lint/git/obligations evidence and no intervening Engram memory writes, archive exactly MemoryItem 019e01f2-0a87-7f73-9b0b-7f2443eac7bb with the archive payload in docs/BRAIN_HARNESS_T248_RESUME_PROBE_STALE_LIFECYCLE_APPROVAL_PACKET_2026-06-04.md. Do not run lint apply_safe, archive any other memory, create replacement memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, runtime configuration, rollback, force-kill, deletion, old-binary reinstall, or user-owned files.
```

Any other reply should be treated as non-authorization for T248.
