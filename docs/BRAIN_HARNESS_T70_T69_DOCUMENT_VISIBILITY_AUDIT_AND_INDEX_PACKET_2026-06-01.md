# Brain Harness T70 T69 Document Visibility Audit And Index Packet

Status: Pending explicit user approval. No document indexing has been run.
Date: 2026-06-01
Scope: Read-only document visibility audit plus proposed exact-file indexing for T59/T68/T69

This packet asks whether to authorize a bounded document-index write for three existing Brain
Harness evidence documents. It does not authorize M6 review-export inspection, review apply,
candidate decisions, deletion, lifecycle mutation, schema changes, document-index behavior changes,
public MCP changes, ranking changes, `orient` changes, or harness adapter/hook changes.

T69 remains the gate for the T68 count-drift inspection. This packet is only about document search
visibility for the repo evidence that explains that gate.

## Current Evidence

- T67 indexed the then-current T58, T59, and T64 documents. After that, T59 was edited again and
  T68/T69 were created.
- Read-only document searches for the T69 title, filename, and exact approval phrase did not
  surface the T69 packet in the top results.
- Read-only document searches for the T68 title and count-drift wording did not surface the T68
  result report in the top results.
- Read-only document search still surfaced stale T59 indexed text saying the review export was
  pending and no export had run, which conflicts with the current repo document after T68.
- Source inspection confirms the existing exact-file indexing path can refresh an already-indexed
  file without a code or schema change:
  - `DocumentService::index_file` finds an existing source by path and reuses its source identity.
  - `reuse_existing_source_identity` assigns the existing `doc_source` id to the new chunks.
  - `DocumentRepo::save_chunks` deletes existing chunks for that source before inserting the new
    chunks.
  - The MCP `docs(action="index", path=...)` handler accepts a single file path and calls
    `service.index_file(path)`.

## Research Question

Can Engram safely refresh document-search visibility for the T59 review-export packet and the new
T68/T69 count-drift evidence without changing retrieval code, creating MemoryItems, or crossing any
M6 migration gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly T59, T68, and T69 makes current evidence recoverable through document search and replaces stale chunks for the already-indexed T59 source. |
| Null | The files index successfully, but semantic document search still does not reliably surface them, so repo docs remain manually authoritative and further retrieval work needs a separate approved slice. |
| Simpler alternative | Defer indexing and continue requiring repo-file inspection before count-drift or M6 decisions. |
| Failure | The operation expands into directory indexing, cleanup/reindex, code changes, MemoryItem creation, M6 inspection, lifecycle mutation, or an implied approval to proceed with migration. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T68_T59_REVIEW_EXPORT_RESULT_2026-06-01.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T69_T68_COUNT_DRIFT_DECISION_PACKET_2026-06-01.md`
2. Index exactly those three files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `Brain Harness T59 M6 Review Export Scope Proposal`
   - `Brain Harness T68 T59 Review Export Result count drift 0012-skip-plan`
   - `Brain Harness T69 T68 Count Drift Decision Packet`
   - `Approve T69: inspect index.md and 0012-skip-plan.md.`
4. Record a Markdown result report and commit documentation if documentation changes are made.
5. Submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- T59 appears in the top five document results for exact title or filename-stem query with current
  post-T68 content.
- T68 appears in the top five document results for title or count-drift query.
- T69 appears in the top five document results for title or exact approval-phrase query.
- The T69 inspection gate remains unchanged: the exact T69 approval phrase is still required before
  reading `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/index.md` or
  `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0012-skip-plan.md`.
- No MemoryItem is created for T59, T68, or T69.
- No M6 review-export inspection, review apply, candidate decision, deletion, lifecycle mutation,
  schema change, document-index behavior change, public MCP change, ranking change, `orient`
  change, or harness write occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, or ambiguous.
- Any of the three exact files is missing.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, run cleanup/reindex, or require
  a schema/storage migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems.
- The result appears to run or require M6 review-export inspection, review apply, candidate
  decisions, lifecycle mutation, deletion, cleanup, or harness writes.

## Approval Question

Reply exactly:

`Approve T70: index exact files T59, T68, and T69.`
