# T145 Runtime Refresh Validation Result

Date: 2026-06-02
Status: operational refresh completed; validation is partial failure

## Research Question

Can the approved T145 binary-source-invariant runtime refresh install the current Engram MCP
runtime, restart the daemon, and validate the T140/T143 continuation/current-plan
approval-gate-context query class without crossing unrelated gates?

## Hypotheses

- Preferred: if the T145 first checks still match the packet, installing `engram-cli`, restarting
  the daemon, and running the listed read-only MCP checks will validate installed runtime parity and
  rank active current-plan guidance above rolling handoff noise for the T140/T143 query class.
- Null: the live runtime remains stale or the listed query class still ranks rolling handoffs above
  active current-plan guidance after install/restart.
- Simpler alternative: leave the stale runtime in place and rely on source tests only. This would
  not answer the installed-runtime parity question.
- Failure: any listed T145 validation class fails, or the refresh crosses harness, lifecycle, M6,
  `orient`, ranking-source, public MCP, schema/storage/index, document-index, user-owned file, PATH,
  rollback, force-kill, deletion, or old-binary reinstall gates.

## Actions

- Ran the T145 required first checks before install. The committed, unstaged, and staged
  binary-relevant diff checks over `Cargo.toml`, `Cargo.lock`, `engram-core`, `engram-store`,
  `engram-embed`, `engram-index`, `engram-mcp`, `engram-cli`, `engram-tests`, `scripts`, and
  `.cargo` returned empty output.
- Confirmed `git status --short` showed only the known untracked root `AGENTS.md`.
- Confirmed `command -v engram` resolved to `/Users/yuval.meiri/.local/bin/engram`.
- Confirmed the pre-install hashes:
  - `/Users/yuval.meiri/.local/bin/engram`:
    `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`
  - `/Users/yuval.meiri/.cargo/bin/engram`:
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`
- Confirmed the pre-install daemon was running on port 8765 with PID `23341`.
- Ran the approved install command:
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
- Confirmed the post-install `/Users/yuval.meiri/.local/bin/engram` hash changed to
  `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`.
- Ran the approved daemon restart commands: `engram daemon stop`, then `engram daemon start`.
- Confirmed the restarted daemon is running on port 8765 with PID `10768`.
- Ran the listed read-only local validation commands and MCP validation commands.

## Evidence

- `command -v engram` after restart still resolves to `/Users/yuval.meiri/.local/bin/engram`.
- `engram --version` reports `engram 0.1.0`.
- Post-refresh hashes:
  - `/Users/yuval.meiri/.local/bin/engram`:
    `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`
  - `/Users/yuval.meiri/.cargo/bin/engram`:
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`
- `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work",
  response_shape="lean")` trace `019e89b6-6fa0-71f2-977a-f9046eaabbdf` returned compact output
  and no open obligations, but its top guidance was generic plan-work guidance:
  `Lean orient contract is a presentation option, not a hot-path expansion`,
  `Commit every meaningful Engram step`, `capture_current_plan auto-supersedes and requires title`,
  `BAF008 scored as sealed recovery and accepted`, and
  `Telemetry feedback expectations are structured weak-signal evidence`.
- The no-prompt lean `orient` call did not return the active T145 current-plan memory
  `019e889b-5453-7dc2-9e34-a72538ac65a4`.
- T140 live-search query trace `019e89b6-6ff0-72a1-bc53-96aa4d1b5819` returned
  `Current plan after T145 binary-source runtime packet`
  (`019e889b-5453-7dc2-9e34-a72538ac65a4`) first, above active rolling handoff noise.
- T143 live-search query trace `019e89b6-7037-7271-933d-71f1ba12cfb3` returned
  `Current plan after T145 binary-source runtime packet`
  (`019e889b-5453-7dc2-9e34-a72538ac65a4`) first, above active rolling handoff noise.
- T142/T143 fixture-shaped live-search query trace `019e89b6-7081-7dd0-9b5f-988c1e838c4f`
  returned `Current plan after T145 binary-source runtime packet`
  (`019e889b-5453-7dc2-9e34-a72538ac65a4`) first, above active rolling handoff noise.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  returned no open obligations and no warnings.
- `git status --short` still shows only the pre-existing untracked root `AGENTS.md`.
- `git diff --check` passed.

## Outcome

T145 validates installed runtime parity and the direct T140/T143 live-search query class: the
installed binary changed, the daemon restarted with a new PID, and all three listed direct search
queries rank active current-plan guidance above active rolling handoff noise.

T145 does not pass overall. The exact no-prompt lean `orient` validation returned compact output
but did not return current-plan guidance. Per the T145 packet's stop condition, this is recorded as
partial validation failure, not a pass.

This result does not authorize an `orient` hot-path change, ranking-source change, public MCP
change, lifecycle mutation, harness install, hook/settings/adapter write, M6/migration/quarantine
action, schema/storage/index change, document-index behavior change, user-owned file edit, PATH or
service configuration change, rollback command, force-kill, deletion, or old-binary reinstall.

## Completion Matrix Delta

| Area | Status | Evidence |
| --- | --- | --- |
| Binary-source precondition | Validated | Required binary-relevant committed/staged/unstaged diff checks were empty. |
| Installed runtime parity | Validated | `.local/bin/engram` hash changed from `837ef2...` to `3d801be9...`; daemon PID changed from `23341` to `10768`. |
| T140/T143 direct search | Validated | All three listed live search traces returned T145 current-plan memory first. |
| Lean `orient` no-prompt current-plan guidance | Missing | Trace `019e89b6-6fa0-71f2-977a-f9046eaabbdf` returned generic plan-work guidance, not the T145 current plan. |
| `orient` hot path | Preserved but risky | No `orient` behavior changed; exact no-prompt validation exposes a gap needing separate approval before any hot-path change. |
| Harness readiness | Still gated | No harness install or hook/settings/adapter write ran. |
| Lifecycle cleanup | Still gated | No archive/apply, `lint apply_safe`, or memory lifecycle mutation ran. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action ran. |

## Next Gate

The next narrow gate is a read-only root-cause and fixture proposal for the no-prompt lean
`orient` current-plan miss. Any implementation that changes `orient`, ranking-source behavior,
public MCP contracts, schema/storage/index behavior, or lifecycle state requires separate exact
approval.
