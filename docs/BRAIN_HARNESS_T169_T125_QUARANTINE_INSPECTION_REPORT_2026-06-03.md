# Brain Harness T169 T125 Quarantine Inspection Report

Date: 2026-06-03
Status: complete as exact-approved read-only inspection
Scope: T125 inspection of quarantine candidate files 0010-0011 from the written T68 M6
review-export snapshot.

## Approval

The user approved the exact T125 wording:

```text
Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.
```

This report is the only write made for T125. The slice did not read review candidates 0001-0009,
read `0012-skip-plan.md`, run `migration_review_status`, run `migration_review_prioritize`, run
`migration_review_apply`, rerun export, query live store state to decide a candidate, mutate Memory
OS lifecycle state, change ranking or `orient`, change public MCP/schema/storage/index behavior,
change document-index behavior, run native Claude or Claude Bridge, edit harness files, or touch
user-owned files.

## Research Question

After T123 and T124 inspected all nine `review` candidates from the written T68 snapshot, can
Engram inspect only the two remaining `quarantine` candidates without bundling candidate decisions
or M6 commands?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The two quarantine candidate files can be inspected and summarized without reading other candidates or running M6 commands. | Supported. Both approved files were regular, small, non-symlink text files and were summarized from their own content only. |
| Null | The files cannot be safely inspected without broader workspace or live-store context. | Not supported for this read-only summary. Both files contained enough generated metadata and machine records for inspection. |
| Simpler alternative | Leave quarantine inspection unexecuted until a later M6 packet. | Rejected by exact user approval for T125. |
| Failure | The slice reads extra candidates, makes candidate decisions, or triggers M6/status/apply/lifecycle work. | Not observed. |

## Path Validation

| Candidate | Approved path | Result |
| --- | --- | --- |
| 0010 | `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0010-quarantine-telemetry-recall-432971.md` | Regular file, size 3082 bytes, ASCII text, not a symlink. |
| 0011 | `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0011-quarantine-gotchas-shared-worktree-branch-loss.md` | Regular file, size 2966 bytes, ASCII text, not a symlink. |

The `file` command classified the first as exported SGML/ASCII text and the second as HTML/ASCII
text because the generated Markdown includes front matter and generated markers. Both files were
small text files and were read with `sed` after path validation.

## Candidate 0010 Inspection

| Field | Value |
| --- | --- |
| File | `0010-quarantine-telemetry-recall-432971.md` |
| Source kind | `entity_observation` |
| Source id | `019e023a-eb53-7d01-8263-022801db1ab6` |
| Source label | `entity:review-all-system observation` |
| Source key | `telemetry.recall.432971` |
| Proposed memory kind | `project_fact` |
| Proposed scope | `entity:review-all-system` |
| Generated disposition | `quarantine` |
| Confidence | `0.650` |
| Staleness | 24 days |

Summary: the candidate describes review-all v3 recall telemetry for PR `#432971` in dd-source,
including perfect recall-by-class values for several classes and notes that a preflight exited 141
after triggering Codex, requiring manual fallback. It also records external Codex comments about
stale `captureExpressions` path guidance and missing capture-expression name validation.

Quarantine reason recorded by the generator: Layer 1 entity observations map to entity-scoped
memory, and entity observations may be linked broadly across projects until scope is confirmed.

Inspection notes: the content is entity-scoped to `review-all-system`, not project-scoped to
Engram. The candidate may be useful as review-all-system telemetry evidence, but T125 does not
authorize deciding whether to accept, edit, reject, or migrate it.

Missing or ambiguous evidence: no direct PR artifact, source observation body beyond the exported
candidate, or scope confirmation was inspected under T125. Resolving those would require a later
approved candidate-decision slice.

## Candidate 0011 Inspection

| Field | Value |
| --- | --- |
| File | `0011-quarantine-gotchas-shared-worktree-branch-loss.md` |
| Source kind | `entity_observation` |
| Source id | `019dd936-90f2-7d53-95f3-f4787042ea47` |
| Source label | `entity:review-all-system observation` |
| Source key | `gotchas.shared-worktree-branch-loss` |
| Proposed memory kind | `limitation` |
| Proposed scope | `entity:review-all-system` |
| Generated disposition | `quarantine` |
| Confidence | `0.650` |
| Staleness | 32 days |

Summary: the candidate describes a review-all PR `#415140` gotcha in dd-source where subagents
reported branch checkout loss while reading. The exported fallback guidance is to avoid subagent
file reads and review immutable objects with `git diff/show <base>...<head>` and
`git show <head>:path`. It also notes that external `codex-fetch.sh` can poll silently until
timeout and return pending.

Quarantine reason recorded by the generator: Layer 1 entity observations map to entity-scoped
memory, and entity observations may be linked broadly across projects until scope is confirmed.

Inspection notes: the content is entity-scoped to `review-all-system` and appears potentially
operationally useful for review-all workflows, but it is not Engram project guidance by itself.
T125 does not authorize deciding whether to accept, edit, reject, or migrate it.

Missing or ambiguous evidence: the export does not include the original review-all session
transcript, the PR state, or confirmation that the gotcha remains valid in current review-all
operations. Checking those would require a later approved candidate-decision slice.

## Completion Matrix Delta

| Area | State After T169 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Review candidate inspection | Complete before T169 | T123 and T124 inspected files 0001-0009 | Candidate decisions remain separate |
| Quarantine candidate inspection | Complete for files 0010-0011 | This report summarizes both approved quarantine files | Candidate decisions remain separate |
| Candidate decisions | Not made | T125 explicitly forbids candidate decisions | Needs later human-approved accept/edit/reject/quarantine decision slice |
| M6 status/prioritize/apply | Not run | No migration commands executed | Needs later dry-run/status/apply approval with rollback and deletion gates |
| Hot path/ranking/orient | Unchanged | No source/runtime behavior changed | Keep separate from M6 inspection |
| Native Claude | Unchanged by T125 | No Claude or Claude Bridge use in this slice | T154 is separately approved and executed separately |

## Decision

T125 is complete as a bounded read-only inspection. It finishes inspection of the two remaining
quarantine candidates from the written T68 review-export snapshot, but it does not decide,
promote, archive, delete, apply, or migrate anything.

The next M6 step is not automatic. It requires a separate approval packet for candidate decisions
and/or a reviewed-candidate status/dry-run/apply plan.
