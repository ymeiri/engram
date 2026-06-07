# Brain Harness T237 T233 Runtime Gate Freshness Audit

Date: 2026-06-04
Status: completed read-only gate-freshness audit. No runtime, lifecycle, migration, harness, source,
ranking, `orient`, public MCP, schema/storage/index, document-index behavior, deletion, rollback,
old-binary, or user-owned-file change was executed.

## Scope

T237 audits whether the exact T233 runtime-refresh packet is still the current product-moving gate
after the docs-only T234, T235, and T236 commits.

This audit is not runtime-refresh approval and does not execute T233. T233 still requires its exact
approval phrase and its required first checks immediately before any install/restart.

## Research Question

After T234/T235/T236 changed only documentation, is T233 still a fresh exact runtime-refresh gate,
or did the later commits invalidate the packet enough to require a new runtime approval packet
before asking the user to execute it?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T233 remains the current exact runtime-refresh gate because fresh checks show no binary-relevant committed, staged, or unstaged drift since source baseline `cd59424f9cb4ae9ec90aa5af7328774c0f7784a8`, and the installed runtime pre-state still matches T233. |
| Null | T233 is stale because later commits changed binary-relevant source, the installed binary, daemon process, or parent environment in a way that makes its required first checks fail. |
| Simpler alternative | Do nothing and continue to point at T233 from T236. Rejected because T233 is deliberately strict about baseline freshness, and later docs commits could confuse future agents unless the invariant is rechecked. |
| Failure | The audit is mistaken for T233 execution approval, hides runtime install/restart, or treats rolling telemetry as Brain Harness completion proof. |

## Measurement

Fresh read-only checks from the T233 source baseline:

- `git diff --name-status cd59424f9cb4ae9ec90aa5af7328774c0f7784a8..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo` returned empty output.
- `git diff --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo` returned empty output.
- `git diff --cached --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo` returned empty output.
- `git diff --name-status cd59424f9cb4ae9ec90aa5af7328774c0f7784a8..HEAD` returned only docs paths:
  `docs/BRAIN_HARNESS_ARCHITECTURE.md`,
  `docs/BRAIN_HARNESS_T233_T217_T221_T223_T225_T227_T229_T232_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md`,
  `docs/BRAIN_HARNESS_T234_STALE_MIGRATION_COMPLETION_LIFECYCLE_APPROVAL_PACKET_2026-06-04.md`,
  `docs/BRAIN_HARNESS_T235_COMPLETION_MATRIX_HEAD_RECONCILIATION_2026-06-04.md`,
  `docs/BRAIN_HARNESS_T236_ROLLING_TELEMETRY_GATE_AUDIT_2026-06-04.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`.
- `git status --short` showed only the known user-owned untracked root `AGENTS.md`.
- `command -v engram` resolved to `/Users/yuval.meiri/.local/bin/engram`.
- `shasum -a 256 /Users/yuval.meiri/.local/bin/engram` returned
  `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`, matching the T233 pre-state.
- `shasum -a 256 /Users/yuval.meiri/.cargo/bin/engram` returned
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- `/Users/yuval.meiri/.local/bin/engram daemon status` reported port `8765`, PID `21398`.
- `ps -axo pid,ppid,command | rg '^ *21398 '` returned
  `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.
- `printenv ENGRAM_EXTERNAL_SESSION_ID` returned no value in the authoring shell.

Fresh risk evidence:

- `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-04T09:25:49.558014Z` returned `feedback_coverage=0.4399999976158142`,
  `distinct_intent_count=2`, `task_failure_count=0`, `bad_memory_used_count=0`,
  `missing_context_count=0`, and `confidence_gate.passed=false`.
- `lint(action="run", limit=20)` still reported wrong-scope active-memory feedback and
  superseded-active lifecycle pressure with `applied_safe_actions=0`.

## Completion Matrix

| Area | Implemented | Validated | Missing Or Risky | Gate |
| --- | --- | --- | --- | --- |
| `orient` hot path | Brain Loop v1, lean shape, current-plan calibration, obligation summary | Source fixtures, Codex/Claude Code smokes from earlier runtime validations | Must remain compact; no payload expansion justified by T237 | No change in this slice |
| MCP memory-list fixes | T221/T223 source behavior plus T225/T227/T232 focused fixtures | No binary-relevant drift since `cd59424`; source tests already recorded in T233 | Installed daemon still stale until runtime refresh | Exact T233 runtime execution |
| External-session fallback | T217/T229 source and fixture coverage | T233 pre-state still matches; parent shell env empty | Live daemon validation still pending; host labels sparse | Exact T233 runtime execution |
| Harness readiness | Generated adapter readiness reported ready in prior matrix | Docs and prior doctor evidence record readiness | Prompt-bearing native Claude behavior and lifecycle compliance remain bounded | No hook/settings writes here |
| Telemetry evidence loop | Real-session report exists with outcome fields | Fresh report shows no task failures or bad-memory-used in sampled feedback | Confidence gate false: 44% feedback coverage and only two intents | More real feedback; no completion claim |
| M6 migration | Review-gated machinery and candidate inspections through 0011 exist | Docs record undecided candidates and T210A/T210B disposition path | Migration completion memory remains stale; candidate dispositions/deferral missing | Human dispositions or explicit deferral; no apply |
| Lifecycle cleanup | Lint and exact lifecycle packets expose stale/superseded items | Fresh lint reports pressure and applies no actions | Archives remain manual/exact-gated | Exact lifecycle approval only |

## Decision

T233 remains the current exact runtime-refresh approval packet. The later T234/T235/T236 commits are
docs-only under the T233 allow-list, and fresh read-only checks show no binary-relevant drift,
unchanged installed binary hash, unchanged daemon PID, and no parent-shell external-session label.

This does not authorize execution. Any future T233 run must repeat the packet's first checks at
execution start and stop if the installed binary hash, daemon PID, parent environment, working tree,
or binary-relevant diff has changed.

## Validation

Validation for T237 is docs-only:

- Fresh Engram `orient` and direct searches before acting.
- Actual repo docs read before planning.
- Fresh git, binary-hash, daemon, environment, telemetry, lint, and obligation dry-run evidence.
- `git diff --check`.
- Exact document indexing for this report and `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`.
- Document-search visibility probe for T237.
- Post-commit `orient`, obligation doctor, current-plan capture, and telemetry feedback.

No Rust build or test is required because this slice changes documentation only and does not touch
binary-relevant source.
