# Brain Harness T82 Controlled Outcome Artifact Pilot

Status: Complete; doc-only pilot snapshot
Date: 2026-06-01
Scope: Manual trace-to-evidence outcome-link artifact format

T82 PILOT ONLY. This file is an immutable snapshot for this slice; do not append new rows after
the T82 commit. It does not change source behavior, telemetry schema, storage, indexes, public MCP
request parameters, ranking, harness adapters/hooks, migration, lifecycle state, document indexing,
or the `orient` payload. It writes no new telemetry feedback for the sampled rows.

Pointers in this artifact indicate temporal and contextual correlation between an Engram trace and
durable evidence. They do not assert exclusive causal attribution to the agent or prove that memory
improved the outcome.

## Research Question

Can a doc-only artifact provide enough evidence discipline to pilot trace-to-outcome linkage before
any implementation surface is justified?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A bounded document artifact can link traces to durable evidence and expose the minimum field shape needed for future review, while refusing causal claims and leaving live telemetry unchanged. |
| Null | The artifact adds ceremony but no useful signal beyond the existing feedback `note` text. |
| Simpler alternative | Run a larger read-only feedback audit instead of building rows. |
| Failure | The artifact becomes a standing process, selects only easy positive cases, or treats agent self-report as independent outcome evidence. |

## Consultation Synthesis

AI Council recall surfaced prior eval guidance: do not treat limited no-memory or passive feedback
evidence as causal proof, and keep `orient` compact. AI Council broadcast and Claude Bridge
supported the doc-only pilot only with narrower criteria:

- define durable evidence before selecting rows;
- include at least one non-assessable or weakly assessable trace;
- separate evidence existence from evidence strength for an outcome claim;
- pre-state thresholds before row classification;
- include an explicit sunset/immutability clause;
- keep reviewer agreement pending rather than pretending this is independent judgment.

T82 applies those constraints below.

## Measurement Before Rows

### Durable Evidence Predicate

For this pilot, durable evidence must be independently inspectable from the repository or Engram
telemetry reads captured during this slice. Accepted refs are:

- full git commit SHA;
- committed file path with line or section anchor;
- trace ID or feedback ID returned by `telemetry(action="get_trace")` or
  `telemetry(action="list_feedback")`;
- test command or git-status evidence only when the result is already recorded in a committed doc.

Rejected refs are chat assertions alone, hidden transcript state, unstored terminal output,
telemetry outcome fields by themselves, and agent memory of what happened.

### Sampling Rule

Use five rows:

1. The four T78 rows that T81 identified as the only sampled rows with explicit
   `ASSESSABLE_TASK_OUTCOME` labels.
2. The first available post-T78 startup feedback row from the same evidence chain that has positive
   task outcome fields but no `ASSESSABLE_TASK_OUTCOME` label or durable outcome pointer in the
   feedback text.

This deliberately includes one weak/self-reported row so the artifact tests refusal behavior, not
only positive linkage.

### Classification Rule

Use only the T80 outcome classes:

- `TRANSCRIPT_VISIBLE_OUTCOME` when the trace, feedback, and committed repo doc together expose the
  downstream state change.
- `SELF_REPORTED_OUTCOME` when feedback reports success but the durable refs only prove retrieval
  or process context.
- `HARNESS_INCONCLUSIVE` when no Engram trace exists for the attempted task.

`CONTROLLED_LINKED_OUTCOME` is not used in T82 because this pilot has no independent human,
eval-agent, or automated-harness judge. Reviewer agreement starts as `PENDING`.

### Future-Field Threshold

T82 cannot authorize schema/API/storage work. At most, it can justify a future approval packet if
both conditions hold:

1. The same compact field shape is useful for at least four of five rows.
2. The weak/self-reported row becomes clearer, not noisier, when represented in that field shape.

Even if both hold, implementation still requires explicit user approval.

## Artifact Rows

