# T176 T175 Document Index Result

Date: 2026-06-03
Status: complete as exact-file document-index visibility repair

## Scope

The user approved:

```text
Approve T175: index exact files T172, T173, and T174.
```

This execution indexed only the three files named by T175. It did not execute native Claude,
Claude Bridge, Claude `/hooks`, prompt-bearing Claude, harness install/settings/hook/adapter
writes, lifecycle archive, `lint apply_safe`, M6 migration or quarantine work, candidate
decisions, deletion, cleanup, schema/storage/index behavior changes, document-index behavior
changes, public MCP changes, ranking changes, or `orient` changes.

## Research Framing

Question: can Engram safely make the newest Brain Harness approval and gate-state documents visible
through document search without changing retrieval code, creating MemoryItems for packet docs, or
crossing any approval-gated product surface?

| Type | Result |
| --- | --- |
| Preferred | Supported. Exact-file indexing made T172, T173, and T174 visible in the approved validation probes. |
| Null | Not supported for the approved probes. Search no longer missed the target docs in the top five. |
| Simpler alternative | Still available for unindexed future packets, but unnecessary for T172-T174 after this run. |
| Failure | Not observed. The operation stayed bounded to three exact file index writes and one report. |

## Preflight

All approved paths existed as regular files and were not symlinks.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md` | 10842 | `115181a2a5283606d7124735d537bd3b38b5c937a4f66f2d168d0e2aae3704ae` |
| `docs/BRAIN_HARNESS_T173_TELEMETRY_AND_STALE_APPROVAL_FOLLOW_THROUGH_2026-06-03.md` | 7350 | `dc66f8ff6d312784168388ee0c98cd7c8a40d6bd307c92ffe80bd60cecf0057e` |
| `docs/BRAIN_HARNESS_T174_M6_CANDIDATE_DECISION_DRY_RUN_SCOPING_APPROVAL_PACKET_2026-06-03.md` | 10914 | `f434608c36d28bdf42af93f6761ba12e78a9a0dae16505f5e782bf97ffb7ced3` |

## Index Execution

Pre-index document stats:

| Metric | Value |
| --- | ---: |
| Source count | 85 |
| Chunk count | 4203 |
| Searchable chunk count | 2191 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

Exact index calls:

| File | Documents Indexed | Chunks Created | Warnings |
| --- | ---: | ---: | --- |
| T172 packet | 1 | 15 | none |
| T173 report | 1 | 8 | none |
| T174 packet | 1 | 14 | none |

Post-index document stats:

| Metric | Value |
| --- | ---: |
| Source count | 88 |
| Chunk count | 4240 |
| Searchable chunk count | 2228 |
| Orphan chunk count | 2012 |
| Embedding dimension | 384 |

The stats delta matches the exact-file writes: three additional sources and 37 additional chunks.

## Validation Searches

Fresh baseline searches before indexing missed the target docs in the top five for all approved
probes. After indexing:

| Query | Target Result | Rank Evidence |
| --- | --- | --- |
| `T172 Native Claude Effective-Hook Validation Approval Packet` | T172 packet | Ranks 1-5 were all T172 chunks; top score `0.899001`. |
| `Approve T172: execute the native Claude effective-hook validation` | T172 packet | Ranks 1-5 were all T172 chunks; top score `0.81915694`. |
| `T173 Telemetry And Stale Approval Follow-Through` | T173 report | T173 ranked first through fourth; top score `0.9342734`. |
| `T174 M6 Candidate-Decision And Dry-Run Scoping Approval Packet` | T174 packet | Ranks 1-5 were all T174 chunks; top score `0.90834147`. |
| `Approve T174: execute the M6 candidate-decision and dry-run scoping packet` | T174 packet | Ranks 1-5 were all T174 chunks; top score `0.91186684`. |

## Completion Matrix Delta

| Area | State After T175 Execution | Remaining Gate |
| --- | --- | --- |
| T172 document visibility | Validated in document search | Exact T172 approval still required before one native `/hooks` PTY session. |
| T173 document visibility | Validated in document search | Telemetry confidence remains a sliding-window weak signal, not completion proof. |
| T174 document visibility | Validated in document search | Exact T174 approval still required before read-only M6 scoping execution. |
| M6 migration completion | Unchanged | Candidate decisions, dry-run/apply plan, rollback evidence, and exact approval remain separate. |
| `orient` and ranking | Unchanged | No hot-path or ranking work was run. |

## Decision

T175 is complete: the three recent gate documents are now indexed and visible through the approved
document-search probes. This improves document evidence retrieval only. It does not approve or
execute the T172 native Claude session, the T174 M6 scoping packet, candidate decisions, migration
apply, lifecycle cleanup, harness writes, or any hot-path behavior change.
