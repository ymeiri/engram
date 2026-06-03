# T177 T176 Matrix Document Index Approval Packet

Date: 2026-06-03
Status: pending exact user approval. No document indexing has been run by this packet.

## Scope

This is a docs-only/default-deny approval packet for a bounded document-visibility repair after
T176. It asks whether to index exactly two authoritative files:

- `docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

This packet does not execute native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude,
harness install/settings/hook/adapter writes, lifecycle archive, `lint apply_safe`, M6 migration or
quarantine work, candidate decisions, deletion, cleanup, schema/storage/index behavior changes,
document-index behavior changes, public MCP changes, ranking changes, `orient` changes, rollback,
force-kill, old-binary reinstall, or user-owned-file edits.

## Current Evidence

- T175 indexed exactly T172, T173, and T174 and committed the result as T176.
- T176 then updated the central implementation-plan matrix with commit `e957073`.
- Fresh document stats still match the T175 post-index state:
  - `source_count=88`
  - `chunk_count=4240`
  - `searchable_chunk_count=2228`
  - `orphan_chunk_count=2012`
- Fresh document search for `T176 T175 Document Index Result` did not return the T176 report in the
  top five.
- Fresh document search for `T176 matrix note next product-moving gates exact T172 approval exact
  T174 approval` returned older indexed docs and an old `MEMORY_OS_IMPLEMENTATION_PLAN.md` chunk,
  not the newly committed T176 matrix note.
- Lean `orient` and direct unified `search` recover current-plan memory
  `019e8de9-b376-7f23-8faa-d2e5dab5e935` first, so this is a document-search visibility gap, not
  evidence for ranking or `orient` changes.

## Research Question

Can Engram safely make the latest T176 result report and the updated central implementation-plan
matrix visible through document search without changing retrieval code, creating MemoryItems for
packet docs, or crossing any approval-gated product surface?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly the T176 result report and the central implementation plan makes the latest T175/T176 state recoverable through document search while preserving every underlying approval gate. |
| Null | The files index successfully, but semantic search still returns older chunks; repo files and current-plan memory remain authoritative. |
| Simpler alternative | Defer indexing and keep requiring repo-file inspection for the latest T176 result and matrix note. |
| Failure | The operation expands into directory indexing, cleanup/reindex, code changes, MemoryItem creation, lifecycle mutation, M6 work, native Claude execution, ranking changes, or implied approval for the T172/T174 gates described by the docs. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
2. Index exactly those two files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `T176 T175 Document Index Result`
   - `T176 matrix note next product-moving gates exact T172 approval exact T174 approval`
   - `T175 document-index execution T172 T173 T174 37 searchable chunks`
4. Record a Markdown result report and commit documentation if documentation changes are made.
5. Submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- The T176 report appears in the top five document results for its exact title.
- The updated `MEMORY_OS_IMPLEMENTATION_PLAN.md` T176 matrix note appears in the top five for the
  matrix-note or T175 execution probe.
- Every underlying approval gate remains unchanged:
  - T172 still requires exact approval before native Claude effective-hook validation.
  - T174 still requires exact approval before read-only M6 candidate-decision/dry-run scoping.
  - Candidate decisions, dry-run apply, write apply, deletion, native Claude prompt-bearing
    validation, and lifecycle cleanup remain separately gated.
- No MemoryItem is created for these document artifacts.
- No native Claude, Claude Bridge, Claude `/hooks`, harness write, lifecycle archive,
  `lint apply_safe`, M6/migration/quarantine action, candidate decision, deletion, cleanup,
  schema/storage/index behavior change, document-index behavior change, public MCP change,
  ranking change, or `orient` change occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- Either exact file is missing.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, run cleanup/reindex, or require
  a schema/storage migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems for packet docs.
- The result appears to run or require native Claude, Claude Bridge, Claude `/hooks`,
  prompt-bearing Claude, harness writes, lifecycle mutation, `lint apply_safe`, M6 migration or
  quarantine work, candidate decisions, deletion, or cleanup.

## Approval Question

Reply exactly:

```text
Approve T177: index exact files T176 and MEMORY_OS_IMPLEMENTATION_PLAN.
```
