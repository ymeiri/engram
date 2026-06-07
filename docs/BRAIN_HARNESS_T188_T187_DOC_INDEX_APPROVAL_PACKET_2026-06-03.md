# T188 T187 Document Index Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval to index two repository documents into Engram document
search:

- `docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

It does not run `docs(action="index")`, change document-index behavior, archive memory, run
`lint apply_safe`, signal PID `49349`, send native Claude input, launch Claude or Claude Bridge,
run harness install, mutate lifecycle or migration state, inspect M6/quarantine candidates, change
ranking or `orient`, change public MCP/schema/storage/index behavior, delete, roll back, reinstall
binaries, or touch user-owned files.

## Research Question

Can Engram safely ask for future exact approval to index the newest T187 lifecycle packet and the
matching implementation-plan note, so document search can recover the latest gate evidence, without
changing document-index behavior or running indexing now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A two-file exact-index packet is the smallest safe follow-up because memory search and lean `orient` recover the current plan, but document-layer searches still miss T187 and return older lifecycle documents. |
| Null | MemoryItem current-plan retrieval is enough; the document layer can remain stale for T187. |
| Simpler alternative | Do nothing until the user asks for document indexing explicitly. |
| Failure | The packet is mistaken for indexing approval or bundled with T186 process cleanup, T187 lifecycle archive, M6/migration/quarantine work, ranking/orient/source changes, schema/storage/index changes, document-index behavior changes, harness edits, deletion, rollback, or user-owned-file edits. |

## Measurement

This packet used read-only evidence only:

- Lean startup `orient` trace `019e8e75-31b6-7042-9c1b-8730e81bee07` returned current-plan
  memory `019e8e70-d568-7e90-8f16-6405dd191b27` first and no open obligations.
- Direct current-plan search trace `019e8e75-5d19-78b0-9275-bab086ea93eb` returned that current-plan
  memory first and confirmed T186 remains the immediate cleanup gate.
- Direct architecture/document-lifecycle search trace `019e8e75-5f2a-72d2-b7b9-289d3275c424`
  returned current-plan memory and older indexed documents; the document layer did not surface T187.
- Exact document-layer query trace `019e8e76-1826-7062-8227-91c58f2eb372` for the T187 title and
  all three target IDs returned T159, T157, T160, T176, T58, and the live feedback batch, not T187.
- `docs(action="search")` for the same exact T187 title and target IDs also returned T159, T157,
  T160, T176, T58, and the live feedback batch, not T187.
- `git status --short --branch` showed branch `yuval.meiri/memory-os-phase0` with only the known
  user-owned untracked root `AGENTS.md`.
- Recent commits show T187 is committed as `88100e1 Record T187 stale handoff lifecycle packet`.
- PID `49349` remains live as `/Users/yuval.meiri/.local/bin/claude`; no process signal or PTY input
  was sent.
- Prior AI Council recall for T136 supports docs-only/default-deny audit packets for active rolling
  handoff search noise, while exact-gating lifecycle archive/apply, handoff semantics repair,
  ranking, `orient`, schema/storage/index, document-index changes, M6, and harness/settings writes.

Current file fingerprints before this packet:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 13294 | `24cdc620dd282e51f63f3c20e50ce76719cbac08ac2499cbf47896f343a294ca` |
| `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` | 315541 | `4b35f3e55152bf5aa1ba0339e6398538c5e0b0d7d440f99ad848b2a893da165e` |

## Completion Matrix Delta

| Area | State After T188 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | Recoverable and first | `orient` and direct search traces return current-plan memory first | T186 cleanup remains exact-gated |
| T187 repo evidence | Committed locally | Commit `88100e1` and file read | Document search still misses T187 until indexing is approved and run |
| Document visibility | Gap identified | Exact document-layer probes return older documents, not T187 | Requires exact T188 approval before indexing |
| Document-index behavior | Unchanged | This packet does not run indexing or change source | Future T188 execution may index only exact files |
| Lifecycle cleanup | Still gated | No archive, no `lint apply_safe` | Requires exact T187 approval |
| Native Claude cleanup | Still gated | PID `49349` remains live | Requires exact T186 approval |
| M6/migration | Still high-risk and gated | No M6 action in T188 | Requires separate approved scoping/dry-run/apply path |

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T188: index exact files T187 and MEMORY_OS_IMPLEMENTATION_PLAN from docs/BRAIN_HARNESS_T188_T187_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md. After fresh git/path/document-search/obligations evidence and no intervening writes, run docs(action="index") only for docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md and docs/MEMORY_OS_IMPLEMENTATION_PLAN.md, then run read-only document-search validation and write/commit the result report. Do not run T186, T187, lifecycle archive, lint apply_safe, ranking/orient/source changes, public MCP/schema/storage/index/document-index behavior changes, M6/migration/quarantine, native Claude, Claude Bridge, process signals, harness installs/settings/hooks/adapters, deletion, rollback, or user-owned-file edits.
```

Shorter approval, generic continuation, T186 approval, or T187 approval must not be treated as T188
approval.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before indexing:

- `git status --short --branch`
- path existence and regular-file checks for the two approved files
- byte count and SHA-256 for the two approved files
- read-only document-search probes proving whether T187 is still missing
- `obligations(action="doctor", project="engram")`

### Exact-File Indexing

Allowed only if preflight still matches this packet and no intervening writes occurred:

```text
docs(action="index", path="docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md")
docs(action="index", path="docs/MEMORY_OS_IMPLEMENTATION_PLAN.md")
```

No directory indexing, reindex plan/execute, cleanup plan/execute, quarantine review action,
document-index behavior change, schema/storage/index change, or source change is authorized.

### Post-Index Read-Only Validation

Allowed after the exact indexing calls:

- Re-run exact document-search probes for the T187 title and all three target IDs.
- Verify the T187 packet and implementation-plan note are visible, or record the exact miss.
- Write one result report under `docs/`.
- Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Commit only the intended documentation files.
- Capture current-plan memory after the commit.
- Submit telemetry feedback for assessed retrieval traces.

## Explicitly Forbidden

T188 does not authorize:

- executing T186 or sending any process signal;
- executing T187 or archiving any MemoryItem;
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

- approval is missing, conditional, ambiguous, or does not include the exact T188 wording and both
  target files;
- either target path is missing, not a regular file, or unexpectedly changed after final preflight
  without user re-approval;
- git status has unexpected tracked changes;
- obligations doctor reports an open obligation that changes the scope;
- any write occurs after the final fresh pre-index read and before the indexing calls;
- indexing appears to require directory-wide ingestion, behavior changes, schema/storage/index
  changes, reindex/cleanup/quarantine actions, source edits, process signals, lifecycle writes,
  M6/migration work, native Claude, Claude Bridge, or harness writes.
