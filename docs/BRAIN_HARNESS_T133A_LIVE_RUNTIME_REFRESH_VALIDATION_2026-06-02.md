# T133A Live Runtime Refresh Validation

Date: 2026-06-02
Status: completed with residual harness-repair gates

## Research Question

Can the committed T130 hook-template repair be validated in the live Engram MCP runtime without
crossing the separate hook/settings repair gate?

## Hypotheses

- Preferred: installing the current `engram-cli` binary and restarting the Engram daemon is enough
  for live `harness(render_adapter)` to render `.write_policy // "nudge"`.
- Null: the live render still shows `.write_policy // "durable"` after binary refresh, meaning the
  running MCP path is not using the current source or another generated-template path exists.
- Simpler alternative: do not attempt installed hook/settings repair until binary-refresh
  validation proves whether the live runtime itself is current.
- Failure: editing installed hooks/settings, running `harness install`, changing daemon write
  semantics, or crossing ranking/orient/migration/schema/storage/index/lifecycle gates would make
  this slice answer the wrong question.

## Actions

- Installed the current repo binary:
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
- Restarted only the Engram daemon:
  `engram daemon stop`, then `engram daemon start`.
- Ran read-only live validation:
  `harness(render_adapter, claude_code, claude-session-end-hook)`, harness doctors for Claude Code,
  Codex, Gemini CLI, and Cursor, source/installed-hook `rg` checks, daemon status, binary hashes,
  lint, obligations doctor, `git status --short`, and `git diff --check`.

## Evidence

- Before install, `/Users/yuval.meiri/.local/bin/engram` and `/Users/yuval.meiri/.cargo/bin/engram`
  had hash `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- After install, `/Users/yuval.meiri/.local/bin/engram` has hash
  `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724`.
- `/Users/yuval.meiri/.cargo/bin/engram` remains at the old hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`; the active `engram`
  path resolves to `/Users/yuval.meiri/.local/bin/engram`.
- The daemon restarted on port 8765 and changed PID from 85557 to 23341.
- Live `harness(render_adapter, claude_code, claude-session-end-hook)` now renders
  `.write_policy // "nudge"`.
- Source check: `engram-index/src/harness.rs:2085` renders
  `WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')`.
- Source tests still assert the rendered hook contains `.write_policy // "nudge"` and not
  `.write_policy // "durable"`.
- Installed-hook check: `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh:25` still renders
  `WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "durable"')`.
- `git diff --check` passed.
- `git status --short` still shows only the pre-existing untracked root `AGENTS.md` before doc
  updates; it was not touched or staged.

## Outcome

T133A validates the live-runtime-refresh hypothesis: the restarted MCP runtime renders the T130
non-durable default for missing Claude `SessionEnd` hook input.

T133A does not validate installed Claude hook parity or full harness readiness. Claude Code doctor
still reports `ready=false` because the installed `claude-session-end-hook` is drifted, the settings
snippet is user-owned, `SessionStart` and `SessionEnd` settings registrations are missing, and extra
legacy Engram permissions remain in settings. Codex, Gemini CLI, and Cursor doctors also still
report `ready=false` because generated adapters have drifted.

The global obligations doctor still reports pre-existing open obligations from other project scopes.
This slice did not mutate or skip those unrelated obligations. T133A-local document and handoff
obligations are handled by this report, Engram memory, and the focused commit.

## Completion Matrix Delta

| Area | Status | Evidence |
| --- | --- | --- |
| T130 source hook default | Implemented and validated | Source and tests render `nudge` for missing input. |
| Live MCP render default | Validated by T133A | Restarted daemon renders `nudge`. |
| Installed Claude hook parity | Missing, gated | Hook still renders `durable`; no hook edit. |
| Claude Code readiness | Partially validated, not ready | Drifted hook and missing settings. |
| Codex/Gemini/Cursor readiness | Partially validated, not ready | Drifted generated adapters. |
| M6 migration completion | High-risk, gated | No M6 candidate/status/prioritize/apply action was taken. |
| Lifecycle cleanup | Risky, gated | Lint still reports stale active memory and superseded-active pressure. |

## Next Gate

The next narrow gate is an exact harness-write repair approval, if the user wants installed harness
state repaired. That approval should name the target harnesses and paths, and should separately
authorize any `harness install`, user-owned adapter adoption, or Claude settings edits. It should not
be bundled with migration, ranking, orient, schema/storage/index, lifecycle, or document-index work.
