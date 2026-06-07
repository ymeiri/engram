# Brain Harness T160 Wrong-Scope Claude Prompt Lifecycle Approval Packet

Date: 2026-06-03
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Ask for future exact approval to archive exactly one wrong-scope active prompt-capture
MemoryItem: `019e7f52-4fc2-7f61-93b4-9a741aba966e`.

This packet is a request for approval, not approval itself. It does not archive, supersede, reject,
review, delete, or edit any MemoryItem. It does not run `lint apply_safe`, change handoff
semantics, change search ranking or `orient`, run native Claude or Claude Bridge, write harness
files, run M6/migration/quarantine actions, change public MCP/schema/storage/index/document-index
behavior, or touch user-owned files.

The user's late "I approve T135 harness repair" message is not treated as approval for this packet.
T135 was already executed and validated in T152; fresh read-only T160 status/doctor checks still
show all five harnesses `ready=true`.

## Research Question

Can Engram safely ask for exact future approval to archive wrong-scope active Claude Code prompt
capture `019e7f52-4fc2-7f61-93b4-9a741aba966e`, using read-only evidence and without performing the
archive now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single-target, docs-only lifecycle packet is the smallest safe follow-up because the item is a one-time Claude Code prompt captured as an active `rule`, lint flags wrong-scope feedback with `safe_action=none`, and graph shows no dependent MemoryItem. |
| Null | The active prompt capture is tolerable because current T135 approval-oriented `orient` did not surface it and exact-query visibility alone is not enough to archive it. |
| Simpler alternative | Defer this target and wait for broader lifecycle cleanup approval. |
| Failure | The packet is mistaken for approval, archives the wrong item, treats lint feedback as automatic cleanup, sweeps old handoffs, reruns T135 harness writes, or bundles ranking, `orient`, Claude, M6, schema/storage/index, public MCP, document-index, or user-owned-file changes. |

## Measurement

This packet used read-only evidence only:

- Lean startup `orient` trace `019e8d1d-1444-76d3-b9d7-b7cf35296690` returned the current T159
  plan first, plus the harness-write gate, M6 gate, and commit preference. The target did not
  surface in this current T135 approval-oriented orientation, so this packet does not claim a fresh
  broad-orient failure.
- Direct current-plan search trace `019e8d1e-94a8-7bc1-9939-310d2750c05e` returned the active T159
  plan first. The stale repository-scoped current-plan target and old handoffs remained noisy
  below it.
- `memory(action="get", id="019e7f52-4fc2-7f61-93b4-9a741aba966e")` confirmed the target remains
  active, kind `rule`, title `Claude Code user-stated instruction`, project-scoped to `engram`,
  tagged `claude-code`, `hook-event`, and `user-stated`, last updated at
  `2026-05-31T18:36:01.346882Z`.
- The target content is a specific one-time Claude Code prompt for a read-only critique of a
  telemetry evidence-loop fix involving `real_session_eval_report_scoped` and
  `list_feedback_scoped`. It is not durable general guidance for future Engram work.
- Direct exact search trace `019e8d1e-945e-7df3-9bb1-a1b062f500f2` returned the target first for
  the prompt-specific telemetry-fix query, proving it remains active and retrievable.
- `lint(action="run", limit=100, write=false)` reported
  `feedback_wrong_scope_active_memory` for the target with 4 recent wrong-scope feedback records
  and `safe_action=none`. This is a human-review signal, not automatic cleanup permission.
- `graph(action="around", node="019e7f52-4fc2-7f61-93b4-9a741aba966e", depth=1)` showed only
  manual-review prompt evidence and project scope. It showed no direct dependent MemoryItem.
- Fresh read-only `harness(status)` and `harness(doctor)` checks still returned `ready=true` for
  generic, Codex, Gemini CLI, Cursor, and Claude Code. Claude Code keeps the documented soft
  warnings about user-owned snippet preservation, split settings, extra legacy permissions, and
  effective hook validation.
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
  M6, harness writes, Claude execution, schema/storage/index, public MCP, and document-index
  changes remain exact-gated.
- Git status showed only the known user-owned untracked root `AGENTS.md`.

## Completion Matrix Delta

| Area | State After T160 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T135 harness repair | Already executed and still ready | T152 plus fresh T160 status/doctor checks | Native Claude behavior remains separate |
| Wrong-scope Claude prompt capture | Exact future archive target identified | `memory(get)`, exact search, lint wrong-scope feedback, graph check | Requires exact T160 approval before archive |
| Active current plan | Recoverable | Startup orient and direct current-plan search return T159 first | Old current-plan/handoff noise remains separate lifecycle debt |
| Lint automation | Not applicable to this target | Target has `safe_action=none` | Archive must be human-approved, not `lint apply_safe` |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Do not change ranking or `orient` from this packet |
| M6 migration | Still gated | No M6 action in T160 | Exact T125/M6 approval remains separate |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e7f52-4fc2-7f61-93b4-9a741aba966e",
  archive_reason="Active Claude Code prompt capture from 2026-05-31 telemetry evidence-loop work is wrong-scope durable guidance: it is a one-time critique request about real_session_eval_report_scoped/list_feedback_scoped, lint reported feedback_wrong_scope_active_memory with 4 recent wrong-scope records and safe_action=none, exact search still returned it as active guidance, and graph depth 1 showed only manual-review prompt evidence and project scope. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `rule`, scope is project `engram`, tags still include `claude-code`, `hook-event`, and `user-stated`, and `updated_at` is not later than `2026-05-31T18:36:01.346882Z` unless the user re-approves after seeing the drift. |
| Current-plan orient or direct search | Current Engram project guidance remains recoverable before the prompt capture is archived. |
| Target visibility check | Exact or related search still shows the target as active wrong-scope prompt guidance, or lint still reports wrong-scope feedback for the target. |
| `lint(action="run", write=false)` | The result is read and recorded. The target may or may not be flagged; either way this remains human-approved, not automatic. |
| `graph(action="around", node=..., depth=1)` | No direct dependent MemoryItem appears. Existing prompt evidence and project-scope edges are acceptable. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T160 does not authorize:

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

- approval is missing, conditional, ambiguous, or does not include the exact T160 wording and target
  ID;
- the target UUID, title, kind, scope, status, tags, or archive payload differs from this packet;
- the target `updated_at` is later than `2026-05-31T18:36:01.346882Z` and the user has not
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
Approve T160: after fresh matching read-only get/orient-or-search/target-visibility/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e7f52-4fc2-7f61-93b4-9a741aba966e with the archive payload in docs/BRAIN_HARNESS_T160_WRONG_SCOPE_CLAUDE_PROMPT_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```
