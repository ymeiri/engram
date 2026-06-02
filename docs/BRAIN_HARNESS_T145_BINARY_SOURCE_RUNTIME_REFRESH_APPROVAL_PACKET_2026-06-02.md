# T145 Approval Packet: Binary-Source Runtime Refresh

Date: 2026-06-02
Status: pending user approval
Scope: refreshed approval request for binary install, daemon restart, and read-only live validation
of the T140/T143 continuation/current-plan approval-gate-context query class

This packet is a request for approval, not approval itself. No binary install, daemon restart,
harness install, hook/settings/adapter write, lifecycle archive/apply, M6 action, quarantine
inspection, migration action, schema/storage/index change, public MCP change, document-index
behavior change, `orient` payload change, ranking-source change, shell profile/PATH/auth/service
change, rollback, force-kill, deletion, old-binary reinstall, or user-owned file change has been
run for T145.

T145 supersedes T141 and T144 as runtime-refresh approval packets. T144 was authored while full
repository `HEAD` was `ab2f5e25b78f1224a7dbc4d5615c143f286a750b`, then the docs-only T144 packet
commit moved `HEAD` to `7baf1365ff72ad3007082be0763a28d5918b0b3f`. T144's full-HEAD stop
condition is therefore self-stale even though no binary-relevant source paths changed. T141 or T144
approval does not authorize the current runtime refresh. Any runtime refresh now requires exact
T145 approval.

## Research Question

Can Engram safely request exact approval to refresh the installed MCP runtime while preserving
approval-gate discipline, by anchoring execution to binary-relevant source invariance from source
baseline `ab2f5e25b78f1224a7dbc4d5615c143f286a750b` instead of full repository `HEAD`, so
docs-only approval/report commits do not self-invalidate the packet?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T144's failed execution precondition is a packet-design issue, not a source/runtime issue: the committed drift from `ab2f5e25` to current `HEAD` is docs-only, binary-relevant paths are unchanged, and exact T145 approval can safely authorize the same install/restart/read-only validation sequence if fresh binary-relevant drift checks pass first. |
| Null | T145 cannot define a clear binary-relevant invariant; full-HEAD pinning is still required, meaning any committed approval packet makes runtime refresh approval self-stale. |
| Simpler alternative | Keep T144 as-is and ask the user to approve it anyway. This weakens the gate because T144 explicitly says to stop on HEAD drift. |
| Failure | The packet permits ambiguous drift, treats docs-only evidence as authorization to execute without approval, or hides runtime, lifecycle, harness, M6, `orient`, ranking, public MCP, schema/storage/index, or document-index changes inside the refresh. |

## Current Evidence

- Binary-relevant source baseline remains
  `ab2f5e25b78f1224a7dbc4d5615c143f286a750b`
  (`Harden current-plan search against fresh handoffs`).
- Current repository `HEAD` after committing T144 is
  `7baf1365ff72ad3007082be0763a28d5918b0b3f`
  (`Record T144 runtime refresh approval packet`).
