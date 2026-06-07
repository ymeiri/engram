# Brain Harness T240 T233 Post-T239 Freshness Audit

Date: 2026-06-04
Status: completed read-only T233 runtime-gate freshness audit after T238/T239 docs-only commits.
No install, daemon restart, temporary environment injection, lifecycle mutation, M6 action,
migration action, quarantine action, harness write, source change, ranking/`orient` change, public
MCP change, schema/storage/index behavior change, document-index behavior change, deletion,
rollback, old-binary reinstall, or user-owned-file edit was executed.

## Scope

T240 checks whether the pending T233 runtime-refresh packet remains fresh after the later T238 and
T239 documentation commits. T233 requires exact approval before runtime execution, but its first
checks are read-only and can be repeated to keep the gate state current.

This audit reads source diffs, git state, installed binary hashes, daemon state, shell environment,
and one live read-only memory-list probe. It does not execute T233.

## Research Question

After T238/T239 docs-only commits, is T233 still the exact runtime-refresh gate for the T217/T221/
T223/T225/T227/T229/T232 source changes, and does live runtime still show the stale behavior that
T233 is meant to refresh?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T238/T239 changed only docs, so binary-relevant diffs from T233 baseline remain empty. Installed runtime is still the old local binary and still leaks the out-of-scope current-plan item for the T232 live memory-list shape, keeping T233 valid and pending. |
| Null | T238/T239 or other intervening work changed binary-relevant source, runtime state, daemon process, or parent env enough to invalidate T233. |
| Simpler alternative | Rely on T237 freshness. Rejected because T238/T239 added commits after T237, and T233 requires repeat first checks immediately before execution. |
| Failure | The audit performs part of T233 execution, mutates runtime state, or treats a fresh read-only audit as approval to install/restart. |

## Measurement

Read-only packet and binary-invariant checks:

- Read `docs/BRAIN_HARNESS_T233_T217_T221_T223_T225_T227_T229_T232_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md`.
- `git diff --name-only cd59424f9cb4ae9ec90aa5af7328774c0f7784a8..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo`
  returned empty output.
- `git diff --name-only -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo`
  returned empty output.
- `git diff --cached --name-only -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo`
  returned empty output.
- `git status --short` returned only `?? AGENTS.md`.

Read-only runtime pre-state:

- `command -v engram` returned `/Users/yuval.meiri/.local/bin/engram`.
- `/Users/yuval.meiri/.local/bin/engram --version` returned `engram 0.1.0`.
- `shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram`
  returned:
  - `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`
    for `/Users/yuval.meiri/.local/bin/engram`;
  - `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`
    for `/Users/yuval.meiri/.cargo/bin/engram`.
- `/Users/yuval.meiri/.local/bin/engram daemon status` returned global daemon port `8765`,
  PID `21398`.
- `ps -axo pid,ppid,command | rg '^ *21398 '` returned
  `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.
- `printenv ENGRAM_EXTERNAL_SESSION_ID` returned no value.

Read-only live stale-behavior probe:

- `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"], limit=5)` returned `count=2`.
- The first item was the active Engram current plan
  `019e9204-0991-7c73-a5ac-9434cb48adfa`, `Current plan after T239 telemetry closeout`.
- The second item was the out-of-scope `voice-layer` current-plan item
  `019e1d28-1e80-77b3-8c32-cd470498fab9`, `Output-only rerun still leaked Insight block; stronger
  top-level rule added`.

## Interpretation

T233 remains fresh after T238/T239. There is no binary-relevant committed, staged, or unstaged drift
from T233 source baseline `cd59424f9cb4ae9ec90aa5af7328774c0f7784a8` to current HEAD. The installed
local binary and daemon pre-state also still match T233's expected old runtime: local binary hash
`1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`, daemon PID `21398`, and no
parent-shell `ENGRAM_EXTERNAL_SESSION_ID`.

The live memory-list probe still reproduces the stale runtime behavior that T227/T232 are meant to
fix: a project-name-only current-plan-tag list for `engram` still includes an out-of-scope
`voice-layer` current-plan item. This confirms the source fixes are still not installed in the live
runtime.

## Decision

T240 keeps T233 as the next product-moving gate. Exact T233 runtime refresh/live validation remains
pending and must still repeat its first checks immediately before any install/restart sequence.

T240 is not approval for T233 execution. The runtime step writes outside the repo by installing
`/Users/yuval.meiri/.local/bin/engram` and restarting the daemon, so it remains an explicit
runtime-approval gate.

## Validation

Validation for this docs-only/read-only slice:

- T233 packet read before checking runtime state;
- binary-relevant committed, unstaged, and staged diff checks all empty;
- git status showed only known user-owned untracked root `AGENTS.md`;
- installed binary hashes, daemon status, serving process, and parent shell env checked read-only;
- live memory-list stale-behavior probe reproduced the out-of-scope `voice-layer` current-plan
  leak;
- planned validation is `git diff --check`, exact document indexing for this report and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`, commit, current-plan capture, and obligation doctor.

No Rust build or test is required because T240 changes documentation only and does not touch
binary-relevant source.
