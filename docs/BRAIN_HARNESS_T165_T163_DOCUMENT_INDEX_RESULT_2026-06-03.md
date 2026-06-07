# Brain Harness T165 T163 Document Index Result

Date: 2026-06-03
Status: complete as exact-file document-index visibility repair
Scope: Execute the approved T163 exact-file indexing packet for T154, T157, T158, T159,
T160, T161, and T162

## Status

The user approved the exact T163 phrase:

```text
Approve T163: index exact files T154, T157, T158, T159, T160, T161, and T162.
```

This result records execution of only that approved scope. It did not run native Claude, use
Claude Bridge, install or edit harness adapters/settings/hooks, archive lifecycle memory, run
`lint apply_safe`, run M6 migration/quarantine commands, inspect quarantine candidates, make
candidate decisions, delete data, change ranking or `orient`, change public MCP parameters,
change schema/storage/index behavior, change document-index behavior, create MemoryItems for
packet docs, or touch user-owned files.

## Research Question

Can Engram safely make the recent Brain Harness approval packets and gate-state audits visible
through document search without changing retrieval code, creating MemoryItems for packet docs, or
crossing any approval-gated product surface?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Exact-file indexing makes T154/T157/T158/T159/T160/T161/T162 recoverable through document search while preserving every underlying approval gate. | Mostly supported. Six target title probes and the T154 approval phrase probe returned target docs first; the synthetic T154 title wording from T163 remained noisy. |
| Null | The files index successfully, but semantic search remains noisy for exact approval phrases. | Partially supported for the synthetic T154 title wording only. |
| Simpler alternative | Keep requiring repo-file inspection for exact approval phrases. | No longer needed for these target docs, except repo files remain authoritative for exact approval text. |
| Failure | The operation expands beyond exact files or implies approval for underlying gates. | Not observed. |

## Preflight

- Git status before indexing remained clean except the pre-existing user-owned untracked root
  `AGENTS.md`.
- All seven approved target files existed.
- Pre-index document stats:
  - `source_count=78`
  - `chunk_count=4131`
  - `searchable_chunk_count=2119`
  - `orphan_chunk_count=2012`

## Index Execution

| File | Documents Indexed | Chunks Created | Warnings |
| --- | ---: | ---: | --- |
| `BRAIN_HARNESS_T154_NATIVE_CLAUDE_VALIDATION_APPROVAL_PACKET_2026-06-03.md` | 1 | 11 | none |
| `BRAIN_HARNESS_T157_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 1 | 12 | none |
| `BRAIN_HARNESS_T158_T125_QUARANTINE_INSPECTION_APPROVAL_PACKET_2026-06-03.md` | 1 | 10 | none |
| `BRAIN_HARNESS_T159_STALE_T146_LIMITATION_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 1 | 12 | none |
| `BRAIN_HARNESS_T160_WRONG_SCOPE_CLAUDE_PROMPT_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` | 1 | 12 | none |
| `BRAIN_HARNESS_T161_DUPLICATE_T135_COMPLETION_GATE_AUDIT_2026-06-03.md` | 1 | 9 | none |
| `BRAIN_HARNESS_T162_TELEMETRY_COVERAGE_FOLLOW_THROUGH_2026-06-03.md` | 1 | 6 | none |

Total: 7 documents indexed, 72 chunks created, no warnings.

Post-index document stats:

- `source_count=85`
- `chunk_count=4203`
- `searchable_chunk_count=2191`
- `orphan_chunk_count=2012`

The stats moved by exactly the approved write shape: +7 sources, +72 chunks, +72 searchable
chunks, and no orphan increase.

## Validation

| Query | Result |
| --- | --- |
| `Brain Harness T154 Native Claude Validation Approval Packet` | Did not return T154 in the top five. The indexed T154 document's actual title is `T154 Native Claude Non-Session Smoke Approval Packet`, so this probe remains noisy. |
| `Approve T154 native Claude non-session smoke.` | Passed. T154 returned first and the approval wording chunk returned second. |
| `T154 Native Claude Non-Session Smoke Approval Packet` | Passed. T154 returned first. |
| `Brain Harness T157 Stale Current Plan Lifecycle Approval Packet` | Passed. T157 returned first. |
| `Brain Harness T158 T125 Quarantine Inspection Approval Packet` | Passed. T158 returned first. |
| `Brain Harness T159 Stale T146 Limitation Lifecycle Approval Packet` | Passed. T159 returned first. |
| `Brain Harness T160 Wrong Scope Claude Prompt Lifecycle Approval Packet` | Passed. T160 returned first. |
| `Brain Harness T161 Duplicate T135 Completion Gate Audit` | Passed. T161 returned first. |
| `Brain Harness T162 Telemetry Coverage Follow-Through` | Passed. T162 returned first. |

## Completion Matrix

| Area | Current State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T163 document visibility | Implemented and validated with caveat | Seven files indexed; stats +7/+72; six target title probes plus T154 approval phrase and actual title probe pass | Synthetic T154 title wording remains noisy |
| Current-plan retrieval | Healthy for this slice | Lean `orient` trace `019e8d45-ef96-78c2-97b2-6475c9382d66` returned T164 first | Older handoffs and stale current-plan memory remain noisy |
| Lifecycle cleanup | Missing / exact-gated | T157/T159/T160 docs are now searchable | Exact lifecycle approvals still required before any archive/reject/supersede write |
| Native Claude/effective hooks | Missing / exact-gated | T154 approval packet is now searchable by actual title and approval phrase | Exact T154 approval still required before native Claude non-session smoke |
| M6 migration/quarantine | Missing / high-risk gated | T158/T125 packet is now searchable | Exact quarantine inspection and later separate apply/deletion approval remain required |
| Legacy substrate | Preserved | No migration apply, deletion, schema/storage/index behavior change, or legacy simplification occurred | Simplification remains eval- and approval-gated |

## Decision

T163 is complete as a bounded document-visibility repair. The indexed packet docs are now
recoverable enough to support future exact-gate work, with one recorded caveat: future agents
should query the actual T154 title or approval phrase, not the synthetic T163 wording
`Brain Harness T154 Native Claude Validation Approval Packet`.

The next product-moving work remains separately exact-gated. T163 does not approve T154 native
Claude execution, T157/T159/T160 lifecycle archives, or T125/T158 M6 quarantine inspection.
