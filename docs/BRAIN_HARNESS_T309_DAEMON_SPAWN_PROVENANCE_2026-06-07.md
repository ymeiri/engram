# T309 Daemon Spawn Provenance Diagnostic

Status: complete
Date: 2026-06-07
Scope: production-hardening diagnostic; no daemon restart, install, release action, or destructive
cleanup

## Research Question

Can Engram make installed-runtime drift easier to detect without requiring ad hoc `lsof`, hash, or
manual PID inspection?

## Hypotheses

| Type | Hypothesis |
|---|---|
| Preferred | Persisting daemon spawn provenance and printing it in `daemon status` gives maintainers a cheap first-class drift diagnostic. |
| Null | Existing `daemon status` already gives enough evidence because port, PID, and health are sufficient. |
| Simpler alternative | Keep relying on release notes and manual hash checks. |
| Failure | The diagnostic becomes a hard readiness gate or changes daemon lifecycle behavior. |

## Result

The null is rejected for production hardening. Existing `daemon status` reported only port, PID, and
health, while prior runtime-refresh slices repeatedly needed separate binary path/hash/PID evidence.

T309 adds best-effort spawn metadata beside the existing daemon pid/port files when a daemon is
started:

- executable path,
- Engram package version,
- daemon PID,
- daemon port.

`engram daemon status` now prints that metadata when present, prints the current CLI executable
path, and warns when the daemon was spawned by a different executable path, different version, or
metadata that no longer matches pid/port files. Daemons started by older Engram builds continue to
work; status reports metadata as unavailable instead of failing. Metadata write failure is logged
and does not block daemon pid/port persistence or startup.

## Validation

- `cargo fmt --all --check`
- `cargo test -p engram-cli daemon::tests`
- `cargo check -p engram-cli`
- `cargo run -q -p engram-cli -- daemon status`

The live read-only status check against the current global daemon reported metadata unavailable,
which is expected for a daemon that was started before T309 metadata support:

```text
Daemon status: 🟢 running
  Port: 8765
  PID:  65155
  Spawn metadata: unavailable (daemon may have been started by an older Engram binary)
  Current CLI: /Users/yuval.meiri/projects/engram/target/debug/engram
```

## Boundary

T309 does not restart the daemon, install a binary, change MCP schema, alter proxy behavior, run
native Claude, prove effective hooks or host labels, mutate lifecycle state, run `lint apply_safe`,
or mark/merge/publish PR #3.
