# T199 External Session Caller Audit

Date: 2026-06-03
Status: Completed docs-only/read-only caller audit
Scope: External-session telemetry label propagation after T198

## Decision

External-session joinability remains incomplete because live callers are not supplying
`external_session_id`, not because the core telemetry path drops a supplied label.

The next implementation that would make live labels non-null is a host/caller contract or harness
integration change. That is an approval-gated slice. Do not patch CLI defaults, MCP proxy behavior,
Codex host integration, Claude hooks, generated harness adapters, or public request semantics from
this audit.

## Research Question

Where does `external_session_id` become absent in current live Engram traces?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Core telemetry pass-through is intact and live nulls come from callers that omit the label. | Supported. |
| Null | A lower telemetry/storage/reporting layer accepts the label but loses it before persistence or reporting. | Not supported by source or prior tests. |
| Simpler alternative | Agents only need to pass the existing field manually when the host label is known. | Partially supported, but current host label is not available to this Codex thread. |
| Failure | A live fix requires host/harness contract work, not a local telemetry patch. | Supported. |

## Measurement

- Source map every production `external_session_id` call site.
- Re-check current 50-trace telemetry and newest trace rows.
- Use AI Council and Claude Bridge for blind-spot review before classifying the next slice.
- Make no source changes and no gated writes.

## Source Evidence

`rg -n "external_session_id" -S .` found these production paths:

- `engram-core/src/telemetry.rs` stores the optional label on `BrainHarnessTrace` and
  `AgentFeedback`, and provides `BrainHarnessTrace::with_external_session_id`.
- `engram-index/src/telemetry.rs` records traces with the provided label, inherits the trace label
  into feedback when feedback omits one, validates non-empty and maximum-length labels, and
  aggregates trace and feedback external-session counts.
- `engram-store/src/repos/telemetry.rs` persists trace and feedback labels and defines indexes for
  them.
- `engram-mcp/src/tools.rs` exposes `external_session_id` on `SearchRequest`, `OrientRequest`,
  `TelemetryRequest`, and `MemoryRequestNew`, then passes the supplied request field to trace
  creation for search, orient, telemetry record_trace/submit_feedback, and memory changes_since.
- `engram-cli/src/main.rs` hard-codes `external_session_id: None` in the direct CLI `orient` path.
- `engram-cli/src/main.rs` hard-codes `external_session_id: None` in the direct CLI
  `memory changes-since` path.

The MCP stdio proxy in `engram-cli/src/proxy.rs` captures and forwards the MCP transport
`mcp-session-id` header. It does not mutate JSON-RPC tool arguments or inject
`external_session_id` into `tools/call` requests. That transport session ID is not currently the
Brain Harness external-session label.

The Claude project hook settings and scripts pass Claude's `${session_id}` into `harness`
`hook_event` inputs, but they do not set telemetry `external_session_id` for ordinary `orient`,
`search`, or `memory changes_since` calls. Reading those files was observational only; T199 did
not edit installed or project-local hooks.

## Runtime Evidence

`telemetry(action="real_session_eval", project="engram", limit=50)` at
`2026-06-03T18:13:56Z` returned:

- `trace_count=50`
- `feedback_trace_count=20`
- `feedback_coverage=0.4000000059604645`
- `confidence_gate.passed=false`
- `external_session_trace_count=0`
- `distinct_external_session_count=0`
- `unspecified_external_session_trace_count=50`
- `external_session_feedback_count=0`
- `unspecified_external_session_feedback_count=20`
- `task_failure_count=0`
- `bad_memory_used_count=0`

`telemetry(action="list_traces", project="engram", limit=20)` showed all 20 newest project traces
have `external_session_id=null`, including the current startup/search traces
`019e8eae-3162-7021-b322-1225b5ed737c`,
`019e8eae-94a9-79e0-a37f-df335f9b65ba`,
`019e8eae-9651-71c0-ad43-3306f87e5a1e`,
`019e8eae-97ed-7790-8a7a-c874df47193e`,
`019e8eae-99d5-7203-ab1e-809d12a3aac9`, and
`019e8eae-9b84-7793-b01c-ce03b73295ab`.

