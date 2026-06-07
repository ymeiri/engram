# T174 M6 Candidate-Decision And Dry-Run Scoping Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval for the next M6 migration-readiness step after the
T68 review-export snapshot was fully inspected.

It does not execute M6 status, prioritize, apply, rerun, review export, lifecycle mutation,
candidate decisioning, deletion, ranking/`orient` changes, public MCP changes, schema/storage/index
changes, document-index behavior changes, native Claude, Claude Bridge, harness install, settings
or hook edits, rollback, force-kill, old-binary reinstall, or user-owned-file adoption.

The future approved slice is intentionally not a migration apply. It is only meant to produce a
read-only decision-readiness matrix, a dry-run observability plan, and a later approval packet shape
for candidate decisions or write-apply if evidence supports that path.

## Current Evidence

- T68 wrote the review-export snapshot at:
  `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export`.
- T123 and T124 inspected the nine `review` candidates, files 0001-0009, from that snapshot.
- T169 inspected the two `quarantine` candidates, files 0010-0011, from that snapshot.
- T169 explicitly did not decide, accept, edit, reject, promote, archive, delete, apply, or migrate
  any candidate and did not run status/prioritize/apply/rerun.
- T173 restored the current 50-trace telemetry confidence gate to 30/50 feedback traces, 60%
  coverage, five feedback-bearing intents, and zero bad-memory use, while preserving that the pass
  is a sliding-window weak signal and not migration readiness.
- T172 native Claude effective-hook validation remains separately exact-gated and unexecuted.
- AI Council recall found no prior matching decision for this exact T174 packet scope. A new
  AI Council broadcast agreed on default-deny boundaries and warned that T173 telemetry must not be
  reinterpreted as migration readiness.

## Research Question

Can Engram prepare a narrowly bounded, read-only M6 candidate-decision readiness and dry-run
scoping slice that moves migration completion forward without making candidate decisions or
crossing into write apply, deletion, lifecycle mutation, ranking changes, or harness/native-Claude
work?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A default-deny packet can define the next read-only M6 step: consolidate inspected candidate evidence, run only a fresh status/readiness check if approved, define dry-run observability requirements, and produce a later decision/apply approval shape. |
| Null | Preparing this packet adds no useful progress because candidate decisions and write apply are still gated. |
| Simpler alternative | Stop at T172 and wait for exact native Claude approval before any M6 planning. |
| Failure | The packet smuggles in candidate decisions, status/prioritize/apply/rerun, lifecycle mutation, deletion, ranking/`orient` change, or a claim that telemetry confidence equals migration readiness. |

## AI Council Synthesis

The useful consensus from the Council was:

- keep the packet docs-only/default-deny until exact user approval;
- separate evidence completeness from candidate decisions;
- require hard stops before any write-capable path;
- record T173 as observability confidence, not migration readiness;
- require a later dry-run/apply approval with exact planned effects, rollback, and fresh evidence.

One recommendation is intentionally not adopted: simulated accept/reject/defer dispositions are too
close to candidate decisions for this slice. T174 should allow a readiness matrix with
`known`/`unknown`/`blocked` evidence states only, not proposed dispositions.

## Proposed Approved Read-Only Scope

To authorize execution, reply exactly:

```text
Approve T174: execute the M6 candidate-decision and dry-run scoping packet from docs/BRAIN_HARNESS_T174_M6_CANDIDATE_DECISION_DRY_RUN_SCOPING_APPROVAL_PACKET_2026-06-03.md. Read only the committed T123/T124/T169/T173 reports, the existing T68 review-export snapshot index and exact candidate files 0001-0011, and at most one read-only M6 review status/readiness check for that snapshot; write only the T174 result report and implementation-plan note. Do not run prioritize/apply/rerun/review-export, make candidate decisions, mutate lifecycle state, delete, change ranking/orient/public MCP/schema/storage/index/document-index behavior, run native Claude or Claude Bridge, edit harness files, or touch user-owned files.
```

Shorter approval, generic continuation, T172 approval, or M6 write-apply approval wording must not
be treated as T174 approval.

## If Approved: Authorized Operations

The future T174 execution may:

1. Re-read committed reports:
   - `docs/BRAIN_HARNESS_T123_M6_CANDIDATE_0001_0004_INSPECTION_RESULT_2026-06-02.md`
   - `docs/BRAIN_HARNESS_T124_M6_CANDIDATE_0005_0009_INSPECTION_RESULT_2026-06-02.md`
   - `docs/BRAIN_HARNESS_T169_T125_QUARANTINE_INSPECTION_REPORT_2026-06-03.md`
   - `docs/BRAIN_HARNESS_T173_TELEMETRY_AND_STALE_APPROVAL_FOLLOW_THROUGH_2026-06-03.md`
