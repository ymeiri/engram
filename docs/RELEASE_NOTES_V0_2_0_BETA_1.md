# Engram v0.2.0-beta.1 Release Notes

Release date: 2026-06-07
Status: Pre-release candidate

## Supported Beta Path

This beta is scoped to the local/Codex Brain Loop path:

- local `engram serve` MCP operation,
- lean `orient` with current-plan retrieval, trace/cursor fields, used-memory candidate IDs, and
  obligation summary,
- generated Memory OS vault inspection,
- review-gated M6 inventory/export/status paths,
- scoped obligations doctor and advisory harness lifecycle guidance,
- preserved approval boundaries for destructive or broad writes.

## Deferred From This Beta

The following remain production-hardening or host-parity gates, not blockers for this beta:

- native Claude prompt-bearing proof,
- effective-hook visibility proof,
- live Claude host-label proof,
- full multi-host parity,
- direct legacy deprecation/deletion,
- broad lifecycle cleanup or broad `lint apply_safe`,
- exhaustive telemetry coverage,
- native auth/debugging edge cases.

## Release Gate

Before tagging this beta, the candidate commit must have:

- exact-head CI green for Format, Docs, Check, Clippy, and Test,
- `cargo fmt --all --check` passing locally,
- a focused local/Codex smoke confirming current source-rendered harness guidance,
- canonical generated vault status count-aligned,
- `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)` clean,
- a refreshed installed runtime/adapters check, or an explicit release note that the source-rendered
  adapter is current while installed local adapters still need refresh.

## Current Installation Caveat

Source-rendered Codex harness guidance includes scoped final-response obligation checks:

```text
obligations(action=doctor, project=..., cwd=...)
```

The currently installed local Codex skill may still render the older unscoped form until the
runtime and adapters are refreshed from this beta commit. Do not claim installed-runtime parity
until that refresh and smoke test are completed.
