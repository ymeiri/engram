# T192 T191 Document Index Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval to index two repository documents into Engram document
search:

- `docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

It does not run `docs(action="index")`, change document-index behavior, archive memory, run
`lint apply_safe`, signal PID `49349`, send native Claude input, launch Claude or Claude Bridge,
run harness install, mutate lifecycle or migration state, inspect M6/quarantine candidates, change
ranking or `orient`, change public MCP/schema/storage/index behavior, delete, roll back, reinstall
binaries, or touch user-owned files.

## Research Question

Can Engram safely ask for future exact approval to index the newest T191 lifecycle packet and the
matching implementation-plan note, so document search can recover the latest stale-handoff approval
evidence, without changing document-index behavior or running indexing now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A two-file exact-index packet is the smallest safe follow-up because memory search and lean `orient` recover the current plan, but document-layer searches still miss T191 and return older lifecycle or document-index artifacts. |
| Null | MemoryItem current-plan retrieval is enough; the document layer can remain stale for T191. |
| Simpler alternative | Do nothing until the user asks for document indexing explicitly. |
| Failure | The packet is mistaken for indexing approval or bundled with T186 process cleanup, T191/T187 lifecycle archive, M6/migration/quarantine work, ranking/orient/source changes, schema/storage/index changes, document-index behavior changes, harness edits, deletion, rollback, or user-owned-file edits. |

## Measurement

This packet used read-only evidence only:

- Lean `orient` trace `019e8e8d-5e29-7581-b681-ddde8f33879f` returned current-plan memory
  `019e8e8a-a124-7822-87df-817e2d78be05` first and no open obligations.
- Direct current-plan search trace `019e8e8d-66a5-7993-9a0c-31ddfd3991f7` returned that
  current-plan memory first, then the latest and stale active rolling handoffs. This confirms the
  current-plan path is healthy while stale handoff search noise remains visible.
- Recent-risk search trace `019e8e8c-8435-7801-84ee-787214db8a3e` returned the active T191
  current-plan memory first, then latest and stale active rolling handoffs before older document
  evidence.
- `docs(action="search")` for the exact T191 title plus all five target IDs returned older indexed
  lifecycle documents, including T159, T157, and T160, not T191.
- `docs(action="search")` for the T191 filename stem returned older indexed lifecycle documents and
  T176, not T191.
- `docs(action="search")` for commit probe `bc25df8 Record T191 stale handoff lifecycle packet`
  returned an older indexed `MEMORY_OS_IMPLEMENTATION_PLAN.md` chunk and other older documents, not
  the T191 file.
- Direct unified search for the current T191 indexing continuation returned document results from
  T176, the live feedback batch, T159, and T157, not T191.
- `docs(action="stats")` reported `source_count=89`, `chunk_count=4346`,
  `searchable_chunk_count=2334`, and `orphan_chunk_count=2012`.
- `obligations(action="doctor", project="engram")` returned `open=[]`, `warnings=[]`.
- `git status --short --branch` showed branch `yuval.meiri/memory-os-phase0` with only the known
  user-owned untracked root `AGENTS.md`.
- Recent commits show T191 is committed as `bc25df8 Record T191 stale handoff lifecycle packet`.

Current file fingerprints before this packet:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 14481 | `d733f8a1c7fb0dfd2b0f741f0263c33b7c98abad3755e2d0870511bedc9be07b` |
| `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` | 321330 | `6d9c0fccdccc70e3a7e9a390aa8567d34b92567982214c082054cea1f80fd9f3` |

## Completion Matrix Delta

| Area | State After T192 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | Recoverable and first | `orient` and direct search traces return T191 current-plan memory first | Goal still incomplete; cleanup, lifecycle, migration, and indexing gates remain |
| T191 repo evidence | Committed locally | Commit `bc25df8` and file read | Document search still misses T191 until indexing is approved and run |
| Document visibility | Gap identified | Exact document-layer probes return older documents, not T191 | Requires exact T192 approval before indexing |
| Document-index behavior | Unchanged | This packet does not run indexing or change source | Future T192 execution may index only exact files |
| Lifecycle cleanup | Still gated | No archive, no `lint apply_safe` | T191 and T187 archive packets remain separate exact gates |
| Native Claude cleanup | Still gated | T190 records PID `49349` remained live | Requires exact T186 approval |
| M6/migration | Still high-risk and gated | No M6 action in T192 | Requires separate approved scoping/dry-run/apply path |

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T192: index exact files T191 and MEMORY_OS_IMPLEMENTATION_PLAN from docs/BRAIN_HARNESS_T192_T191_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md. After fresh git/path/document-search/obligations evidence and no intervening writes, run docs(action="index") only for docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md and docs/MEMORY_OS_IMPLEMENTATION_PLAN.md, then run read-only document-search validation and write/commit the result report. Do not run T186, T191, T187, lifecycle archive, lint apply_safe, ranking/orient/source changes, public MCP/schema/storage/index/document-index behavior changes, M6/migration/quarantine, native Claude, Claude Bridge, process signals, harness installs/settings/hooks/adapters, deletion, rollback, or user-owned-file edits.
```

Shorter approval, generic continuation, T186 approval, T191 approval, or T187 approval must not be
treated as T192 approval.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before indexing:

- `git status --short --branch`
- path existence and regular-file checks for the two approved files
- byte count and SHA-256 for the two approved files
- read-only document-search probes proving whether T191 is still missing
- `obligations(action="doctor", project="engram")`

### Exact-File Indexing

Allowed only if preflight still matches this packet and no intervening writes occurred:

```text
docs(action="index", path="docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md")
docs(action="index", path="docs/MEMORY_OS_IMPLEMENTATION_PLAN.md")
```

No directory indexing, reindex plan/execute, cleanup plan/execute, quarantine review action,
document-index behavior change, schema/storage/index change, or source change is authorized.

### Post-Index Read-Only Validation

Allowed after the exact indexing calls:

- Re-run exact document-search probes for the T191 title and all five target IDs.
- Verify the T191 packet and implementation-plan note are visible, or record the exact miss.
- Write one result report under `docs/`.
- Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Commit only the intended documentation files.
- Capture current-plan memory after the commit.
- Submit telemetry feedback for assessed retrieval traces.

## Explicitly Forbidden

T192 does not authorize:

- executing T186 or sending any process signal;
- executing T191, T187, or archiving any MemoryItem;
- running `lint(action="apply_safe", write=true)` or broad lifecycle cleanup;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules, telemetry
  formulas, source code, or document-index behavior;
- running M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion,
  quarantine inspection, candidate decisions, or legacy simplification;
- launching native Claude or Claude Bridge;
- sending native Claude input, EOF, Ctrl-C bytes, `/hooks`, slash commands, or prompt-bearing input;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- editing root `AGENTS.md` or other user-owned files.

## Stop Conditions

Stop without indexing if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T192 wording and both
  target files;
- either target path is missing, not a regular file, or unexpectedly changed after final preflight
  without user re-approval;
- git status has unexpected tracked changes;
- obligations doctor reports an open obligation that changes the scope;
- any write occurs after the final fresh pre-index read and before the indexing calls;
- indexing appears to require directory-wide ingestion, behavior changes, schema/storage/index
  changes, reindex/cleanup/quarantine actions, source edits, process signals, lifecycle writes,
  M6/migration work, native Claude, Claude Bridge, or harness writes.
