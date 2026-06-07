# T219 Approval Packet: T217 MCP External Session Runtime Refresh

Date: 2026-06-04
Status: pending user approval
Scope: approval request for installing the current `engram-cli` binary, restarting the Engram
daemon, and running telemetry-only live validation for the committed T217 MCP
`ENGRAM_EXTERNAL_SESSION_ID` fallback.

This packet is a request for approval, not approval itself. No binary install, daemon restart,
temporary daemon environment injection, harness install, hook/settings/adapter write, lifecycle
archive/apply, `lint apply_safe`, M6 action, quarantine inspection, migration action,
schema/storage/index change, public MCP change, document-index behavior change, `orient` payload
change, ranking-source change, shell profile/PATH/auth/service configuration change, rollback,
force-kill, deletion, old-binary reinstall, or user-owned file change has been run for T219.

T219 intentionally allows only telemetry trace/feedback writes required to prove the fallback. It
does not authorize memory, lifecycle, migration, document-index, harness, or user-owned-file writes.

## Research Question

Can Engram safely refresh the installed MCP runtime to include T217, then prove through live daemon
MCP calls that omitted, empty, and whitespace `external_session_id` request values use
`ENGRAM_EXTERNAL_SESSION_ID` from the serving daemon process environment, while explicit request
values still win and overlong explicit request values still fail validation?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T217 is source-complete but not installed. If binary-relevant source remains at the T217 baseline, installing `/Users/yuval.meiri/.local/bin/engram`, cleanly restarting the daemon once with a unique temporary `ENGRAM_EXTERNAL_SESSION_ID`, and exercising the touched MCP telemetry call sites will prove the fallback in live runtime. A final restart without the temporary env will prove the validation label is not sticky. |
| Null | Runtime refresh or validation does not demonstrate the fallback: the wrong binary is serving MCP, the daemon did not inherit the temporary env, the telemetry oracle is ambiguous, or source tests missed a live-path issue. |
| Simpler alternative | Leave the installed runtime as-is and rely on T217 source tests only. This avoids daemon churn but leaves live MCP external-session joinability unvalidated. |
| Failure | The packet hides unrelated runtime, public MCP, payload, ranking, `orient`, schema/storage/index/document-index, lifecycle, harness, M6/migration/quarantine, user-owned-file, PATH/profile/auth/service configuration, rollback, force-kill, deletion, or old-binary reinstall work inside the refresh. |

## Current Evidence

- T217 source implementation is committed as
  `78eba3c643e3921fb1c19311aef2d1e0cd95b2d0`
  (`Add MCP external session env fallback`).
- T217 result doc:
  `docs/BRAIN_HARNESS_T217_MCP_EXTERNAL_SESSION_ENV_FALLBACK_2026-06-04.md`.
- T217 source validation passed:
  - `cargo test -p engram-mcp external_session_id`
  - `cargo test -p engram-tests --test telemetry_tests mcp_telemetry_tool_rejects_too_long_external_session_id -- --exact`
  - `cargo test -p engram-tests --test telemetry_tests`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- T217 changed only the private MCP-side resolver and existing telemetry call sites:
  - unified `search` trace recording;
  - `orient` trace recording through `OrientInput`;
  - `telemetry(action="record_trace")`;
  - `telemetry(action="submit_feedback")`;
  - `memory(action="changes_since")` trace recording.
- T218 reconciled startup-facing docs after T217 but did not refresh runtime:
  `docs/BRAIN_HARNESS_T218_EXTERNAL_SESSION_STARTUP_DOC_RECONCILIATION_2026-06-04.md`.
- Current read-only pre-state:
  - `command -v engram` resolves to `/Users/yuval.meiri/.local/bin/engram`.
  - `/Users/yuval.meiri/.local/bin/engram --version` reports `engram 0.1.0`.
  - `/Users/yuval.meiri/.local/bin/engram` hash is
    `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`.
  - `/Users/yuval.meiri/.cargo/bin/engram` hash is
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
  - `/Users/yuval.meiri/.local/bin/engram daemon status` reports the global daemon running on
    port `8765`, PID `21398`.
  - `git status --short` shows only the known user-owned untracked root `AGENTS.md`.
  - Binary-relevant committed, unstaged, and staged diff checks from T217 baseline are empty.
- Current telemetry quality is improving but does not prove T217. A fresh
  `telemetry(action="real_session_eval", project="engram", limit=50)` report showed
  `external_session_trace_count=15`, `distinct_external_session_count=1`, and
  `unspecified_external_session_trace_count=35`; those labels are not evidence that the T217 MCP env
  fallback is installed or live.

## AI Review

- AI Council recall found no prior exact consultation for this runtime-env fallback validation.
- Fresh AI Council broadcast supported the proposed temporary-env daemon validation and emphasized
  additional stop conditions: prove the actual serving process and binary path, use unique stamped
  trace scenarios, define the telemetry oracle through `telemetry(get_trace)`/feedback responses,
  cover absent/null/empty/whitespace/padded semantics explicitly, verify the 256/257 character
  boundary, stop on daemon supervision or PID ambiguity, and prove the cleanup restart emits a fresh
  non-temporary trace instead of merely checking old data.
