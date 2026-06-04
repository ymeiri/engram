# T233 Approval Packet: T217/T221/T223/T225/T227/T229/T232 Runtime Refresh

Date: 2026-06-04
Status: pending exact user approval
Scope: refreshed approval request for installing the current `engram-cli` binary, restarting the
Engram daemon, temporarily injecting a daemon-process `ENGRAM_EXTERNAL_SESSION_ID`, and running
telemetry/read-only live validation for the committed T217, T221, and T223 MCP source fixes plus
the T225, T227, T229, and T232 focused source fixtures.

This packet is a request for approval, not approval itself. No binary install, daemon restart,
temporary daemon environment injection, harness install, hook/settings/adapter write, lifecycle
archive/apply, `lint apply_safe`, M6 action, quarantine inspection, migration action,
schema/storage/index change, public MCP change, document-index behavior change, `orient` payload
change, ranking-source change, shell profile/PATH/auth/service configuration change, rollback,
force-kill, deletion, old-binary reinstall, or user-owned file change has been run for T233.

T233 supersedes T230. T230 was correct when authored, but T232 intentionally changed
binary-relevant `engram-tests/tests/memory_tests.rs` after T230 to add combined source fixture
coverage for the stale live `memory(action="list", project_name="engram", status_filter="active",
tags=["current-plan"], limit=5)` request shape. The old T230 first checks must now stop if
executed. Any future runtime refresh should use this T233 packet instead.

T233 intentionally allows only:

- installing the current `engram-cli` binary to `/Users/yuval.meiri/.local`;
- clean daemon stop/start for the global Engram daemon;
- temporary daemon-process `ENGRAM_EXTERNAL_SESSION_ID` injection for telemetry-only T217/T229
  validation;
- telemetry trace/feedback writes needed to prove T217/T229 fallback behavior; and
- read-only live MCP memory-list validation needed to prove T221 project-scope inference, T223
  scoped post-filter limit behavior, T225 project-name-plus-limit behavior, T227
  project-name-plus-current-plan-tag behavior, and T232 combined project-name-plus-current-plan-tag
  plus limit behavior.

It does not authorize active memory writes beyond telemetry trace/feedback records, lifecycle
mutation, migration work, document-index behavior changes, harness writes, native Claude runs, or
user-owned-file writes.

## Research Question

Can Engram safely refresh the installed MCP runtime to include T217, T221, T223, T225, T227, T229,
and T232, then prove live that:

1. omitted, empty, and whitespace `external_session_id` request values use
   `ENGRAM_EXTERNAL_SESSION_ID` from the serving daemon process environment;
2. the exact T229 `telemetry(action="record_trace")` omitted-label path persists that runtime env
   label in the returned trace;
3. explicit `external_session_id` request values still win and 256/257 boundary validation remains
   intact;
4. `memory(action="list", project_name="engram")` behaves as a project-scoped list request when
   `scope_type` is omitted;
5. scoped `memory(action="list", ..., limit=1)` applies the requested limit after scope filtering;
6. the T225 combined project-name-only plus `limit=1` path works in live runtime;
7. the T227 startup-style project-name-only plus `tags=["current-plan"]` path works in live runtime;
   and
8. the T232 combined project-name-only plus `tags=["current-plan"]` plus `limit=5` path excludes
   out-of-scope current-plan items in live runtime.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | If binary-relevant source remains at baseline `cd59424`, installing `/Users/yuval.meiri/.local/bin/engram`, cleanly restarting the daemon once with a unique temporary `ENGRAM_EXTERNAL_SESSION_ID`, and exercising the T217/T221/T223/T225/T227/T229/T232 live MCP paths will prove all source fixes in installed runtime. A final restart without the temporary env will prove the label is not sticky. |
| Null | Runtime refresh or validation does not demonstrate one or more fixes: the wrong binary is serving MCP, daemon env did not propagate, telemetry observation is ambiguous, live memory data cannot demonstrate the scoped/tag/limit invariants, or source tests missed a live-path issue. |
| Simpler alternative | Keep T230 as the active packet and ask for approval anyway. Rejected because T230's binary-relevant invariant is now stale after T232. |
| Failure | The packet hides unrelated runtime, public MCP, payload, ranking, `orient`, schema/storage/index/document-index, lifecycle, harness, native Claude, M6/migration/quarantine, user-owned-file, PATH/profile/auth/service configuration, rollback, force-kill, deletion, or old-binary reinstall work inside the refresh. |

## Current Evidence

- T217 source implementation is committed as
  `78eba3c643e3921fb1c19311aef2d1e0cd95b2d0`
  (`Add MCP external session env fallback`).
- T221 source implementation is committed as
  `e8b1cc732a4108b827fb8dea6b2be43d095dfe66`
  (`Infer project scope for memory list`).
