# Brain Harness T159 Stale T146 Limitation Lifecycle Approval Packet

Date: 2026-06-03
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Refresh the lifecycle request for exactly one stale T146 runtime-refresh limitation:
`019e89f4-7dba-7ae1-a559-85d924af31a3`.

This packet is a request for approval, not approval itself. It does not archive, supersede,
reject, review, delete, or edit any MemoryItem. It does not run `lint apply_safe`, mutate
lifecycle state, run native Claude, use Claude Bridge, install or edit harness files, run M6 or
migration commands, inspect quarantine candidates, change ranking or `orient`, change public MCP,
schema/storage/index behavior, document-index behavior, or touch user-owned files.

The user's late "I approve T135 harness repair" message is not treated as approval for this
packet. T135 was already executed and validated in T152, and late duplicate T135 approval does not
reopen harness writes.

## Research Question

Can Engram safely ask for exact future approval to archive the stale T146 runtime-refresh limitation
MemoryItem `019e89f4-7dba-7ae1-a559-85d924af31a3`, using current read-only evidence and without
performing the archive now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single-target, docs-only lifecycle packet is the smallest safe follow-up because T147 has contradicted the limitation, but the item still appears as active guidance for targeted prompts. |
| Null | The active stale limitation is tolerable because it is only surfaced for targeted prompts and lint does not flag it. |
| Simpler alternative | Defer this target and wait for broader lifecycle cleanup approval. |
| Failure | The packet is mistaken for approval, archives the wrong item, treats manual stale assessment as automatic lint proof, sweeps old handoffs, or bundles ranking, `orient`, M6, Claude, harness, schema/storage/index, public MCP, document-index, or user-owned-file changes. |

## Measurement

This packet used read-only evidence only:

- Lean startup `orient` trace `019e8d16-b7e6-7fa1-9837-dfe3e1aed825` returned the active T158
  current plan first, plus the harness-write gate, M6 gate, and commit preference.
- Direct search trace `019e8d16-fba3-71d0-be8d-dea170622e0f` returned the active T158 current
  plan first for current-plan/T135 continuation context, but old handoffs remained noisy below it.
- Repo docs confirmed T135 is already complete:
  `docs/BRAIN_HARNESS_T152_T135_HARNESS_REPAIR_VALIDATION_RESULT_2026-06-03.md` says all five
  approved harness writes were installed and validated, and
  `docs/BRAIN_HARNESS_T155_COMPLETION_GATE_AUDIT_2026-06-03.md` says duplicate or late T135
  approval does not reopen harness writes.
- `memory(action="get", id="019e89f4-7dba-7ae1-a559-85d924af31a3")` confirmed the target remains
  `status=active`, `kind=limitation`, project-scoped to `engram`, tagged `runtime-refresh-gate`
  and `t146`, titled `T146 source fix requires runtime refresh before live no-prompt orient
  changes`, and last updated at `2026-06-02T20:09:22.10605Z`.
- T147 validation contradicts the target. The target says live no-prompt `plan_work` `orient`
  required runtime refresh after source commit `d12b2ca`; T147 then installed the binary, restarted
  the daemon, and validated that no-prompt and empty-prompt project-scoped `plan_work` `orient`
  returned the active current-plan item first.
- Direct targeted search trace `019e8d17-da96-71b2-b529-f8eeb27fb652` returned the stale target
  first for a T146/T147 stale-limitation query.
- Lean targeted `orient` trace `019e8d17-db79-7b12-b8d6-8c7ac6d5d65a` surfaced the stale target
  first and the active T158 current plan second when planning this exact lifecycle packet.
- `lint(action="run", limit=80, write=false)` did not flag this target. It did flag unrelated
  current-plan/lifecycle debt: stale and wrong-scope feedback for `019e5e0a...`, plus many
  `superseded_item_still_active` findings. That lint gap is evidence that this target requires
  manual approval, not automatic cleanup.
- `graph(action="around", node="019e89f4-7dba-7ae1-a559-85d924af31a3", depth=1)` showed only
  evidence edges, project scope, and the writer session. It showed no direct dependent MemoryItem.
