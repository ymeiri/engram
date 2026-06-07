# Brain Harness T95 Stale Handoff T91 Lifecycle Approval Packet

Status: Pending explicit user approval. No lifecycle write has been run.
Date: 2026-06-01
Scope: Read-only post-T94 stale handoff audit plus one exact lifecycle approval request

This packet asks whether to authorize one bounded archive action for the T91 rolling handoff that
T94 now supersedes and that still appears beside the active T94 handoff in direct search. It does
not archive anything by itself.

This packet does not authorize T69 count-drift inspection, T70 document indexing, T88 archive, M6
review apply, candidate decisions, deletion, schema/storage/index behavior changes, public MCP
changes, ranking changes, `orient` changes, document-index behavior changes, or harness
adapter/hook changes. Generic `i approve` remains insufficient for T69, T70, T88, and this T95
lifecycle action.

## Research Question

After T94 refreshed the rolling handoff, can Engram reduce resume-source ambiguity by preparing an
exact approval request to archive the newly superseded T91 handoff, without changing search
ranking, expanding `orient`, or mutating lifecycle state before explicit approval?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T94 supersedes T91 and direct search still returns both handoffs at the same score; an exact approval packet for only the newly superseded T91 handoff is the smallest safe next step. |
| Null | The stale T91 handoff is harmless enough to leave active, so no lifecycle packet is needed. |
| Simpler alternative | Keep only the T94 source-precedence note and rely on agents to use `handoff(get)`. |
| Failure | The packet becomes a broad handoff cleanup, implies a ranking fix, or treats generic approval as lifecycle authorization. |

## Measurement

Read-only evidence collected before drafting the approval target:

- `handoff(action="get", project="engram")` returned active handoff
  `019e8352-a610-7f92-859f-f9d74b026ba7`.
- `memory(action="get", id="019e8352-a610-7f92-859f-f9d74b026ba7")` shows that active handoff
  explicitly supersedes `019e8316-ebd1-7220-b18e-f0d33110131a`.
- `memory(action="get", id="019e8316-ebd1-7220-b18e-f0d33110131a")` shows it is still active,
  still `kind=handoff`, and contains T90/T91 context rather than T92/T93/T94.
- Direct memory search trace `019e8356-94fb-7d01-91bf-b23ab13d752c` returned the T94 current plan
  first, then active handoff `019e8352...` and superseded handoff `019e8316...` at equal score
  `0.8894`.
- Architecture/risk search trace `019e8356-a12a-7582-b150-c6d109a5c526` returned the T94 current
  plan first, then active handoff `019e8352...`, superseded handoff `019e8316...`, and older
  handoff `019e82f8...` at equal score `0.8894`.
- `lint(action="run", limit=40)` remained read-only with `applied_safe_actions=0`. It reported
  stale repository-scoped current-plan feedback first, then wrong-scope feedback, then
  `superseded_item_still_active` findings with safe action `archive_memory_item`. The first 40
  findings did not include this newer handoff because the active supersession backlog is much
  larger than one item.
- `graph(action="around", node="019e8352-a610-7f92-859f-f9d74b026ba7", depth=2)` showed the edge
  `019e8352...` supersedes `019e8316...`.
- Source inspection shows `MemoryService::archive_memory` calls `with_archive`, which changes item
  status to `archived` and records archive metadata; it does not delete the item.
- `engram-core/src/memory.rs` shows `with_archive` sets `MemoryStatus::Archived`, archive metadata,
  and `updated_at`.

## Consultation

AI Council recall surfaced the T88 lifecycle approval consultation. The stored consensus was to
freeze exact target IDs, avoid presenting archive as a ranking fix, and require exact ID-scoped
approval rather than a generic affirmative reply. T95 applies that already reviewed pattern to the
newly superseded T91 handoff; no new broadcast was run because this packet does not introduce a new
architecture, ranking, migration, or lifecycle mechanism.

## Completion Matrix

| Area | Status | Current evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Active resume source | Validated | `handoff(get)` returns `019e8352...`; T94 records source precedence | Future agents must still start with lean `orient`, direct search, and `handoff(get)` |
| Newly superseded T91 handoff noise | Validated as a narrow risk | Search traces `019e8356-94fb...` and `019e8356-a12a...` return `019e8352...` and `019e8316...` at equal scores | Archiving requires exact T95 approval |
| Older active handoff noise | Still present | Searches also return `019e82f8...` and `019e82f3...`; T88 already freezes one older exact target | Too broad and mixed for this slice; do not sweep |
| Lifecycle mechanism | Implemented | Source shows archive marks status and metadata; no delete path is involved | No lifecycle write is authorized by this packet |
| Ranking / `orient` behavior | Intentionally unchanged | T95 uses lifecycle packet, not ranking or payload changes | Equal-score symptom may recur for future superseded handoffs |
| T69/T70/M6/T88 | Still gated | Exact approval phrases remain pending | T95 does not inspect M6 snapshot files, index documents, or authorize T88 |

## Frozen Archive Target

Only this MemoryItem is proposed for future archive approval:

```text
019e8316-ebd1-7220-b18e-f0d33110131a
```

Reason:

- it is an active `handoff` MemoryItem;
- it is explicitly superseded by the current active handoff
  `019e8352-a610-7f92-859f-f9d74b026ba7`;
- it still appears beside the current handoff at equal score in direct search;
- its content predates the T92/T93/T94 context now required for resume.

The following active handoff IDs are not included in this T95 archive request:

```text
019e82f8-cada-7c31-b073-18ac41986b1e
019e82f3-53bc-7a83-9e39-cfdb29b06c44
019e82ec-b571-7830-b8f2-661da91585e7
019e82b0-e41f-7b83-b328-cd3cb2640d1f
```

They need separate exact approval or a separate audit. In particular, T88 already asks for exact
approval to archive `019e82f3-53bc-7a83-9e39-cfdb29b06c44`; T95 does not replace or execute T88.

## Proposed Approved Action

If the user approves this packet exactly, Codex may run only this lifecycle write:

```text
memory(
  action="archive",
  id="019e8316-ebd1-7220-b18e-f0d33110131a",
  archive_reason="Superseded by active rolling handoff 019e8352-a610-7f92-859f-f9d74b026ba7; T95 read-only search traces showed both handoffs surfacing at equal score.",
  archived_by="codex"
)
```

After the archive, Codex may perform only read-only validation:

- `handoff(action="get", project="engram")`;
- `memory(action="get", id="019e8316-ebd1-7220-b18e-f0d33110131a")`;
- direct search probes used above;
- `lint(action="run", limit=40)`;
- a Markdown result report and telemetry feedback for assessable traces.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, generic, or does not name T95 and the exact target ID;
- `handoff(get)` no longer returns `019e8352-a610-7f92-859f-f9d74b026ba7` as the active handoff;
- active handoff `019e8352...` no longer supersedes `019e8316...`;
- target `019e8316...` is no longer active, no longer `kind=handoff`, already archived, or has
  materially changed content;
- the operation would require ranking changes, `orient` changes, schema/storage/index work,
  document indexing, M6 snapshot inspection, migration apply/status/prioritize, deletion, T88, or
  harness adapter/hook writes;
- the user reply also asks to proceed with T69, T70, T88, M6, broad handoff cleanup, or all stale
  memories without a separate exact approval.

## Approval Question

To authorize only the single archive action above, reply exactly:

```text
Approve T95: archive handoff 019e8316-ebd1-7220-b18e-f0d33110131a only.
```

Any other reply should be treated as non-authorization for T95.
