# Brain Harness T158 T125 Quarantine Inspection Approval Packet

Date: 2026-06-03
Status: Pending user approval. No quarantine candidate file has been read by this packet.
Scope: Refresh the remaining T125 read-only M6 inspection gate for the two quarantine candidate
files from the written T68 review-export snapshot.

This packet is a request for approval, not approval itself. It does not run
`migration_review_status`, `migration_review_prioritize`, `migration_review_apply`, rerun export,
make candidate decisions, read quarantine files, mutate lifecycle state, delete data, change
schema/storage/index behavior, change public MCP behavior, change ranking, change `orient`, change
document-index behavior, run native Claude, use Claude Bridge, edit harness files, or touch
user-owned files.

## Research Question

After T123 and T124 inspected all nine `review` candidates from the written T68 snapshot, can
Engram safely ask for exact approval to inspect only the two remaining `quarantine` candidates
without bundling candidate decisions or M6 commands?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A standalone default-deny T125 packet is the smallest useful M6 progress: it completes the read-only candidate-inspection set if approved, while preserving separate gates for decisions, status/prioritize, apply, rollback, deletion, and lifecycle mutation. |
| Null | The existing T122 mention of T125 is enough, and another packet adds process noise without improving safety or clarity. |
| Simpler alternative | Stop M6 progress until the user directly approves T125 from T122. |
| Failure | The packet is mistaken for approval, reads quarantine files now, makes accept/reject/quarantine decisions, or creates pressure to run migration commands without reviewed candidates and explicit write approval. |

## Current Evidence

- T58 ran one approved inventory-only M6 scope and found 9 review candidates plus 2 quarantine
  candidates. It wrote no Memory OS records.
- T68 ran the approved T59 review-export call and wrote the review workspace at:

```text
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export
```

- T121 inspected only `index.md` and `0012-skip-plan.md` from that written snapshot, explaining
  the count drift as one generated `skip` candidate.
- T123 inspected review candidate files 0001-0004 only.
- T124 inspected review candidate files 0005-0009 only, completing inspection of all nine
  `review` candidates.
- The remaining unread candidates are the two `quarantine` files named in T58, T68, and T122.
- AI Council recall recovered the T122 operation-separation guidance: candidate-file inspection,
  `migration_review_status`/`prioritize`, apply, T70 indexing, lifecycle changes, ranking/`orient`,
  schema/storage/index, public MCP, document-index behavior, and harness writes require distinct
  exact approvals.
- A fresh AI Council broadcast on 2026-06-03 agreed that the smallest useful non-gated artifact is
  a docs-only/default-deny packet for read-only inspection of files 0010-0011 only. The Council
  highlighted scope creep, hidden writes, snapshot mismatch, malformed content, and gate collision
  as stop-condition classes.

## Candidate Filename Set

If and only if the user gives the exact approval below, Codex may read only these two files:

```text
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0010-quarantine-telemetry-recall-432971.md
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0011-quarantine-gotchas-shared-worktree-branch-loss.md
```

The packet does not authorize reading:

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
candidates/0012-skip-plan.md
```

## Proposed Approved Read-Only Inspection

To authorize execution, reply exactly:

```text
Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.
```

If approved exactly, the execution slice may:

- verify the two approved paths exist inside the written T68 snapshot;
- read only those two quarantine candidate files;
- record each candidate's source kind, source id, proposed memory kind, disposition, confidence,
  quarantine reason, scope concern, and any missing or ambiguous evidence;
- write one Markdown inspection report and update tracking docs;
- run documentation-only validation such as `git diff --check`;
- submit telemetry feedback and capture current-plan memory after the documentation commit.

## Out Of Scope

T125 does not authorize:

- reading review candidates 0001-0009 or skip candidate 0012;
- reading unrelated files from the review workspace;
- querying live store state to decide a candidate;
- `memory(action="migration_review_status")`;
- `memory(action="migration_review_prioritize")`;
- `memory(action="migration_review_apply")`;
- rerunning inventory or review export;
- accepting, rejecting, editing, promoting, demoting, or quarantining any candidate;
- writing active Memory OS records or KnowledgeCommits;
- deletion, cleanup, archive, supersede, or any lifecycle mutation;
- schema/storage/index changes, public MCP changes, ranking changes, or `orient` changes;
- document indexing or document-index behavior changes;
- native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive Claude;
- harness install, settings edits, hook edits, adapter edits, `adopt_user_owned`, rollback,
  force-kill, old-binary reinstall, or user-owned files.

## Stop Conditions

Stop without reading quarantine files if:

- approval is missing, ambiguous, conditional, or does not exactly name T125, files 0010-0011,
  the written T68 snapshot, read-only inspection, no review files, no M6 commands, no decisions,
  and no writes except the inspection report;
- either approved path is missing, renamed, a symlink to outside the written T68 snapshot, or no
  longer matches the expected `0010-quarantine...` / `0011-quarantine...` identity;
- path validation requires live-store queries, migration commands, or broader filesystem
  exploration.

Stop after reading only the approved files and before any further action if:

- either file is malformed, unexpectedly large, binary, truncated, or structurally inconsistent;
- either file requires reading review candidates, skip candidates, quarantine sources, or live
  store state to summarize safely;
- either file appears to contain a destructive, executable, or irreversible follow-up instruction;
- the inspection would require a candidate accept/reject/quarantine decision;
- the inspection would require `status`, `prioritize`, `apply`, rerun, deletion, lifecycle
  mutation, schema/storage/index work, public MCP changes, ranking, `orient`, document indexing,
  Claude, Claude Bridge, or harness writes.

## Completion Matrix Delta

| Area | State After T158 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Review candidate inspection | Completed before T158 | T123 and T124 inspected files 0001-0009 | Candidate decisions still separate |
| Quarantine candidate inspection | Approval packet refreshed | T158 names only files 0010-0011 and exact approval wording | Requires exact T125 approval before reads |
| M6 candidate decisions | Still gated | No accept/reject/quarantine decision made | Needs human decisions and separate approval |
| M6 dry-run/apply | Still gated | No `status`, `prioritize`, `apply`, or rerun executed | Needs reviewed candidates, dry-run report, rollback plan, and explicit write approval |
| T154/T157 gates | Unchanged | No native Claude or lifecycle write ran | Exact approvals remain required |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Keep `orient` and ranking out of M6 packets |

## Next Step

The next M6 step is a user decision on the exact T125 approval phrase above. Generic continuation,
generic approval, T135 approval, T154 approval, or T157 approval must not be treated as T125
authorization.
