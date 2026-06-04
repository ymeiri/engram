# Brain Harness T247 T76 Telemetry Stale Observation Lifecycle Approval Packet

Date: 2026-06-04
Status: Pending exact user approval. No memory lifecycle write is authorized by this document.
Scope: Proposal for one exact archive action on stale project-scoped telemetry observation
`019e8291-40aa-71a0-b16b-9ba7b6446cc6`.

This packet is a request for explicit approval, not approval itself. No `memory(action="archive")`,
`lint apply_safe`, supersession, rejection, deletion, migration action, M6/quarantine action,
harness write, native Claude action, schema/storage/index change, document-index behavior change,
public MCP change, ranking change, `orient` change, rollback, force-kill, legacy simplification,
or user-owned-file change has been run for T247.

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely ask for exact approval to archive one stale project-scoped telemetry observation
whose content was accurate at T76 but is now contradicted by later T244 telemetry evidence, without
using global lint as proof or mutating lifecycle state?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A default-deny, single-target packet is the smallest safe lifecycle follow-up for the T246 candidate: it records the stale-content evidence and future checks without archiving now. |
| Null | The active observation is harmless historical telemetry context and should remain active while future agents continue marking it stale in feedback. |
| Simpler alternative | Keep only the T246 scoping report and do not prepare an exact packet. |
| Failure | The packet treats sampled lint as target evidence, calls the item technically superseded, archives without exact approval, or bundles broad lifecycle cleanup, M6, harness, ranking, `orient`, schema/storage/index, document-index, native Claude, or user-owned-file work. |

## Measurement

This packet used read-only evidence only:

- Lean `orient` trace `019e926f-d65e-7581-800c-7cbdfc28ebba` returned the T246 current plan
  `019e926e-8426-7220-a142-13af04005791` first and no open obligations.
- `memory(action="get", id="019e8291-40aa-71a0-b16b-9ba7b6446cc6")` confirmed the target is
  `status=active`, kind `custom observation`, scope `project:engram`, title
  `Post-T76 rolling telemetry gate remains false`, tags
  `telemetry`, `t76`, `confidence-gate`, `brain-harness`, and
  `updated_at=2026-06-01T09:43:37.898629Z`.
- The target content records a genuine 2026-06-01 T76 point-in-time failing telemetry gate:
  `feedback_coverage=0.6600000262260437` and `confidence_gate.passed=false` because feedback
  spanned only two intents.
- `graph(action="around", node="019e8291-40aa-71a0-b16b-9ba7b6446cc6", depth=1)` showed the
  item scoped to `project:engram` with evidence edges and no supersedes edge in the depth-1 graph.
  The graph tool-call evidence label is mismatched and must not be used as the target-content
  source; use `memory(get)` and docs instead.
- Direct memory search trace `019e9270-05bb-7cc2-b8c6-5897441829f8` returned the T246 current plan
  first and the target second for the exact target/stale query.
- Fresh `telemetry(action="list_feedback", project="engram", limit=40)` included repeated stale
  marks for this exact item, including T243/T244/T246 follow-through feedback. These are related
  follow-through signals, not independent proof.
- T244 later recorded `telemetry(action="real_session_eval", project="engram", limit=50)` at
  `2026-06-04T11:14:07.108605Z` with `feedback_coverage=0.5199999809265137`,
  `distinct_intent_count=7`, `confidence_gate.passed=true`, `task_failure_count=0`,
  `bad_memory_used_count=0`, `wrong_scope_memory_count=0`, and `missing_context_count=0`.
- Fresh `lint(action="run", write=false, limit=80)` was global and did not surface this target in
  the sampled output; it was dominated by non-Engram wrong-scope findings and globally sorted
  `superseded-active` findings. Absence from sampled global lint is not evidence that the target is
  clean.
- Source inspection confirmed `memory.archive` loads and archives exactly one requested item, while
  `lint apply_safe` may archive every `ArchiveMemoryItem` safe-action finding in the report.
- Source inspection confirmed feedback-stale and wrong-scope lint findings intentionally use
  `safe_action=none`; only `superseded-active` findings add `archive_memory_item`.
- Git status before this packet showed only the known user-owned untracked root `AGENTS.md`.

## AI Review

AI Council recall found T246/T139/T48 lifecycle guidance: lifecycle packets must stay
default-deny, exact-target, fresh-evidence-gated, and must not bundle M6, harness, ranking,
`orient`, schema/storage/index, document-index, or broad cleanup.

AI Council broadcast and Claude Bridge critique agreed on these corrections:

- Do not call the target technically superseded. It has no supersedes edge.
- Frame the issue as content staleness: the item was accurate at T76, then later T244 telemetry
  contradicted its present-tense title and active guidance value.
- Do not claim sampled lint confirms this target; sampled global lint did not show it.
- Explain that `lint apply_safe` cannot be the archive path for this target because feedback-stale
  findings have `safe_action=none`, and global `apply_safe` can affect unrelated safe-action
  findings.
- Treat T244's passing gate as a point-in-time contradiction, not proof of permanent telemetry
  health.
- Carry forward T246's caveat that this is one unranked candidate, not a complete Engram lifecycle
  inventory.

