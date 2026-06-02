# Brain Harness T120 Runtime Refresh Validation

Status: Completed approved runtime refresh validation
Date: 2026-06-02
Scope: Refresh the installed Engram runtime after T118/T119 and verify that the active MCP search
path sees the exact approval-command ranking fix.

This slice updated the installed local binary and restarted the Engram HTTP daemon. It did not
change source code, ranking logic, `orient`, public MCP parameters or response shape,
schema/storage/index behavior, document-index behavior, document indexing, T69 inspection, M6
inventory/export/apply, lifecycle state, `lint(action="apply_safe")`, harness adapters, hooks, or
settings.

## Research Question

After the user-approved runtime refresh, does the active MCP path recover the active current-plan
memory first for the exact approval-command query
`Approve T70: index exact files T59, T68, and T69.`?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T119 was caused by a stale daemon binary. Updating the runtime binary and restarting the daemon should make the committed T118 search-only promotion visible through the live MCP path. |
| Null | The refreshed runtime still ranks old handoffs above current-plan, which would mean the T118 design is insufficient against live data. |
| Simpler alternative | Treat T118 deterministic fixtures as sufficient and defer runtime validation. |
| Failure | Runtime refresh is mistaken for T70 indexing, M6 migration approval, lifecycle cleanup, hook repair, or broad ranking authorization. |

## Measurement

Startup and binary state:

- T119 had installed the current source to `/Users/yuval.meiri/.cargo/bin/engram` with SHA-256
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- `PATH` and the live daemon were still using `/Users/yuval.meiri/.local/bin/engram`, which had
  SHA-256 `aa9f557200ee34367a9218d0b5a2d4a6098286698e2189440b96bfe3b15971f2`.
- `cargo install --path engram-cli --root /Users/yuval.meiri/.local --force` replaced the local
  binary. `/Users/yuval.meiri/.local/bin/engram` then matched the current source hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- The old HTTP daemon was PID `1236` on port `8765`. It was stopped cleanly with
  `/Users/yuval.meiri/.local/bin/engram daemon stop`.
- The refreshed daemon started on port `8765` as PID `85557`.

Post-refresh MCP validation:

- Lean `orient` trace `019e8724-de18-7bd2-abc1-e176a7f6ea6b` recovered T119 current-plan memory
  `019e8506-1b1e-7da0-9a21-96f098765a43` first.
- Exact command search trace `019e8724-de63-7003-8d57-db2a05a53525` returned T119 current-plan
  memory `019e8506-1b1e-7da0-9a21-96f098765a43` first for
  `Approve T70: index exact files T59, T68, and T69.`, above old T110/T109 handoffs.
- Broad continuation search trace `019e8724-dea5-7532-a1f8-9f613fc0a795` also returned T119
  current-plan memory first for `what should happen next Engram Brain Harness`.
- Explicit migration controls preserved default-deny gate behavior:
  - Trace `019e8725-7fdf-76f1-8ae0-8a73419760c5` returned paused migration gate memory
    `019dd35d-1a48-7103-b0e2-390225f8b418` first for
    `Should we run migration_review_apply for M6 now?`.
  - Trace `019e8725-8016-7bb1-aff4-9da9c827384d` returned paused migration gate memory
    `019dd35d-1a48-7103-b0e2-390225f8b418` first and active M6 approval-gate limitation
    `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` second for
    `Approve M6 migration_review_apply write apply deletion rollback plan`.

## Completion Matrix Delta

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| T118 exact approval-command search | Runtime-validated in Codex MCP | Trace `019e8724-de63-7003-8d57-db2a05a53525` ranks active current-plan above old handoffs | Needs current-plan memory refresh so the top item no longer says the runtime still needs refresh. |
| Runtime freshness | Repaired for active daemon | `.local` binary now hash-matches `.cargo` at `ff7e2994...`; daemon PID `85557` is running on port `8765` | Existing stdio proxy processes remain numerous; validate through MCP trace rather than process count alone. |
| Broad next-step retrieval | Healthy for tested prompt | Trace `019e8724-dea5-7532-a1f8-9f613fc0a795` returned T119 current-plan first | This is one live prompt, not broad ranking proof. |
| Explicit M6 apply gates | Preserved | Traces `019e8725-7fdf-76f1-8ae0-8a73419760c5` and `019e8725-8016-7bb1-aff4-9da9c827384d` returned migration gate evidence first | M6 inspection/apply/deletion remains separately approval-gated. |
| Document index visibility | Unchanged/risky | No document indexing was run | T70 exact-file indexing remains a separate gate. |

## Interpretation

T120 supports the preferred T119 hypothesis: the failing active-runtime exact T70 search was caused
by stale runtime state, not by an immediate need for another ranking change. The committed T118
search-only detector is now visible in the active Codex MCP path after refreshing the daemon.

The result is narrow. It validates retrieval of the current gate state for one exact approval
command and confirms migration controls still return default-deny evidence. It does not authorize
T70 exact-file indexing, T69 inspection, M6 apply/deletion/lifecycle mutation, document-index
behavior changes, schema/storage/index changes, public MCP changes, `orient` expansion, broad
ranking changes, or harness writes.
