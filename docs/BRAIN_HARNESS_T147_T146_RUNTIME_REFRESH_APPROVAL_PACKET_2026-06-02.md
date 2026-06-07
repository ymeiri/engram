# T147 Approval Packet: T146 No-Prompt PlanWork Runtime Refresh

Date: 2026-06-02
Status: pending user approval
Scope: approval request for installing the current `engram-cli` binary, restarting the Engram
daemon, and running read-only live validation for the committed T146 no-prompt `plan_work`
`orient` fix

This packet is a request for approval, not approval itself. No binary install, daemon restart,
harness install, hook/settings/adapter write, lifecycle archive/apply, M6 action, quarantine
inspection, migration action, schema/storage/index change, public MCP change, document-index
behavior change, `orient` payload change, ranking-source change, shell profile/PATH/auth/service
configuration change, rollback, force-kill, deletion, old-binary reinstall, or user-owned file
change has been run for T147.

## Research Question

Can Engram safely refresh the installed MCP runtime to source commit `d12b2ca`, then validate that
no-prompt `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram",
intent="plan_work", response_shape="lean")` returns the latest active project current-plan memory
first in live Codex MCP output?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | T146 is source-complete but not installed. If binary-relevant source remains exactly at `d12b2ca` and pre-state matches this packet, installing `/Users/yuval.meiri/.local/bin/engram`, restarting the daemon, and running read-only live checks will make no-prompt `plan_work` lean `orient` surface the active current-plan item first. |
| Null | Runtime refresh does not change live no-prompt `plan_work` `orient`, meaning either the wrong binary is serving MCP, daemon restart did not take effect, or source-level fixture coverage missed a live-path issue. |
| Simpler alternative | Leave the old installed runtime in place and rely on source tests only. This preserves service state but leaves the Brain Loop hot path stale in actual Codex/Claude use. |
| Failure | The refresh crosses unrelated gates: public MCP/payload/schema/storage/index/document-index behavior, lifecycle state, harness files, M6/migration/quarantine, user-owned files, PATH/service configuration, rollback, force-kill, deletion, or old-binary reinstall. |

## Current Evidence

- T146 source implementation is committed as
  `d12b2ca17500d0979852fe9a35ff7dc6468aa091`
  (`Fix no-prompt plan_work current-plan orient`).
- T146 result doc:
  `docs/BRAIN_HARNESS_T146_NO_PROMPT_PLAN_WORK_ORIENT_RESULT_2026-06-02.md`.
- T146 source validation passed:
  - `cargo test -p engram-tests test_mcp_orient_no_prompt_plan_work --test memory_tests`
  - `cargo test -p engram-tests test_mcp_orient_ranks_reviewed_decisions_by_prompt --test memory_tests`
  - `cargo test -p engram-index orient_mission_prompt_diagnostic_distinguishes_intent_from_ranking`
  - `cargo test -p engram-index open_ended_plan_work_prompt_detection_stays_narrow`
  - `cargo test -p engram-tests --test memory_tests`
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p engram-cli`
  - `cargo test -p engram-index orient_`
  - `cargo test -p engram-tests --test search_tests current`
- Live runtime is stale for this path. No-prompt lean `orient` trace
  `019e89f4-3b30-7c81-a1ac-df20d5ce69b9`, run after the T146 source commit but before runtime
  refresh, returned generic Brain Loop items instead of active current-plan memory.
- Startup trace `019e89f5-58ed-7da2-a498-e254d0feeae8` reproduced the same stale-runtime shape.
- Direct search still returns the current-plan memory first. Trace
  `019e89f5-898a-7160-b661-31b8b6aa6c5a` returned
  `019e89f4-09c4-7100-b15a-8d138eb4cd50`
  (`Current plan after T146 no-prompt PlanWork orient source fix`) first.
- Current read-only pre-state:
  - `command -v engram` resolves to `/Users/yuval.meiri/.local/bin/engram`.
  - `engram --version` reports `engram 0.1.0`.
  - `/Users/yuval.meiri/.local/bin/engram` hash is
    `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`.
  - `/Users/yuval.meiri/.cargo/bin/engram` hash is
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
  - `engram daemon status` reports the global daemon running on port `8765`, PID `10768`.
  - `git status --short` shows only the known user-owned untracked root `AGENTS.md`.
  - Binary-relevant unstaged, staged, and `d12b2ca..HEAD` diff checks are empty.

## Binary-Relevant Invariant

T147 allows committed docs-only approval/report drift from source baseline `d12b2ca` only if fresh
execution-start checks prove no binary-relevant committed, staged, or unstaged drift.

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

If the user explicitly approves this packet, run these read-only checks before any install command:

```text
git diff --name-status d12b2ca17500d0979852fe9a35ff7dc6468aa091..HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git diff --cached --name-status HEAD -- Cargo.toml Cargo.lock engram-core engram-store engram-embed engram-index engram-mcp engram-cli engram-tests scripts .cargo
git status --short
command -v engram
engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
engram daemon status
```

The first three diff commands must return empty output. `git status --short` must show only the
known user-owned untracked `AGENTS.md`, or else only paths that are explicitly docs-only and
unmodified before installation. If there is any ambiguity, stop.

At execution start, `/Users/yuval.meiri/.local/bin/engram` must still have hash
`3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`, and
`engram daemon status` must still show PID `10768`. If either already changed, stop before install
because the runtime was mutated outside this approval packet.

## Proposed Approved Sequence

If and only if the required first checks pass after exact approval, the authorized operational
sequence is exactly:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
engram daemon stop
engram daemon start
```

