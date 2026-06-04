# Brain Harness T242 T233 Runtime Refresh Execution Report

Date: 2026-06-04
Status: completed runtime refresh, live validation, daemon pidfile repair, and source hardening.

## Scope

T242 executed the T233 runtime refresh packet and recorded the resulting evidence. During cleanup
validation, T233 exposed a daemon-management race: a stale pidfile could point at a failed child
while another daemon was actually serving the health endpoint. T242 therefore also includes the
minimal source fix and runtime repair needed to make the refresh closeout trustworthy.

T242 does not change public MCP parameters or payload shape, ranking, `orient` responsibilities,
schema/storage/index/document-index behavior, lifecycle state, M6/migration/quarantine state,
harness hooks/settings/adapters, native Claude state, shell profile/PATH/auth configuration, user
owned files, deletion, rollback, or legacy-layer simplification.

## Research Question

Can Engram refresh the installed runtime for T217/T221/T223/T225/T227/T229/T232, prove the live
MCP behavior, and leave the daemon manager with an accurate pidfile/status/process invariant?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Installing the current binary, validating under a temporary daemon env, fixing the discovered pidfile race, and reinstalling the fixed binary will close the runtime gate without changing hot-path payloads or migration/lifecycle state. |
| Null | The live runtime cannot prove one or more T233 invariants, or daemon cleanup leaves ambiguous process state. |
| Simpler alternative | Report only the partial T233 validation and leave pidfile repair for later. Rejected because an inaccurate daemon pidfile would make future status/stop operations untrustworthy. |
| Failure | The slice hides migration, lifecycle, harness, schema/storage/index, ranking, `orient`, rollback, force-kill, deletion, or user-owned-file work inside a runtime refresh. |

## T233 First Checks

The T233 first checks passed before installation:

- binary-relevant committed, staged, and unstaged diffs from baseline
  `cd59424f9cb4ae9ec90aa5af7328774c0f7784a8` were empty;
- `git status --short` showed only known user-owned untracked root `AGENTS.md`;
- `/Users/yuval.meiri/.local/bin/engram` was the old expected hash
  `1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`;
- daemon status reported PID `21398` on port `8765`;
- `ps` confirmed PID `21398` was
  `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`;
- parent-shell `ENGRAM_EXTERNAL_SESSION_ID` was unset.

## Temporary-Env Runtime Validation

The first runtime install completed with installed hash
`31170ebe5227ab144f02cb38e821a09e05c3433dcd5ff054821890462eacb0e6`, then the daemon
was restarted with `ENGRAM_EXTERNAL_SESSION_ID=t233-runtime-env-20260604-31170ebe`.

Telemetry fallback validation passed:

| Case | Trace | Observed external_session_id |
| --- | --- | --- |
| omitted request label | `019e921a-30c1-7ab3-a875-5d2ea7a4be6d` | `t233-runtime-env-20260604-31170ebe` |
| empty request label | `019e921a-315b-7001-8877-b772948e2d0b` | `t233-runtime-env-20260604-31170ebe` |
| whitespace request label | `019e921a-3162-78f1-ad58-699c77ae327f` | `t233-runtime-env-20260604-31170ebe` |
| explicit request label | `019e921a-316c-73b2-a6d1-3d90456eb550` | `t233-explicit-request-label` |
| 256-character request label | `019e921a-3173-7331-9d62-6a0b8a19bffc` | accepted exactly |
| 257-character request label | none | parse error: `external_session_id must be 256 characters or fewer` |

Additional touched call sites were exercised:

- `telemetry(action="submit_feedback")` feedback
  `019e921a-7aed-7e11-8dd9-081284b07c10` inherited the temporary daemon label.
- `search` trace `019e921a-7bb7-7710-960c-0954b9a086d9` inherited the temporary daemon label.
- `orient` trace `019e921a-7c6a-7ca0-853d-822b244a6291` inherited the temporary daemon label.
- `memory(action="changes_since")` trace `019e921a-bb59-7402-97ba-6d09e04605c6` inherited the
  temporary daemon label after passing both cursor `commit_id` and `timestamp`.

Read-only memory-list validation passed:

- `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"], limit=5)` returned `count=1`, only current plan
  `019e920b-949b-7ac3-bea9-ab3f05cd290c`, and no `voice-layer` item.
- `memory(action="list", status_filter="active", project_name="engram", limit=1)` returned
  `count=1` and an Engram project-scoped item.
