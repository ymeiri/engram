# Brain Harness T116 Approval Scope Audit

Status: Completed docs-only approval-scope audit; no gated write run
Date: 2026-06-01
Scope: Verify whether the latest continuation context authorizes any still-pending gated write.

This slice did not run document indexing, document planning, reindex, cleanup, orphan recovery, M6
inspection, migration review export/apply, deletion, lifecycle mutation, `lint(action="apply_safe")`,
ranking changes, `orient` changes, public MCP changes, schema/storage/index behavior changes,
document-index behavior changes, MemoryItem creation for document packets, or harness adapter/hook
changes.

## Research Question

After a stale completed T65 approval phrase and a generic approval in the thread, is any
still-pending gated Brain Harness write authorized, or must Engram continue with non-gated evidence
work until an exact current gate phrase is provided?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The completed T65 approval cannot authorize new T70/T69/T111 work; the generic approval is too ambiguous for a write gate. |
| Null | The latest approval wording is enough to run the current recommended document-index write. |
| Simpler alternative | Stop at the live T115 handoff without recording a fresh approval-scope audit. |
| Failure | The audit is misread as approval to run T70, inspect T69 files, change T111 recommendations, archive lifecycle records, or broaden document indexing. |

## Measurement

Startup and repo evidence were refreshed before choosing a slice:

- Lean `orient` trace `019e8491-9c1f-7f22-ad59-f5784605f75b` returned the active T115
  current-plan memory `019e8490-3439-7ca0-b8b3-a3b8bc3fdf4a`.
- Direct search trace `019e8491-c89b-7160-806c-18e1bcbd896a` returned the T115 current plan first
  for current-plan/next-step context.
- `git status --short` returned only the pre-existing untracked root `AGENTS.md`; latest commit was
  `66e55e2` (`Record T115 document visibility audit`).
- The T65 approval packet is historical and T67 records it as completed: T65 indexed exactly T58,
  T59, and T64.
- The still-pending T70 packet asks for the exact phrase
  `Approve T70: index exact files T59, T68, and T69.`
- Live document stats still report `source_count=76`, `chunk_count=4114`,
  `searchable_chunk_count=2102`, and `orphan_chunk_count=2012`.
- Live document search still returns T59 rank 1 for the T59 title, but the T70 packet query does
  not return T70 in the top five and the T114 title query does not return T114 in the top five.
- `obligations(action="doctor")` returned no open obligations.
- `lint(action="run")` still reports stale current-plan feedback for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with `safe_action=none` plus wrong-scope feedback on
  Claude Code memories; no lifecycle action is safe without explicit approval.
- `telemetry(action="real_session_eval", project="engram", limit=50)` currently fails the
  confidence gate with `feedback_coverage=0.47999998927116394` and only two intents with feedback.

## Completion Matrix

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Lean `orient` hot path | Validated for this startup | T115 current plan returned by lean `orient` | Keep payload compact; do not expand without evidence and approval. |
| Current-plan search | Validated for tested continuation prompt | Direct search returned T115 current plan first | Broad ranking quality remains unproven beyond fixtures and live prompts tested so far. |
| T65 exact-file indexing | Completed historical slice | T67 result report | Does not authorize T70 or later files. |
| T70 exact-file indexing | Missing approval | T70 packet exact phrase absent from current user text | Requires `Approve T70: index exact files T59, T68, and T69.` |
| T69 count-drift inspection | Missing approval | T70/T115 preserve T69 as separate gate | Requires exact T69 inspection approval before reading review-export files. |
| T111 eval recommendation behavior | Missing option choice | T111/T112 remain paused/audited | Requires exact Option A or Option B approval. |
| Document index health | Risky | 2012 orphan chunks out of 4114 total chunks | Cleanup, reindex, orphan recovery, and broader indexing remain gated. |
| Lifecycle cleanup | Risky but gated | Lint reports stale/wrong-scope and superseded-active items | `apply_safe`, archive, scope correction, or deletion require exact approval. |
| Harness readiness | Risky/not ready | T106/T115 handoff keep T47 gate active | Adapter/settings/hook writes require exact T47 approval. |
| Real-session confidence | Not complete | Latest report fails coverage and intent-diversity gates | M6 write-apply remains blocked by evidence and approval gates. |

## Interpretation

No still-pending write gate is authorized. The next recommended write remains T70 exact-file
indexing, but only if the user provides the exact current phrase. Until then, future work should
remain non-gated: evidence-quality audits, deterministic fixtures, cross-harness read-only
validation, or narrowly scoped documentation that prevents mistaken approval carryover.

Do not broaden T70 to include T70/T113/T114. If those newer reports should be indexed, prepare a
separate exact-file approval packet or get an explicit scope that names those files.