- Claude Bridge read-only critique was attempted once and timed out after 90 seconds. This is a
  consultation confound, not validation evidence.

## Measurement Plan

Before execution, T219 must name the expected evidence shape:

| Measurement | Expected Evidence |
| --- | --- |
| Binary-source invariant | Fresh committed, staged, and unstaged binary-relevant diff checks are empty from T217 baseline `78eba3c643e3921fb1c19311aef2d1e0cd95b2d0`. |
| Runtime identity | `command -v engram`, binary hash, daemon status, and process command identify `/Users/yuval.meiri/.local/bin/engram` as the installed binary and serving daemon. |
| Temporary env propagation | Live MCP calls made while the daemon is started with unique `ENGRAM_EXTERNAL_SESSION_ID=t219-runtime-env-<stamp>` produce fresh trace/feedback records carrying exactly that label when request label is absent, null, empty, or whitespace. |
| Explicit precedence | Fresh live trace/feedback records created with explicit request labels carry the trimmed explicit label, not the daemon env label. |
| Boundary validation | A 256-character explicit request label is accepted; a 257-character explicit request label returns the existing validation error containing `external_session_id must be 256 characters or fewer`. |
| Call-site coverage | Fresh trace IDs from `search`, `orient`, `telemetry(record_trace)`, `telemetry(submit_feedback)`, and `memory(changes_since)` prove the fallback or explicit precedence at each T217 call site. |
| Non-sticky cleanup | After a clean stop and restart with `ENGRAM_EXTERNAL_SESSION_ID` removed from the daemon-start process, a fresh no-request-label telemetry trace does not carry the temporary T219 label. |

Validation must use a unique stamp, for example `t219-runtime-env-20260604-<short-suffix>`, in
`external_session_id`, `scenario_id`, `arm`, and/or query text so historical telemetry cannot satisfy
the checks.

## Binary-Relevant Invariant

T219 allows committed docs-only approval/report drift from source baseline
`78eba3c643e3921fb1c19311aef2d1e0cd95b2d0` only if fresh execution-start checks prove no
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
git diff --name-status 78eba3c643e3921fb1c19311aef2d1e0cd95b2d0..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
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
env ENGRAM_EXTERNAL_SESSION_ID=t219-runtime-env-<stamp> /Users/yuval.meiri/.local/bin/engram daemon start
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

Use the running daemon with a unique stamp. The following validation classes are authorized. They
write only telemetry trace/feedback records needed for validation.

| Class | Required Check |
| --- | --- |
| `orient` fallback | Call `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", response_shape="lean", scenario_id="<stamp>-orient-omitted")` without `external_session_id`; then `telemetry(get_trace)` for the returned trace must show `external_session_id="t219-runtime-env-<stamp>"`. |
| `search` fallback | Call `search(project="engram", cwd="/Users/yuval.meiri/projects/engram", query="<stamp> external session fallback search", scenario_id="<stamp>-search-omitted")` without `external_session_id`; then `telemetry(get_trace)` for the returned trace must show the temporary env label. |
| `memory(changes_since)` fallback | Use the `orient` memory cursor and call `memory(action="changes_since", timestamp=<cursor.timestamp>, commit_id=<cursor.commit_id>, project_name="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", query="<stamp> changes since fallback")` without `external_session_id`; then `telemetry(get_trace)` for the returned trace must show the temporary env label. |
| `telemetry(record_trace)` fallback | Call `telemetry(action="record_trace", operation="orient", project="engram", scenario_id="<stamp>-record-omitted")` without `external_session_id`; the returned trace must show the temporary env label. |
| `telemetry(submit_feedback)` fallback | Submit feedback for a T219 trace without `external_session_id`; the returned feedback must show the temporary env label. |
| Empty and whitespace request fallback | Use `telemetry(record_trace)` with `external_session_id=""` and with `external_session_id=" \t\n "`; both returned traces must show the temporary env label. |
| Null request fallback | If the live MCP client can send JSON `null` distinctly from an omitted field, use `telemetry(record_trace)` with `external_session_id=null`; the returned trace must show the temporary env label. If the client cannot express null distinctly, record that as a client limitation rather than a pass. |
| Explicit request precedence | Use `telemetry(record_trace)` with `external_session_id=" t219-explicit-<stamp> "`; the returned trace must show `external_session_id="t219-explicit-<stamp>"`, not the temporary env label. |
| Boundary validation | Use an explicit 256-character `external_session_id` and confirm it is accepted. Use an explicit 257-character non-whitespace `external_session_id` and confirm the call returns the existing validation error containing `external_session_id must be 256 characters or fewer`. |

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
with `telemetry(get_trace)`, and verify it does not carry `t219-runtime-env-<stamp>`. A missing
trace, old trace, historical list result, or ambiguous value is not a pass.