- Source inspection confirmed:
  - `engram-core/src/memory.rs` `with_archive` sets `status=Archived`, records archive metadata,
    and updates `updated_at`;
  - `engram-index/src/memory.rs` `archive_memory` loads exactly one requested item, applies
    `with_archive`, and saves that item;
  - `engram-index/src/memory_ranker.rs` assigns archived memory status score `0.0`;
  - `engram-index/src/lint.rs` `apply_safe` can archive every matching safe-action finding in a
    report, so it remains too broad for this target.
- AI Council recall recovered prior default-deny lifecycle guidance: single-target docs-only
  packets are acceptable, but archive writes, `lint apply_safe`, broad cleanup, ranking/`orient`,
  M6, harness, Claude, schema/storage/index, public MCP, and document-index changes remain
  exact-gated.
- Git status showed only the known user-owned untracked root `AGENTS.md`.

## Completion Matrix Delta

| Area | State After T159 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T146 installed-runtime gap | Closed | T147 validation result and live traces | None for T146 no/empty-prompt runtime parity |
| Stale T146 runtime-refresh limitation | Exact future archive target refreshed | `memory(get)`, targeted search/orient, T147 contradiction, graph check | Requires exact T159 approval before archive |
| Lint coverage for this target | Missing | `lint(run)` did not flag `019e89f4...` | Archive must be human-approved, not automatic |
| Active current plan | T158 remains active | Startup orient and `memory(get)` for `019e8d14...` | T125, T154, and T157 remain separate exact gates |
| Harness readiness | T135 already executed | T152 validation and T155 duplicate-approval note | Native Claude behavior still separate |
| M6 migration | Still gated | T158 packet only asks for future T125 read-only quarantine inspection | Exact T125 approval before quarantine reads |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Do not change ranking or `orient` from this packet |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e89f4-7dba-7ae1-a559-85d924af31a3",
  archive_reason="Stale T146 runtime-refresh limitation contradicted by T147 installed-runtime validation: after installing binary hash 0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22 and restarting the daemon to PID 68053, live no-prompt and empty-prompt project-scoped plan_work orient traces returned the active current-plan item first. Read-only 2026-06-03 search/orient still surfaced this limitation as active guidance; graph depth 1 showed only evidence, project scope, and writer-session edges; lint did not flag this item, so this is a human-approved manual archive, not a lint safe action.",
  archived_by="codex"
)
```

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `limitation`, scope is project `engram`, tags still include `runtime-refresh-gate` and `t146`, and `updated_at` is not later than `2026-06-02T20:09:22.10605Z` unless the user re-approves after seeing the drift. |
| Current-plan orient or direct search | Current Engram project guidance remains recoverable before the stale item is archived. |
| T147 evidence check | `docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_VALIDATION_RESULT_2026-06-03.md` or fresher evidence still proves the T146 runtime refresh passed. |
| `lint(action="run", write=false)` | The result is read and recorded. The target may or may not be flagged; either way this remains human-approved, not automatic. |
| `graph(action="around", node=..., depth=1)` | No direct dependent MemoryItem appears. Existing evidence, project, and writer-session edges are acceptable. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T159 does not authorize:

- archiving, superseding, rejecting, reviewing, editing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- changing `handoff(update)` semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- running native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive
  Claude commands;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine
  inspection, candidate decisions, or legacy simplification;
- editing root `AGENTS.md` or other user-owned files.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T159 wording and target
  ID;
- the target UUID, title, kind, scope, status, tags, or archive payload differs from this packet;
- the target `updated_at` is later than `2026-06-02T20:09:22.10605Z` and the user has not
  re-approved after seeing the fresh item contents;
- the target is already archived, superseded, rejected, deleted, or missing;
- active current-plan guidance for Engram cannot be identified before the archive;
- fresh graph depth 1 shows a direct MemoryItem dependency on the target;
- any write occurs after the final fresh pre-write read and before the archive;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, executing Claude, or executing harness writes.

## Approval Wording

To authorize only the single archive action above, reply exactly:

```text
Approve T159: after fresh matching read-only get/orient-or-search/T147-evidence/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e89f4-7dba-7ae1-a559-85d924af31a3 with the archive payload in docs/BRAIN_HARNESS_T159_STALE_T146_LIMITATION_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

Any other reply should be treated as non-authorization for T159.
