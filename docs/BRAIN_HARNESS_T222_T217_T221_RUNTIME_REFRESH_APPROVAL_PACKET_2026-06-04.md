# T222 Approval Packet: T217/T221 Runtime Refresh

Date: 2026-06-04
Status: pending user approval
Scope: refreshed approval request for installing the current `engram-cli` binary, restarting the
Engram daemon, and running telemetry/read-only live validation for the committed T217 MCP
`ENGRAM_EXTERNAL_SESSION_ID` fallback plus the committed T221 `memory(action="list")`
project-scope inference fix.

This packet is a request for approval, not approval itself. No binary install, daemon restart,
temporary daemon environment injection, harness install, hook/settings/adapter write, lifecycle
archive/apply, `lint apply_safe`, M6 action, quarantine inspection, migration action,
schema/storage/index change, public MCP change, document-index behavior change, `orient` payload
change, ranking-source change, shell profile/PATH/auth/service configuration change, rollback,
force-kill, deletion, old-binary reinstall, or user-owned file change has been run for T222.

T222 supersedes T219. T219 was correct when authored, but T221 introduced intentional
binary-relevant source drift in `engram-mcp/src/tools.rs` and `engram-tests/tests/memory_tests.rs`.
The old T219 first checks must now stop if executed. Any future runtime refresh should use this
T222 packet instead.

T222 intentionally allows only:

- telemetry trace/feedback writes needed to prove the T217 fallback; and
- read-only live MCP list validation needed to prove the T221 project-scope filter.

It does not authorize memory writes, lifecycle mutation, migration work, document-index behavior
changes, harness writes, or user-owned-file writes.

## Research Question

Can Engram safely refresh the installed MCP runtime to include both T217 and T221, then prove live
that:

1. omitted, empty, and whitespace `external_session_id` request values use
   `ENGRAM_EXTERNAL_SESSION_ID` from the serving daemon process environment;
2. explicit `external_session_id` request values still win and boundary validation remains intact;
3. `memory(action="list", project_name="engram")` behaves as a project-scoped list request even
   when `scope_type` is omitted?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | If binary-relevant source remains at baseline `e8b1cc7`, installing `/Users/yuval.meiri/.local/bin/engram`, cleanly restarting the daemon once with a unique temporary `ENGRAM_EXTERNAL_SESSION_ID`, and exercising the T217/T221 live MCP paths will prove both source fixes in installed runtime. A final restart without the temporary env will prove the validation label is not sticky. |
| Null | Runtime refresh or validation does not demonstrate one or both fixes: the wrong binary is serving MCP, daemon env did not propagate, telemetry observation is ambiguous, or source tests missed a live-path issue. |
| Simpler alternative | Keep T219 as the active packet and ask for approval anyway. Rejected because T219's binary-relevant invariant is now stale after T221. |
| Failure | The packet hides unrelated runtime, public MCP, payload, ranking, `orient`, schema/storage/index/document-index, lifecycle, harness, M6/migration/quarantine, user-owned-file, PATH/profile/auth/service configuration, rollback, force-kill, deletion, or old-binary reinstall work inside the refresh. |

## Current Evidence

- T217 source implementation is committed as
  `78eba3c643e3921fb1c19311aef2d1e0cd95b2d0`
  (`Add MCP external session env fallback`).
- T219 approval packet is committed as `1ac3579` but is now stale for execution because T221 changed
  binary-relevant source after it.
- T221 source implementation is committed as
  `e8b1cc732a4108b827fb8dea6b2be43d095dfe66`
  (`Infer project scope for memory list`).
- Fresh read-only diff checks show binary-relevant drift from T219 to current HEAD:
  - `M engram-mcp/src/tools.rs`
  - `M engram-tests/tests/memory_tests.rs`
- Fresh read-only diff checks from T221 baseline `e8b1cc7` to current HEAD show no binary-relevant
  committed drift.
- T217 validation passed:
  - `cargo test -p engram-mcp external_session_id`
  - focused overlong-label MCP telemetry test
  - full telemetry integration target
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T221 validation passed:
  - `cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact`
  - `cargo test -p engram-tests test_mcp_memory_list_filters_by_scope_before_limit --test memory_tests -- --exact`
  - `cargo test -p engram-tests --test memory_tests`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- Current read-only pre-state:
  - `command -v engram` resolves to `/Users/yuval.meiri/.local/bin/engram`.
  - `/Users/yuval.meiri/.local/bin/engram` hash is
    `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`.
  - `/Users/yuval.meiri/.cargo/bin/engram` hash is
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
  - `/Users/yuval.meiri/.local/bin/engram daemon status` reports the global daemon running on
    port `8765`, PID `21398`.
  - `git status --short` shows only the known user-owned untracked root `AGENTS.md`.

