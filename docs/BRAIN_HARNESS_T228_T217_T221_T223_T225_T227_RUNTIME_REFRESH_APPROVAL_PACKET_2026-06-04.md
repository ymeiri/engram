# T228 Approval Packet: T217/T221/T223/T225/T227 Runtime Refresh

Date: 2026-06-04
Status: pending exact user approval
Scope: refreshed approval request for installing the current `engram-cli` binary, restarting the
Engram daemon, temporarily injecting a daemon-process `ENGRAM_EXTERNAL_SESSION_ID`, and running
telemetry/read-only live validation for the committed T217, T221, and T223 MCP source fixes plus
the T225 and T227 focused source fixtures.

This packet is a request for approval, not approval itself. No binary install, daemon restart,
temporary daemon environment injection, harness install, hook/settings/adapter write, lifecycle
archive/apply, `lint apply_safe`, M6 action, quarantine inspection, migration action,
schema/storage/index change, public MCP change, document-index behavior change, `orient` payload
change, ranking-source change, shell profile/PATH/auth/service configuration change, rollback,
force-kill, deletion, old-binary reinstall, or user-owned file change has been run for T228.

T228 supersedes T226. T226 was correct when authored, but T227 intentionally changed
binary-relevant `engram-tests/tests/memory_tests.rs` after T226 to add startup-style
project-name-plus-current-plan-tag coverage. The old T226 first checks must now stop if executed.
Any future runtime refresh should use this T228 packet instead.

T228 intentionally allows only:

- installing the current `engram-cli` binary to `/Users/yuval.meiri/.local`;
- clean daemon stop/start for the global Engram daemon;
- temporary daemon-process `ENGRAM_EXTERNAL_SESSION_ID` injection for telemetry-only T217
  validation;
- telemetry trace/feedback writes needed to prove T217 fallback behavior; and
- read-only live MCP memory-list validation needed to prove T221 project-scope inference, T223
  scoped post-filter limit behavior, T225 combined project-name-plus-limit behavior, and T227
  project-name-plus-current-plan-tag behavior.

It does not authorize memory writes, lifecycle mutation, migration work, document-index behavior
changes, harness writes, native Claude runs, or user-owned-file writes.

## Research Question

Can Engram safely refresh the installed MCP runtime to include T217, T221, T223, T225, and T227,
then prove live that:

1. omitted, empty, and whitespace `external_session_id` request values use
   `ENGRAM_EXTERNAL_SESSION_ID` from the serving daemon process environment;
2. explicit `external_session_id` request values still win and 256/257 boundary validation remains
   intact;
3. `memory(action="list", project_name="engram")` behaves as a project-scoped list request when
   `scope_type` is omitted;
4. scoped `memory(action="list", ..., limit=1)` applies the requested limit after scope filtering,
   not only after tag filtering;
5. the T225 combined project-name-only plus `limit=1` path works in live runtime; and
6. the T227 startup-style project-name-only plus `tags=["current-plan"]` path works in live
   runtime.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | If binary-relevant source remains at baseline `993e3c9`, installing `/Users/yuval.meiri/.local/bin/engram`, cleanly restarting the daemon once with a unique temporary `ENGRAM_EXTERNAL_SESSION_ID`, and exercising the T217/T221/T223/T225/T227 live MCP paths will prove all source fixes in installed runtime. A final restart without the temporary env will prove the label is not sticky. |
| Null | Runtime refresh or validation does not demonstrate one or more fixes: the wrong binary is serving MCP, daemon env did not propagate, telemetry observation is ambiguous, live memory data cannot demonstrate the scoped/tag/limit invariants, or source tests missed a live-path issue. |
| Simpler alternative | Keep T226 as the active packet and ask for approval anyway. Rejected because T226's binary-relevant invariant is now stale after T227. |
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
- T226 approval packet is committed as `66f4801` but is now stale for execution because T227
  changed binary-relevant test source after it.
- T227 source-fixture hardening is committed as
  `993e3c991e70247abc74477c0879a633410858ec`
  (`Harden memory list project-name tag fixture`).
- Fresh read-only diff checks from T227 baseline `993e3c9` to current HEAD show no binary-relevant
  committed, staged, or unstaged drift.
