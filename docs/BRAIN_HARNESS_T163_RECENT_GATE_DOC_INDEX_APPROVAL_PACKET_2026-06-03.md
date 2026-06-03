# Brain Harness T163 Recent Gate Document Index Approval Packet

Status: pending exact user approval. No document indexing has been run.
Date: 2026-06-03
Scope: Proposed exact-file document-index visibility repair for recent Brain Harness gate docs

This packet asks whether to authorize a bounded document-index write for seven existing Brain
Harness documents. It does not authorize native Claude execution, Claude Bridge execution, harness
install/settings/hook/adapter writes, lifecycle archive, `lint apply_safe`, M6 migration or
quarantine work, candidate decisions, deletion, cleanup, schema/storage/index behavior changes,
document-index behavior changes, public MCP changes, ranking changes, or `orient` changes.

## Current Evidence

- T162 recorded that exact approval-packet searches for T154, T157, T160, and T125/T158 produced
  missing-context feedback because packet docs did not reliably surface.
- Fresh read-only `docs(action="stats")` still reports `source_count=78`, `chunk_count=4131`,
  `searchable_chunk_count=2119`, and `orphan_chunk_count=2012`, matching the post-T70 index state
  rather than showing any later exact-file indexing.
- Fresh read-only document searches did not surface the target recent packet/audit docs in the top
  five for exact title or approval-scope probes:
  - T162 query returned only older live-feedback and Claude smoke chunks.
  - T154 native-Claude approval query returned older live-feedback and research-method chunks.
  - T158/T125 quarantine approval query returned T40/T58/live-feedback/T69 chunks, not T158.
  - T160 wrong-scope lifecycle approval query returned T40/live-feedback/T58 chunks, not T160.
- T65 and T70 established the safe operating pattern for this class: prepare a default-deny packet,
  require exact user approval, index only named file paths through MCP `docs(action="index",
  path=...)`, then validate with read-only searches and record a result report.
- T70 also showed exact-file indexing is document visibility work only; it does not approve M6,
  lifecycle, ranking, `orient`, public MCP, schema/storage/index, document-index behavior, or
  harness changes.

## Research Question

Can Engram safely make the recent Brain Harness approval packets and gate-state audits visible
through document search without changing retrieval code, creating MemoryItems for packet docs, or
crossing any approval-gated product surface?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly the seven named recent gate/audit docs makes T154/T157/T158/T159/T160/T161/T162 recoverable through document search while preserving every underlying approval gate. |
| Null | The files index successfully, but semantic search remains noisy for exact approval phrases; repo files and current-plan memory remain authoritative. |
| Simpler alternative | Defer indexing and keep requiring repo-file inspection for exact approval phrases. |
| Failure | The operation expands into directory indexing, cleanup/reindex, code changes, MemoryItem creation, lifecycle mutation, M6/quarantine work, native Claude execution, or implied approval for the gates described by the docs. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T154_NATIVE_CLAUDE_VALIDATION_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T157_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T158_T125_QUARANTINE_INSPECTION_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T159_STALE_T146_LIMITATION_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T160_WRONG_SCOPE_CLAUDE_PROMPT_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T161_DUPLICATE_T135_COMPLETION_GATE_AUDIT_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T162_TELEMETRY_COVERAGE_FOLLOW_THROUGH_2026-06-03.md`
2. Index exactly those seven files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `Brain Harness T154 Native Claude Validation Approval Packet`
   - `Approve T154 native Claude non-session smoke.`
   - `Brain Harness T157 Stale Current Plan Lifecycle Approval Packet`
   - `Brain Harness T158 T125 Quarantine Inspection Approval Packet`
   - `Brain Harness T159 Stale T146 Limitation Lifecycle Approval Packet`
   - `Brain Harness T160 Wrong Scope Claude Prompt Lifecycle Approval Packet`
   - `Brain Harness T162 Telemetry Coverage Follow-Through`
4. Record a Markdown result report and commit documentation if documentation changes are made.
5. Submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- T154 appears in the top five document results for the exact title or exact approval phrase.
- T157, T158, T159, and T160 appear in the top five document results for exact title queries.
- T161 and T162 appear in the top five document results for exact title queries.
- Every underlying approval gate remains unchanged: T154, T157, T159, T160, and T125/T158 still
  require their own exact approval phrases before execution.
- No MemoryItem is created for these packet docs.
- No native Claude, Claude Bridge, harness write, lifecycle archive, `lint apply_safe`, M6/
  migration/quarantine action, candidate decision, deletion, cleanup, schema/storage/index behavior
  change, document-index behavior change, public MCP change, ranking change, or `orient` change
  occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- Any of the seven exact files is missing.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, run cleanup/reindex, or require
  a schema/storage migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems for packet docs.
- The result appears to run or require native Claude, Claude Bridge, harness writes, lifecycle
  mutation, `lint apply_safe`, M6 migration/quarantine work, candidate decisions, deletion, or
  cleanup.

## Approval Question

Reply exactly:

`Approve T163: index exact files T154, T157, T158, T159, T160, T161, and T162.`
