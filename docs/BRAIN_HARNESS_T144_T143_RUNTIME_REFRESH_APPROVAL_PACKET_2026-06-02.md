# T144 Approval Packet: T143 Runtime Refresh

Date: 2026-06-02
Status: pending user approval
Scope: refreshed approval request for binary install, daemon restart, and read-only live validation
of the T140/T143 continuation/current-plan approval-gate-context query class

This packet is a request for approval, not approval itself. No binary install, daemon restart,
harness install, hook/settings/adapter write, lifecycle archive/apply, M6 action, quarantine
inspection, migration action, schema/storage/index change, public MCP change, document-index
behavior change, `orient` payload change, ranking-source change, shell profile/PATH/auth/service
change, or user-owned file change has been run for T144.

T144 supersedes T141 as the runtime-refresh approval packet. T141 names older HEAD and pre-T143
evidence; quoting or approving T141 does not authorize the current runtime refresh. Any runtime
refresh now requires exact T144 approval.

## Research Question

Can Engram safely request exact approval to refresh the installed MCP runtime at current HEAD
`ab2f5e25b78f1224a7dbc4d5615c143f286a750b`, so the live daemon can validate the source-proven
T140/T143 direct-search behavior without bundling any harness, lifecycle, migration, schema, index,
document-index, public MCP, `orient`, ranking-source, shell, auth, or service-configuration work?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The current live direct-search miss is stale-runtime behavior: installing current HEAD and restarting the daemon will make live search rank active current-plan guidance above active rolling handoff noise for both the T140 and T143 continuation-with-approval-gate-context query shapes. |
| Null | The live miss remains after confirmed runtime refresh, meaning the source fixture did not cover the real live behavior or another runtime/corpus factor dominates. |
| Simpler alternative | Keep T140/T143 as source/test-only evidence and do not refresh the runtime. This leaves a production-facing daemon stale against committed source. |
| Failure | The refresh requires commands or effects beyond the exact install/restart and read-only validation scope, or partial validation passes one query shape but fails another. |

## Current Evidence

- Current repository HEAD is `ab2f5e25b78f1224a7dbc4d5615c143f286a750b`
  (`Harden current-plan search against fresh handoffs`).
- T140 source ranking repair is committed as `e26bcf8`.
- T142 source baseline is committed as `293b322`; validation passed:
  `cargo fmt --all --check`, focused T140 ranker tests, focused `search_tests`,
  `cargo check -p engram-cli`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, and `git diff --check`.
- T143 source fixture is committed as `ab2f5e2`; validation passed:
  `cargo fmt --all --check`, the focused T143 search test,
  `cargo test -p engram-index memory_ranker::tests -- --nocapture`,
  `cargo test -p engram-tests --test search_tests -- --nocapture`,
  `cargo check -p engram-cli`, and `git diff --check`.
- The active `engram` path resolves to `/Users/yuval.meiri/.local/bin/engram`.
- The active installed binary reports `engram 0.1.0` and still has hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- `/Users/yuval.meiri/.cargo/bin/engram` remains at hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- `engram daemon status` reports the global daemon running on port `8765`, PID `23341`.
- Lean startup `orient` trace `019e8889-f44b-7d32-8363-b0105366eb8a` returned T143
  current-plan memory `019e8888-f5f2-7560-9bea-ac6dcbdf3ff9` first.
- Direct live search trace `019e888a-116a-7923-940c-cc5668240877` for
  `current plan next step continue move forward Engram Brain Harness after T143 T141 T140 approval
  gates` still ranked active rolling handoff `019e8889-12f2-7d73-8633-110a704cef36` first.
- Direct live search trace `019e888a-125f-7273-93a9-ea1a21bc34d4` returned stale T141 handoff
  `019e887a-60e5-7662-82fe-ca1b9ee8726d` above the T143 current-plan memory for a runtime-refresh
  query. This is additional active-handoff noise, not approval to mutate lifecycle state.

## AI Review

- AI Council recall found the T141 consensus: runtime refresh should be a docs-only/default-deny
  packet until exact user approval, with only the known install/restart sequence and read-only
  validation authorized.
- Fresh AI Council broadcast agreed T144 is the right next non-gated slice because T141 is stale
  after T143. The models emphasized exact commands, HEAD pinning, read-only validation boundaries,
  and stop conditions for partial validation or unexpected side effects.
- Claude Bridge read-only critique agreed the boundary is sound and requested four constraints:
  anchor to `ab2f5e2`, cite T142/T143 evidence, include the T143 live-shaped validation query, and
  explicitly say T144 supersedes T141. Claude also recommended stopping if the post-install hash
  still equals the pre-refresh hash or if T140 and T143 query validation disagree.

## Proposed Approval

