# Brain Harness T88 Stale Handoff Lifecycle Approval Packet

Status: Pending explicit user approval. No lifecycle write has been run.
Date: 2026-06-01
Scope: Read-only stale handoff search-risk audit plus one exact lifecycle approval request

This packet asks whether to authorize one bounded archive action for the immediately superseded
rolling handoff that still appears beside the active handoff in direct search. It does not archive
anything by itself.

This packet does not authorize T69 count-drift inspection, T70 document indexing, M6 review apply,
candidate decisions, deletion, schema/storage/index behavior changes, public MCP changes, ranking
changes, `orient` changes, or harness adapter/hook changes. Generic `i approve` remains
insufficient for T69, T70, and this T88 lifecycle action.

## Research Question

Can Engram reduce resume-source ambiguity by preparing an exact approval request to archive the
immediately superseded rolling handoff, without changing search ranking, expanding `orient`, or
mutating lifecycle state before explicit approval?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The active handoff already supersedes one stale handoff that direct search still returns at the same score; an exact approval packet is the smallest safe next step. |
| Null | The stale handoff is harmless enough to leave active, so no lifecycle packet is needed. |
| Simpler alternative | Keep only the T87 source-precedence note and rely on agents to use `handoff(get)`. |
| Failure | The packet becomes a broad handoff cleanup, implies a ranking fix, or treats generic approval as lifecycle authorization. |

## Measurement

Read-only evidence collected before drafting the approval target:

- `handoff(action="get", project="engram")` returned active handoff
  `019e82f8-cada-7c31-b073-18ac41986b1e`.
- `memory(action="get", id="019e82f8-cada-7c31-b073-18ac41986b1e")` shows that active handoff
  explicitly supersedes `019e82f3-53bc-7a83-9e39-cfdb29b06c44`.
- `memory(action="get", id="019e82f3-53bc-7a83-9e39-cfdb29b06c44")` shows it is still active,
  still `kind=handoff`, and contains the pre-T87 rolling handoff context.
- Direct memory search trace `019e8300-656d-7933-aac3-56b4d9a031ee` returned the current plan
  first, then active handoff `019e82f8...` and superseded handoff `019e82f3...` at equal score
  `0.8894`.
- Direct memory search trace `019e8300-6530-7793-988d-b3c76cd7f2d5` returned the current plan
  first, then active handoff `019e82f8...` and superseded handoff `019e82f3...` at equal score
  `0.86600006`, with older handoff memories lower in the result set.
- Source inspection shows `MemoryService::archive_memory` calls `with_archive`, which changes the
  item status to `archived` and records archive metadata; it does not delete the item.
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` states that archived memory remains searchable but is not
  loaded by default.

## Consultation

AI Council recall surfaced prior guidance to keep hot-path, lifecycle, and ranking work strictly
scope-bound. A fresh AI Council broadcast to Claude Sonnet 4.6, GPT-5.4, and Gemini 3.1 Pro agreed
that a docs-only approval packet is the correct non-gated slice, with three constraints:

- freeze the exact target IDs before asking for approval;
- do not present archiving as a ranking or search-quality fix;
- require exact ID-scoped approval, not a generic affirmative reply.

Claude Bridge read-only critique agreed with the packet shape and added guardrails: collect exact
UUID evidence before drafting the approval phrase, do not sweep all stale handoffs, and stop if the
active handoff or target status changes before any future archive attempt.

## Completion Matrix

| Area | Status | Current evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Active resume source | Validated | `handoff(get)` returns `019e82f8...`; T87 records source precedence | Future agents must still start with lean `orient`, direct search, and `handoff(get)` |
| Immediate stale handoff noise | Validated as a narrow risk | Search traces `019e8300-656d...` and `019e8300-6530...` return `019e82f8...` and `019e82f3...` at equal scores | Archiving requires exact T88 approval |
| Broader active handoff noise | Partially validated | The same searches return older active handoffs lower in results | Too broad and mixed for this slice; needs separate audit before any broader cleanup |
| Lifecycle mechanism | Implemented | Source shows archive marks status and metadata; docs say archive is not deletion | No lifecycle write is authorized by this packet |
| Ranking / `orient` behavior | Intentionally unchanged | T88 uses lifecycle packet, not ranking or payload changes | Equal-score symptom may recur for future superseded handoffs |
| T69/T70/M6 | Still gated | T69 and T70 exact approval phrases remain pending | T88 does not inspect M6 snapshot files or index documents |

## Frozen Archive Target

Only this MemoryItem is proposed for future archive approval:

```text
019e82f3-53bc-7a83-9e39-cfdb29b06c44
```

Reason:

- it is an active `handoff` MemoryItem;
- it is explicitly superseded by the current active handoff
  `019e82f8-cada-7c31-b073-18ac41986b1e`;
- it still appears beside the current handoff at equal score in direct search;
- its content predates the T87 source-precedence clarification.

The following older active handoff IDs appeared in the read-only search evidence or the active
supersession chain, but they are not included in this T88 archive request:

```text
019e82ec-b571-7830-b8f2-661da91585e7
019e82b0-e41f-7b83-b328-cd3cb2640d1f
019e82ad-5046-7331-a6aa-f5be399fe03f
019e82a5-dbf0-7911-bd06-acdd1307ff4b
019e829c-6bc1-7683-b96e-5c8aa78789b0
019e829c-6bc1-7683-b96e-5c927ecd2bed
019e58e2-0048-7f00-82d4-52157136980c
019dd509-5ef0-7850-ac9d-69828a16a785
019dd3ff-4170-7550-b047-19369d276ea5
```

They need a separate audit because some are older substantive handoffs, some are low-information
Claude session-end handoffs, and the broader active supersession graph is much larger than this
slice.

## Proposed Approved Action

If the user approves this packet exactly, Codex may run only this lifecycle write:

```text
memory(
  action="archive",
  id="019e82f3-53bc-7a83-9e39-cfdb29b06c44",
  archive_reason="Superseded by active rolling handoff 019e82f8-cada-7c31-b073-18ac41986b1e; T88 read-only search traces showed both handoffs surfacing at equal score.",
  archived_by="codex"
)
```

After the archive, Codex may perform only read-only validation:

- `handoff(action="get", project="engram")`;
- `memory(action="get", id="019e82f3-53bc-7a83-9e39-cfdb29b06c44")`;
- the same direct search probes used above;
- a Markdown result report and telemetry feedback for assessable traces.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, generic, or does not name T88 and the exact target ID;
- `handoff(get)` no longer returns `019e82f8-cada-7c31-b073-18ac41986b1e` as the active handoff;
- active handoff `019e82f8...` no longer supersedes `019e82f3...`;
- target `019e82f3...` is no longer active, no longer `kind=handoff`, already archived, or has
  materially changed content;
- the operation would require ranking changes, `orient` changes, schema/storage/index work,
  document indexing, M6 snapshot inspection, migration apply/status/prioritize, deletion, or
  harness adapter/hook writes;
- the user reply also asks to proceed with T69, T70, M6, broad handoff cleanup, or all stale
  memories without a separate exact approval.

## Approval Question

To authorize only the single archive action above, reply exactly:

```text
Approve T88: archive handoff 019e82f3-53bc-7a83-9e39-cfdb29b06c44 only.
```

Any other reply should be treated as non-authorization for T88.
