# Brain Harness T69 T68 Count Drift Decision Packet

Status: Pending explicit user approval
Date: 2026-06-01
Scope: Approval packet for read-only inspection of the T68 review-export count drift

T68 correctly stopped after the approved T59 review-export-only call because the export returned
12 candidates instead of the expected 11. The later user reply, `i approve`, is not safely scoped
to a count-drift option. This packet therefore asks for explicit approval before any further M6
inspection.

This packet does not inspect candidate contents, rerun export, run review apply, make candidate
decisions, delete data, mutate lifecycle state, change schema/storage/index behavior, change public
MCP behavior, change ranking, expand `orient`, or write harness adapters/hooks.

## Research Question

Can Engram safely perform a narrow read-only inspection of the T68 count drift by reading exactly
two files from the already-written review-export snapshot, without making migration decisions or
changing any state?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The written export snapshot explains the drift through `index.md` and the `skip` candidate file, allowing the next gate to be framed from evidence. |
| Null | The two files do not explain the drift, so M6 remains paused until a narrower or revised read-only scope is approved. |
| Simpler alternative | Leave M6 paused and use T68 as the current evidence boundary. |
| Failure | Ambiguous approval is treated as authorization, or inspection expands into candidate decisions, rerun export, apply, lifecycle mutation, deletion, or ranking/hot-path work. |

## Current Evidence

- T58 inventory-only M6 scope scanned 115 sources, returned 11 candidates, and wrote no records.
- T59 approved exactly one review-export-only call and required stopping on zero candidates, more
  than 11 candidates, or count mismatch.
- T68 ran exactly the approved T59 call. It wrote:
  `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export`.
- T68 output reported 116 sources and 12 candidates: 9 review, 2 quarantine, and 1 skip.
- The additional file was listed as:
  `candidates/0012-skip-plan.md`.
- The tool reported dry-run-only behavior and no Memory OS records were written.

## Consultation

AI Council recall surfaced prior guidance that M6 and ranking work must stay prompt-class- and
approval-scope-specific. A fresh Council broadcast to Claude Sonnet 4.6, GPT-5.4, and Gemini 3.1
Pro unanimously recommended a documentation-only packet asking for exact read-only inspection
approval, not treating `i approve` as scoped authorization.

Claude Bridge read-only critique agreed with that recommendation and added two safeguards:

- pin inspection to the written export snapshot, not the live store;
- require an approval reply that names both inspected files.

## Completion Matrix

| Area | State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Memory OS substrate and MCP surfaces | Implemented | Migration inventory/export/status/apply surfaces exist with review gates | Does not justify broad legacy simplification or deletion |
| `orient` hot path | Implemented and validated for current contract | Lean shape, current-plan continuity, gate prompt fixtures, Codex and Claude Code traces | Keep payload expansion and approval-audit behavior gated |
| Current-plan / next-step retrieval | Validated for approved prompt classes | T64/T67/T68 startup searches recovered current plan and active M6 gate | Broad implementation-plan searches can still surface stale history |
| T58 inventory | Validated | 115 sources, 11 candidates, no writes | Spent scope; do not rerun without fresh approval |
| T67 document visibility | Partially validated | T59 title and filename-stem document searches improved | T59 was later edited after indexing; absolute-path semantic search remains weak |
| T68 review export | Partially validated | Review workspace exists; 12 candidates, no writes | Count drift requires explicit user decision before further M6 progress |
| T69 inspection | Missing and gated | Council and Claude recommend exact read-only inspection packet | Requires approval phrase below |
| M6 apply/deletion/lifecycle | Blocked | No reviewed candidates, no dry-run apply, no rollback plan | Requires separate explicit approval after candidate review evidence |

## Proposed Approved Read-Only Inspection

If approved exactly, Codex may read only these two files from the written T68 export snapshot:

```text
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/index.md
/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export/candidates/0012-skip-plan.md
```

The inspection result may be summarized in a new Markdown report and committed with documentation
updates. The report may recommend a next gate, but it must not make candidate accept/reject/skip
decisions or run any migration status, prioritize, apply, rerun, deletion, lifecycle, index,
ranking, `orient`, public MCP, or harness-write action.

## Stop Conditions

Stop without reading further if any of these occur:

- approval does not name T69 and both target files;
- either target file is missing or has moved;
- either target file points to required evidence outside the two-file scope;
- explaining the drift requires live-store queries, candidate decisions, rerun export, status,
  prioritize, apply, deletion, lifecycle mutation, schema/storage/index work, ranking, `orient`,
  public MCP changes, or harness writes.

## Approval Question

T69 requires explicit scoped authorization. The count drift from T68 is: sources 115 to 116, and
candidates 11 to 12, with the extra candidate listed as type `skip` at
`candidates/0012-skip-plan.md`.

To authorize read-only inspection of exactly two files, reply with:

```text
Approve T69: inspect index.md and 0012-skip-plan.md.
```

Any other reply should be treated as non-authorization for T69.
