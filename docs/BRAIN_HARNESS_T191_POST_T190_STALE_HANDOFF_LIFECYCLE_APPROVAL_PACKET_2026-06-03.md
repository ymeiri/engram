# Brain Harness T191 Post-T190 Stale Handoff Lifecycle Approval Packet

Date: 2026-06-03
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Ask for future exact approval to archive exactly five superseded rolling handoff
MemoryItems created before the latest T190 handoff:

- `019e8e83-4461-7500-909c-241183737348`
- `019e8e7b-5977-7f93-be7c-742da46f6831`
- `019e8e77-dfbc-7ae2-990a-df9368b75fc3`
- `019e8e71-1932-72e3-bac4-bd5abe9248f5`
- `019e8e6c-361a-73a0-933e-fcb12c599247`

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

This packet is a request for approval, not approval itself. It does not archive, supersede,
reject, review, delete, or edit any MemoryItem. It does not run `lint apply_safe`, change handoff
semantics, change search ranking or `orient`, run native Claude or Claude Bridge, write harness
files, run M6/migration/quarantine actions, change public MCP/schema/storage/index/document-index
behavior, signal PID `49349`, send PTY input, or touch user-owned files.

## Research Question

Can Engram safely ask for exact future approval to archive the five active rolling handoffs now
superseded by the latest T190 handoff, using read-only evidence and without performing lifecycle
writes now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A five-target, docs-only lifecycle packet is the smallest safe follow-up because the latest active handoff directly supersedes these records in a chain, direct search still surfaces them as active handoff noise, and current-plan retrieval remains healthy. |
| Null | The stale handoff records are tolerable because `handoff(get)` returns the latest handoff and explicit current-plan search still returns the current-plan item first. |
| Simpler alternative | Submit telemetry for stale handoff noise and defer lifecycle cleanup entirely. |
| Failure | The packet is mistaken for approval, archives the wrong handoff, duplicates T187 targets, applies broad lint cleanup, sweeps older handoffs without target-local evidence, changes handoff semantics, ranking, `orient`, M6, harness behavior, Claude process state, schema/storage/index, document-index behavior, or user-owned files. |

## Measurement

This packet used read-only evidence only:

- Lean startup `orient` trace `019e8e85-149a-77b3-b359-b10df8ec0cd6` returned active
  current-plan memory `019e8e84-1927-7a11-85f0-36792b244ad1` first and no open obligations.
- Direct current-plan search trace `019e8e85-3434-7c82-932a-f1d4869e2bea` returned active
  current-plan memory `019e8e84-1927-7a11-85f0-36792b244ad1` first, latest handoff
  `019e8e84-44bf-7d31-bded-88fe36f96659` second, then stale active handoffs including
  `019e8e83-4461-7500-909c-241183737348`,
  `019e8e7b-5977-7f93-be7c-742da46f6831`,
  `019e8e77-dfbc-7ae2-990a-df9368b75fc3`,
  `019e8e71-1932-72e3-bac4-bd5abe9248f5`,
  `019e8e6c-361a-73a0-933e-fcb12c599247`, and the already-packeted T187 target
  `019e8e6b-bb32-7832-9389-22dd04cbfcda`.
- Architecture/open-risk search trace `019e8e85-35f6-7932-91cb-95505796230f` was dominated by
  older rolling handoffs before returning architecture documents, confirming the stale handoff
  search-noise problem remains visible outside exact current-plan prompts.
- Recent-risk search trace `019e8e85-3994-75e3-9a53-24ae487a0faf` returned the latest handoff
  and stale handoff chain as top memory results before T174/telemetry document evidence.
- Focused handoff search trace `019e8e86-5fc9-7012-a570-59ffd4a42ade` returned latest handoff
  `019e8e84-44bf-7d31-bded-88fe36f96659` first, then target stale handoffs
  `019e8e83-4461-7500-909c-241183737348`,
  `019e8e7b-5977-7f93-be7c-742da46f6831`,
  `019e8e77-dfbc-7ae2-990a-df9368b75fc3`,
  `019e8e71-1932-72e3-bac4-bd5abe9248f5`, and then T187 target
  `019e8e6b-bb32-7832-9389-22dd04cbfcda`.