## AI Review

AI Council recall surfaced the earlier external-session guidance: use caller-supplied
`external_session_id` as secondary correlation metadata, and do not treat sparse labels as proof
of core telemetry failure.

AI Council broadcast agreed that the highest-value next line is external-session caller/host
labeling, but model responses differed on whether a patch could be considered non-gated. Claude
Bridge resolved the boundary more conservatively: a read-only call-site audit is safe and useful,
but any code change that makes live labels non-null is effectively a host/harness contract change
and needs exact approval.

## Completion Matrix

| Area | Status After T199 | Evidence | Remaining Risk |
| --- | --- | --- | --- |
| Core telemetry label storage and reporting | Implemented and validated | T198 source/tests; T199 source map | None found in this slice |
| MCP request pass-through | Implemented | `SearchRequest`, `OrientRequest`, `TelemetryRequest`, and `MemoryRequestNew` pass the supplied label | Callers omit the field |
| CLI direct commands | Implemented but unlabeled | CLI `orient` and `memory changes-since` pass `None` | Adding flags/env defaults is a CLI/host contract change |
| Codex live MCP calls | Missing labels | Current traces all have `external_session_id=null` | Codex host/thread ID is not exposed to this agent as an approved label |
| Claude hook/event labels | Partially separate | Hooks pass Claude `${session_id}` to `harness hook_event` | Ordinary orient/search telemetry is still caller-supplied; hook edits remain gated |
| Current-plan retrieval | Validated for this continuation | Startup orient/search returned T198 first | Stale handoff noise remains below the top item |
| Telemetry feedback confidence | Currently failing | Latest 50-trace window coverage is 40% | Needs feedback scoring and stable coverage before migration confidence claims |
| External-session joinability | Incomplete | `external_session_trace_count=0` | Requires exact-approved host/caller label contract |
| M6 migration | Blocked by approval | Existing T174 packet and inspection reports | Candidate decisions/dry-run/apply remain separate gates |
| Native Claude effective hook behavior | Incomplete | T197 resolved live process only | Effective-hook visibility and prompt-bearing behavior remain separate gates |
| Lifecycle cleanup | Blocked by approval | T187/T191/T193 packets | Archive writes require exact approval |
| Document visibility | Blocked by approval | T181/T184/T188/T192/T194 packets | Exact-file indexing requires exact approval |

## Next Approval Boundary

The smallest product-moving implementation after this audit is not a telemetry-core patch. It is an
exact-approved host/caller label contract slice. A safe approval should name the concrete caller
surface and forbid adjacent changes.

Recommended approval wording:

> Approve T200: implement the smallest external-session label contract for one concrete
> non-harness caller surface identified by T199, with focused tests proving supplied labels reach
> live telemetry; do not change public MCP parameters or payload shape, orient/ranking,
> schema/storage/index/document-index behavior, harness hooks/settings/adapters, native Claude,
> Claude Bridge, M6/migration/quarantine, lifecycle state, deletion, rollback, old-binary
> reinstall, or user-owned files.

If the intended surface is Codex Desktop or another host outside this repository, the repo cannot
complete that implementation without a host API or explicit host integration mechanism. In that
case the next safe repo-local gate is T174 M6 read-only scoping, but it still requires exact user
approval and does not close external-session joinability.

## Non-Actions

T199 did not:

- change source code,
- add CLI flags or environment defaults,
- mutate MCP request/response shape,
- inject transport `mcp-session-id` into Brain Harness telemetry,
- edit Claude hooks/settings/adapters,
- run native Claude or process signals,
- run document indexing,
- archive memory or run `lint apply_safe`,
- run M6/migration/quarantine actions,
- change ranking, `orient`, schema/storage/index, or document-index behavior,
- touch the pre-existing untracked root `AGENTS.md`.
