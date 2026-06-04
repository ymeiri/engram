# Brain Harness T210 M6 Candidate-Disposition Authorization Packet

Date: 2026-06-04
Status: docs-only/default-deny authorization packet. Not executed.

## Scope

This packet defines the next M6 gate after T209. It does not execute that gate.

T210 is not a migration apply, not a prioritize run, not a review-export rerun, and not an
agent-generated candidate decision. It is a future authorization shape for recording explicit
human-provided candidate dispositions in the existing generated review workspace, then confirming
the resulting state with one read-only status check.

This packet does not edit the T68 review workspace, make candidate decisions, run
`migration_review_status`, run `migration_review_prioritize`, run `migration_review_apply`, rerun
inventory/export, mutate active Memory OS lifecycle state, archive memory, delete data, change
ranking or `orient`, change public MCP/schema/storage/index/document-index behavior, run native
Claude or Claude Bridge writes, edit harness files, change runtime configuration, or touch
user-owned files.

## Current Evidence

- T58 inventory found 11 M6 candidates: 9 review and 2 quarantine.
- T68 review export wrote:
  `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export`.
- T68 stopped because the export contained 12 candidates: the T58 11 plus
  `candidates/0012-skip-plan.md`.
- T123 and T124 inspected candidates 0001-0009 without decisions.
- T169 inspected candidates 0010-0011 without decisions.
- T209 validated the snapshot as generated `index.md` plus 12 regular candidate files with no
  symlinks, confirmed `memory(action="migration_review_status")` is a read-only status path from
  source inspection, and ran exactly one status check.
- T209 status result: 12 files scanned, all 12 in `files_with_no_decision`, no skipped/conflict/
  missing/not-in-index files, accepted/planned/written counts 0, `ready_to_apply=false`, warnings
  empty.

## Research Question

What exact future gate would let Engram record candidate dispositions for the existing M6 review
workspace without letting that step become migration apply, agent judgment, lifecycle cleanup, or
legacy simplification?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The next safe M6 step is an exact, human-disposition-only authorization: record explicit human choices in generated review pages, then run one read-only status check and write a result report. |
| Null | Candidate dispositions cannot be authorized safely yet because the 0012 count-drift artifact and quarantine scope gaps make the review workspace too ambiguous. |
| Simpler alternative | Keep M6 paused after T209 and index/report the evidence only. |
| Failure | The packet lets Codex infer dispositions, write active MemoryItems, run apply/prioritize/export/rerun, or treat `ready_to_apply=false` as a near-apply state. |

## Recommended Future Approval

Use the conservative 0001-0011 shape unless the user explicitly wants 0012 included.

```text
Approve T210A: execute the M6 human-disposition recording gate from docs/BRAIN_HARNESS_T210_M6_CANDIDATE_DISPOSITION_AUTHORIZATION_PACKET_2026-06-04.md. I will provide explicit dispositions for candidates 0001-0011 and a separate explicit instruction for 0012. Record only those human-provided choices in the generated T68 review workspace, run one read-only memory(action="migration_review_status") check afterward, and write a T210 result report plus implementation-plan note. Do not infer candidate decisions, edit content beyond the provided choices/notes, run migration_review_prioritize/apply/export/rerun, write active MemoryItems, mutate lifecycle state, delete data, change ranking/orient/public MCP/schema/storage/index/document-index behavior, run native Claude or Claude Bridge, edit harness files, change runtime configuration, or touch user-owned files.
```

If the user wants all 12 generated snapshot files to share the same decision gate, use a separate
approval that explicitly names the scope expansion:

```text
Approve T210B: execute the M6 human-disposition recording gate for snapshot candidates 0001-0012, intentionally including 0012 as a scope expansion beyond the T58 11-candidate inventory. I will provide explicit dispositions for all 12 candidates. Record only those human-provided choices in the generated T68 review workspace, run one read-only memory(action="migration_review_status") check afterward, and write a T210 result report plus implementation-plan note. Do not infer candidate decisions, edit content beyond the provided choices/notes, run migration_review_prioritize/apply/export/rerun, write active MemoryItems, mutate lifecycle state, delete data, change ranking/orient/public MCP/schema/storage/index/document-index behavior, run native Claude or Claude Bridge, edit harness files, change runtime configuration, or touch user-owned files.
```

