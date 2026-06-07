# Brain Harness T119 Exact Approval Command Runtime Gap

Status: Completed docs-only runtime-gap audit; no daemon restart
Date: 2026-06-01
Scope: Verify whether T118 exact approval-command ranking is visible through the active runtime
path, and preserve the current-plan gate text needed by the T118 promotion rule.

This slice did not change code, ranking logic, `orient`, public MCP parameters or response shape,
schema/storage/index behavior, document-index behavior, document indexing, T69 inspection, M6
inventory/export/apply, lifecycle state, `lint(action="apply_safe")`, harness adapters, hooks, or
settings. It installed the current CLI binary with `cargo install --path engram-cli`, but did not
restart or kill any running Engram server process.

## Research Question

After T118, can the live Engram runtime recover the active current-plan memory first for the exact
approval-command query `Approve T70: index exact files T59, T68, and T69.`?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The active runtime still uses stale pre-T118 search code; once a fresh runtime can access the store, the T118 search-only promotion plus exact current-plan text should rank current-plan first. |
| Null | The T118 code is running, but the promotion design is insufficient for live data. |
| Simpler alternative | Treat deterministic fixture coverage as enough and defer runtime verification. |
| Failure | Runtime verification is mistaken for T70 approval, document indexing is run, or a daemon restart disrupts current MCP access. |

## Measurement

Startup state:

- Lean `orient` trace `019e84ac-f47b-7a41-933b-7aef0602e45b` returned T118 current-plan memory
  `019e84aa-e9fe-7092-8aee-0559b4f96567` first.
- Direct current-plan search trace `019e84ac-f4c5-7470-a610-fd56969e16e4` returned T118
  current-plan memory first.
- Exact command memory search trace `019e84ad-5db8-77d2-809a-b7d4626a3635` still ranked old T110
  and T109 handoffs above active current-plan for
  `Approve T70: index exact files T59, T68, and T69.`.
- Exact command document search still returned older T64/T59/T58 material before recent T70/T118
  packets, and `docs(action="stats")` still reported `source_count=76`, `chunk_count=4114`,
  `searchable_chunk_count=2102`, `orphan_chunk_count=2012`.
- `real_session_eval(project="engram", limit=50)` still failed the confidence gate with
  `feedback_coverage=0.47999998927116394` and feedback across only two intents.

Current-plan continuity repair:

- T118's ranker intentionally promotes only active current-plan Decision/Rule items containing the
  exact normalized command text.
- Capturing the T118 completion current-plan superseded the prior current-plan memory that carried
  the literal T70 command. That left the active current-plan with generic `Approve T<number>:`
  guidance but without the literal T70 phrase.
- Memory `019e84ae-3a80-7af2-9f15-2dc45977e596` now preserves the exact command text
  `Approve T70: index exact files T59, T68, and T69.` while keeping T70, T69, M6, lifecycle,
  harness, public MCP, schema/storage/index, and document-index gates closed.
- Lean `orient` trace `019e84ae-4e28-76a0-8f35-907786146c1b` returned that T119 current-plan
  memory first.
- Exact command search trace `019e84ae-4d75-7b40-a68b-7a48cf1d5fae` still ranked old handoffs
  above T119 current-plan, proving that memory content alone does not fix the active runtime path.

Runtime check:

- `cargo install --path engram-cli` completed and installed the current repo binary to
  `/Users/yuval.meiri/.cargo/bin/engram`.
- Starting a fresh HTTP MCP server with that binary on port `8799` against the global store first
  failed under sandboxing when RocksDB attempted to rotate `/Users/yuval.meiri/.engram/data/LOG`.
- The same command with approved escalation reached the store but failed with
  `LOCK: Resource temporarily unavailable`, meaning an existing Engram process owns the RocksDB
  lock.
- No `kill`, daemon stop, daemon restart, or session-disrupting action was run.

## Completion Matrix Delta

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| T118 fixture behavior | Implemented and test-validated | Commit `8f06943`; T118 ranker/search tests passed | Does not prove active MCP runtime was restarted. |
| Active current-plan continuity | Repaired for orientation | T119 memory `019e84ae-3a80-7af2-9f15-2dc45977e596`; orient trace `019e84ae-4e28-76a0-8f35-907786146c1b` first | Exact direct search still depends on runtime code freshness. |
| Active MCP exact T70 search | Not validated | Traces `019e84ad-5db8-77d2-809a-b7d4626a3635` and `019e84ae-4d75-7b40-a68b-7a48cf1d5fae` still rank old handoffs first | Requires a fresh runtime with access to the global store or an approved daemon restart/maintenance window. |
| Document index visibility | Risky/unchanged | Exact doc search still returns older packets; stats unchanged | T70 exact-file indexing remains separately gated. |
| M6 confidence gate | Blocked | `real_session_eval(limit=50)` fails feedback coverage and intent diversity | M6 write/apply remains blocked. |

## Interpretation

T119 shows that T118's code change is not enough until the running MCP path is refreshed. The
current in-thread MCP server still behaves like the pre-T118 ranker for the exact T70 command, even
after current-plan memory contains the exact phrase. The likely cause is runtime staleness, not a
new ranking-design failure, because the committed deterministic fixture exercises the intended
promotion rule and the fresh binary cannot be attached to the live store while the existing process
holds the RocksDB lock.

Do not run T70 indexing from this state. Exact approval-command retrieval is still operationally
risky in the active MCP runtime. The next safe step is either:

- an explicit user-approved Engram daemon/runtime refresh window followed by the same exact T70
  search smoke, or
- a non-disruptive source-runtime smoke against an isolated copied/synthetic store, documented as
  weaker than live-global verification.