- `handoff(get)` returned latest active handoff `019e8e84-44bf-7d31-bded-88fe36f96659`, not
  any of the proposed archive targets.
- `memory(get)` for `019e8e84-44bf-7d31-bded-88fe36f96659` confirmed it is active,
  project-scoped to `engram`, tagged `handoff` and `rolling`, and directly supersedes
  `019e8e83-4461-7500-909c-241183737348`.
- `memory(get)` for each proposed archive target confirmed the target is still active,
  project-scoped to `engram`, tagged `handoff` and `rolling`, and part of this chain:

```text
019e8e84-44bf... latest active handoff
  -> 019e8e83-4461...
  -> 019e8e7b-5977...
  -> 019e8e77-dfbc...
  -> 019e8e71-1932...
  -> 019e8e6c-361a...
  -> 019e8e6b-bb32...  (already covered by T187)
```

- `graph(action="around", node="019e8e84-44bf-7d31-bded-88fe36f96659", depth=8)` confirmed the
  same supersession chain.
- `memory(changes_since)` from the startup cursor returned `item_count=0`, `commit_count=0`, trace
  `019e8e86-6004-7a01-b3c7-ad051cfbe1a8`.
- `lint(action="run", limit=40)` was read-only and applied zero safe actions. It reported broad
  superseded-active lifecycle findings, which are intentionally out of scope for this packet.
- `obligations(action="doctor", project="engram")` returned `open=[]`, `warnings=[]`.
- Git status showed only the known user-owned untracked root `AGENTS.md`.

## Completion Matrix Delta

| Area | State After T191 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | Recoverable and first for explicit next-step prompt | `orient` trace `019e8e85-149a...`; search trace `019e8e85-3434...` | Goal still incomplete; cleanup and migration gates remain |
| Active rolling handoff | Latest handoff available | `handoff(get)` returns `019e8e84-44bf...` | Older active handoffs still pollute search until archived |
| Post-T190 stale handoff chain | Exact archive targets identified | `memory(get)`, focused search, graph chain | Requires exact T191 approval before archive |
| T187 stale handoff chain | Already packeted separately | T187 packet targets `019e8e6b-bb32...`, `019e8e6a-dd68...`, `019e8bc0-59a...` | Requires exact T187 approval; excluded from T191 to avoid duplicate scope |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")` | Archive/write approval required |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Do not change ranking or `orient` from this packet |
| Native Claude process | Still unresolved | T190 records PID `49349` remained live | T186 exact approval remains required |
| M6/migration | Still gated | No M6/migration/quarantine action ran | Separate reviewed-candidate and dry-run/apply approval |

## Proposed Approved Archives

If and only if the user approves with the exact phrase below, Codex may run these five Memory OS
archive writes and no others:

