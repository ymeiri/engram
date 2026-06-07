# Brain Harness T115 Post-T114 Document Visibility Audit

Status: Completed read-only document visibility audit; no indexing run
Date: 2026-06-01
Scope: Check whether document search can recover the latest Brain Harness evidence after T114.

This slice did not run `docs(action="index")`, `docs(action="plan")`, reindex, cleanup, orphan
recovery, M6 inspection, migration review export/apply, deletion, lifecycle mutation, ranking
changes, `orient` changes, public MCP changes, schema/storage/index behavior changes, document-index
behavior changes, MemoryItem creation for document packets, or harness adapter/hook changes.

## Research Question

After T114, does the existing document index recover the current Brain Harness evidence documents
well enough for agents to rely on document search, or do repo files/current-plan memory remain the
authoritative startup source until an exact indexing gate is approved?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T59 remains visible from the earlier T67 indexing, but newer T68/T69/T70/T113/T114 documents are not reliably recoverable because no later exact indexing was run. |
| Null | Document search now recovers T68/T69/T70/T113/T114 without additional indexing, so the T70 gate is less urgent. |
| Simpler alternative | Skip document search and keep relying on repo-file reads for all recent evidence. |
| Failure | The audit is misread as approval to index documents, inspect M6 export files, or broaden T70 beyond its exact approved phrase. |

## Measurement

Read-only document stats returned:

- `source_count=76`
- `chunk_count=4114`
- `searchable_chunk_count=2102`
- `orphan_chunk_count=2012`
- `embedding_dimension=384`

Read-only document-search results:

| Query | Top-five result | Interpretation |
| --- | --- | --- |
| `Brain Harness T59 M6 Review Export Scope Proposal` | T59 packet rank 1, score `0.7712747` | Earlier T67 indexing still makes T59 recoverable. |
| `Brain Harness T68 T59 Review Export Result count drift 0012-skip-plan` | T64 rank 1; T59/T58 also appear; T68 absent from top five | T68 remains weak or absent in document search. |
| `Brain Harness T69 T68 Count Drift Decision Packet Approve T69 inspect index.md and 0012-skip-plan.md` | T58 rank 1; T64 and T59 appear; T69 absent from top five | T69 remains weak or absent in document search. |
| `Brain Harness T70 T69 Document Visibility Audit And Index Packet Approve T70 index exact files T59 T68 T69` | T64 rank 1; T70 absent from top five | The T70 packet itself is not recoverable through document search. |
| `Brain Harness T114 Current-Plan Noise Fixture T114 current-plan noise fixture Claude Code user-stated instruction` | Research Method and Architecture chunks; T114 absent from top five | The latest T114 report is not recoverable through document search. |
| `Brain Harness T113 Post-T112 Startup Retrieval Validation` | Live Feedback, T58, and Architecture chunks; T113 absent from top five | The T113 report is not recoverable through document search. |

The T70 packet itself says no indexing has run since T67 and asks for this exact phrase before any
write:

`Approve T70: index exact files T59, T68, and T69.`

## Completion Matrix Delta

| Area | T115 state | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| T59 document visibility | Still validated | T59 title query returns T59 rank 1 | T59 content may still need refresh if edited again. |
| T68/T69 visibility | Still missing/weak | T68 and T69 absent from top-five document results for title/gate queries | T70 exact approval remains required before indexing T59/T68/T69. |
| T70 packet visibility | Missing/weak | T70 absent from top-five document results for exact packet query | Agents must read repo files/current memory before acting; no indexing approved. |
| T113/T114 visibility | Missing/weak | T113 and T114 absent from top-five document results for exact report queries | Do not silently broaden T70; prepare a separate exact packet if latest reports should be indexed. |
| Document index health | Risky | `orphan_chunk_count=2012` of `chunk_count=4114` | Cleanup/reindex/orphan recovery remains gated and was not run. |
| M6 migration | Still gated | No M6 inspection/export/apply ran | T69 count-drift inspection and later M6 writes require explicit approval. |

## Interpretation

Document search is not yet a trustworthy standalone source for recent Brain Harness continuity.
Agents should continue to treat `orient`, direct memory search, `handoff(get)`, and repo-file reads
as authoritative during startup. The original T70 exact-file indexing gate remains useful for
T59/T68/T69, but this audit also shows newer reports are missing from document search. Do not
broaden T70 implicitly; indexing T70/T113/T114 would require a separate exact-file approval packet
or a new explicitly approved scope.