| Row | Trace ID | Feedback ID | Intent | Durable evidence refs | Verifiable state change | Evidence strength | T80 class | Confounds | Reviewer agreement | Future-field signal |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| T82-1 | `019e82a7-c5f1-7c73-987c-63f31d105a92` | `019e82a8-82da-7b22-bd31-cfed0f458fb7` | `verify_decision` | Commit `3a67fa2c589473b6b51c5f897cb35e97935eae31`; `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` lines 47, 102, 119; `telemetry(get_trace)` returned active T77 plan before stale plan. | Pre-registered T78-V1 direct search returned active T77 current-plan memory above stale repository current-plan memory. | Direct for retrieval ordering; indirect for downstream task success because transcript state is summarized in T78 doc. | `TRANSCRIPT_VISIBLE_OUTCOME` | Prospective task was selected for assessability; no independent judge beyond Codex-authored evidence. | `PENDING` | Field shape useful: trace, feedback, evidence refs, evidence strength, class, confounds. |
| T82-2 | `019e82a7-c6b7-7742-ae61-f244a67bb4c9` | `019e82a8-9499-7c03-b983-08c5387281cc` | `verify_decision` | Commit `3a67fa2c589473b6b51c5f897cb35e97935eae31`; `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` lines 48, 103, 120; `telemetry(get_trace)` returned active T77 plan first in orient memory IDs. | Pre-registered T78-V2 lean `orient` returned active T77 current-plan memory first and did not rank stale repository plan above it. | Direct for orient retrieval shape; indirect for downstream task success because compactness and transcript state are summarized in T78 doc. | `TRANSCRIPT_VISIBLE_OUTCOME` | Prospective task was selected for assessability; no independent judge. | `PENDING` | Field shape useful; evidence strength prevents overclaiming beyond retrieval shape. |
| T82-3 | `019e82a7-c86e-7aa3-a7fd-109edf7a9672` | `019e82a8-a7be-7fb2-8c4e-6b7db8cc5f58` | `follow_user_preference` | Commit `3a67fa2c589473b6b51c5f897cb35e97935eae31`; `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` lines 49, 104, 121; `docs/BRAIN_HARNESS_T81_FEEDBACK_OUTCOME_POINTER_PROXY_AUDIT_2026-06-01.md` lines 34-40. | Pre-registered T78-P1 search returned the reviewed Ousterhout/evidence preference first; T78 remained documentation/evidence-only. | Direct for returned preference; indirect for no product-surface behavior change because that is proven by the committed T78 report and commit diff, not by the trace alone. | `TRANSCRIPT_VISIBLE_OUTCOME` | Outcome depends on repo diff interpretation; no independent judge. | `PENDING` | Field shape useful and shows why trace alone was insufficient without external doc refs. |
| T82-4 | `019e82a7-c9ff-7d01-b4cd-8f802044bca8` | `019e82a8-ba3e-7fc3-a5b8-0dd0ecde4ced` | `follow_user_preference` | Commit `eee76d292b615d09e011baacf04c5102c408e5d3`; commit `3a67fa2c589473b6b51c5f897cb35e97935eae31`; `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` lines 50, 105, 122. | Pre-registered T78-P2 search returned commit-discipline preference first; T78 commits staged intended docs and left root `AGENTS.md` untracked. | Direct for returned preference; indirect for staged-file discipline through committed T78 report. | `TRANSCRIPT_VISIBLE_OUTCOME` | Git status evidence is summarized in T78 doc rather than preserved as raw terminal output. | `PENDING` | Field shape useful; durable refs can preserve what the feedback note only stated informally. |
| T82-5 | `019e82af-52a0-7fd1-ab7f-33060f04d4a2` | `019e82b3-52cb-7480-8f20-4924bfb22baf` | `plan_work` | `telemetry(get_trace)` returned startup orient query and memory IDs; `telemetry(list_feedback)` note said orient surfaced T78 plan, Claude Bridge caveat, harness gate, and commit preference; `docs/BRAIN_HARNESS_T79_CLAUDE_BRIDGE_OBSERVABLE_TASK_AUDIT_2026-06-01.md` lines 124-145 records later harness inconclusive outcome. | Retrieval/process context was available for T79 startup. The feedback row does not itself point to a durable downstream outcome artifact. | Insufficient for downstream task outcome; direct only for retrieval/process context. | `SELF_REPORTED_OUTCOME` | Later T79 doc exists, but importing it would turn a generic startup feedback note into a post-hoc outcome link. This row tests refusal to over-link. | `PENDING` | Field shape useful because it distinguishes positive self-report from durable outcome evidence. |

## Result Against Threshold

The same field shape was useful for all five rows. The weak T82-5 row became clearer because the
artifact could say exactly why positive task outcome fields should remain `SELF_REPORTED_OUTCOME`
instead of being promoted to task-outcome evidence.

This supports a future approval packet for a small, reviewable controlled-outcome artifact format.
It does not justify schema/API/storage work yet, because all classifications are still authored by
the using agent and reviewer agreement remains `PENDING`.

## Decision

Do not add `outcome_evidence` fields or any new storage/public MCP surface.

T82 validates the value of an explicit outcome-link artifact shape, not an implementation. The next
non-gated slice should be either:

1. A second-reader review of this T82 artifact, ideally through a harness that can inspect the file
   and evidence refs without writes.
2. A controlled artifact pilot with an independent reviewer column filled after blind review.

Any stored schema, MCP API, new tool, harness write, ranking change, document indexing, migration,
lifecycle mutation, or `orient` expansion still requires explicit user approval.
