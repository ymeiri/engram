# Brain Harness T234 Stale Migration-Completion Lifecycle Approval Packet

Date: 2026-06-04
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Ask for future exact approval to archive exactly one stale migration-completion MemoryItem:
`019dd3fe-ec94-7122-af04-1f35b839387f`.

This packet is a request for approval, not approval itself. It does not archive, supersede,
reject, review, delete, or edit any MemoryItem. It does not run `lint apply_safe`, mutate
lifecycle state, run M6/migration/quarantine actions, change search ranking or `orient`, change
public MCP/schema/storage/index/document-index behavior, run native Claude or Claude Bridge, edit
harness files/settings/hooks/adapters, change runtime configuration, or touch user-owned files.

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely ask for exact future approval to archive active MemoryItem
`019dd3fe-ec94-7122-af04-1f35b839387f` because its "migration completion" guidance is now stale
and contradicted by current M6 review-gate evidence, without performing the archive now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single-target, docs-only lifecycle packet is the smallest safe follow-up because the item remains active and retrievable, but newer M6 docs say migration is not complete and the next M6 progress requires human dispositions or explicit deferral. |
| Null | The active migration-completion item is tolerable historical context and should keep being rejected through telemetry instead of archived. |
| Simpler alternative | Defer this target and rely on current-plan/M6 gate records to outrank it. |
| Failure | The packet is mistaken for approval, archives the wrong item, treats stale feedback as proof, sweeps unrelated lifecycle debt, or bundles M6 disposition/apply work, ranking, `orient`, schema/storage/index, document-index, harness, Claude, runtime, deletion, or user-owned-file changes. |

## Measurement

This packet used read-only evidence only:

- `memory(action="get", id="019dd3fe-ec94-7122-af04-1f35b839387f")` confirmed the target remains
  `status=active`, `kind=project_fact`, project-scoped to `engram`, tagged `memory-os`,
  `migration`, `review-gated`, `digest`, `vault`, `lint`, and `completion`, titled
  `Memory OS migration completion run finished`, and last updated at
  `2026-04-28T12:09:52.532338Z`.
- The target content claims the Memory OS migration completion run finished on 2026-04-28 and that
  legacy migration inventory was exhausted after digest review, source indexing, vault compile, and
  safe lint actions.
- Direct memory search trace `019e91e8-32ed-7d53-89ba-c243daee4af2` returned the active target in
  the top eight for `Memory OS migration completion run finished stale active M6 migration
  completion review-gated`, alongside active M6 gate memory `019dd35d-1a48-7103-b0e2-390225f8b418`
  and reviewed migration-gate decision `019dc9ce-3b4e-7b02-80b5-04f56c84624e`.
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` T213 records the current M6 state: candidate inspection
  is complete for 0001-0011, 0012 is count-drift provenance requiring explicit-scope handling, all
  12 generated snapshot files remain undecided by read-only status, and next M6 progress requires
  human-provided dispositions or explicit deferral.
- `docs/BRAIN_HARNESS_T210_M6_CANDIDATE_DISPOSITION_AUTHORIZATION_PACKET_2026-06-04.md` defines the
  next M6 gate as explicit human-disposition recording only, not migration apply, prioritize,
  review-export rerun, active MemoryItem writes, lifecycle cleanup, or legacy simplification.
- Historical feedback docs already flagged the target as stale retrieval noise:
  `docs/BRAIN_HARNESS_LIVE_FEEDBACK_BATCH_2026-05-27.md` says implementation-plan and risk searches
  surfaced the target and feedback marked it stale rather than using it, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` T11 says the target was visible through generic
  `feedback_stale_active_memory` lint with `safe_action=none`.
- Fresh `lint(action="run", limit=120, write=false)` did not return this target in the first 120
  findings. It did return unrelated wrong-scope/stale/superseded lifecycle debt. This non-confirming
  lint result means the future archive, if approved, must be a human-approved manual archive, not a
  lint safe action.
- `graph(action="around", node="019dd3fe-ec94-7122-af04-1f35b839387f", depth=1)` showed the target
  is project-scoped to `engram`, was added by commit `019dd3ff-0dc7-7dc1-a7c6-4a2229761e49`, and
  has only tool-call evidence edges. It showed no direct dependent MemoryItem.
- Source inspection confirmed:
  - `engram-core/src/memory.rs` `with_archive` sets `status=Archived`, records archive metadata,
    and updates `updated_at`;
  - `engram-index/src/memory.rs` `archive_memory` loads exactly one requested item, applies
    `with_archive`, and saves that item;
  - `engram-index/src/memory_ranker.rs` assigns archived memory status score `0.0`;
  - `engram-index/src/lint.rs` `apply_safe` can archive every matching
    `ArchiveMemoryItem` safe-action finding in its report, so it remains too broad for this target.
