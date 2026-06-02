# Brain Harness T70 Exact File Index Result

Status: Completed approved exact-file indexing
Date: 2026-06-02
Scope: Index exactly T59, T68, and T69 report files

The user approved the exact T70 gate:

```text
Approve T70: index exact files T59, T68, and T69.
```

Codex ran exactly three `docs(action="index", path=...)` calls:

```text
/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md
/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T68_T59_REVIEW_EXPORT_RESULT_2026-06-01.md
/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T69_T68_COUNT_DRIFT_DECISION_PACKET_2026-06-01.md
```

This did not run M6 inventory/export/status/prioritize/apply/rerun, inspect candidate files, make
candidate decisions, delete data, mutate lifecycle state, change schema/storage/index behavior,
change public MCP behavior, change ranking, expand `orient`, or write harness adapters/hooks.

## Research Question

Does exact-file indexing improve document-search visibility for the T59, T68, and T69 evidence
files without broadening document-index behavior or crossing M6 gates?

## Measurement

Before indexing:

- `Brain Harness T59 M6 Review Export Scope Proposal` returned T59 at rank 1.
- `Brain Harness T68 T59 Review Export Result` did not return T68 in the top five.
- `Brain Harness T69 T68 Count Drift Decision Packet` did not return T69 in the top five.

Indexing results:

| File | Documents indexed | Chunks created | Warnings |
| --- | ---: | ---: | --- |
| T59 | 1 | 9 | none |
| T68 | 1 | 8 | none |
| T69 | 1 | 9 | none |

After indexing:

- `Brain Harness T68 T59 Review Export Result` returned T68 at rank 1.
- `Brain Harness T69 T68 Count Drift Decision Packet` returned T69 in the top five, including the
  stop-conditions chunk at rank 1 and the document opening chunk at rank 2.
- `BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md` returned T59 at rank 1.
- `T59 M6 review export scope proposal exact approval packet` returned T59 at rank 1.
- `Brain Harness T59 M6 Review Export Scope Proposal` remained noisy after reindexing and did not
  return T59 in the tested top five.
- `docs(action="stats")` reported `source_count=78`, `chunk_count=4131`,
  `searchable_chunk_count=2119`, and `orphan_chunk_count=2012`.

## Result

T70 partially closes the document-visibility gap. T68 and T69 now surface for exact-title probes,
and T59 remains recoverable through filename-stem and scoped approval phrasing. The exact T59 title
probe is still noisy after reindexing, so repo files and current-plan memory remain authoritative
for M6 decisions.

T70 is document-index visibility work only. It does not approve M6 candidate inspection, M6
status/prioritize/apply/rerun, candidate decisions, deletion, lifecycle mutation, ranking,
`orient`, public MCP/schema/storage/index behavior changes, document-index behavior changes, or
harness writes.