- Read-only `git diff --name-status ab2f5e25b78f1224a7dbc4d5615c143f286a750b..HEAD`
  showed only:
  - `M docs/BRAIN_HARNESS_ARCHITECTURE.md`
  - `M docs/BRAIN_HARNESS_RESEARCH_METHOD.md`
  - `A docs/BRAIN_HARNESS_T144_T143_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md`
  - `M docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- Read-only binary-relevant diff check
  `git diff --name-status ab2f5e25b78f1224a7dbc4d5615c143f286a750b..HEAD -- Cargo.toml Cargo.lock 'engram-*'`
  returned empty output.
- The active `engram` path resolves to `/Users/yuval.meiri/.local/bin/engram`.
- The active installed binary reports `engram 0.1.0` and still has hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- `/Users/yuval.meiri/.cargo/bin/engram` remains at hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- `engram daemon status` still reports the global daemon running on port `8765`, PID `23341`.
- Lean startup `orient` trace `019e8891-56e4-70e1-a35f-c3b4c6a32fbd` returned T144 current-plan
  memory `019e888f-e1e2-7833-b3f7-b01a62563901` first.
- Direct live current-plan/continue search trace `019e8891-8733-7771-95a8-8332a5549b42` remained
  handoff-heavy, returning T144, T143, T142, T140, T138, T133A, T134, and T133 rolling handoffs in
  the memory top results.
- Direct implementation-matrix search trace `019e8891-c4c5-7c81-a9a9-166eefea135a` returned T144
  current-plan memory first, with handoff noise behind it.
- Direct risk search trace `019e8892-00de-7bb0-903b-1184b2abdbcd` returned T144 current-plan
  memory first, but still included active T144/T143/T142/T141/T140/T139/T135 handoffs in the top
  memory results.
- Source reads confirm the known handoff-noise mechanism still applies:
  `engram-index/src/handoff.rs` records a supersedes edge to the previous handoff but saves only the
  new handoff, while `engram-index/src/memory.rs` marks older current-plan items
  `MemoryStatus::Superseded`, and `engram-index/src/memory_ranker.rs` gives active items full
  status score.

## AI Review

- AI Council recall found prior guidance to keep current-plan retrieval work prompt-class local and
  separate from lifecycle cleanup, broad ranking, or `orient` expansion.
- Fresh AI Council broadcast first recommended a docs-only post-T144 continuity audit instead of a
  source fixture, ranking repair, or idle wait.
- After the full-HEAD self-staleness was identified, a second AI Council broadcast unanimously
  recommended a refreshed T145 approval packet that supersedes T144 and anchors execution to
  binary-relevant source invariance, not full repository `HEAD`.
- Claude Bridge read-only critique agreed with T145 and requested explicit source baseline SHA,
  concrete diff commands, binary-relevant deny-list, docs-only allow-list, staged/unstaged checks,
  pre-state hash/PID checks, supersession chain, and preservation of T144's three validation
  queries and partial-failure stop condition.

## Binary-Relevant Invariant

T145 allows committed docs-only approval/report drift from source baseline `ab2f5e25` only if every
fresh execution-start check proves no binary-relevant committed, staged, or unstaged drift.

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
git diff --name-status ab2f5e25b78f1224a7dbc4d5615c143f286a750b..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --cached --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git status --short
command -v engram
engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
engram daemon status
```

The first three diff commands must return empty output. `git status --short` must show only the
known untracked user-owned `AGENTS.md`, or else only paths that are explicitly docs-only and
unmodified before installation. If there is any ambiguity, stop.

At execution start, `/Users/yuval.meiri/.local/bin/engram` must still have hash
`837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`, and `engram daemon status`
must still show PID `23341`. If either already changed, stop before install because the runtime was
mutated outside this approval packet.

## Proposed Approval

If the required first checks pass after exact approval, the authorized operational sequence is
exactly:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
engram daemon stop
engram daemon start
```

After `cargo install` completes and before daemon restart, verify that the
`/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
`837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`. If it does not differ, stop
before daemon restart.

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

Any missing, partial, broad, conditional, stale-T141, stale-T144, or ambiguous approval remains
default-deny.

## Pass Criteria

T145 succeeds only if all of the following are true:

- The required first checks run before `cargo install` and prove no binary-relevant committed,
  staged, or unstaged drift from source baseline `ab2f5e25`.
- `git status --short` has no unexpected local changes and does not stage or modify root
  `AGENTS.md`.
- Pre-install binary hash and daemon PID match the packet's pre-state.
- `cargo install` succeeds with the exact command above and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- `engram daemon stop` and `engram daemon start` complete cleanly.
- `engram daemon status` reports a healthy daemon after restart, with a PID distinct from pre-state
  PID `23341`.
- Lean `orient` still returns compact current-plan guidance and does not require an `orient`
  contract change.
- The T140 live-search query ranks active current-plan guidance above active rolling handoff noise.
- The T143 live-search query ranks active current-plan guidance above active rolling handoff noise.
- The T142/T143 fixture-shaped live-search query ranks active current-plan guidance above active
  rolling handoff noise.
- Approval-gate context remains retrievable in the result set when relevant.
- If one query class passes while another fails, T145 stops and records partial validation as a
  failure, not a pass.
