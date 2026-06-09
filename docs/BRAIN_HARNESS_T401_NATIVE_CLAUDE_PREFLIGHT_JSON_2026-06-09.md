# Brain Harness T401 Native Claude Preflight JSON

Date: 2026-06-09
Status: completed structured-output hardening for the native Claude production gate.

## Scope

T401 adds `--json` to `scripts/native-claude-gate-preflight.sh` and corrects the stale T400
tracked evidence head. The native Claude prompt-bearing, effective-hook, and live host-label gates
remain unexecuted.

This slice does not launch Claude, send prompts, run `/hooks`, signal processes, mutate settings,
adapters, hooks, lifecycle state, M6 state, memory state, release state, PR state, tags, or
published artifacts.

## Research Question

Can the native Claude production preflight provide machine-readable evidence without weakening the
existing text report or fail-closed strict mode?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | `--json` can expose gate state, blockers, current hashes, vault state, harness state, and non-action flags in a stable object while keeping default text output unchanged. | Supported. |
| Null | Text output is sufficient for future release and GA checks. | Rejected. Machine-readable evidence reduces hand-copied stale fields, as shown by the stale T400 head line. |
| Simpler alternative | Only fix the stale T400 head. | Rejected because it would leave the same evidence-copy failure mode in place. |
| Failure | JSON mode changes readiness semantics, hides blockers, or suppresses strict-mode failure. | Avoided. JSON strict mode still exits `2` while the gate is blocked. |

## Structured Output Contract

`scripts/native-claude-gate-preflight.sh --json` emits one JSON object with:

- `gate_state`,
- branch/upstream/head state,
- tracked and extra-untracked source state,
- Claude path, target, version, and SHA-256,
- Engram binary and daemon status,
- harness status/doctor summaries,
- snippet-only dry-run summary,
- obligations summary,
- canonical vault summary,
- native Claude process list,
- blocker list,
- explicit `actions_performed` booleans.

Strict mode keeps the same output shape and exits `2` when blocked:

```bash
scripts/native-claude-gate-preflight.sh --json --require-ready
```

## Validation

Validation performed for this slice:

- `bash -n scripts/native-claude-gate-preflight.sh`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes --json`
- JSON field assertion with `jq -e`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes --json --require-ready`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

The JSON assertion verified:

```text
gate_state == blocked
claude.version == 2.1.169 (Claude Code)
claude.sha256 == 86d8b820ad7eed50e50a130706d3dc5ef70696f91194de1b3897a842182afe3a
blockers includes native Claude CLI processes are already running
actions_performed.native_claude_launch == false
```

Strict JSON mode exited `2` and preserved the blocker list.

## Gate Impact

T401 improves production-gate evidence quality. It does not close native Claude prompt-bearing
proof, effective-hook visibility, live host labels, multi-host parity, hosted CI, M6, lifecycle
cleanup, release, or production/GA readiness.
