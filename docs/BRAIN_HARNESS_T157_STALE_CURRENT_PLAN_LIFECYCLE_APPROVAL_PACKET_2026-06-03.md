# Brain Harness T157 Stale Current-Plan Lifecycle Approval Packet

Date: 2026-06-03
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Refresh the stale current-plan archive request for exactly one MemoryItem:
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`.

This packet supersedes the stale T139 approval packet shape for this target because the active
current plan is now T156, not T138, and the lint feedback counts have changed. It does not archive
anything by itself.

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely refresh the exact approval request to archive the stale repository-scoped
current-plan MemoryItem `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, using current T156 evidence and no
lifecycle write?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A refreshed, single-target archive packet is the smallest safe lifecycle follow-up: it preserves the active T156 project-scoped current plan, avoids broad cleanup, and gives the user an exact approval phrase if they want to remove stale current-plan noise from active retrieval. |
| Null | The stale repository-scoped item is tolerable lower-rank noise and should continue to be rejected through telemetry without lifecycle mutation. |
| Simpler alternative | Keep the old T139 packet and do no refresh. |
| Failure | The packet is mistaken for approval, archives the wrong item, treats lint feedback as proof, sweeps old handoffs, or bundles ranking, `orient`, M6, harness, schema/storage/index, document-index, Claude, or other lifecycle work. |

## Measurement

This packet used read-only evidence only:

- Lean `orient` trace `019e8d07-ed4b-7352-aff6-bed518ffa78d` returned the T156 current-plan
  memory `019e8d05-dce0-7a82-9b23-30ce1405b5bd` first and the stale repository-scoped target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower in the candidate set.
- Direct current-plan search trace `019e8d08-87ac-7be1-adc6-e1b045f177a6` returned the T156
  current plan first and the stale target second.
- Direct architecture/risk searches `019e8d08-88a3-7180-84c8-130eaef7655f` and
  `019e8d08-8a65-7583-8e2b-062241084ccd` were dominated by old active handoffs, confirming
  broader lifecycle noise while not authorizing a sweep.
- Direct lifecycle-target search trace `019e8d09-998e-7d33-9b9e-67b8af1ccefd` returned the T156
  current plan first, then lifecycle/current-plan context and old handoffs.
- `memory(action="get", id="019e5e0a-86b4-73e3-aa9b-ca350e83e915")` confirmed the target remains
  `status=active`, `kind=decision`, repository-scoped to `/Users/yuval.meiri/projects/engram`,
  tagged `current-plan`, titled `Current plan after Codex document lifecycle follow-through`, and
  last updated at `2026-05-25T07:30:08.716259Z`.
- `memory(action="get", id="019e8d05-dce0-7a82-9b23-30ce1405b5bd")` confirmed the active T156
  project-scoped current plan remains active and points to exact T154 approval as the next gate.
- `memory(action="list", project_name="engram", tags=["current-plan"], status_filter="active")`
  returned the active T156 project plan, the stale repository-scoped target, and one unrelated
  `voice-layer` project item from broad tag listing behavior.
- `lint(action="run", limit=50, write=false)` reported:
  - `feedback_stale_current_plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 198 recent stale
    feedback records and `safe_action=none`;
  - `feedback_wrong_scope_active_memory:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 23 recent
    wrong-scope feedback records and `safe_action=none`;
  - many `superseded_item_still_active` findings with `safe_action=archive_memory_item`, which are
    out of scope for this packet.
- `graph(action="around", node="019e5e0a-86b4-73e3-aa9b-ca350e83e915", depth=1)` showed the
  target's evidence, repository scope, capture commit, and its edge superseding older current-plan
  item `019e59f2-524d-76f0-929a-7d2be0cea901`; it did not show a newer direct dependent
  MemoryItem.
- Source inspection confirmed:
  - `engram-core/src/memory.rs` `with_archive` sets status to `Archived`, records archive
    metadata, and updates `updated_at`;
  - `engram-index/src/memory.rs` `archive_memory` loads exactly one requested item, applies
    `with_archive`, and saves that item;
  - `engram-index/src/lint.rs` `apply_safe` can archive every matching safe-action finding in its
    report, so it is too broad for this packet;
  - `engram-index/src/memory_ranker.rs` gives `Archived` memory status score `0.0`, so archived
    items stop behaving as active guidance.