- `obligations(action="doctor")` is clean or any T145-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T145 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T145 status | Evidence |
| --- | --- | --- |
| T140/T143 source behavior | Implemented and source/test validated | T140, T142, and T143 commits plus focused and broad source validation. |
| T144 approval packet | Superseded | T144's full-HEAD stop condition became self-stale after docs-only commit `7baf136`. |
| Binary-source invariance | Documented, pending approval | Current diff from `ab2f5e25` to `HEAD` is docs-only; no `Cargo.toml`, `Cargo.lock`, or `engram-*` drift was observed. |
| Installed runtime parity | Missing, gated | Live daemon still runs pre-refresh binary hash `837ef2...` and PID `23341`; live direct search remains handoff-heavy for broad continuation prompts. |
| Runtime-refresh approval | Refreshed, pending | This packet names first checks, exact commands, validation queries, pass criteria, exclusions, and stop conditions. |
| `orient` hot path | Preserved | T145 requests no `orient` code, payload, ranking-contract, or hot-path responsibility change. |
| Lifecycle hygiene | Still gated | No archive/apply or `lint apply_safe` ran; T139 remains a separate exact archive gate. |
| Harness readiness | Still gated | No `harness install`, hook/settings/adapter write, or user-owned adoption ran; T135 remains a separate exact repair gate. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action ran. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| Executing stale T141 or stale T144 wording | No |
| `harness(action="install")`, hook edits, settings edits, adapter/command/skill writes, or installed user hook/settings changes | No |
| `adopt_user_owned=true` or changing user-owned files | No |
| Editing, staging, committing, or adopting root `AGENTS.md`, `/Users/yuval.meiri/AGENTS.engram.md`, Claude settings snippets, or installed harness files | No |
| Memory lifecycle archive/apply/supersede/reject/delete or `lint(action="apply_safe")` | No |
| T139 stale current-plan archive | No |
| M6 migration inventory/export/status/prioritize/apply, candidate decisions, quarantine inspection, deletion, cleanup, or legacy simplification | No |
| Schema, storage, index, document-index behavior, public MCP, graph, lint rule, telemetry formula, ranking source, or `orient` payload/contract changes | No |
| Broad search/ranking QA beyond the T140/T143 prompt class and explicit listed validation queries | No |
| Shell profile, PATH, package manager beyond the exact `cargo install`, auth, launch agent, service definition, or environment changes | No |
| Rollback commands, force-kill commands, deleting daemon files, or reinstalling old binaries | No |

## Stop Conditions

Stop and ask before continuing if:

- approval is missing, references only stale T141 or stale T144, is conditional, is ambiguous, or
  changes the allowed scope;
- any required first check is skipped, reordered after install, or cannot be interpreted;
- any committed, staged, unstaged, or untracked change outside the docs-only allow-list appears;
- root `AGENTS.md` is staged, modified by this work, or requested for inclusion without explicit
  user approval;
- any binary-relevant diff command returns output;
- any changed path is ambiguous or not clearly documentation-only;
- pre-install `/Users/yuval.meiri/.local/bin/engram` hash differs from
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`;
- pre-install daemon PID differs from `23341`, the daemon is unhealthy, or the MCP runtime becomes
  unreachable;
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
Approve T145: execute the binary-source-invariant T140/T143 runtime refresh from
docs/BRAIN_HARNESS_T145_BINARY_SOURCE_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md. This
supersedes stale T141 and stale T144. Before any install command, verify no committed, staged, or
unstaged binary-relevant drift from source baseline ab2f5e25b78f1224a7dbc4d5615c143f286a750b
using the packet's listed diff, status, hash, and daemon checks. If those checks pass, install with
`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`, restart the Engram
daemon with `engram daemon stop` and `engram daemon start`, then run only read-only live validation
of the listed T140/T143 continuation/current-plan approval-gate-context search queries. Do not run
harness install, edit hooks/settings/adapters/user-owned files, stage or edit AGENTS.md, use
adopt_user_owned, mutate memory lifecycle, run lint apply_safe, run T139 archive, run
M6/migration/quarantine, change orient/ranking source/public MCP/schema/storage/index/document-index
behavior, change shell profile/PATH/auth/service configuration, run rollback commands, force-kill
processes, delete daemon files, or reinstall old binaries.
```
