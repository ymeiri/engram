# Brain Harness T175 Recent Gate Document Index Approval Packet

Status: pending exact user approval. No document indexing has been run.
Date: 2026-06-03
Scope: Proposed exact-file document-index visibility repair for recent Brain Harness gate docs
T172, T173, and T174.

This packet asks whether to authorize a bounded document-index write for three existing Brain
Harness documents. It does not authorize native Claude execution, Claude Bridge execution, Claude
`/hooks`, prompt-bearing Claude, harness install/settings/hook/adapter writes, lifecycle archive,
`lint apply_safe`, M6 migration or quarantine work, candidate decisions, deletion, cleanup,
schema/storage/index behavior changes, document-index behavior changes, public MCP changes,
ranking changes, or `orient` changes.

## Current Evidence

- T163/T165 established the prior safe pattern for this class: prepare a default-deny packet,
  require exact user approval, index only named file paths through MCP `docs(action="index",
  path=...)`, then validate with read-only searches and record a result report.
- T165 indexed T154/T157/T158/T159/T160/T161/T162 and improved visibility for those target docs,
  while preserving every underlying approval gate.
- T172, T173, and T174 were committed after the T163/T165 indexing slice:
  - `e9e4993 Record T172 native Claude hook packet`
  - `c688a4f Record T173 telemetry follow-through`
  - `17f4db0 Record T174 M6 scoping packet`
- Fresh document-only searches on 2026-06-03 did not surface the new target docs in the top five:
  - Trace `019e8db8-3c71-7f03-8890-43d3d9a1ad75` queried
    `T172 Native Claude Effective Hook Validation Approval Packet` and returned T154 chunks.
  - Trace `019e8db8-3d36-70c2-b17e-62d1cc856e07` queried
    `Approve T172 execute the native Claude effective-hook validation` and returned T154 chunks.
  - Trace `019e8db8-3df3-72b3-8db8-13474a36450b` queried
    `T173 Telemetry And Stale Approval Follow-Through` and returned T162/T161 chunks.
  - Trace `019e8db8-3eb1-7173-ba41-555f8b942fff` queried
    `T174 M6 Candidate-Decision And Dry-Run Scoping Approval Packet` and returned
    T162/T69/T158 chunks.
  - Trace `019e8db8-3f74-7cf3-abce-d763884f1c2a` queried
    `Approve T174 execute the M6 candidate-decision and dry-run scoping packet` and returned
    T158/T162/T69 chunks.
  - Trace `019e8db8-4033-7ef2-bdf1-a8560b6a7c31` queried the T174 filename stem and returned
    T160/T159/T157/T69 chunks.
- Current `orient` and direct memory search still recover the active current-plan memory first, so
  this is a document-visibility gap, not evidence for ranking changes or `orient` expansion.

## Research Question

Can Engram safely make the newest Brain Harness approval and gate-state documents visible through
document search without changing retrieval code, creating MemoryItems for packet docs, or crossing
any approval-gated product surface?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly the three named recent gate docs makes T172/T173/T174 recoverable through document search while preserving every underlying approval gate. |
| Null | The files index successfully, but semantic search remains noisy for exact approval phrases; repo files and current-plan memory remain authoritative. |
| Simpler alternative | Defer indexing and keep requiring repo-file inspection for exact approval phrases. |
| Failure | The operation expands into directory indexing, cleanup/reindex, code changes, MemoryItem creation, lifecycle mutation, M6 work, native Claude execution, or implied approval for the gates described by the docs. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T173_TELEMETRY_AND_STALE_APPROVAL_FOLLOW_THROUGH_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T174_M6_CANDIDATE_DECISION_DRY_RUN_SCOPING_APPROVAL_PACKET_2026-06-03.md`
2. Index exactly those three files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `T172 Native Claude Effective-Hook Validation Approval Packet`
   - `Approve T172: execute the native Claude effective-hook validation`
   - `T173 Telemetry And Stale Approval Follow-Through`
   - `T174 M6 Candidate-Decision And Dry-Run Scoping Approval Packet`
   - `Approve T174: execute the M6 candidate-decision and dry-run scoping packet`
4. Record a Markdown result report and commit documentation if documentation changes are made.
5. Submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- T172 appears in the top five document results for exact title or exact approval phrase.
- T173 appears in the top five document results for exact title.
- T174 appears in the top five document results for exact title, filename-stem, or exact approval
  phrase.
- Every underlying approval gate remains unchanged:
  - T172 still requires its own exact approval before any native Claude PTY session.
  - T174 still requires its own exact approval before any M6 readiness/scoping execution.
  - Candidate decisions, dry-run apply, write apply, deletion, native Claude prompt-bearing
    validation, and broad lifecycle cleanup remain separately gated.
- No MemoryItem is created for these packet docs.
- No native Claude, Claude Bridge, Claude `/hooks`, harness write, lifecycle archive,
  `lint apply_safe`, M6/migration/quarantine action, candidate decision, deletion, cleanup,
  schema/storage/index behavior change, document-index behavior change, public MCP change,
  ranking change, or `orient` change occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- Any of the three exact files is missing.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, run cleanup/reindex, or require
  a schema/storage migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems for packet docs.
- The result appears to run or require native Claude, Claude Bridge, Claude `/hooks`,
  prompt-bearing Claude, harness writes, lifecycle mutation, `lint apply_safe`, M6 migration or
  quarantine work, candidate decisions, deletion, or cleanup.

## Approval Question

Reply exactly:

`Approve T175: index exact files T172, T173, and T174.`