```text
memory(
  action="archive",
  id="019e8e83-4461-7500-909c-241183737348",
  archive_reason="Superseded intermediate rolling handoff from T190 handoff-refresh maintenance. Latest active handoff 019e8e84-44bf-7d31-bded-88fe36f96659 supersedes it, and it still appears as active direct-search handoff noise. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8e7b-5977-7f93-be7c-742da46f6831",
  archive_reason="Superseded rolling handoff from T189 telemetry follow-through. Handoff 019e8e83-4461-7500-909c-241183737348 supersedes it, it still appears as active direct-search handoff noise, and its content points at superseded current-plan memory 019e8e7b-273d-7110-a2d8-8543619e4cb5 rather than active current-plan memory 019e8e84-1927-7a11-85f0-36792b244ad1. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8e77-dfbc-7ae2-990a-df9368b75fc3",
  archive_reason="Superseded rolling handoff from T188 document-index packet. Handoff 019e8e7b-5977-7f93-be7c-742da46f6831 supersedes it, and it still appears as active direct-search handoff noise. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8e71-1932-72e3-bac4-bd5abe9248f5",
  archive_reason="Superseded rolling handoff from T187 lifecycle packet. Handoff 019e8e77-dfbc-7ae2-990a-df9368b75fc3 supersedes it, and it still appears as active direct-search handoff noise. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8e6c-361a-73a0-933e-fcb12c599247",
  archive_reason="Superseded rolling handoff from T186 native-Claude SIGINT packet. Handoff 019e8e71-1932-72e3-bac4-bd5abe9248f5 supersedes it, it now predates the latest T190 handoff 019e8e84-44bf-7d31-bded-88fe36f96659, and it still appears as active direct-search handoff noise. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

## Required Fresh Pre-Write Evidence

Immediately before any future archive calls, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the five archive calls:

| Check | Required result |
| --- | --- |
| `handoff(action="get", project="engram")` | Latest active handoff remains `019e8e84-44bf-7d31-bded-88fe36f96659`, or the user re-approves after seeing the newer handoff. |
| `memory(action="get", id="019e8e84-44bf-7d31-bded-88fe36f96659")` | Latest handoff remains active, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e83-4461-7500-909c-241183737348`. |
| `memory(action="get", id="019e8e83-4461-7500-909c-241183737348")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e7b-5977-7f93-be7c-742da46f6831`. |
| `memory(action="get", id="019e8e7b-5977-7f93-be7c-742da46f6831")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e77-dfbc-7ae2-990a-df9368b75fc3`. |
| `memory(action="get", id="019e8e77-dfbc-7ae2-990a-df9368b75fc3")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e71-1932-72e3-bac4-bd5abe9248f5`. |
| `memory(action="get", id="019e8e71-1932-72e3-bac4-bd5abe9248f5")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e6c-361a-73a0-933e-fcb12c599247`. |
| `memory(action="get", id="019e8e6c-361a-73a0-933e-fcb12c599247")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes T187 target `019e8e6b-bb32-7832-9389-22dd04cbfcda`. |
| Current-plan orient or direct search | Current Engram project guidance remains recoverable before the stale handoffs are archived. |
| Handoff search | At least one T191 target still appears as active stale handoff noise, or the graph/supersession chain still proves staleness. |
| `lint(action="run", write=false)` | The result is read and recorded. This packet does not depend on lint safe actions. |
| `graph(action="around", node="019e8e84-44bf-7d31-bded-88fe36f96659", depth=8)` | The direct supersession chain still includes all five targets and no unexpected direct dependency changes require user review. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T191 does not authorize:

- archiving, superseding, rejecting, reviewing, editing, or deleting any other MemoryItem;
- archiving the already-packeted T187 targets unless the user separately approves T187;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- changing `handoff(update)` semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- sending any native Claude input, including EOF, Ctrl-C bytes, `/hooks`, another slash command, or
  prompt-bearing input;
- sending any process signal, including `SIGINT`, `SIGTERM`, `SIGKILL`, or force-kill fallback;
- launching native Claude or Claude Bridge;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine
  inspection, candidate decisions, or legacy simplification;
- editing root `AGENTS.md` or other user-owned files.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T191 wording and all
  five target IDs;
- any target UUID, kind, scope, status, tags, or archive payload differs from this packet;
- any target is already archived, rejected, deleted, missing, or no longer a rolling handoff;
- latest active handoff for Engram cannot be identified before the archive;
- active current-plan guidance for Engram cannot be identified before the archive;
- fresh graph no longer shows the expected supersession chain;
- any write occurs after the final fresh pre-write read and before the archive calls;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, executing Claude, sending process signals, or executing harness writes.

## Approval Wording

To authorize only the five archive actions above, reply exactly:

```text
Approve T191: after fresh matching read-only handoff/get/orient-or-search/handoff-search/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItems 019e8e83-4461-7500-909c-241183737348, 019e8e7b-5977-7f93-be7c-742da46f6831, 019e8e77-dfbc-7ae2-990a-df9368b75fc3, 019e8e71-1932-72e3-bac4-bd5abe9248f5, and 019e8e6c-361a-73a0-933e-fcb12c599247 with the archive payloads in docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, process signals, harness installs/settings/hooks/adapters, or user-owned files.
```

Any other reply should be treated as non-authorization for T191.