After `cargo install` completes and before daemon restart, verify that the
`/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
`3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`. If it does not differ, stop
before daemon restart.

Then run read-only validation only:

```text
command -v engram
engram --version
shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
engram daemon status
```

and MCP validation only:

```text
search(project="engram", cwd="/Users/yuval.meiri/projects/engram", layers=["memory"], query="current plan after T147 T146 runtime refresh no-prompt plan_work orient")
orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", response_shape="lean")
orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", prompt="", response_shape="lean")
orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", prompt="implement request throttling", response_shape="lean")
obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")
git status --short
git diff --check
```

The first search identifies the active current-plan memory expected by the no-prompt checks at
execution time. The no-prompt and empty-prompt `orient` checks must return that active current-plan
memory first in `brain_loop.top_items`. The explicit implementation-prompt guard must not force the
current-plan item above prompt-specific implementation guidance merely because a current-plan item
exists.

If one validation class passes while another fails, T147 stops and records partial validation as a
failure, not a pass.

## Pass Criteria

T147 succeeds only if all of the following are true:

- The required first checks run before `cargo install` and prove no binary-relevant committed,
  staged, or unstaged drift from source baseline `d12b2ca`.
- `git status --short` has no unexpected local changes and does not stage or modify root
  `AGENTS.md`.
- Pre-install binary hash and daemon PID match the packet's pre-state.
- `cargo install` succeeds with the exact command above and installs to
  `/Users/yuval.meiri/.local/bin/engram`.
- The post-install `/Users/yuval.meiri/.local/bin/engram` hash differs from pre-state hash
  `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`.
- `engram daemon stop` and `engram daemon start` complete cleanly.
- `engram daemon status` reports a healthy daemon after restart, with a PID distinct from
  pre-state PID `10768`.
- Direct search identifies the latest active project current-plan memory.
- No-prompt `plan_work` lean `orient` returns that active current-plan memory first in
  `brain_loop.top_items`.
- Empty-prompt `plan_work` lean `orient` returns that active current-plan memory first in
  `brain_loop.top_items`.
- Explicit implementation-prompt `plan_work` does not force current-plan guidance above
  prompt-specific implementation guidance.
- No public MCP parameter, response payload shape, schema/storage/index behavior, document-index
  behavior, lifecycle state, harness file/settings/hook state, M6/migration/quarantine state,
  user-owned file, PATH/profile/auth configuration, rollback, deletion, or old-binary reinstall is
  changed.
- `obligations(action="doctor")` is clean or any T147-local obligation is resolved/skipped with
  evidence.
- `git status --short` shows no unintended repo changes except known user-owned untracked
  `AGENTS.md` and T147 documentation/report changes.
- `git diff --check` passes.

## Completion Matrix Delta

| Area | T147 packet status | Evidence |
| --- | --- | --- |
| T146 source implementation | Complete | Commit `d12b2ca`; T146 result doc and source validation. |
| Installed runtime parity for T146 | Pending approval | Live traces still show old no-prompt `orient` behavior before refresh. |
| Direct search current-plan retrieval | Working | Search trace `019e89f5-898a-7160-b661-31b8b6aa6c5a` returned T146 current-plan memory first. |
| `orient` hot path | Source fixed, live stale | Source fixtures pass; installed daemon has not been refreshed. |
| Harness readiness | Still gated | No harness install or hook/settings/adapter write is authorized. |
| Lifecycle cleanup | Still gated | No archive/apply, `lint apply_safe`, or memory lifecycle mutation is authorized. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action is authorized. |

## Stop Conditions

Stop before installation if:

- any binary-relevant committed, staged, or unstaged drift is present;
- `git status --short` contains anything except known user-owned `AGENTS.md` and clearly docs-only
  approval/report files;
- the pre-install `/Users/yuval.meiri/.local/bin/engram` hash no longer matches
  `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`;
- daemon PID is no longer `10768`;
- the approval phrase is missing, partial, stale, broad, or ambiguous.

Stop after installation if:

- the post-install `.local/bin/engram` hash does not change from pre-state;
- daemon restart fails or reports an ambiguous state;
- live no-prompt or empty-prompt `orient` still fails to return the active current-plan item first;
- explicit implementation-prompt validation regresses;
- any validation requires public MCP, payload, schema/storage/index, document-index, lifecycle,
  harness, M6/migration/quarantine, user-owned-file, PATH/profile/auth, rollback, force-kill,
  deletion, old-binary reinstall, or broad ranking work.

## Exact Approval Phrase

```text
Approve T147: execute the T146 no-prompt PlanWork runtime refresh from
docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md. Install the current
engram-cli binary to /Users/yuval.meiri/.local, restart the Engram daemon, and run read-only live
validation that no-prompt and empty-prompt project-scoped plan_work lean orient return the active
current-plan item first while explicit implementation-prompt plan_work does not force current-plan
promotion. Do not edit installed hooks/settings/adapters, run harness install, use
adopt_user_owned, change public MCP params or payload shape, schema/storage/index,
document-index behavior, lifecycle state, ranking beyond the already-committed T146 source,
M6/migration/quarantine, user-owned files, PATH/profile/auth configuration, rollback, force-kill,
deletion, or old-binary reinstall.
```