## Binary-Relevant Invariant

T222 allows committed docs-only approval/report drift from source baseline
`e8b1cc732a4108b827fb8dea6b2be43d095dfe66` only if fresh execution-start checks prove no
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
git diff --name-status e8b1cc732a4108b827fb8dea6b2be43d095dfe66..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --cached --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git status --short
command -v engram
/Users/yuval.meiri/.local/bin/engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
/Users/yuval.meiri/.local/bin/engram daemon status
ps -p <daemon-pid> -o pid,ppid,command
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
env ENGRAM_EXTERNAL_SESSION_ID=t222-runtime-env-<stamp> /Users/yuval.meiri/.local/bin/engram daemon start
```

After `cargo install` completes and before daemon restart, verify that the
`/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
`1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`. If it does not differ, stop
before daemon restart.

After starting the daemon with the temporary env, verify:

```text
command -v engram
/Users/yuval.meiri/.local/bin/engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
/Users/yuval.meiri/.local/bin/engram daemon status
ps -p <new-daemon-pid> -o pid,ppid,command
```

The daemon PID must differ from pre-state PID `21398`, remain stable through validation, and the
process command must identify `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.
If stop/start fails, the daemon auto-restarts unexpectedly, the PID changes during validation, or
the process path is ambiguous, stop without force-kill or rollback.

## Live MCP Validation

Use the running daemon with a unique stamp such as `t222-runtime-env-20260604-<short-suffix>`.
Validation may write telemetry trace/feedback records only where needed for T217. T221 validation
is read-only.

Required T217 validation classes:

- `orient` without `external_session_id` returns a trace whose `telemetry(get_trace)` record has
  `external_session_id="t222-runtime-env-<stamp>"`.
- `search` without `external_session_id` returns a trace whose `telemetry(get_trace)` record has the
  temporary env label.
- `memory(action="changes_since")` without `external_session_id`, using the `orient` memory cursor,
  returns a trace whose `telemetry(get_trace)` record has the temporary env label.
- `telemetry(action="record_trace")` without `external_session_id` returns a trace with the
  temporary env label.
- `telemetry(action="submit_feedback")` without `external_session_id` returns feedback with the
  temporary env label.
- Empty and whitespace request labels fall back to the temporary env label.
- Explicit request label ` t222-explicit-<stamp> ` trims and wins over the temporary env label.
- A 256-character explicit `external_session_id` is accepted; a 257-character non-whitespace
  explicit `external_session_id` returns the existing validation error containing
  `external_session_id must be 256 characters or fewer`.

Required T221 validation class:

- Call `memory(action="list", project_name="engram", status_filter="active", limit=20)` with
  `scope_type` omitted. Every returned item must have `scope.type="project"` and
  `scope.project_name="engram"`; no returned item may have `scope.project_name="dd-source"` or any
  other project. This is read-only validation against live data and must not add, archive, reject,
  supersede, or mutate MemoryItems.

If any call site cannot be exercised because of missing data, missing MCP tool availability, an
ambiguous telemetry oracle, or unrelated environment failure, stop and record partial validation
instead of inferring a pass.

## Cleanup Restart

If validation reaches the cleanup phase, stop the temporary-env daemon cleanly and restart without
the temporary variable:

```text
/Users/yuval.meiri/.local/bin/engram daemon stop
env -u ENGRAM_EXTERNAL_SESSION_ID /Users/yuval.meiri/.local/bin/engram daemon start
/Users/yuval.meiri/.local/bin/engram daemon status
ps -p <cleanup-daemon-pid> -o pid,ppid,command
```

Then create one fresh no-request-label telemetry trace with a unique cleanup `scenario_id`, fetch it
with `telemetry(get_trace)`, and verify it does not carry `t222-runtime-env-<stamp>`. A missing
trace, old trace, historical list result, or ambiguous value is not a pass.

If the cleanup restart fails or cannot prove the temporary label is gone from fresh telemetry,
stop and report the daemon state. Do not force-kill, roll back, reinstall an older binary, edit
service configuration, or delete data under this packet.

## Pass Criteria

T222 succeeds only if all of the following are true:

- The required first checks run before `cargo install` and prove no binary-relevant committed,
  staged, or unstaged drift from source baseline `e8b1cc732a4108b827fb8dea6b2be43d095dfe66`.
- `git status --short` has no unexpected local changes and does not stage or modify root
  `AGENTS.md`.
- Pre-install binary hash and daemon PID match the packet's pre-state.
- `cargo install --path engram-cli --root /Users/yuval.meiri/.local` succeeds and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
  `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`.
- The daemon stops cleanly, starts with the temporary env, and reports a healthy daemon on port
  `8765` with a new, stable PID and unambiguous process command.
- Fresh T217 validation records show expected env fallback, explicit precedence, and boundary
  validation behavior.
- T221 read-only `memory(list)` validation with only `project_name="engram"` excludes wrong-project
  MemoryItems.
- The cleanup restart without the temporary env completes cleanly and a fresh trace proves the T222
  temporary label is not sticky.
- No public MCP parameter, response payload shape, ranking, `orient` payload, schema/storage/index
  behavior, document-index behavior, lifecycle state, harness file/settings/hook state,
  M6/migration/quarantine state, user-owned file, PATH/profile/auth/service configuration outside
  the temporary daemon-start env, rollback, force-kill, deletion, or old-binary reinstall is
  changed.
- `obligations(action="doctor")` is clean or any T222-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T222 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T222 packet status | Evidence |
| --- | --- | --- |
| T219 runtime packet | Superseded | T221 created binary-relevant drift after T219. |
| T217 source implementation | Complete, not installed | Commit `78eba3c`; T217 validation. |
| T221 source implementation | Complete, not installed | Commit `e8b1cc7`; T221 validation. |
| Installed runtime parity | Pending exact approval | Installed hash remains `1475cd...`, daemon PID `21398`. |
| `orient` hot path | Preserved | T222 requests no ranking, payload, public MCP, or hot-path responsibility change. |
| Lifecycle cleanup | Still gated | No archive/apply or `lint apply_safe` is authorized. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action is authorized. |

## Stop Conditions

Stop before installation if:

- exact T222 approval is missing, partial, stale, broad, conditional, or ambiguous;
- any binary-relevant committed, staged, or unstaged drift is present from `e8b1cc7`;
- `git status --short` contains anything except known user-owned `AGENTS.md` and clearly docs-only
  approval/report files;
- the pre-install `/Users/yuval.meiri/.local/bin/engram` hash no longer matches
  `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`;
- daemon PID is no longer `21398`;
- `command -v engram`, daemon status, or process listing cannot prove the serving binary path;
- `printenv ENGRAM_EXTERNAL_SESSION_ID` returns a parent value.

Stop after installation if:

- the post-install `.local/bin/engram` hash does not change from pre-state;
- daemon stop/start fails, auto-restarts unexpectedly, changes PID during validation, or reports an
  ambiguous process command;
- the T217 temporary env label cannot be observed in fresh telemetry for omitted/empty/whitespace
  request labels;
- explicit request labels do not win;
- the 256/257 boundary behavior differs from source validation;
- overlong-label validation crashes the daemon, drops the MCP session without a structured error, or
  otherwise prevents normal cleanup;
- T221 project-name-only `memory(list)` still returns any wrong-project MemoryItem;
- cleanup restart cannot prove the temporary env label is gone from fresh telemetry;
- any validation requires public MCP, payload, ranking, `orient`, schema/storage/index,
  document-index, lifecycle, harness, M6/migration/quarantine, user-owned-file, PATH/profile/auth,
  rollback, force-kill, deletion, old-binary reinstall, or broad behavior work.

## Exact Approval Phrase

```text
Approve T222: execute the T217/T221 runtime refresh from
docs/BRAIN_HARNESS_T222_T217_T221_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md. Install the
current engram-cli binary to /Users/yuval.meiri/.local, restart the Engram daemon, temporarily start
the daemon with ENGRAM_EXTERNAL_SESSION_ID=t222-runtime-env for telemetry-only T217 live
validation, validate omitted/empty/whitespace env fallback, explicit request precedence, 256/257
external_session_id boundary behavior, touched T217 call sites search/orient/telemetry(record_trace)/
telemetry(submit_feedback)/memory(changes_since), and read-only T221 memory(list) project_name-only
scope filtering, then restart the daemon without the temporary env and prove the label is not
sticky. Do not execute stale T219, edit installed hooks/settings/adapters or user-owned files, run
harness install, use adopt_user_owned, run native Claude, run Claude Bridge writes, run
M6/migration/quarantine, mutate lifecycle state, run lint apply_safe, change public MCP params or
response shape, change ranking/orient payload, schema/storage/index/document-index behavior,
PATH/profile/auth/service configuration outside the temporary process env used by this packet,
rollback, force-kill, deletion, or old-binary reinstall.
```
