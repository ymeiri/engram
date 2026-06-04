# Brain Harness T250 M6 Human-Disposition Worksheet

Date: 2026-06-04
Worksheet status: human input pending; no candidate choices recorded.

This document is a docs-only worksheet for human review. It is compiled only from committed
reports T209, T210, T123, T124, T169, and T121. It does not inspect the generated review
workspace, edit review pages, run `migration_review_status`, run `migration_review_prioritize`,
run `migration_review_apply`, rerun inventory/export, mutate Memory OS lifecycle state, archive
memory, delete data, change ranking/`orient`, change public MCP/schema/storage/index/
document-index behavior, run native Claude or Claude Bridge writes, edit harness/runtime state, or
touch user-owned files.

Authoritative migration state remains T210: all generated files 0001-0012 are undecided,
`ready_to_apply=false`, and candidate 0012 requires explicit handling before any future execution.
Generated labels below are report-derived metadata only. They are not recommendations, human
choices, migration readiness signals, or apply authorization.

## Research Question

Can Engram make the M6 human-input queue easier to review by consolidating already committed
report evidence into a neutral worksheet, without inspecting the generated review workspace or
turning the worksheet into candidate choices?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A report-derived worksheet with explicit pending human fields is a safe M6-unblocking documentation aid because it reduces ambiguity without creating candidate choices or migration state. | Supported. |
| Null | The worksheet would be too easy to mistake for candidate choices and should not be created. | Mitigated by per-row provenance, explicit pending fields, and repeated non-execution boundaries. |
| Simpler alternative | Leave T210 as the only human-input document. | Rejected because T210 defines the authorization shape but does not put the 12 generated files into a compact reviewer worksheet. |
| Failure | The worksheet recommends accept/reject/quarantine, treats generated labels as reviewed choices, inspects the review workspace, or implies `ready_to_apply=true`. | Avoided. |

## Source Boundary

The worksheet uses only these committed reports:

- T209: `docs/BRAIN_HARNESS_T209_M6_READ_ONLY_SCOPING_STATUS_2026-06-04.md`
- T210: `docs/BRAIN_HARNESS_T210_M6_CANDIDATE_DISPOSITION_AUTHORIZATION_PACKET_2026-06-04.md`
- T123: `docs/BRAIN_HARNESS_T123_M6_CANDIDATE_0001_0004_INSPECTION_RESULT_2026-06-02.md`
- T124: `docs/BRAIN_HARNESS_T124_M6_CANDIDATE_0005_0009_INSPECTION_RESULT_2026-06-02.md`
- T169: `docs/BRAIN_HARNESS_T169_T125_QUARANTINE_INSPECTION_REPORT_2026-06-03.md`
- T121: `docs/BRAIN_HARNESS_T121_T69_COUNT_DRIFT_INSPECTION_RESULT_2026-06-02.md`

No generated review-workspace file was opened for T250.

## Candidate Worksheet

Each row is non-authoritative and report-derived. The `Human choice input` fields are intentionally
blank pending values. Future execution must receive explicit human choices outside this worksheet
before editing generated review files.

