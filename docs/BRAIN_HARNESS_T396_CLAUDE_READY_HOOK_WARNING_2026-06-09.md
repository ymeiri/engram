# Brain Harness T396 Claude Ready Hook Warning

Status: completed harness non-overclaim hardening
Date: 2026-06-09

## Research Question

Can Claude Code harness readiness stay useful for static release checks while explicitly warning
that `ready=true` does not prove live effective-hook visibility?

## Hypotheses

| Hypothesis | Expected result |
| --- | --- |
| Preferred | Keep static `ready=true` semantics for generated files and settings entries, but add a warning that live effective hooks still require Claude Code `/hooks` verification. |
| Null | Existing `ready=true` plus soft lifecycle warnings are enough to prevent native-Claude hook overclaims. |
| Simpler alternative | Document the caveat only in release notes. |
| Failure | The readiness warning changes `ready` semantics or implies `/hooks` was executed. |

## Measurement

- `cargo test -p engram-index claude_ready_status_warns_effective_hooks_need_live_hooks_proof`
- `cargo test -p engram-tests --test harness_tests test_mcp_harness_claude_ready_warns_effective_hooks_need_live_hooks_proof`
- `cargo fmt --all --check`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

## Evidence

`HarnessService::status` already distinguishes static adapter/settings readiness from missing
files, user-owned files, drifted files, missing settings entries, and checked MCP-tool gaps.
However, a statically ready Claude Code report could still be misread as proof that a live Claude
host will show the Engram hooks as effective. T396 preserves `ready=true` for static readiness and
adds a Claude-only warning:

- static readiness confirms generated adapter files and settings entries;
- it does not prove live effective-hook visibility;
- live effective-hook claims still require Claude Code `/hooks` verification.

The focused service and MCP regression tests install the generated Claude Code harness into a
temporary root, read the status report, assert `ready=true`, and assert the `/hooks` caveat is
present. They do not run native Claude, execute `/hooks`, mutate the real project `.claude`
settings, or claim effective-hook visibility.

## Validation

Validation passed:

- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p engram-index claude_ready_status_warns_effective_hooks_need_live_hooks_proof`
- `cargo test -p engram-tests --test harness_tests test_mcp_harness_claude_ready_warns_effective_hooks_need_live_hooks_proof`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

The package/install smoke rebuilt the release package, verified the checksum, inspected archive
paths, installed the packaged binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`, and
verified packaged HTTP `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Decision

Accept the preferred hypothesis. The beta can keep static readiness checks useful while making the
native-Claude/effective-hook proof boundary visible in the same structured report agents and
release checks already consume.

## Boundary

T396 does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native
Claude, run `/hooks`, prove prompt-bearing behavior, prove effective-hook visibility, prove live
host labels, mutate M6, run lifecycle cleanup, or make Engram production/GA ready.
