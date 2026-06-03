# T147 Runtime Refresh Validation Result

Date: 2026-06-03
Status: operational refresh completed; validation passed
Scope: approved runtime refresh and read-only live validation for the committed T146 no-prompt
`plan_work` `orient` fix

## Research Question

Can Engram safely install the current `engram-cli` binary, restart the live daemon, and validate
that no-prompt and empty-prompt project-scoped `plan_work` lean `orient` calls return the active
current-plan item first without crossing unrelated gates?

## Hypotheses

- Preferred: if binary-relevant source still has no drift from `d12b2ca`, installing the current
  binary to `/Users/yuval.meiri/.local`, restarting the daemon, and running the listed read-only
  checks will make live no/empty-prompt `plan_work` `orient` return the active current-plan item
  first.
- Null: the runtime refresh does not change live no/empty-prompt `plan_work` `orient`, which would
  mean the wrong binary is serving MCP, the daemon did not restart cleanly, or T146 source fixtures
  missed the live path.
- Simpler alternative: leave the old runtime in place and rely on source tests only. This would not
  answer the installed-runtime parity question.
- Failure: the refresh crosses unrelated gates, including harness settings/hooks/adapters, lifecycle
  state, M6/migration/quarantine, public MCP shape, schema/storage/index, document-index behavior,
  ranking beyond the committed T146 source, user-owned files, PATH/profile/auth configuration,
  rollback, force-kill, deletion, or old-binary reinstall.

## Actions

- Ran the T147 required first checks before install. The committed, unstaged, and staged
  binary-relevant diff checks over `Cargo.toml`, `Cargo.lock`, `engram-core`, `engram-store`,
  `engram-embed`, `engram-index`, `engram-mcp`, `engram-cli`, `engram-tests`, `scripts`, and
  `.cargo` returned empty output.
- Confirmed `git status --short` showed only the known user-owned untracked root `AGENTS.md`.
- Confirmed `command -v engram` resolved to `/Users/yuval.meiri/.local/bin/engram` and
  `engram --version` reported `engram 0.1.0`.
- Confirmed the pre-install hashes:
  - `/Users/yuval.meiri/.local/bin/engram`:
    `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`
  - `/Users/yuval.meiri/.cargo/bin/engram`:
    `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`
- Confirmed the pre-install daemon was running on port 8765 with PID `10768`.
- Ran the approved install command:
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
- Confirmed the post-install `/Users/yuval.meiri/.local/bin/engram` hash changed to
  `0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22`.
- Ran the approved daemon restart commands: `engram daemon stop`, then `engram daemon start`.
- Confirmed the restarted daemon is running on port 8765 with PID `68053`.
- Ran the listed read-only local validation commands and MCP validation commands.

## Evidence

| Check | Evidence | Result |
| --- | --- | --- |
| Direct current-plan search | Search trace `019e8bb8-b9e1-7ff3-921f-f3a5589b5ed2` returned `Current plan after T149 T147 preflight recheck` (`019e8a01-0720-7903-814c-db9eb4eb4d6f`) first. | Passed |
| No-prompt `plan_work` lean `orient` | Trace `019e8bb8-ba85-7230-aede-84266c5721c6` returned `019e8a01-0720-7903-814c-db9eb4eb4d6f` first in `brain_loop.top_items`. | Passed |
| Empty-prompt `plan_work` lean `orient` | Trace `019e8bb8-bb3e-7af2-a765-fcbd5bbc4c50` returned `019e8a01-0720-7903-814c-db9eb4eb4d6f` first in `brain_loop.top_items`. | Passed |
| Explicit implementation-prompt guard | Trace `019e8bb8-bbf7-7e21-9dac-fd1e72d91a41` did not force current-plan promotion; the current-plan item was not first and was not in the top five. | Passed |
| Obligation doctor | `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")` returned no open obligations and no warnings. | Passed |
| Repo hygiene | `git status --short` showed only the pre-existing untracked `AGENTS.md`; `git diff --check` passed. | Passed |

Final read-only runtime snapshot:

```text
command -v engram
/Users/yuval.meiri/.local/bin/engram

engram --version
engram 0.1.0

shasum -a 256 /Users/yuval.meiri/.local/bin/engram /Users/yuval.meiri/.cargo/bin/engram
0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22  /Users/yuval.meiri/.local/bin/engram
ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726  /Users/yuval.meiri/.cargo/bin/engram

engram daemon status
Daemon status: running
  Port: 8765
  PID:  68053
```

## Outcome

T147 passes. The installed `.local/bin/engram` binary changed from the T145/T147 pre-state hash to
`0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22`, the daemon restarted with a
new PID, and the live no-prompt and empty-prompt project-scoped `plan_work` lean `orient` paths now
return the active current-plan item first.

The explicit implementation-prompt guard also passed: `implement request throttling` did not force
current-plan promotion merely because an active current-plan item exists. This guard is not evidence
of broad implementation-prompt ranking quality; it only validates the T147 non-promotion condition.

This result closes the installed-runtime gap for the committed T146 no-prompt `plan_work` `orient`
fix. It does not authorize a harness install, hook/settings/adapter write, lifecycle mutation,
M6/migration/quarantine action, schema/storage/index change, document-index behavior change, public
MCP change, payload change, PATH/profile/auth configuration change, rollback command, force-kill,
deletion, old-binary reinstall, or user-owned file edit.

## Completion Matrix Delta

| Area | Status | Evidence |
| --- | --- | --- |
| Binary-source precondition | Validated | Required binary-relevant committed/staged/unstaged diff checks were empty. |
| Installed runtime parity for T146 | Validated | `.local/bin/engram` hash changed from `3d801be9...` to `0cbbbc82...`; daemon PID changed from `10768` to `68053`. |
| Direct current-plan retrieval | Validated | Search trace `019e8bb8-b9e1-7ff3-921f-f3a5589b5ed2` returned the active current-plan memory first. |
| No-prompt project `plan_work` `orient` | Validated | Trace `019e8bb8-ba85-7230-aede-84266c5721c6` returned the active current-plan memory first in Brain Loop. |
| Empty-prompt project `plan_work` `orient` | Validated | Trace `019e8bb8-bb3e-7af2-a765-fcbd5bbc4c50` returned the active current-plan memory first in Brain Loop. |
| Explicit implementation-prompt guard | Validated | Trace `019e8bb8-bbf7-7e21-9dac-fd1e72d91a41` did not force current-plan promotion. |
| `orient` hot path | Preserved | No public request/response shape or additional hot-path responsibility changed during T147. |
| Harness readiness | Still gated | No harness install or hook/settings/adapter write ran. |
| Lifecycle cleanup | Still gated | No archive/apply, `lint apply_safe`, or memory lifecycle mutation ran. |
| M6 migration completion | Still gated | No M6, migration, quarantine, deletion, cleanup, or legacy simplification action ran. |

## Next Gate

The next product-moving gates remain outside T147: harness readiness validation, M6/quarantine
review-gated migration completion, and lifecycle cleanup for stale/wrong-scope memory. Each remains
separately approval-gated.
