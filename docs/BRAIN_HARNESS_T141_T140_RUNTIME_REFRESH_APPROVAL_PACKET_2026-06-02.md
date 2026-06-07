# T141 Approval Packet: T140 Runtime Refresh

Date: 2026-06-02
Status: pending user approval
Scope: approval request for binary install, daemon restart, and read-only live T140 validation

This packet is a request for approval, not approval itself. No binary install, daemon restart,
harness install, hook/settings/adapter write, lifecycle archive/apply, M6 action, quarantine
inspection, migration action, schema/storage/index change, public MCP change, document-index
behavior change, `orient` payload change, or ranking change has been run for T141.

The document is docs-only. The operational actions it requests are not docs-only and require exact
user approval before execution.

## Research Question

Can Engram safely request exact approval to refresh the installed MCP runtime after the T140 source
fix, so the live daemon can validate the approved T140 direct-search behavior without bundling any
harness, lifecycle, migration, schema, index, document-index, or `orient` work?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The current live direct-search miss is a stale-runtime problem: installing the current `engram-cli` binary and restarting the daemon will make the live daemon use the committed T140 ranker, after which read-only direct-search validation should rank current-plan guidance above old active rolling handoffs for the tested continuation-with-approval-gate-context prompt class. |
| Null | The live miss is not caused by stale runtime state, and the same ranking problem remains after a confirmed binary refresh and daemon restart. |
| Simpler alternative | Do not refresh the runtime; keep T140 as source/test-only evidence and continue to treat live behavior as stale. |
| Failure | The refresh requires hook/settings/adapters, `harness install`, user-owned adoption, lifecycle cleanup, M6/migration/quarantine work, schema/storage/index/document-index changes, public MCP changes, `orient` changes, or broad diagnosis beyond the approved T140 query class. |

## Current Evidence

- T140 is committed as `e26bcf8` (`Calibrate approval-gate continuation search`).
- T140 source validation passed for the final tree:
  `cargo test -p engram-index memory_ranker::tests -- --nocapture`,
  `cargo test -p engram-tests --test search_tests -- --nocapture`,
  `cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`.
- The current repository HEAD is `e26bcf875c4cfc78bc003bddb8e05e197a11b27a`.
- The current active `engram` path resolves to `/Users/yuval.meiri/.local/bin/engram`.
- The active installed binary reports `engram 0.1.0` and has hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- `/Users/yuval.meiri/.cargo/bin/engram` remains at hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- `engram daemon status` reports the global daemon running on port `8765`, PID `23341`.
- Lean startup `orient` trace `019e8874-f501-7863-b67e-2c6e7cca890f` returned the T140
  current-plan memory `019e8872-9952-7943-a910-f5952064eb82` first.
- Direct live search trace `019e8875-01bb-7763-a9d9-86c10830e3fc` for
  `current plan next step continue move forward Engram Brain Harness after T140 T139 T135 approval
  gates` still ranked active rolling handoff `019e8872-cb39-74b0-9594-e052aeb6d993` above current
  plan. That is consistent with the installed daemon not yet serving the T140 source fix.
- Prior T133A used the same runtime-refresh shape successfully: install with
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`, then
  `engram daemon stop` and `engram daemon start`.
- Source inspection of `engram-cli/src/daemon.rs` shows daemon start uses the current executable,
  runs `serve --http --port <port>`, writes daemon pid/port files, and waits up to 30 seconds for
  `/health`. Stop sends SIGTERM, waits briefly, and cleans daemon pid/port files.

## AI Review

- AI Council recall found prior approval-packet guidance: keep approval requests pending/default
  deny, exact in scope, and separate from lifecycle, migration, harness, `orient`, ranking, public
  MCP, schema/storage/index, and document-index work.
- Fresh AI Council broadcast agreed that a docs-only T141 approval packet is the right next
  non-gated slice. The models converged on exact scope: install the current binary, restart the
  daemon, and run read-only live validation for the T140 prompt class, with hard stop conditions for
  any required config, hook, harness, lifecycle, migration, schema, storage, index, document-index,
  or `orient` work.
- Claude Bridge read-only critique agreed with the packet direction but warned that the packet must
  distinguish the docs-only approval request from the later operational actions. It also requested
  an exact install command, explicit daemon restart approval, version identity evidence, and a
  concrete pass/fail validation criterion.

## Proposed Approval

If the user explicitly approves this packet, the authorized operational sequence is exactly:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
engram daemon stop
engram daemon start
```

Then run read-only live validation only:

```text
command -v engram
engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
engram daemon status
```

and read-only MCP validation:

```text
orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", response_shape="lean")
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan next step continue move forward Engram Brain Harness after T140 T139 T135 approval gates")
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan next step continue move forward Engram Brain Harness after T139 T135 T139 approval gate")
obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")
git status --short
git diff --check
```

