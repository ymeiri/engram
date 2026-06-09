# T307 Claude Hook Settings Diagnostic

Date: 2026-06-07
Status: completed narrow harness-diagnostics hardening slice

## Question

Can Claude harness status make the effective-hook gap clearer when generated hook files are present
but Claude settings do not register the corresponding hook, without running Claude or mutating user
settings?

## Hypotheses

| Hypothesis | Result |
| --- | --- |
| Preferred | Status can add an actionable warning for installed `SessionStart` and `SessionEnd` hook files whose required settings registrations are missing. | Supported. |
| Null | Existing missing-settings warnings are sufficient to distinguish installed files from effective hook configuration. | Not supported. |
| Failure | The slice changes readiness semantics, settings merge behavior, installed hooks, or public report shape. | Avoided. |

## Evidence

Before T307, `HarnessService::status` independently reported adapter file status and missing Claude
settings entries. A generated hook script could be installed while its required `settings.json` or
`settings.local.json` registration was missing, leaving users to infer that Claude would not run the
file.

T307 keeps the existing `ready` calculation and report schema. It adds a read-only warning when:

- `claude-session-start-hook` is installed but `SessionStart:startup|resume|compact` is missing.
- `claude-session-end-hook` is installed but `SessionEnd` is missing.

The `Stop` path is not included in this file-to-settings warning because current generated settings
register the MCP hook handler for `Stop`, not the generated stop-nudge script.

## Validation

These commands passed after the change:

```bash
cargo fmt --all --check
cargo test -p engram-index status_warns_when_claude_hook_files_are_installed_but_settings_missing
cargo test -p engram-index harness::tests
cargo test -p engram-tests --test harness_tests
```

## Boundary

T307 does not run native Claude, edit Claude settings, install adapters, adopt user-owned files,
change hook behavior, change readiness semantics, change MCP request/response shape, mutate Memory
OS lifecycle state, run M6, mark PR #3 ready, merge, tag, or publish.