| Candidate | Filename / label | Report-derived generated label | Source kind | Proposed memory kind (non-binding) | Provenance | Report coverage note | Human choice input | Human rationale / reviewer / date |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0001 | `0001-review-dogfood-baf008-accepted-live-2026-05-24.md` / `Dogfood Baf008 Accepted Live 2026 05 24` | `review` | `project_observation` | `project_fact` | T209, T210, T123 | Historical BAF008 treatment outcome; T123 says later review should check whether newer Memory OS records already represent it. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0002 | `0002-review-dogfood-baf008-prearm-setup-2026-05-24.md` / `Dogfood Baf008 Prearm Setup 2026 05 24` | `review` | `project_observation` | `decision` | T209, T210, T123 | Historical BAF008 pre-arm setup; T123 says human review must decide whether it is durable memory or experiment provenance. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0003 | `0003-review-dogfood-claude-code-scoped-obligation-smoke-2026-05-24.md` / `Dogfood Claude Code Scoped Obligation Smoke 2026 05 24` | `review` | `project_observation` | `decision` | T209, T210, T123 | Historical Claude Code scoped-obligation smoke; T123 notes `used_memory_ids` was empty, so evidence is more about connectivity/scope behavior than memory utility. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0004 | `0004-review-dogfood-claude-code-obligation-list-scope-fix-2026-05-24.md` / `Dogfood Claude Code Obligation List Scope Fix 2026 05 24` | `review` | `project_observation` | `project_fact` | T209, T210, T123 | Historical scope-fix observation; T123 flags the `ready=true` harness wording as time-bound or stale relative to later readiness audits. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0005 | `0005-review-dogfood-claude-code-2026-05-24-review.md` / `Dogfood Claude Code 2026 05 24 Review` | `review` | `project_observation` | `decision` | T209, T210, T124 | Historical Claude Code dogfood review; T124 says later evidence narrows both the old `ready=true` claim and the obligation-list scope issue. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0006 | `0006-review-decisions-claude-hook-reenable-prompt-2026-05-24.md` / `Decisions Claude Hook Reenable Prompt 2026 05 24` | `review` | `project_observation` | `decision` | T209, T210, T124 | Historical conservative prompt for re-enabling Claude hooks; T124 says any accepted memory would still need to preserve the separate exact harness-write gate. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0007 | `0007-review-maintenance-disk-cleanup-2026-05-24.md` / `Maintenance Disk Cleanup 2026 05 24` | `review` | `project_observation` | `project_fact` | T209, T210, T124 | Historical cleanup fact; T124 says no current action is implied. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0008 | `0008-review-decisions-orient-recent-git-context.md` / `Decisions Orient Recent Git Context` | `review` | `project_observation` | `decision` | T209, T210, T124 | Historical orient recent-commit context; T124 says later current-plan work may have narrowed the old Brain Loop limitation wording. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0009 | `0009-review-testing-dogfood-pilot-2026-05-07.md` / `Testing Dogfood Pilot 2026 05 07` | `review` | `project_observation` | `decision` | T209, T210, T124 | Historical Brain Harness dogfood pilot; T124 says later current-plan and retrieval work likely supersedes parts of its next-step guidance. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0010 | `0010-quarantine-telemetry-recall-432971.md` / `Telemetry Recall 432971` | `quarantine` | `entity_observation` | `project_fact` | T209, T210, T169 | Entity-scoped `review-all-system` telemetry evidence; T169 says scope confirmation and original artifact review remain outside the inspected evidence. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0011 | `0011-quarantine-gotchas-shared-worktree-branch-loss.md` / `Gotchas Shared Worktree Branch Loss` | `quarantine` | `entity_observation` | `limitation` | T209, T210, T169 | Entity-scoped `review-all-system` gotcha; T169 says current validity and original transcript/PR context were not inspected. | `<PENDING_HUMAN_INPUT: DO_NOT_INFER>` | `<PENDING_HUMAN_INPUT>` |
| 0012 | `0012-skip-plan.md` / `Skip Plan` | `skip` | `session_event` | `session_insight` | T209, T210, T121 | Count-drift provenance from a session-following plan source; T121 says it explains the extra generated file but does not change the 11 non-skip review-actionable queue. | `<PENDING_HUMAN_INPUT_FOR_0012: EXPLICIT_HANDLING_REQUIRED>` | `<PENDING_HUMAN_INPUT>` |

## Candidate 0012 Handling

Candidate 0012 is not part of the original T58 11-candidate inventory. T210 requires one explicit
human handling choice before any future execution:

- leave 0012 unscoped and do not edit it;
- include 0012 in the same future recording gate with an explicit human choice;
- record a separate count-drift note only, without treating it as part of the T58 candidate set.

T250 does not satisfy that requirement.

## Human Input Requirements For Future Execution

For candidates 0001-0011, future execution requires exactly one explicit human choice per file:

- accept for migration;
- accept with edits;
- quarantine;
- reject / skip.

For every selected choice, human rationale, reviewer identity, and decision date should be supplied
or explicitly left blank by the human. An agent must not fill these fields by inference.

## AI Consultation

AI Council recall found prior M6 guidance that operation classes must stay separate and that
candidate-file inspection, status/prioritize/apply, document indexing, lifecycle changes,
ranking/`orient`, public MCP/schema/storage/index/document-index behavior, and harness writes all
require distinct gates. A fresh AI Council broadcast agreed that T250 needs source-boundary
language, generated-label caveats, explicit pending human fields, per-row provenance, and a
separate 0012 warning.

Claude Bridge read-only critique warned that even neutral evidence language can imply
apply-readiness unless the worksheet distinguishes report-derived facts from human inputs. T250
therefore uses explicit pending fields and provenance per row.

## Decision

T250 is a neutral documentation aid. It does not change the M6 gate: future progress still requires
human-provided choices under T210A or T210B, explicit 0012 handling, a later status check, and
separate dry-run/apply/rollback/write-approval gates before any migration write can occur.

## Validation

Validation for this docs-only slice:

- lean `orient` and direct Engram search for T249/T210/M6 state
- reread T209, T210, T123, T124, T169, and T121
- AI Council recall and bounded broadcast
- read-only Claude Bridge critique
- `git diff --check`
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- document-search visibility for T250
- `obligations(action="doctor", project="engram")`
- focused commit with only intended repo docs

End of worksheet: zero human choices recorded, zero apply signals, all generated files 0001-0012
remain undecided, and `ready_to_apply=false` remains authoritative per T210.