- T217 validation passed:
  - `cargo test -p engram-mcp external_session_id`
  - focused overlong-label MCP telemetry test
  - full telemetry integration target
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T221 validation passed:
  - project-name-only memory-list scope inference fixture
  - explicit-scope list fixture
  - full `memory_tests`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T223 validation passed:
  - the new scoped-limit regression failed before the fix with `count=2` for `limit=1`;
  - explicit scoped-limit fixture
  - adjacent scope/tag regressions
  - full `memory_tests`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T225 validation passed:
  - project-name-only plus `limit=1` fixture
  - adjacent T221/T223 fixtures
  - full `memory_tests` with 33 passing tests
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T227 validation passed:
  - `cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags --test memory_tests -- --exact`
  - adjacent project-name/tag/limit fixtures
  - `cargo test -p engram-tests --test memory_tests` with 34 passing tests
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
  - `printenv ENGRAM_EXTERNAL_SESSION_ID` returns no value in the authoring shell.
  - `git status --short` shows only the known user-owned untracked root `AGENTS.md`.
  - read-only live `memory(action="list", project_name="engram", status_filter="active",
    tags=["current-plan"], limit=5)` still returned an out-of-scope `voice-layer` current-plan item
    during T227 startup, proving the installed runtime has not picked up the memory-list fixes yet.

## Binary-Relevant Invariant

T228 allows committed docs-only approval/report drift from source baseline
`993e3c991e70247abc74477c0879a633410858ec` only if fresh execution-start checks prove no
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
git diff --name-status 993e3c991e70247abc74477c0879a633410858ec..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
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
env ENGRAM_EXTERNAL_SESSION_ID=t228-runtime-env-<stamp> /Users/yuval.meiri/.local/bin/engram daemon start
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

T217 telemetry validation:

- record telemetry with omitted `external_session_id`; the recorded trace must use the temporary
  daemon env label;
- record telemetry with empty and whitespace `external_session_id`; both must use the temporary
  daemon env label;
- record telemetry with an explicit request label; that explicit label must win;
- validate the 256-character boundary succeeds and the 257-character boundary fails;
- exercise the touched MCP call sites for `search`, `orient`, `telemetry(record_trace)`,
  `telemetry(submit_feedback)`, and `memory(changes_since)`.

T221/T223/T225/T227 read-only memory-list validation:

- run `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"], limit=5)`; every returned item must be Engram project-scoped, proving the
  startup-style T227 path;
- run `memory(action="list", status_filter="active", project_name="engram", limit=1)`;
  it must return `count == 1` and an Engram project-scoped item, proving project-name-only scope
  inference plus post-filter limit preservation on the live path;
- run `memory(action="list", status_filter="active", scope_type="project",
  project_name="engram")`; if the result has fewer than two active Engram project items, stop and
  report that the live dataset cannot demonstrate T223 beyond source tests;
- if the unbounded scoped list has at least two active Engram project items, run
  `memory(action="list", status_filter="active", scope_type="project",
  project_name="engram", limit=1)`; it must return `count == 1` and an Engram project-scoped item.

All validation output should be recorded in a T228 execution report. Do not write active memory
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
- an omitted-label telemetry trace no longer uses the `t228-runtime-env-<stamp>` label.

## Exact Approval Phrase

Approve T228: execute the T217/T221/T223/T225/T227 runtime refresh from
`docs/BRAIN_HARNESS_T228_T217_T221_T223_T225_T227_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md`.
Install the current `engram-cli` binary to `/Users/yuval.meiri/.local`, restart the Engram daemon,
temporarily start the daemon with `ENGRAM_EXTERNAL_SESSION_ID=t228-runtime-env-<stamp>` for
telemetry-only T217 live validation, validate omitted/empty/whitespace env fallback, explicit
request precedence, 256/257 external_session_id boundary behavior, touched T217 call sites
`search`/`orient`/`telemetry(record_trace)`/`telemetry(submit_feedback)`/`memory(changes_since)`,
read-only T227 `memory(list)` project_name-only current-plan tag filtering, read-only T221/T225
`memory(list)` project_name-only scope filtering with `limit=1`, read-only T223 explicit scoped
post-filter limit behavior, then restart the daemon without the temporary env and prove the label
is not sticky. Do not execute stale T219, T222, T224, or T226, edit installed
hooks/settings/adapters or user-owned files, run harness install, use adopt_user_owned, run native
Claude, run Claude Bridge writes, run M6/migration/quarantine, mutate lifecycle state, run lint
apply_safe, change public MCP params or response shape, change ranking/`orient` payload,
schema/storage/index/document-index behavior, PATH/profile/auth/service configuration outside the
temporary process env used by this packet, rollback, force-kill, deletion, or old-binary reinstall.