2. Validate paths and read only these existing snapshot files:
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/index.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0001-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0002-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0003-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0004-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0005-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0006-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0007-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0008-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0009-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0010-*.md`
   - `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0011-*.md`
3. Run at most one read-only M6 review status/readiness check for the exact T68 snapshot if the
   tool path is confirmed to be read-only before invocation.
4. Produce a docs-only result report with:
   - a candidate evidence-readiness matrix using only `known`, `unknown`, `blocked`, or
     `not-applicable`;
   - snapshot provenance consistency;
   - quarantine boundary notes for candidates 0010-0011;
   - dry-run observability requirements;
   - explicit stop conditions for any later candidate-decision or write-apply packet;
   - a statement of whether a later approval packet is justified.
5. Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
6. Commit only the intended documentation files.
7. Capture current-plan memory and submit telemetry feedback for assessed retrieval traces.

## Explicitly Forbidden

T174 does not authorize:

- candidate decisions, including accept, reject, edit, defer, promote, archive, unquarantine, or
  final disposition recommendations;
- `migration_review_prioritize`, `migration_review_apply`, `migration_review_export`, reruns, or
  any equivalent write-capable M6 command;
- applying a dry-run result or treating read-only status output as approval to apply;
- writing or mutating Memory OS candidate state, lifecycle state, active memory, review queues,
  quarantine state, KnowledgeCommits, vault files, or legacy data;
- deletion, cleanup, broad legacy simplification, direct legacy deprecation, or rollback;
- ranking/`orient`, public MCP request/response, schema/storage/index, or document-index behavior
  changes;
- native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, harness install, settings
  edits, hook edits, adapter edits, or `adopt_user_owned`;
- editing or staging root `AGENTS.md` or other user-owned files.

## Measurements

The T174 result, if approved and executed, must measure:

| Measurement | Required Output |
| --- | --- |
| Candidate evidence completeness | For each candidate 0001-0011: evidence inspected, source kind, proposed scope, direct/inferred evidence, unresolved ambiguity, and readiness state. |
| Snapshot consistency | Whether the inspected reports, snapshot index, and candidate files agree on candidate count, IDs, dispositions, and provenance. |
| Quarantine boundary | For 0010-0011: quarantine reason, missing scope/evidence, and what evidence would be required before any future unquarantine or decision. |
| Status/readiness check safety | Whether the read-only status/readiness path was confirmed safe before use; if not safe, the check must be skipped and recorded as blocked. |
| Telemetry caveat | Whether the current telemetry window still passes and why that is only observability evidence, not migration readiness. |
| Future dry-run requirements | Exact non-mutating outputs a later dry-run must emit: candidate traceability, would-write preview, no-write assertion, error classes, and halt reasons. |
| Later write-apply prerequisites | Reviewed candidate decisions, dry-run apply report, rollback plan, fresh no-intervening-write evidence, and exact user approval. |

## Hard Stops

Do not execute T174 if:

- exact approval is missing, shortened, ambiguous, conditional, or combined with another gate;
- git status has unexpected tracked changes;
- the T68 snapshot path is missing, renamed, or not the same snapshot referenced by T68/T123/T124/T169;
- any expected candidate file 0001-0011 is missing, duplicated, symlinked outside the snapshot, or
  has a mismatched ID/disposition;
- the M6 status/readiness check cannot be proven read-only before invocation;
- executing the slice would require candidate decisions, status/prioritize/apply/rerun,
  lifecycle writes, deletion, native Claude, Claude Bridge, harness writes, or ranking/`orient`
  changes.

Stop immediately and report without cleanup if:

- any candidate file or status output contradicts the committed inspection reports;
- any command attempts or proposes a write-capable path;
- any unexpected Memory OS write, git change, candidate-state change, lifecycle mutation, or
  user-owned-file change appears;
- continuing would require inference-heavy candidate judgment rather than evidence-readiness
  classification.

## Completion Criteria

T174 execution can be marked complete only if it produces a committed result report and
implementation-plan note that preserve all default-deny boundaries. It must leave candidate
decisions, migration apply, deletion, lifecycle cleanup, native Claude validation, and broad
legacy simplification behind separate exact approval gates.