If the cleanup restart fails or cannot prove the temporary label is gone from fresh telemetry,
stop and report the daemon state. Do not force-kill, roll back, reinstall an older binary, edit
service configuration, or delete data under this packet.

## Pass Criteria

T219 succeeds only if all of the following are true:

- The required first checks run before `cargo install` and prove no binary-relevant committed,
  staged, or unstaged drift from source baseline `78eba3c643e3921fb1c19311aef2d1e0cd95b2d0`.
- `git status --short` has no unexpected local changes and does not stage or modify root
  `AGENTS.md`.
- Pre-install binary hash and daemon PID match the packet's pre-state.
- `cargo install --path engram-cli --root /Users/yuval.meiri/.local` succeeds and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
  `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`.
- The daemon stops cleanly, starts with the temporary env, and reports a healthy daemon on port
  `8765` with a new, stable PID and unambiguous process command.
- Fresh `orient`, `search`, `telemetry(record_trace)`, `telemetry(submit_feedback)`, and
  `memory(changes_since)` validation records show the expected env fallback or explicit precedence.
- Empty and whitespace request labels fall back to the temporary env label.
- Null request fallback is either validated through a client capable of sending JSON null or
  explicitly recorded as untested client-surface coverage.
- A padded explicit request label trims and wins over the temporary env label.
- A 256-character explicit request label is accepted and a 257-character explicit request label
  returns the existing validation error.
- The cleanup restart without the temporary env completes cleanly and a fresh trace proves the T219
  temporary label is not sticky.
- No public MCP parameter, response payload shape, ranking, `orient` payload, schema/storage/index
  behavior, document-index behavior, lifecycle state, harness file/settings/hook state,
  M6/migration/quarantine state, user-owned file, PATH/profile/auth/service configuration outside
  the temporary daemon-start env, rollback, force-kill, deletion, or old-binary reinstall is
  changed.
- `obligations(action="doctor")` is clean or any T219-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T219 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T219 packet status | Evidence |
| --- | --- | --- |
| T217 source implementation | Complete | Commit `78eba3c`; T217 report and source validation. |
| T217 installed runtime parity | Pending exact approval | Installed hash remains `1475cd...`, from the prior T207 refresh; T217 has not been installed. |
| MCP env fallback live validation | Pending exact approval | Packet defines temporary daemon env, telemetry oracle, call-site coverage, boundary checks, and cleanup restart. |
| External-session joinability | Source improved, live incomplete | Hosts still need real `ENGRAM_EXTERNAL_SESSION_ID`; Engram does not synthesize labels. |
| `orient` hot path | Preserved | T219 requests no ranking, payload, public MCP, or hot-path responsibility change. |
| Harness readiness | Still gated | No harness install or hook/settings/adapter write is authorized. |
| Lifecycle cleanup | Still gated | No archive/apply or `lint apply_safe` is authorized. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action is authorized. |

## Stop Conditions

Stop before installation if:

- exact T219 approval is missing, partial, stale, broad, conditional, or ambiguous;
- any binary-relevant committed, staged, or unstaged drift is present;
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
- the temporary env label cannot be observed in fresh telemetry for omitted/empty/whitespace
  request labels;
- explicit request labels do not win;
- the 256/257 boundary behavior differs from source validation;
- overlong-label validation crashes the daemon, drops the MCP session without a structured error, or
  otherwise prevents normal cleanup;
- any T217 call site cannot be validated with a reliable telemetry oracle;
- cleanup restart cannot prove the temporary env label is gone from fresh telemetry;
- any validation requires public MCP, payload, ranking, `orient`, schema/storage/index,
  document-index, lifecycle, harness, M6/migration/quarantine, user-owned-file, PATH/profile/auth,
  rollback, force-kill, deletion, old-binary reinstall, or broad behavior work.

## Exact Approval Phrase

```text
Approve T219: execute the T217 MCP external-session env-fallback runtime refresh from
docs/BRAIN_HARNESS_T219_T217_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-04.md. Install the current
engram-cli binary to /Users/yuval.meiri/.local, restart the Engram daemon, temporarily start the
daemon with ENGRAM_EXTERNAL_SESSION_ID=t219-runtime-env for telemetry-only live validation, validate
omitted/empty/whitespace env fallback, explicit request precedence, 256/257 external_session_id
boundary behavior, and touched call sites search/orient/telemetry(record_trace)/
telemetry(submit_feedback)/memory(changes_since), then restart the daemon without the temporary env
and prove the label is not sticky. Do not edit installed hooks/settings/adapters or user-owned
files, run harness install, use adopt_user_owned, run native Claude, run Claude Bridge writes, run
M6/migration/quarantine, mutate lifecycle state, run lint apply_safe, change public MCP params or
response shape, change ranking/orient payload, schema/storage/index/document-index behavior,
PATH/profile/auth/service configuration outside the temporary process env used by this packet,
rollback, force-kill, deletion, or old-binary reinstall.
```