- explicit `scope_type="project"` unbounded listing had at least 20 active Engram project items.
- explicit `scope_type="project", limit=1` returned `count=1` and an Engram project-scoped item.

## Cleanup Anomaly

The required cleanup restart without the temporary env proved the label was not sticky:

- cleanup trace `019e9226-7ba4-7413-877c-f5e15bbe3eef` had
  `external_session_id=null`.

However, the process invariant failed:

- `daemon status` and `daemon.pid` reported PID `71174`;
- `ps` showed PID `71174` as `<defunct>`;
- the actual serving process was PID `71020`;
- `daemon.log` showed the failed child hit a SurrealDB lock while another daemon was already
  serving port `8765`.

Source inspection found the race in `engram-cli/src/daemon.rs`: `spawn_daemon` wrote the child PID
immediately, before proving that child remained alive after health checks. If another proxy started
a healthy daemon concurrently, health could pass against the other daemon while the recorded child
failed.

## Source Fix

Commit `e2da668` (`Harden daemon start pid validation`) fixes the race narrowly:

- `spawn_daemon` now returns the `Child` instead of writing the pidfile immediately;
- `ensure_daemon_running` waits for health through `wait_for_spawned_daemon`;
- the spawned child must still be running before and after a short post-health stability delay;
- pid/port files are written only after that validation succeeds;
- focused unit tests cover running and already-exited child liveness checks.

Validation for the source fix:

- `cargo test -p engram-cli daemon::tests::spawned_child_liveness -- --nocapture`
- `cargo fmt --all --check`
- `git diff --check`
- `cargo check -p engram-cli`
- `cargo clippy -p engram-cli --all-targets -- -D warnings`
- `cargo test -p engram-tests test_daemon_starts_and_responds_to_health --test multi_session_tests -- --exact --nocapture`
- `cargo test -p engram-tests test_daemon_health_endpoint --test multi_session_tests -- --exact --nocapture`

The first integration test hit a macOS `system-configuration` dynamic-store panic under the
sandbox, then passed when rerun with approved escalation. The second was run with the same
escalation path and passed.

## Final Runtime Repair

The committed fix was installed to `/Users/yuval.meiri/.local/bin/engram`, producing final hash
`1059ae2f44bdcddc56ff88f2a1ed441f51459572d24d9b429248e38df1e6e2dc`.

Runtime repair sequence:

1. Sent clean `SIGTERM` to actual stale HTTP daemon PID `71020`.
2. Ran `/Users/yuval.meiri/.local/bin/engram daemon stop` to clean stale pid/port files.
3. Ran `env -u ENGRAM_EXTERNAL_SESSION_ID /Users/yuval.meiri/.local/bin/engram daemon start`.

Final live state:

- `daemon status` reports PID `14310`, port `8765`.
- `daemon.pid` contains `14310`; `daemon.port` contains `8765`.
- `ps` reports PID `14310` as
  `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.
- parent-shell `ENGRAM_EXTERNAL_SESSION_ID` is unset.
- final omitted-label telemetry trace `019e9247-3f50-7830-b2eb-0d9cbe7beeb9` has
  `external_session_id=null`.
- final `memory(action="list", project_name="engram", tags=["current-plan"], limit=5)` returns
  only current plan `019e920b-949b-7ac3-bea9-ab3f05cd290c`.
- final lean `orient` trace `019e9247-408c-73c2-8df9-01aebdcccb66` works on the repaired daemon.
- `obligations(action="doctor", project="engram")` returns `open=[]`, `warnings=[]`.
- `lint(action="run", limit=5)` still reports the known wrong-scope and superseded-active
  lifecycle findings with no safe action applied.

Residual caveat: older proxy-owned defunct children remain visible under existing proxy processes,
but they are not the active HTTP daemon and were not force-killed. The active daemon invariant is
now correct for PID `14310`.

## Decision

T233 is now executed and closed for the installed runtime. The live daemon has the T217/T221/T223
source fixes, T225/T227/T229/T232 fixture-backed behavior, and the T242 pidfile hardening. The
previous live `voice-layer` current-plan leak for project-scoped current-plan listing is no longer
present.

Remaining goal gates are unchanged:

- M6 still requires human-provided candidate dispositions through T210 or an explicit
  user-approved deferral rationale.
- Lifecycle cleanup remains exact-gated; current lint findings are review pressure, not approval.
- No migration apply, lifecycle archive, harness write, public MCP/schema/storage/index change,
  document-index behavior change, ranking/`orient` expansion, deletion, rollback, or legacy-layer
  simplification was introduced by T242.
