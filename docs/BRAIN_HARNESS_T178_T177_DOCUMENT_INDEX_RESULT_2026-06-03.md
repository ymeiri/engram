# T178 T177 Document Index Result

Date: 2026-06-03
Status: complete as exact-file document-index visibility repair

## Scope

The user approved:

```text
Approve T177: index exact files T176 and MEMORY_OS_IMPLEMENTATION_PLAN.
```

This execution indexed only the two files named by T177:

- `docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

It did not execute native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, harness
install/settings/hook/adapter writes, lifecycle archive, `lint apply_safe`, M6 migration or
quarantine work, candidate decisions, deletion, cleanup, schema/storage/index behavior changes,
document-index behavior changes, public MCP changes, ranking changes, `orient` changes, rollback,
force-kill, old-binary reinstall, or user-owned-file edits.

## Research Framing

Question: can Engram safely make the latest T176 result report and updated central
implementation-plan matrix visible through document search without changing retrieval code,
creating MemoryItems for packet docs, or crossing any approval-gated product surface?

| Type | Result |
| --- | --- |
| Preferred | Partially supported. The T176 report now ranks first for its exact-title probe, and the implementation-plan T175/T176 matrix chunk is visible in the approved T175 execution probe. |
| Null | Still partly plausible for the exact T176 matrix-note wording: that probe remains dominated by older T174/T173/T161 content. |
| Simpler alternative | Still required for ambiguous future gates: repo files remain authoritative when semantic search returns noisy chunks. |
| Failure | Not observed. The operation stayed bounded to two exact file index writes and this report. |

## Preflight

Both approved paths existed as regular files.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md` | 4791 | `a1c002c64a84ed23a2385732f8a47f1c0049dbfbe6e1c4c48629419fe62e9d03` |
| `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` | 300477 | `c79e59287318947d73b5e20e80e1bceb11635d459b73a746ca3d4d9740efad37` |

## Index Execution

Pre-index document stats:

| Metric | Value |
| --- | ---: |
| Source count | 88 |
| Chunk count | 4240 |
| Searchable chunk count | 2228 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

Exact index calls:

| File | Documents Indexed | Chunks Created | Warnings |
| --- | ---: | ---: | --- |
| T176 result report | 1 | 8 | none |
| `MEMORY_OS_IMPLEMENTATION_PLAN.md` | 1 | 336 | none |

Post-index document stats:

| Metric | Value |
| --- | ---: |
| Source count | 89 |
| Chunk count | 4346 |
| Searchable chunk count | 2334 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

The stats show one new source and 106 additional searchable chunks. This is consistent with adding
the previously unindexed T176 result report and replacing the existing implementation-plan source
chunks with the current file contents.

## Validation Searches

Baseline searches before indexing:

| Query | Baseline Result |
| --- | --- |
| `T176 T175 Document Index Result` | T176 did not appear in the top five. |
| `T176 matrix note next product-moving gates exact T172 approval exact T174 approval` | Returned older indexed documents and an older implementation-plan chunk, not the newest T176/T177 matrix content. |
| `T175 document-index execution T172 T173 T174 37 searchable chunks` | Did not return the T176 report in the top five. |

Post-index validation:

| Query | Result |
| --- | --- |
| `T176 T175 Document Index Result` | T176 result report ranked first through third; top score `0.77586216`. |
| `T176 matrix note next product-moving gates exact T172 approval exact T174 approval` | Still noisy: top four results were T174, T173, T161, and T174; `MEMORY_OS_IMPLEMENTATION_PLAN.md` appeared fifth but on an older M6-gate matrix chunk, not the newest T176/T177 note. |
| `T175 document-index execution T172 T173 T174 37 searchable chunks` | T176 result report ranked first through fourth, and the newly indexed implementation-plan chunk containing the T175/T176 matrix notes ranked fifth with score `0.6187497`. |

## Completion Matrix Delta

| Area | State After T177 Execution | Remaining Gate |
| --- | --- | --- |
| T176 report visibility | Validated. Exact-title document search returns T176 first. | None for this document visibility target. |
| Central matrix visibility | Partially validated. The T175/T176 matrix chunk is reachable through the approved T175 execution probe, but the exact T176 matrix-note probe remains noisy. | Read repo docs for high-stakes gates; future document visibility repairs still require exact approval. |
| T172 native Claude validation | Unchanged. | Exact T172 approval still required before one native `/hooks` PTY session. |
| T174 M6 scoping | Unchanged. | Exact T174 approval still required before read-only M6 scoping execution. |
| M6 migration completion | Unchanged. | Candidate decisions, dry-run/apply plan, rollback evidence, telemetry readiness, and exact approval remain separate. |
| `orient`, ranking, schema/storage/index behavior | Unchanged. | No hot-path, ranking, public MCP, schema/storage, or document-index behavior work was run. |

## Decision

T177 is complete as a bounded document visibility repair. It materially improves T176 report
retrieval and partially improves central matrix retrieval for the approved execution-state probe.
It does not complete the Brain Harness goal and does not approve or execute T172, T174, candidate
decisions, migration apply, lifecycle cleanup, harness writes, ranking, `orient`, or any behavior
change.
