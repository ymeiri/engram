# Brain Harness T67 T65 Document Index Result

Status: Completed approved T65 exact-file document indexing
Date: 2026-06-01
Scope: Exact document-index visibility repair for the T58, T59, and T64 evidence docs

This report records the result of the user-approved T65 indexing slice. It ran only three exact
`docs(action="index", path=...)` calls through the Engram MCP surface. It did not run M6 review
export, review apply, candidate decisions, deletion, lifecycle mutation, schema/storage or
document-index behavior changes, public MCP changes, ranking changes, `orient` changes, MemoryItem
creation for T59, or harness adapter/hook changes.

## Approved Files

- `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md`
- `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`
- `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T64_POST_T63_CONTINUITY_AND_T59_VISIBILITY_AUDIT_2026-05-31.md`

## Research Question

Can exact-file MCP document indexing make the authoritative T59 review-export packet recoverable
through document search without changing ranking behavior, expanding `orient`, creating a parallel
T59 MemoryItem, or running any M6 migration action?

## Hypotheses

- Preferred: exact-file indexing improves T59 document-search visibility for title, filename, and
  explicit review-export prompts while preserving the M6 default-deny gate.
- Null: indexing succeeds but T59 is still not visible in useful document-search probes.
- Simpler alternative: continue relying on repo docs as source of truth and require agents to read
  the T59 file directly before M6 decisions.
- Failure: the slice broadens beyond the approved files, mutates non-document state, or is mistaken
  for M6 review-export approval.

## Measurement

Preflight confirmed the three approved files existed:

- T58 report: `5991` bytes.
- T59 review-export packet: `5197` bytes.
- T64 continuity/visibility audit: `5404` bytes.

Index results:

- T58: `documents_indexed=1`, `chunks_created=11`, no warnings.
- T59: `documents_indexed=1`, `chunks_created=9`, no warnings.
- T64: `documents_indexed=1`, `chunks_created=8`, no warnings.

Document-search validation:

- Query `Brain Harness T59 M6 Review Export Scope Proposal` returned the T59 packet rank 1 with
  score `0.7712747`.
- Query `BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31` returned the T59 packet rank
  1 with score `0.7408934`.
- Query `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md` returned T59 chunks
  at ranks 4 and 5. This is useful top-five recovery, but not strong exact-path ranking.
- Query `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md`
  did not return T59 in the top five. Absolute path search remains weak.
- Query `Should we run migration_review_export now for the T58 M6 candidates?` returned T59 chunks
  at ranks 1, 3, and 5, and T64 chunks at ranks 2 and 4.

Unified-search validation:

- Trace `019e8216-1263-7282-883b-3e0bd0c2b6ce` for `Should we run migration_review_export now for
  the T58 M6 candidates?` returned active M6 migration-gate memory first and also returned T59/T64
  document evidence. This preserves default-deny behavior: the search provides evidence for the
  pending review-export packet, not authorization to run review export.

Incidental read-only `tool_intel_stats` calls ran during tool-surface setup. They did not mutate
state and were not part of the approved index operation.

## Result

T65 indexing is complete with a partial-success result:

- Useful T59 visibility is repaired for title, filename-stem, and explicit review-export document
  probes.
- Relative-path semantic search finds T59 in the top five but not first.
- Absolute-path semantic search remains unreliable.
- M6 review export remains unapproved. The T59 document is now easier to recover, but it is still
  an approval packet only.

## Completion Matrix Delta

| Area | Status After T67 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T59 document visibility | Partially improved | Exact-file indexing produced 9 T59 chunks; title and filename-stem document searches return T59 rank 1; explicit review-export prompt returns T59/T64 document evidence | Absolute-path semantic search does not return T59 top-five; this slice does not change ranking behavior |
| M6 migration gate | Still gated | Unified search trace `019e8216-1263-7282-883b-3e0bd0c2b6ce` returns active migration-gate memory first for explicit review-export prompt | Do not run review export without separate explicit approval of the T59 scope |
| Hot path / ranking / schema | Unchanged | No source edits and no public MCP, `orient`, ranking, schema, or storage changes | Future ranking work still requires a separate approved prompt-class slice |

## Next Gate

The next executable M6 action remains the T59 review-export scope. It requires separate explicit
user approval and should still stop on path preflight failure, candidate-count drift, unexpected
write/apply behavior, or any request to include lifecycle, deletion, schema/storage/index,
ranking, public MCP, `orient`, or harness changes.