Any missing, partial, broad, conditional, or ambiguous approval remains default-deny.

## Pass Criteria

T141 succeeds only if all of the following are true:

- `cargo install` succeeds with the exact command above and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- `engram daemon stop` and `engram daemon start` complete cleanly.
- `engram daemon status` reports a healthy daemon after restart, with a PID different from the
  pre-refresh PID `23341` unless the stop/start command reports an already-stopped daemon and then
  starts a new healthy daemon.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from the pre-refresh hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`, or the command output
  otherwise proves the current source was already installed.
- Lean `orient` still returns compact current-plan guidance and does not require an `orient`
  contract change.
- Direct search for the T140 prompt class ranks current-plan guidance above old active rolling
  handoff noise.
- Approval-gate context remains retrievable in the result set when relevant.
- `obligations(action="doctor")` is clean or any T141-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T141 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T141 status | Evidence |
| --- | --- | --- |
| T140 source behavior | Implemented and source/test validated | Commit `e26bcf8` and the T140 report preserve focused ranker and search fixture validation. |
| Installed runtime parity | Missing, gated | Live direct search trace `019e8875-01bb-7763-a9d9-86c10830e3fc` still shows handoff-first behavior until an approved binary install and daemon restart occur. |
| Runtime-refresh approval | Prepared, pending | This packet names the exact install/restart commands, validation queries, pass criteria, exclusions, and stop conditions. |
| `orient` hot path | Preserved | T141 requests no `orient` code, payload, ranking-contract, or hot-path responsibility change. |
| Lifecycle hygiene | Still gated | No archive/apply or `lint apply_safe` ran; T139 remains a separate exact archive gate. |
| Harness readiness | Still gated | No `harness install`, hook/settings/adapter write, or user-owned adoption ran; T135 remains a separate exact repair gate. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action ran. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| `harness(action="install")`, hook edits, settings edits, adapter/command/skill writes, or installed user hook/settings changes | No |
| `adopt_user_owned=true` or changing user-owned files | No |
| Editing root `AGENTS.md`, `/Users/yuval.meiri/AGENTS.engram.md`, Claude settings snippets, or installed harness files | No |
| Memory lifecycle archive/apply/supersede/reject/delete or `lint(action="apply_safe")` | No |
| T139 stale current-plan archive | No |
| M6 migration inventory/export/status/prioritize/apply, candidate decisions, quarantine inspection, deletion, cleanup, or legacy simplification | No |
| Schema, storage, index, document-index behavior, public MCP, graph, lint rule, telemetry formula, ranking source, or `orient` payload/contract changes | No |
| Broad search/ranking QA beyond the T140 prompt class and one small negative-scope sanity check | No |
| Shell profile, PATH, package manager, auth, launch agent, service definition, or environment changes | No |

## Stop Conditions

Stop and ask before continuing if:

- approval is missing, conditional, ambiguous, or changes the allowed scope;
- `cargo install` requires any command other than the exact command above;
- the install attempts to modify shell profiles, PATH, auth, services, hooks, settings, adapters,
  user-owned files, schema/storage/index data, document indexes, or migration/lifecycle state;
- daemon stop/start fails, the daemon is unhealthy after 30 seconds, or the MCP runtime becomes
  unreachable;
- daemon logs or command output show unexpected schema migration, index rebuild, lifecycle action,
  harness install, hook/settings write, M6/migration/quarantine action, or data rewrite;
- it is ambiguous which binary the restarted daemon is running;
- post-refresh direct search still ranks old active rolling handoff noise above current-plan
  guidance for the T140 query class after runtime identity is confirmed;
- validation exposes a broader ranking regression outside the T140 prompt class;
- validating the result would require changing source code, ranking, `orient`, lifecycle state,
  schema/storage/index, document-index behavior, public MCP, harness files, or M6 state;
- any write occurs after the final pre-validation read-only check other than the exact approved
  binary install and daemon restart sequence;
- rollback appears to require deleting daemon data, RocksDB files, user hooks, settings, generated
  adapters, or user-owned files.

## Exact Approval Wording

A safe approval phrase is:

```text
Approve T141: execute the T140 runtime refresh from
docs/BRAIN_HARNESS_T141_T140_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md. Install the current
engram-cli binary with `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`,
restart the Engram daemon with `engram daemon stop` and `engram daemon start`, then run only
read-only live validation of the T140 continuation/current-plan approval-gate-context search class.
Do not run harness install, edit hooks/settings/adapters/user-owned files, use adopt_user_owned,
mutate memory lifecycle, run lint apply_safe, run T139 archive, run M6/migration/quarantine,
change orient/ranking source/public MCP/schema/storage/index/document-index behavior, or change
shell profile/PATH/auth/service configuration.
```
