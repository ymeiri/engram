# Brain Harness T212 T211 Document Index Result

Date: 2026-06-04
Status: completed exact-file document indexing

## Scope

This slice indexed only:

- `docs/BRAIN_HARNESS_T211_T209_T210_DOC_INDEX_RESULT_2026-06-04.md`

It did not index the whole docs tree, reindex existing sources, run M6 candidate decisions, run
`migration_review_status`, run apply/prioritize/export/rerun, mutate lifecycle state, archive
memory, delete data, change ranking or `orient`, change public MCP/schema/storage/index behavior,
change document-index behavior, run native Claude, run Claude Bridge, edit harness files, change
runtime configuration, or touch user-owned files.

## Before Indexing

Document stats before indexing:

| Metric | Value |
| --- | ---: |
| Source count | 97 |
| Chunk count | 4408 |
| Searchable chunk count | 2396 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

Pre-index search results:

- `Brain Harness T211 T209/T210 Document Index Result` did not return T211 in the top five; the top
  result was T40.
- `T211 T209 T210 document index result source_count 97 chunk_count 4408` returned older index
  reports above T211.

## Indexing Result

| File | Documents indexed | Chunks created | Warnings |
| --- | ---: | ---: | --- |
| T211 report | 1 | 1 | none |

## Validation

Post-index exact title search:

| Query | Result |
| --- | --- |
| `Brain Harness T211 T209/T210 Document Index Result` | T211 returned first with score `1.0`. |

Additional content search:

| Query | Result |
| --- | --- |
| `T211 T209 T210 document index result source_count 97 chunk_count 4408` | T211 returned in the top five, behind older index-result reports with similar stats language. |

Post-index document stats:

| Metric | Value |
| --- | ---: |
| Source count | 98 |
| Chunk count | 4409 |
| Searchable chunk count | 2397 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

The orphan count did not increase.

## Decision

T212 closes document visibility for the T211 report. It does not make candidate decisions,
authorize migration apply, or change the T210 future gate. The next M6 step remains human-provided
candidate-disposition authorization under T210A or T210B, or a separate deferral decision.