- T223 source implementation is committed as
  `19707e60b9126b2fcdfabbe5fe9c0562a44c7f03`
  (`Apply memory list limits after scope filters`).
- T225 source-fixture hardening is committed as
  `ff2d6fd5199279eb96b9d2e2e044cece4cd23607`
  (`Harden memory list project-name limit fixture`).
- T227 source-fixture hardening is committed as
  `993e3c991e70247abc74477c0879a633410858ec`
  (`Harden memory list project-name tag fixture`).
- T229 source-fixture hardening is committed as
  `d953d16d857d0457d3d1d951eccbc630d33a28b1`
  (`Harden telemetry env fallback fixture`).
- T230 approval packet is committed as `1c2665f` but is now stale for execution because T232
  changed binary-relevant MCP integration tests after it.
- T232 source-fixture hardening is committed as
  `cd59424f9cb4ae9ec90aa5af7328774c0f7784a8`
  (`Harden memory list tag limit fixture`).
- Fresh read-only diff checks from T232 baseline
  `cd59424f9cb4ae9ec90aa5af7328774c0f7784a8` to current HEAD show no binary-relevant committed,
  staged, or unstaged drift.
- T232 validation passed:
  - `cargo test -p engram-tests test_mcp_memory_list_project_name_current_plan_tags_preserves_limit --test memory_tests -- --exact`
  - adjacent T221/T223/T225/T227 memory-list fixtures
  - full `cargo test -p engram-tests --test memory_tests` with 35 passing tests
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- Current read-only live pre-state:
  - `command -v engram` resolves to `/Users/yuval.meiri/.local/bin/engram`.
  - `/Users/yuval.meiri/.local/bin/engram --version` reports `engram 0.1.0`.
  - `/Users/yuval.meiri/.local/bin/engram` hash is
    `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`.
  - `/Users/yuval.meiri/.cargo/bin/engram` hash is
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
  - `/Users/yuval.meiri/.local/bin/engram daemon status` reports the global daemon running on
    port `8765`, PID `21398`.
  - `ps -axo pid,ppid,command | rg '^ *21398 '` reports
    `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.
  - `printenv ENGRAM_EXTERNAL_SESSION_ID` returns no value in the authoring shell.
  - `git status --short` shows only the known user-owned untracked root `AGENTS.md`.
  - read-only live `memory(action="list", project_name="engram", status_filter="active",
    tags=["current-plan"], limit=5)` still returns the active Engram current-plan item plus an
    out-of-scope `voice-layer` current-plan item, proving the installed runtime has not picked up
    the memory-list fixes yet.

## Binary-Relevant Invariant

T233 allows committed docs-only approval/report drift from source baseline
`cd59424f9cb4ae9ec90aa5af7328774c0f7784a8` only if fresh execution-start checks prove no
binary-relevant committed, staged, or unstaged drift.

Binary-relevant paths are deny-by-default and include:

- `Cargo.toml`
- `Cargo.lock`
- `engram-core/`
- `engram-store/`
- `engram-embed/`
- `engram-index/`
- `engram-mcp/`
- `engram-cli/`
- `engram-tests/`
- `scripts/`
- `.cargo/`
- any other changed path that is not clearly documentation-only

Docs-only paths are:

- `docs/**`
- tracked root Markdown documentation files such as `README.md`, `CHANGELOG.md`,
  `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md`

The existing untracked root `AGENTS.md` may remain untracked and user-owned. It must not be staged,
edited, committed, adopted, or treated as part of this approval.

Any untracked, staged, unstaged, or committed path outside the docs-only allow-list is invalidating
unless the user sees the exact path and explicitly re-approves a refreshed packet.

## Required First Checks After Exact Approval

If the user explicitly approves this packet, the first actions must be read-only checks. Run them
before any install command:

```text
git diff --name-status cd59424f9cb4ae9ec90aa5af7328774c0f7784a8..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --cached --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git status --short
command -v engram
/Users/yuval.meiri/.local/bin/engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
/Users/yuval.meiri/.local/bin/engram daemon status
ps -axo pid,ppid,command | rg '^ *<daemon-pid> '
printenv ENGRAM_EXTERNAL_SESSION_ID
```

The first three diff commands must return empty output. `git status --short` must show only the
known untracked user-owned `AGENTS.md`, or else only paths that are explicitly docs-only and
unmodified before installation. If there is any ambiguity, stop.

At execution start, `/Users/yuval.meiri/.local/bin/engram` must still have hash
`1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`, and
`/Users/yuval.meiri/.local/bin/engram daemon status` must still show PID `21398`. If either already
changed, stop before install because the runtime was mutated outside this approval packet.

If `printenv ENGRAM_EXTERNAL_SESSION_ID` returns a value, stop and report it. The validation needs
to control daemon env explicitly with `env ENGRAM_EXTERNAL_SESSION_ID=...` and
`env -u ENGRAM_EXTERNAL_SESSION_ID ...`; a pre-existing parent value makes cleanup evidence
ambiguous.

## Proposed Approved Sequence

If and only if the required first checks pass after exact approval, the authorized operational
sequence is exactly:

```text
cargo install --path engram-cli --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram daemon stop
env ENGRAM_EXTERNAL_SESSION_ID=t233-runtime-env-<stamp> /Users/yuval.meiri/.local/bin/engram daemon start
```

After `cargo install` completes and before daemon restart, verify that
`/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
`1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`. If it does not differ, stop
before daemon restart.

After starting the daemon with the temporary env, verify:

```text
command -v engram
/Users/yuval.meiri/.local/bin/engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
/Users/yuval.meiri/.local/bin/engram daemon status
ps -axo pid,ppid,command | rg '^ *<new-daemon-pid> '
```

The daemon PID must differ from pre-state PID `21398`, remain stable through validation, and the
binary hash must remain stable through validation.

## Live Validation

T217/T229 telemetry validation:

- record telemetry with omitted `external_session_id`; the recorded trace must use the temporary
  daemon env label, proving the exact T229 `telemetry(record_trace)` path;
- record telemetry with empty and whitespace `external_session_id`; both must use the temporary
  daemon env label;
- record telemetry with an explicit request label; that explicit label must win;
- validate the 256-character boundary succeeds and the 257-character boundary fails;
- exercise the touched MCP call sites for `search`, `orient`, `telemetry(record_trace)`,
  `telemetry(submit_feedback)`, and `memory(changes_since)`.

T221/T223/T225/T227/T232 read-only memory-list validation:

- run `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"], limit=5)`; every returned item must be Engram project-scoped and tagged
  `current-plan`, proving the startup-style combined T227/T232 path on live data;
- run `memory(action="list", status_filter="active", project_name="engram", limit=1)`;
  it must return `count == 1` and an Engram project-scoped item, proving project-name-only scope
  inference plus post-filter limit preservation on the live path;
- run `memory(action="list", status_filter="active", scope_type="project",
  project_name="engram")`; if the result has fewer than two active Engram project items, stop and
  report that the live dataset cannot demonstrate T223 beyond source tests;
- if the unbounded scoped list has at least two active Engram project items, run
  `memory(action="list", status_filter="active", scope_type="project",
  project_name="engram", limit=1)`; it must return `count == 1` and an Engram project-scoped item.

All validation output should be recorded in a T233 execution report. Do not write active memory
other than telemetry trace/feedback records during the temporary-env validation window.

## Cleanup Validation

After validation, restart the daemon without the temporary env:

```text
/Users/yuval.meiri/.local/bin/engram daemon stop
env -u ENGRAM_EXTERNAL_SESSION_ID /Users/yuval.meiri/.local/bin/engram daemon start
```

Then prove the temporary label is not sticky:

- daemon PID differs from the temporary-env PID;
- `printenv ENGRAM_EXTERNAL_SESSION_ID` is still empty in the caller shell;
- an omitted-label telemetry trace no longer uses the `t233-runtime-env-<stamp>` label.

## Exact Approval Phrase

Approve T233: execute the T217/T221/T223/T225/T227/T229/T232 runtime refresh from
`docs/BRAIN_HARNESS_T233_T217_T221_T223_T225_T227_T229_T232_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md`.
Install the current `engram-cli` binary to `/Users/yuval.meiri/.local`, restart the Engram daemon,
temporarily start the daemon with `ENGRAM_EXTERNAL_SESSION_ID=t233-runtime-env-<stamp>` for
telemetry-only T217/T229 live validation, validate omitted/empty/whitespace env fallback,
explicit request precedence, 256/257 external_session_id boundary behavior, touched T217/T229 call
sites `search`/`orient`/`telemetry(record_trace)`/`telemetry(submit_feedback)`/
`memory(changes_since)`, read-only T227/T232 `memory(list)` project_name-only current-plan tag
filtering with `limit=5`, read-only T221/T225 `memory(list)` project_name-only scope filtering
with `limit=1`, read-only T223 explicit scoped post-filter limit behavior, then restart the daemon
without the temporary env and prove the label is not sticky. Do not execute stale T219, T222, T224,
T226, T228, or T230, edit installed hooks/settings/adapters or user-owned files, run harness
install, use adopt_user_owned, run native Claude, run Claude Bridge writes, run
M6/migration/quarantine, mutate lifecycle state, run lint apply_safe, change public MCP params or
response shape, change ranking/`orient` payload, schema/storage/index/document-index behavior,
PATH/profile/auth/service configuration outside the temporary process env used by this packet,
rollback, force-kill, deletion, or old-binary reinstall.
