# Brain Harness T65 T59 Document Index Visibility Approval Packet

Status: Pending user approval. No document indexing has been run.
Date: 2026-06-01
Scope: Proposed bounded document-index visibility repair for T58/T59/T64 evidence docs

This packet asks whether to authorize a bounded document-index operation for three existing
Brain Harness evidence documents. It does not authorize M6 review export, review apply, candidate
decisions, deletion, lifecycle mutation, schema changes, document-index behavior changes, public
MCP changes, ranking changes, `orient` changes, or harness adapter/hook changes.

## Current Evidence

- T58 completed the approved inventory-only M6 scoping run and wrote
  `docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md`.
- T59 prepared the pending review-export approval packet at
  `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`.
- T64 found that current-plan retrieval is healthy, but exact T59 approval-packet visibility is
  incomplete for explicit `migration_review_export` prompts.
- Live document search did not surface the T59 document for exact title/path probes:
  - trace `019e7f6f-62df-7760-897b-0324806e5858`,
    query `Brain Harness T59 M6 Review Export Scope Proposal`;
  - trace `019e7f6f-bb49-7001-9d55-1aa91d99bafe`,
    query `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`;
  - trace `019e7f6f-638c-7f00-889a-594af7651f22`,
    query `Should we run migration_review_export now for the T58 M6 candidates?`.
- Source inspection shows unified document search currently uses semantic search over indexed
  chunks in `engram-index/src/search.rs`; ranking cannot surface a document that is not a candidate.
- A direct CLI check against the embedded store could not run while the daemon owned the RocksDB
  lock, so the MCP traces above are the live evidence.
- AI Council and Claude Bridge agreed that creating another T59 MemoryItem would create a parallel
  source of truth, and that ranking changes would not address an unindexed or non-candidate
  document.

## Research Question

Can Engram safely make the existing T58/T59/T64 evidence documents visible through document search
without changing retrieval code, creating parallel MemoryItems, or crossing any M6 migration gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly the T58, T59, and T64 evidence documents makes the authoritative T59 packet visible for exact title/path and review-export queries while preserving M6 default-deny behavior. |
| Null | The documents index successfully but semantic search still does not surface T59 reliably, so source docs remain manually authoritative and further retrieval design needs separate approval. |
| Simpler alternative | Defer indexing and continue requiring repo-file inspection before any T59/M6 gate decision. |
| Failure | The operation requires recursive indexing, changes document-index behavior, creates unrelated document churn, writes MemoryItems, or is mistaken for approval to run M6 review export/apply. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T64_POST_T63_CONTINUITY_AND_T59_VISIBILITY_AUDIT_2026-05-31.md`
2. Index exactly those three files through the existing document-index surface. Do not index the
   whole `docs/` directory and do not use recursive indexing.
3. Run read-only validation searches:
   - `Brain Harness T59 M6 Review Export Scope Proposal`
   - `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`
   - `Should we run migration_review_export now for the T58 M6 candidates?`
4. Record a Markdown result report and commit documentation if documentation changes are made.
5. Submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- The T59 document appears in the top five document results for exact title or exact path query.
- The explicit review-export query still preserves default-deny context and does not state or
  imply that `migration_review_export` is approved.
- No MemoryItem is created for T59.
- No M6 inventory, review export, review apply, candidate decision, lifecycle mutation, deletion,
  schema change, document-index behavior change, public MCP change, ranking change, `orient` change,
  or harness write occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, or ambiguous.
- Any of the three exact files is missing.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, or require a schema/storage
  migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems.
- The result appears to run or require M6 inventory, review export, review apply, candidate
  decisions, lifecycle mutation, deletion, cleanup, or harness writes.

## Approval Question

Do you approve a bounded document-index visibility repair that indexes exactly the three existing
files listed above, followed only by read-only retrieval validation, telemetry feedback, and a
Markdown report, with no M6 migration action, no MemoryItem creation, no lifecycle mutation, no
schema/storage/index behavior change, no public MCP change, no ranking or `orient` change, and no
harness adapter/hook change?
