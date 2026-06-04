# T207 T206 Runtime Refresh Validation

Date: 2026-06-04
Status: installed runtime refreshed and live validation complete
Scope: Install current `engram-cli`, restart the Engram daemon, and validate T206 document source
metadata search live

## Decision

The installed runtime now includes commit `de18584` (`Repair document source metadata search`).
After installing the current `engram-cli` binary and restarting the daemon, live MCP document search
returns the indexed T202 report first for both exact title and filename-stem known-item queries.

This closes the immediate installed-runtime caveat recorded in T206.

## Runtime Evidence

Install command:

```text
cargo install --path engram-cli --root /Users/yuval.meiri/.local
```

Install result:

```text
Finished release profile in 5m 44s
Replaced /Users/yuval.meiri/.local/bin/engram
```

Installed binary hash:

```text
1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058  /Users/yuval.meiri/.local/bin/engram
```

Daemon restart:

```text
Before: port 8765, PID 91929
daemon stop: Daemon stopped
daemon start: Daemon running on port 8765
After: port 8765, PID 21398
```

## Before/After Live Search Evidence

Before refresh, live MCP `docs(action="search")` showed the T205 caveat still present:

- exact title query `T202 Handoff Supersession MCP Boundary Validation` did not return T202 in the
  top five;
- filename-stem query
  `BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04` did not return T202
  in the top ten;
- content query `test_mcp_handoff_update_supersedes_previous_handoff` returned T202 first with
  score `0.6488516`.

After refresh, live MCP validation returned:

- exact title query `T202 Handoff Supersession MCP Boundary Validation` returned T202 first with
  score `1.0`;
- filename-stem query
  `BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04` returned T202 first
  with score `1.0`;
- unified `search(layers=["document"])` for exact title returned T202 first with score `1.0`;
- generic query `Validation` did not promote T202 into the top five;
- content query `test_mcp_handoff_update_supersedes_previous_handoff` still returned T202 first
  with semantic score `0.6488516`.

## Completion Matrix Delta

| Area | State After T207 | Remaining Risk |
| --- | --- | --- |
| T202 exact title search | Live daemon returns T202 first | None found for tested query |
| T202 filename-stem search | Live daemon returns T202 first | None found for tested query |
| T202 content search | Still returns T202 first at prior semantic score | None found |
| Unified document search | Live document-layer query returns T202 first | Only validated for the tested exact-title query |
| Generic metadata noise | `Validation` did not promote T202 top five | Broader lexical-noise eval remains future work |
| M6/migration | Unchanged | Candidate decisions, dry-run/apply evidence, rollback plan, and explicit migration completion/defer decision remain incomplete |

## Non-Actions

T207 did not:

- change source code after T206;
- change public MCP request or response shape;
- change `orient`, memory ranking, memory lifecycle state, schema/storage definitions, document
  indexing/chunking/embedding behavior, or document cleanup;
- run `lint apply_safe`, migration status/prioritize/apply, deletion, rollback, or M6 work;
- edit hooks, settings, adapters, or user-owned files;
- run native Claude, Claude Bridge write actions, harness install, force-kill, or old-binary
  reinstall.