## Completion Matrix Delta

| Area | State After T247 Packet | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| Target item `019e8291...` | Exact future archive candidate documented | Fresh get/search/graph/feedback, T244 docs | Requires exact user approval and fresh pre-write checks |
| Target technical state | Active project-scoped custom observation, not superseded | `memory(get)`, graph depth 1 | Archive only by direct exact `memory.archive`, not `apply_safe` |
| Lint signal | Global lint pressure remains, but sampled lint did not show this target | `lint(limit=80)` | Do not require sampled global lint visibility for this target |
| Lifecycle cleanup | Still incomplete | No archive and no `apply_safe` ran | Exact lifecycle write approval remains required |
| M6/cross-harness/hot path | Unchanged | No migration, harness, ranking, or `orient` change | Separate gates remain |

## Proposed Approved Archive

If and only if the user approves with the exact T247 approval wording below, Codex may run one
Memory OS archive write for this single ID:

```text
memory(
  action="archive",
  id="019e8291-40aa-71a0-b16b-9ba7b6446cc6",
  archive_reason="Content-stale project-scoped telemetry observation: MemoryItem 019e8291-40aa-71a0-b16b-9ba7b6446cc6 accurately recorded a T76 point-in-time rolling telemetry gate failure on 2026-06-01, but later T244 evidence recorded telemetry(action=real_session_eval, project=engram, limit=50) at 2026-06-04T11:14:07.108605Z with confidence_gate.passed=true, feedback_coverage=0.5199999809265137, task_failure_count=0, bad_memory_used_count=0, wrong_scope_memory_count=0, and missing_context_count=0; repeated T243/T244/T246 feedback marks this exact item stale. This is a direct exact-target archive, not lint apply_safe and not graph supersession.",
  archived_by="codex"
)
```

This is a human approval exception to feedback-stale lint's `safe_action=none`, not a lint-approved
safe action. `lint apply_safe` is out of scope.

## Required Fresh Pre-Write Evidence

Immediately before any future archive call, in the same execution session, collect fresh read-only
evidence with no intervening Engram memory writes between the final read-only check and archive:

| Check | Required result |
| --- | --- |
| `memory(action="get", id=...)` | Target exists, is `active`, title is unchanged, kind is `custom observation`, scope is `project:engram`, tags still include `telemetry`, `t76`, `confidence-gate`, and `brain-harness`, and `updated_at` is not later than `2026-06-01T09:43:37.898629Z` unless the user re-approves after seeing the drift. |
| Lean `orient` or direct target search scoped to `project=engram` | Current Engram plan remains recoverable, and the target is still interpretable as stale-content evidence rather than current guidance. |
| `graph(action="around", node=..., depth=1)` | Target remains project-scoped; no new direct MemoryItem dependency or replacement relation appears. The known mismatched evidence label alone is not a stop condition if `memory(get)` and docs still match. |
| `telemetry(action="list_feedback", project="engram")` | Recent feedback still names this target as stale, rejected, or review-worthy, or a fresh telemetry/doc check independently confirms the T76 claim remains outdated. |
| Current telemetry/docs check | T244's passing gate evidence still stands as the latest relevant point-in-time telemetry state, or the user re-approves after seeing any newer contradictory telemetry. |
| Optional `lint(action="run", write=false)` | If checked, sampled global lint absence is not a failure; a matching feedback-stale finding remains review evidence only with `safe_action=none`. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T247 does not authorize:

- archiving, superseding, rejecting, editing, reviewing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- treating the target as graph-superseded;
- creating replacement telemetry memory;
- changing handoff semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine,
  candidate decisions, or legacy simplification;
- native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, or interactive Claude
  commands;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- editing root `AGENTS.md`.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T247 wording and target
  ID;
- the target UUID, title, kind, scope, status, tags, `updated_at`, or archive payload differs from
  this packet;
- the target is already archived, superseded, rejected, deleted, or missing;
- fresh evidence suggests the observation remains useful current guidance rather than stale
  historical context;
- fresh graph depth 1 shows a new direct MemoryItem dependency or replacement relation for the
  target;
- fresh feedback/telemetry/docs evidence is inconclusive about the T76 claim's current staleness;
- any Engram memory write occurs after the final fresh pre-write read and before the archive;
- any step appears to require applying automatic lint cleanup, mutating other memories, changing
  ranking, changing `orient`, running M6, inspecting quarantine candidates, executing Claude, or
  executing harness writes.

## Approval Wording

To authorize only the single archive action above, reply exactly:

```text
Approve T247: after fresh matching read-only get/orient-or-search/graph/telemetry-feedback/git/obligations evidence and no intervening Engram memory writes, archive exactly MemoryItem 019e8291-40aa-71a0-b16b-9ba7b6446cc6 with the archive payload in docs/BRAIN_HARNESS_T247_T76_TELEMETRY_STALE_OBSERVATION_LIFECYCLE_APPROVAL_PACKET_2026-06-04.md. Do not run lint apply_safe, archive any other memory, treat the item as graph-superseded, create replacement memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

Any other reply should be treated as non-authorization for T247.