If the user explicitly approves this packet, the authorized operational sequence is exactly:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
engram daemon stop
engram daemon start
```

Then run read-only validation only:

```text
command -v engram
engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
engram daemon status
```

and read-only MCP validation only:

```text
orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", response_shape="lean")
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan next step continue move forward Engram Brain Harness after T140 T139 T135 approval gates")
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan next step continue move forward Engram Brain Harness after T143 T141 T140 approval gates")
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan next step continue move forward Engram Brain Harness after T142 T141 T140 approval gates")
obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")
git status --short
git diff --check
```

Any missing, partial, broad, conditional, stale-T141, or ambiguous approval remains default-deny.

## Pass Criteria

T144 succeeds only if all of the following are true:

- `cargo install` succeeds with the exact command above and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- `engram daemon stop` and `engram daemon start` complete cleanly.
- `engram daemon status` reports a healthy daemon after restart, with a PID distinct from pre-state
  PID `23341`.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- Lean `orient` still returns compact current-plan guidance and does not require an `orient`
  contract change.
- The T140 live-search query ranks active current-plan guidance above active rolling handoff noise.
- The T143 live-search query ranks active current-plan guidance above active rolling handoff noise.
- The T142/T143 fixture-shaped live-search query ranks active current-plan guidance above active
  rolling handoff noise.
- Approval-gate context remains retrievable in the result set when relevant.
- `obligations(action="doctor")` is clean or any T144-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T144 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T144 status | Evidence |
| --- | --- | --- |
| T140/T143 source behavior | Implemented and source/test validated | T140, T142, and T143 commits plus focused and broad source validation. |
| Installed runtime parity | Missing, gated | Live daemon still runs pre-refresh binary hash `837ef2...` and PID `23341`; live direct search remains handoff-first. |
| Runtime-refresh approval | Refreshed, pending | This packet updates stale T141 to current HEAD `ab2f5e2` and names exact commands, validation queries, pass criteria, exclusions, and stop conditions. |
| `orient` hot path | Preserved | T144 requests no `orient` code, payload, ranking-contract, or hot-path responsibility change. |
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
| Broad search/ranking QA beyond the T140/T143 prompt class and explicit listed validation queries | No |
| Shell profile, PATH, package manager beyond the exact `cargo install`, auth, launch agent, service definition, or environment changes | No |
| Rollback commands, force-kill commands, deleting daemon files, or reinstalling old binaries | No |

## Stop Conditions

Stop and ask before continuing if:

- approval is missing, references only stale T141, is conditional, is ambiguous, or changes the
  allowed scope;
- `HEAD` changes from `ab2f5e25b78f1224a7dbc4d5615c143f286a750b` before execution;
- `cargo install` requires any command other than the exact command above;
- `cargo install` completes but the installed binary hash still equals pre-refresh hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`;
- the install attempts to modify shell profiles, PATH, auth, services, hooks, settings, adapters,
  user-owned files, schema/storage/index data, document indexes, or migration/lifecycle state;
- daemon stop/start fails, PID `23341` remains the running daemon after restart, the daemon is
  unhealthy after 30 seconds, or the MCP runtime becomes unreachable;
- daemon logs or command output show unexpected schema migration, index rebuild, lifecycle action,
  harness install, hook/settings write, M6/migration/quarantine action, or data rewrite;
- it is ambiguous which binary the restarted daemon is running;
- any T140/T143 validation query still ranks active rolling handoff noise above active current-plan
  guidance after runtime identity is confirmed;
- one query class passes while another fails;
- validating the result would require changing source code, ranking, `orient`, lifecycle state,
  schema/storage/index, document-index behavior, public MCP, harness files, or M6 state;
- any write occurs after the final pre-validation read-only check other than the exact approved
  binary install and daemon restart sequence;
- rollback or recovery appears to require deleting daemon data, RocksDB files, user hooks, settings,
  generated adapters, user-owned files, force-killing processes, or reinstalling old binaries.

## Exact Approval Wording

A safe approval phrase is:

```text
Approve T144: execute the refreshed T140/T143 runtime refresh from
docs/BRAIN_HARNESS_T144_T143_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md. This supersedes stale
T141. Install current HEAD ab2f5e25b78f1224a7dbc4d5615c143f286a750b with
`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`, restart the Engram
daemon with `engram daemon stop` and `engram daemon start`, then run only read-only live validation
of the listed T140/T143 continuation/current-plan approval-gate-context search queries. Do not run
harness install, edit hooks/settings/adapters/user-owned files, use adopt_user_owned, mutate memory
lifecycle, run lint apply_safe, run T139 archive, run M6/migration/quarantine, change
orient/ranking source/public MCP/schema/storage/index/document-index behavior, change shell
profile/PATH/auth/service configuration, run rollback commands, force-kill processes, delete daemon
files, or reinstall old binaries.
```
