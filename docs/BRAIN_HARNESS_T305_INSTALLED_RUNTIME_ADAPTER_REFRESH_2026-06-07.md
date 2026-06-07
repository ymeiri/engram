# T305 Installed Runtime Adapter Refresh

Date: 2026-06-07
Branch: `yuval.meiri/memory-os-phase1`
Previous repo head: `e15aaf75fbf99e294f00ad362ffbc42e8bcb2696`
PR: <https://github.com/ymeiri/engram/pull/3>

## Question

Can the `0.2.0-beta.1` candidate close the T304 installed-runtime/adapters drift gate for the
local/Codex beta path?

## Hypotheses

- Preferred: installing the exact current `engram-cli` source, refreshing generated Codex adapters,
  and restarting the global daemon makes the installed runtime match source and keeps MCP usable.
- Null: the installed binary or adapter remains stale after refresh.
- Simpler alternative: refreshing only the adapter would make the visible skill file current, but
  would leave the running daemon and installed CLI on stale code.
- Failure: the generated adapter is user-owned, the daemon cannot restart cleanly on port `8765`,
  or Codex MCP cannot call the refreshed daemon after restart.

## Authorization Boundary

T304 did not authorize runtime or adapter writes. This T305 execution uses the standing `/goal`
authorization for Engram-scoped daemon/runtime refresh and harness adapter writes. The executed
scope stayed bounded to:

- `/Users/yuval.meiri/.local/bin/engram`,
- `/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md`,
- the global Engram daemon on port `8765`.

No native Claude process was launched or signaled, no PR readiness/merge/tag action was taken, and
root `AGENTS.md` remained untracked and unstaged.

## Preflight

- `target/debug/engram harness status --harness codex` reported the Codex memory-session skill as
  `drifted` and the Codex harness as `ready=false`.
- `target/debug/engram harness install --harness codex --json` planned exactly one write:
  `/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md`.
- The dry-run skipped the resume-session skill and `AGENTS.engram.md` as already installed and
  returned no warnings.
- Installed binary baseline:

```text
engram 0.1.0
cb814e3f1a3c55b33d47ce15d4058e054cb7864c2303b94e06e98183f6584ea4
```

- The pre-refresh daemon was running on port `8765` as PID `25189`, and `lsof` showed it was still
  mapped to the old installed binary inode.

## Execution

```bash
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram harness install --harness codex --write --json
/Users/yuval.meiri/.local/bin/engram daemon stop
/Users/yuval.meiri/.local/bin/engram daemon start --port 8765
```

The adapter install wrote exactly one generated adapter:

```text
/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md
```

The installed binary is now:

```text
engram 0.2.0-beta.1
99bf7b9f680435ebaa7aa59a4c9c60e7ee477163c798694c13f86e516551eff5
```

The refreshed daemon is PID `65155` on port `8765`, and `lsof` confirms its text segment is the new
installed binary inode `544423940`.

## Validation

- `/Users/yuval.meiri/.local/bin/engram harness status --harness codex` reports `Ready: true`.
- `/Users/yuval.meiri/.local/bin/engram harness doctor --harness codex` reports `Ready: true` with
  only the expected soft lifecycle warning.
- `target/debug/engram harness status --harness codex` also reports `Ready: true`.
- The installed render path now includes:

```text
obligations(action=doctor, project=..., cwd=...)
```

- The installed skill file at
  `/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md` contains the same scoped final
  obligation guidance.
- The installed CLI can run `orient` against an isolated data dir and produced trace
  `019ea053-7511-7e92-b2df-df4ac1a26883`.
- Live MCP through Codex recovered after the daemon restart and returned lean `orient` trace
  `019ea054-61fa-79d2-96e7-8f0780f82b82` with obligation summary available and zero open
  obligations.

## Limitations

- Direct installed-CLI `orient` against the default global data dir failed while the daemon owned
  the RocksDB lock. This is expected for direct embedded-store access with a live daemon; live
  Codex uses the MCP daemon path instead.
- Already-open agent UI sessions may need a fresh session or tool reload to ingest the updated
  Codex skill text. The installed file and live daemon are current, but that does not prove every
  already-open host has reloaded its instruction cache.
- This closes the local/Codex installed-runtime/adapters drift gate only. It does not prove native
  Claude prompt-bearing behavior, effective-hook visibility, live Claude host-label proof, Gemini
  labels, full multi-host parity, broad lifecycle cleanup, or direct legacy deprecation/deletion.

## Next Closure Condition

After committing this report, PR #3 needs a fresh exact-head CI pass before any PR-ready, merge,
tag, or GitHub release action.
