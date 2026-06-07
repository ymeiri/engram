# T204 T203 Runtime Refresh Validation

Date: 2026-06-04
Status: installed runtime refreshed and read-only validation complete
Scope: Install current `engram-cli`, restart the Engram daemon, and validate T203 handoff
supersession convergence without writing a handoff

## Decision

The installed runtime now includes commit `b086bd6` (`Converge rolling handoff supersession`).
After installing the current `engram-cli` binary and restarting the daemon, a read-only
`handoff(action="update", dry_run=true)` live MCP check planned to supersede all active
project-scoped rolling handoff predecessors instead of only the newest predecessor.

This validates the installed MCP path for T203. It does not mutate live handoff memory because the
validation call used `dry_run=true` and returned `written=false`.

## Runtime Actions

Commands run:

```text
cargo install --path engram-cli --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram daemon status
shasum -a 256 /Users/yuval.meiri/.local/bin/engram
/Users/yuval.meiri/.local/bin/engram daemon stop
/Users/yuval.meiri/.local/bin/engram daemon start
/Users/yuval.meiri/.local/bin/engram daemon status
```

Results:

- installed binary hash:
  `39ee3b6491dca33267019376be07dd43a51b3772ffc24829cb3cf5f07385cd0c`;
- daemon before refresh: running on port `8765`, PID `6516`;
- daemon stopped cleanly;
- daemon after refresh: running on port `8765`, PID `91929`, command
  `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.

## Read-Only Validation

Before the daemon restart, the same live dry-run handoff request returned only one predecessor:

```text
previous_id = 019e9130-ea5e-7752-befe-1f23face22af
item.supersedes = [019e9130-ea5e-7752-befe-1f23face22af]
written = false
```

After installing and restarting, the same dry-run request returned the same newest
`previous_id` but a long `item.supersedes` vector containing the newest predecessor plus many older
active project-scoped rolling handoffs, beginning with:

```text
019e9130-ea5e-7752-befe-1f23face22af
019e8ea5-663e-7152-b346-9c5ab7ddc93b
019e8e9d-1e08-76d1-ab53-3c7f63ca0baa
019e8e9a-292c-7be2-9ee5-441415aafd27
019e8e97-1346-74c2-8d9e-8d89e2457f89
```

and ending with older active handoff IDs including:

```text
019dce53-7420-7a81-8395-fdafcc020d91
019dce4f-c367-7ed3-8598-cc271129a2e7
019dce4a-d70e-7661-974f-fd2e0f0470bf
019dce37-d1a5-70c2-b020-9e5ba1fa50bb
019dca5b-134e-7e62-9508-c2ef389fa29a
019dca4d-535b-7e11-b2ea-e49d51fc9ad1
019dca43-d6fd-7462-955e-ff6b8d7b9b49
```

The response still reported:

```text
dry_run = true
written = false
```

This proves the installed runtime now exercises the T203 all-predecessor planning semantics through
the live MCP handoff path.

## Completion Matrix Delta

| Area | State After T204 | Remaining Risk |
| --- | --- | --- |
| Installed runtime | Refreshed to binary hash `39ee3b6491dca33267019376be07dd43a51b3772ffc24829cb3cf5f07385cd0c` | None found for binary drift |
| Daemon | Restarted cleanly on port `8765`, PID `91929` | Existing MCP clients may need normal reconnect behavior if they held old connections |
| T203 live behavior | Read-only dry-run now plans all active same-scope predecessors | Actual convergence still requires a future non-dry-run handoff update |
| Live stale handoffs | Not mutated | They remain active until a future handoff write converges them or lifecycle cleanup is explicitly run |
| Search and `orient` | Unchanged | Old active handoffs can still surface until live convergence happens |

## Non-Actions

T204 did not:

- run a non-dry-run handoff update;
- archive, reject, delete, review, or mutate existing live MemoryItems;
- run `lint(action="apply_safe")`;
- change search ranking, `orient`, public MCP request parameters, or payload shape;
- change schema/storage/index/document-index behavior;
- edit hooks, settings, adapters, or user-owned files;
- run native Claude, Claude Bridge write actions, M6/migration/quarantine actions, deletion,
  rollback, force-kill, or old-binary reinstall.