Generic continuation, approval without explicit dispositions, or approval that omits 0012 handling
must not be treated as authorization to edit the review workspace.

## Required Human Inputs

The future execution must receive explicit human-provided decisions. Codex must not infer them from
candidate content.

For candidates 0001-0011, each input must name one generated candidate file and one generated
review choice:

- accept for migration;
- accept with edits;
- quarantine;
- reject / skip.

For candidate 0012, the human input must choose one of:

- leave unscoped and do not edit 0012;
- include 0012 in the same disposition recording gate with an explicit generated review choice;
- record a separate count-drift note only, without treating it as part of the T58 candidate set.

If any candidate is missing an explicit input, the future execution must stop before editing review
files.

## If Approved: Authorized Operations

The future execution may:

1. Re-read T209, the T68 index, and the exact candidate files named by the approved scope.
2. Re-check that the review workspace contains no symlinks and no unexpected generated candidate
   files beyond the approved scope.
3. Edit only generated review pages in:
   `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates`.
4. Mark exactly one generated review checkbox per human-provided decision.
5. Add only human-provided reviewer notes when supplied.
6. Run exactly one read-only `memory(action="migration_review_status")` check after editing.
7. Write a T210 result report and implementation-plan note.
8. Commit only the repo report and implementation-plan note.
9. Capture current-plan memory and submit telemetry feedback.

## Explicitly Forbidden

The future execution must not:

- infer or recommend candidate dispositions;
- edit candidate content unless the human explicitly provides edited wording;
- change any candidate file outside the approved scope;
- run `migration_review_prioritize`, `migration_review_apply`, `migration_review_export`, rerun
  inventory/export, or any equivalent write-capable migration command;
- apply a status result or write active MemoryItems;
- mutate lifecycle state, archive memory, delete data, or simplify legacy layers;
- change ranking/`orient`, public MCP contracts, schema/storage/index behavior, or document-index
  behavior;
- run native Claude, Claude Bridge write tasks, harness install, settings edits, hook edits,
  adapter edits, runtime refresh, rollback, force-kill, or old-binary reinstall;
- edit or stage root `AGENTS.md` or other user-owned files.

## Measurements For Future Execution

| Measurement | Required Output |
| --- | --- |
| Input completeness | Every approved candidate has exactly one human-provided choice; 0012 handling is explicit. |
| Review-workspace writes | Exact files edited, exact checkbox selected, and any human-provided note added. |
| Status after recording | `files_with_no_decision`, conflict/missing/not-in-index lists, accepted/quarantined/rejected/planned counts, `ready_to_apply`, and warnings. |
| No active memory writes | Confirm no active MemoryItems were written by this gate. |
| Next apply prerequisites | If status becomes apply-shaped, still require a separate dry-run apply report, rollback plan, fresh no-intervening-write evidence, and exact write-apply approval. |

## Hard Stops

Stop before editing review files if:

- explicit candidate choices are missing, contradictory, or ambiguous;
- 0012 handling is not explicit;
- git status has unexpected tracked changes;
- the review workspace path is missing or contains unexpected symlinks;
- any candidate file is missing, duplicated, or not listed by the generated index;
- any requested operation would require active Memory OS writes, apply/prioritize/export/rerun,
  lifecycle mutation, deletion, ranking/`orient` changes, public MCP/schema/storage/index behavior
  changes, harness/native-Claude/runtime work, or user-owned-file edits.

## Completion Criteria For Future Execution

The future T210 execution can be marked complete only if it records exactly the human-provided
review decisions in the generated workspace, runs the single read-only status check, commits the
result report and implementation-plan note, captures current-plan memory, and leaves migration
apply behind a separate exact gate.
