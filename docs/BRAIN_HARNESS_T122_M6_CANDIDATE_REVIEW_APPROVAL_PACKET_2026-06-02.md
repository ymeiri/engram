# Brain Harness T122 M6 Candidate Review Approval Packet

Status: Approved and partially executed by T123
Date: 2026-06-02
Scope: Approval packet for the next read-only M6 candidate-review inspection

Execution note, 2026-06-02: the user approved the exact T123 phrase. Codex read only candidate
files 0001-0004 and recorded the result in
`docs/BRAIN_HARNESS_T123_M6_CANDIDATE_0001_0004_INSPECTION_RESULT_2026-06-02.md`.

T121 explained the T68 count drift without making migration decisions. The written T68 review
export snapshot still has 9 review candidates, 2 quarantine candidates, and 1 skip candidate. T122
does not inspect any candidate file and does not run migration status, prioritize, apply, rerun,
deletion, lifecycle mutation, document indexing, schema/storage/index behavior change, public MCP
change, ranking, `orient`, or harness-write action.

## Research Question

What is the smallest useful next approval gate for M6 after T121 explained the count drift?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only packet should ask for exact approval to inspect a small first batch of review candidate files from the written snapshot, leaving quarantine and all M6 commands behind separate gates. |
| Null | Candidate review should remain paused because even a small read-only batch is too broad or underspecified. |
| Simpler alternative | Ask only for the existing T70 exact-file indexing approval and defer M6 candidate review. |
| Failure | A generic continuation prompt is treated as approval to read candidate files, run status/prioritize/apply, index documents, or mutate lifecycle/storage state. |

## Current Evidence

The only candidate-list source used for this packet is the already-approved T69 `index.md` read:

```text
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/index.md
```

The snapshot anchors are:

- sources scanned: 116
- total candidates: 12
- dispositions: `review: 9`, `quarantine: 2`, `skip: 1`
- source kinds: `entity_observation: 2`, `project_observation: 9`, `session_event: 1`
- proposed kinds: `decision: 6`, `project_fact: 4`, `limitation: 1`, `session_insight: 1`
- dry run only: no Memory OS records were written

The skip candidate was already inspected by T121 and is excluded from the proposed next read:

```text
candidates/0012-skip-plan.md
```

## Candidate Filename Set

Review candidates, not yet inspected:

```text
candidates/0001-review-dogfood-baf008-accepted-live-2026-05-24.md
candidates/0002-review-dogfood-baf008-prearm-setup-2026-05-24.md
candidates/0003-review-dogfood-claude-code-scoped-obligation-smoke-2026-05-24.md
candidates/0004-review-dogfood-claude-code-obligation-list-scope-fix-2026-05-24.md
candidates/0005-review-dogfood-claude-code-2026-05-24-review.md
candidates/0006-review-decisions-claude-hook-reenable-prompt-2026-05-24.md
candidates/0007-review-maintenance-disk-cleanup-2026-05-24.md
candidates/0008-review-decisions-orient-recent-git-context.md
candidates/0009-review-testing-dogfood-pilot-2026-05-07.md
```

Quarantine candidates, not yet inspected:

```text
candidates/0010-quarantine-telemetry-recall-432971.md
candidates/0011-quarantine-gotchas-shared-worktree-branch-loss.md
```

## Historical T123 Gate

Before T123 was approved, the recommended next slice was a small first-batch read-only inspection
of the first four review candidate files. It was intentionally smaller than all 11 non-skip files
so candidate-review quality, report shape, and stop conditions could be validated before reading
the remaining queue.

To authorize this exact read-only inspection, reply with:

```text
Approve T123: read-only inspect candidate files 0001-0004 from the written T68 M6 review-export snapshot; no quarantine files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.
```

If approved exactly, Codex may read only:

```text
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0001-review-dogfood-baf008-accepted-live-2026-05-24.md
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0002-review-dogfood-baf008-prearm-setup-2026-05-24.md
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0003-review-dogfood-claude-code-scoped-obligation-smoke-2026-05-24.md
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0004-review-dogfood-claude-code-obligation-list-scope-fix-2026-05-24.md
```

The inspection result may be summarized in a new Markdown report and committed with documentation
updates. It must not make accept/reject/skip decisions, write Memory OS records, run
status/prioritize/apply/rerun, read other candidates, index documents, mutate lifecycle state,
change schema/storage/index or public MCP behavior, change ranking, expand `orient`, or write
harness adapters/hooks.

## Later Gates, Not Approved By T122

These are examples of future gates, not approval to execute them now:

```text
Approve T124: read-only inspect candidate files 0005-0009 from the written T68 M6 review-export snapshot; no quarantine files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.
```

```text
Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.
```

Any future `migration_review_status`, `migration_review_prioritize`, or `migration_review_apply`
step needs a separate exact approval phrase. Status/prioritize must not be bundled with candidate
file inspection. Apply/deletion/lifecycle mutation requires reviewed candidates, a dry-run report,
a rollback plan, and explicit write approval.

The separate T70 document-visibility gate was later approved and executed:

```text
Approve T70: index exact files T59, T68, and T69.
```

T70 indexing did not approve M6 candidate review/apply, and M6 candidate review does not approve
document indexing.

## Historical Measurement For T123

Before reading candidate files:

- verify the four approved filenames still exist in the written snapshot;
- verify the approval phrase names T123, files 0001-0004, the written T68 snapshot, read-only
  inspection, no quarantine files, no M6 commands, no decisions, and no writes except the
  inspection report;
- do not query live store state or run migration commands.

After reading candidate files:

- record each candidate's disposition, proposed memory kind, source id, evidence target, and any
  missing or ambiguous evidence;
- preserve failures and confounds;
- recommend the next gate without making a candidate decision.

## Stop Conditions

Stop without reading candidate files if any of these occur:

- approval does not exactly name T123 and files 0001-0004;
- approval tries to combine candidate inspection with T70 indexing, status, prioritize, apply,
  rerun, deletion, lifecycle mutation, ranking, `orient`, public MCP/schema/storage/index,
  document-index behavior, or harness writes;
- any approved candidate file is missing, renamed, or points outside the written T68 snapshot;
- explaining a candidate requires reading additional candidate files or querying live store state;
- the candidate file contents imply a destructive or irreversible follow-up.
