# T194 T193 Document Index Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval to index two repository documents into Engram document
search:

- `docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

It does not run `docs(action="index")`, change document-index behavior, archive memory, run
`lint apply_safe`, signal PID `49349`, send native Claude input, launch Claude or Claude Bridge,
run harness install, mutate lifecycle or migration state, inspect M6/quarantine candidates, change
ranking or `orient`, change public MCP/schema/storage/index behavior, delete, roll back, reinstall
binaries, or touch user-owned files.

## Research Question

Can Engram safely ask for future exact approval to index the newest T193 stale-handoff lifecycle
packet and the matching implementation-plan note, so document search can recover the latest
single-target stale-handoff approval evidence, without changing document-index behavior or running
indexing now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A two-file exact-index packet is the smallest safe follow-up because current-plan memory and repo files recover T193, but document-layer searches still miss T193 and return older lifecycle or document-index artifacts. |
| Null | MemoryItem current-plan retrieval is enough; the document layer can remain stale for T193. |
| Simpler alternative | Do nothing until the user asks for document indexing explicitly. |
| Failure | The packet is mistaken for indexing approval or bundled with T193/T191/T187 lifecycle archive, T192 indexing, T186 process cleanup, M6/migration/quarantine work, ranking/orient/source changes, schema/storage/index changes, document-index behavior changes, harness edits, deletion, rollback, or user-owned-file edits. |

## Measurement

This packet used read-only evidence only:

- Lean `orient` trace `019e8e98-107c-7830-b41e-d8e1586237ed` returned current-plan memory
  `019e8e96-deb0-7bb2-918d-2f167c15430e` first and no open obligations.
- Direct current-plan search trace `019e8e98-2ac4-7992-b3a2-a9b21c5e4a9d` returned that
  current-plan memory first, then stale active rolling handoff noise. This confirms the current-plan
  path is healthy while stale handoff search noise remains visible.
- Memory OS implementation/search trace `019e8e98-2c6f-7bc3-a49b-3b62c9864372` returned the
  T193 current-plan memory first, then the latest handoff and older completion/migration facts.
- `docs(action="search")` for the exact T193 title and target ID returned older indexed lifecycle
  documents, including T159, T157, T160, and T58, not T193.
- Direct unified document search trace `019e8e98-ad54-7932-8d25-1161e763df0a` for the T193 filename
  stem and target ID returned T176, T159, T157, T160, and an older implementation-plan chunk, not
  T193.
- Direct unified document search trace `019e8e98-ae1b-7c30-8bac-a2ff3fc51c5e` for the T193 matrix
  note and commit probe returned older implementation-plan chunks, T157, and T64, not the current
  T193 matrix note.
- `docs(action="stats")` reported `source_count=89`, `chunk_count=4346`,
  `searchable_chunk_count=2334`, and `orphan_chunk_count=2012`.
- `obligations(action="doctor", project="engram")` returned `open=[]`, `warnings=[]`.
- `git status --short --branch` showed branch `yuval.meiri/memory-os-phase0` with only the known
  user-owned untracked root `AGENTS.md`.
- Recent commits show T193 is committed as `f1653a9 Record T193 stale handoff lifecycle packet`.

Current file fingerprints before this packet:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 7266 | `d44f4ce54f9e9bb1498dbe949a627489027dd6c1fafafeba1cceaca690a60251` |
| `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` | 324441 | `c4936b929b3f8c1d0a86c0bca6d80e0802d18d854e85050b2ac8dfc2bbcda0e3` |

## Completion Matrix Delta

| Area | State After T194 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | Recoverable and first | `orient` and direct search traces return T193 current-plan memory first | Goal still incomplete; cleanup, lifecycle, migration, native Claude, and indexing gates remain |
| T193 repo evidence | Committed locally | Commit `f1653a9` and file read | Document search still misses T193 until indexing is approved and run |
| Document visibility | Gap identified | Exact document-layer probes return older documents, not T193 | Requires exact T194 approval before indexing |
| Document-index behavior | Unchanged | This packet does not run indexing or change source | Future T194 execution may index only exact files |
| Lifecycle cleanup | Still gated | No archive, no `lint apply_safe` | T193, T191, and T187 archive packets remain separate exact gates |
| Native Claude cleanup | Still gated | T190 records PID `49349` remained live | Requires exact T186 approval |
| M6/migration | Still high-risk and gated | No M6 action in T194 | Requires separate approved scoping/dry-run/apply path |

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T194: index exact files T193 and MEMORY_OS_IMPLEMENTATION_PLAN from docs/BRAIN_HARNESS_T194_T193_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md. After fresh git/path/document-search/obligations evidence and no intervening writes, run docs(action="index") only for docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md and docs/MEMORY_OS_IMPLEMENTATION_PLAN.md, then run read-only document-search validation and write/commit the result report. Do not run T193, T192, T191, T187, T186, lifecycle archive, lint apply_safe, ranking/orient/source changes, public MCP/schema/storage/index/document-index behavior changes, M6/migration/quarantine, native Claude, Claude Bridge, process signals, harness installs/settings/hooks/adapters, deletion, rollback, or user-owned-file edits.
```

Shorter approval, generic continuation, T193 approval, T192 approval, T191 approval, T187 approval,
or T186 approval must not be treated as T194 approval.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before indexing:

- `git status --short --branch`
- path existence and regular-file checks for the two approved files
- byte count and SHA-256 for the two approved files
- read-only document-search probes proving whether T193 is still missing
- `obligations(action="doctor", project="engram")`

### Exact-File Indexing

Allowed only if preflight still matches this packet and no intervening writes occurred:

```text
docs(action="index", path="docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md")
docs(action="index", path="docs/MEMORY_OS_IMPLEMENTATION_PLAN.md")
```

No directory indexing, reindex plan/execute, cleanup plan/execute, quarantine review action,
document-index behavior change, schema/storage/index change, or source change is authorized.

### Post-Index Read-Only Validation

Allowed after the exact indexing calls:

- Re-run exact document-search probes for the T193 title and target ID.
- Verify the T193 packet and implementation-plan note are visible, or record the exact miss.
- Write one result report under `docs/`.
- Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Commit only the intended documentation files.
- Capture current-plan memory after the commit.
- Submit telemetry feedback for assessed retrieval traces.

## Explicitly Forbidden

T194 does not authorize:

- executing T193, T191, T187, or archiving any MemoryItem;
- executing T192 document indexing;
- executing T186 or sending any process signal;
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

- approval is missing, conditional, ambiguous, or does not include the exact T194 wording and both
  target files;
- either target path is missing, not a regular file, or unexpectedly changed after final preflight
  without user re-approval;
- git status has unexpected tracked changes;
- obligations doctor reports an open obligation that changes the scope;
- any write occurs after the final fresh pre-index read and before the indexing calls;
- indexing appears to require directory-wide ingestion, behavior changes, schema/storage/index
  changes, reindex/cleanup/quarantine actions, source edits, process signals, lifecycle writes,
  M6/migration work, native Claude, Claude Bridge, or harness writes.