- AI Council recall found the T139 and T136 lifecycle guidance: docs-only read-only packets are
  acceptable, but lifecycle archive/apply, broad ranking, `orient`, schema/storage/index,
  document-index, M6, or harness/settings writes remain exact-gated.
- Git status before this packet showed only the user-owned untracked root `AGENTS.md`.

## Completion Matrix Delta

| Area | State After T157 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | T156 is active and first for the current continuation prompt class | `orient`, direct search, and `memory(get)` for `019e8d05...` | T154 native Claude smoke remains exact-gated |
| Stale repository-scoped current-plan target | Exact future archive target refreshed | `memory(get)`, lint stale/wrong-scope feedback, direct searches | Requires exact T157 approval before archive |
| Old active handoff noise | Real but not this target | Direct architecture/risk/lifecycle searches return old active handoffs | Needs separate target-local packets or explicit safe-action policy |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")`; lint applied zero safe actions | Archive/write approval required target by target |
| M6 migration | Still gated | No M6/migration/quarantine action ran | Separate reviewed-candidate and dry-run/apply approval |
| Hot path and ranking | Unchanged | No source/runtime change | Do not change ranking or `orient` from this packet |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e5e0a-86b4-73e3-aa9b-ca350e83e915",
  archive_reason="Stale repository-scoped current-plan guidance superseded by active T156 project-scoped current plan 019e8d05-dce0-7a82-9b23-30ce1405b5bd; read-only lint reported feedback_stale_current_plan with 198 recent stale-feedback records and feedback_wrong_scope_active_memory with 23 recent wrong-scope records, both with safe_action=none.",
  archived_by="codex"
)
```

This is a human approval exception to lint's `safe_action=none`, not a lint-approved safe action.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `decision`, scope is repository `/Users/yuval.meiri/projects/engram`, tags still include `current-plan`, and `updated_at` is not later than `2026-05-25T07:30:08.716259Z` unless the user re-approves after seeing the drift. |
| `memory(action="get", id="019e8d05-dce0-7a82-9b23-30ce1405b5bd")` | T156 project-scoped current plan remains active, or a newer current-plan item is clearly active and the user re-approves after seeing the replacement. |
| Lean `orient` or direct current-plan search | Current Engram project guidance remains recoverable before the stale item is archived. Ranking is evidence only, not proof. |
| `lint(action="run", write=false)` | The target is still reported as stale or wrong-scope active memory with `safe_action=none`. |
| `graph(action="around", node=..., depth=1)` | No new direct MemoryItem dependency on the target appears. Existing evidence, scope, and supersedes edges are acceptable. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T157 does not authorize:

- archiving, superseding, rejecting, editing, reviewing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- changing `handoff(update)` semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- running native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive
  Claude commands;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine,
  candidate decisions, or legacy simplification;
- editing root `AGENTS.md`.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T157 wording and target
  ID;
- the target UUID, title, kind, scope, status, `current-plan` tag, or archive payload differs from
  this packet;
- the target `updated_at` is later than `2026-05-25T07:30:08.716259Z` and the user has not
  re-approved after seeing the fresh item contents;
- the target is already archived, superseded, rejected, deleted, or missing;
- the active current-plan item for Engram cannot be identified before the archive;
- fresh lint no longer reports the target as stale or wrong-scope active memory with
  `safe_action=none`;
- fresh graph depth 1 shows a new direct MemoryItem dependency on the target;
- any write occurs after the final fresh pre-write read and before the archive;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, executing Claude, or executing harness writes.

## Approval Wording

To authorize only the single archive action above, reply exactly:

```text
Approve T157: after fresh matching read-only get/orient-or-search/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e5e0a-86b4-73e3-aa9b-ca350e83e915 with the archive payload in docs/BRAIN_HARNESS_T157_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

Any other reply should be treated as non-authorization for T157.