- AI Council recall recovered prior default-deny lifecycle guidance: docs-only single-target
  packets are acceptable, but archive writes, `lint apply_safe`, broad cleanup, ranking/`orient`,
  M6, harness, Claude, schema/storage/index, public MCP, and document-index changes remain
  exact-gated.
- Git status showed only the known user-owned untracked root `AGENTS.md`.
- `obligations(action="doctor", project="engram")` returned no open obligations or warnings.

## Completion Matrix Delta

| Area | State After T234 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Stale migration-completion memory | Exact future archive target identified | `memory(get)`, direct search, historical stale feedback docs, current M6 gate docs, graph check | Requires exact T234 approval before archive |
| Current M6 state | Not complete; review/disposition gate remains | T209/T210/T213 docs and implementation-plan note | Requires human dispositions or explicit deferral before M6 progress |
| Lint automation | Not applicable to this target | Fresh lint did not return the target; historical lint had `safe_action=none` | Archive must be human-approved, not `lint apply_safe` |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Do not change ranking or `orient` from this packet |
| Lifecycle cleanup | Still target-gated | No `memory(action="archive")`; no `lint(action="apply_safe")` | Separate exact approval for each lifecycle write |
| Runtime refresh | Still T233-gated | T233 remains docs-only | Exact T233 approval required before install/restart/live validation |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019dd3fe-ec94-7122-af04-1f35b839387f",
  archive_reason="Stale migration-completion project_fact contradicted by current M6 evidence: T209/T213 show the T68 snapshot is still undecided with ready_to_apply=false, and T210 defines the next progress as human-disposition recording or explicit deferral rather than completed migration. The target remains active and retrievable for migration-completion/M6 queries; historical feedback marked it stale with safe_action=none, while fresh lint did not return it, so this is a human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `project_fact`, scope is project `engram`, tags still include `memory-os`, `migration`, `review-gated`, and `completion`, and `updated_at` is not later than `2026-04-28T12:09:52.532338Z` unless the user re-approves after seeing the drift. |
| Current-plan orient or direct search | Current Engram project guidance remains recoverable before the stale item is archived. |
| M6 evidence check | Current repo docs or fresher read-only evidence still show M6 migration is not complete and remains gated by human dispositions, explicit deferral, or a separate apply approval. |
| Target visibility check | Exact or related search still shows the target as active migration-completion guidance, or the target is otherwise shown to be active stale migration-completion memory through `memory(get)` plus current M6 contradiction evidence. |
| `lint(action="run", write=false)` | The result is read and recorded. The target may or may not be flagged; either way this remains human-approved, not automatic. |
| `graph(action="around", node=..., depth=1)` | No direct dependent MemoryItem appears. Existing evidence, project, and commit edges are acceptable. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T234 does not authorize:

- archiving, superseding, rejecting, reviewing, editing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- running M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion,
  quarantine inspection, candidate decisions, human-disposition recording, or legacy
  simplification;
- running native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive
  Claude commands;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, runtime refresh,
  rollback, force-kill, deletion, or old-binary reinstall;
- editing root `AGENTS.md` or other user-owned files.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T234 wording and target
  ID;
- the target UUID, title, kind, scope, status, tags, or archive payload differs from this packet;
- the target `updated_at` is later than `2026-04-28T12:09:52.532338Z` and the user has not
  re-approved after seeing the fresh item contents;
- the target is already archived, superseded, rejected, deleted, or missing;
- active current-plan guidance for Engram cannot be identified before the archive;
- current M6 evidence no longer contradicts the target because the migration has actually completed
  or been explicitly deferred with user-approved evidence;
- fresh graph depth 1 shows a direct MemoryItem dependency on the target;
- any write occurs after the final fresh pre-write read and before the archive;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, executing Claude, executing harness writes, or changing runtime configuration.

## Approval Wording

To authorize only the single archive action above, reply exactly:

```text
Approve T234: after fresh matching read-only get/orient-or-search/M6-evidence/target-visibility/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019dd3fe-ec94-7122-af04-1f35b839387f with the archive payload in docs/BRAIN_HARNESS_T234_STALE_MIGRATION_COMPLETION_LIFECYCLE_APPROVAL_PACKET_2026-06-04.md. Do not run lint apply_safe, archive any other memory, change ranking, orient, public MCP, schema/storage/index/document-index behavior, run M6/migration/quarantine actions, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, runtime refresh, deletion, rollback, old-binary reinstall, or touch user-owned files.
```

Any other reply should be treated as non-authorization for T234.
