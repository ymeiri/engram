# T134 Approval Packet: T133A Live Runtime Refresh

Date: 2026-06-02
Status: pending user approval

## Research Question

Can the committed T130 hook-template repair be validated in the live Engram MCP runtime without
crossing the separate hook/settings repair gate?

## Hypotheses

- Preferred: installing the current `engram-cli` binary and restarting the Engram daemon is enough
  for live `harness(render_adapter)` to render `.write_policy // "nudge"`.
- Null: the live render still shows `.write_policy // "durable"` after binary refresh, meaning the
  running MCP path is not using the current source or another generated-template path exists.
- Simpler alternative: no hook/settings repair should be attempted until binary-refresh validation
  proves whether the live runtime itself is current.
- Failure: editing installed hooks/settings, running `harness install`, or changing daemon hook
  semantics during this slice would blur the source-vs-live question and cross a separate gate.

## Current Evidence

- Source is correct: `engram-index/src/harness.rs` defaults missing hook input `write_policy` to
  `nudge`.
- Live runtime is stale: live `harness(render_adapter)` still emits `.write_policy // "durable"`.
- Installed generated Claude hook is stale: `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`
  still emits `.write_policy // "durable"`.
- Installed binary hash for both `/Users/yuval.meiri/.local/bin/engram` and
  `/Users/yuval.meiri/.cargo/bin/engram` is
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- Claude Code doctor remains `ready=false` because `SessionStart` and `SessionEnd` settings
  registrations are missing.
- Codex, Gemini CLI, and Cursor doctors remain `ready=false` because generated adapters drifted.
- AI Council recall on harness readiness found an existing high-importance decision: fix/validate
  Claude readiness carefully before more cross-harness dogfood, but do not expand `orient`, run
  migration apply, deletion, or legacy simplification.
- Claude Bridge was not consulted for this packet because the installed Claude `SessionEnd` hook is
  currently part of the known accidental durable handoff-write path.

## Proposed T133A Scope

Approved actions:

1. Install the current repo binary with `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
2. Restart only the Engram daemon needed for the current MCP runtime.
3. Re-run read-only validation:
   - `harness(action="render_adapter", harness="claude_code", adapter="claude-session-end-hook")`
   - `harness(action="doctor", harness="claude_code", root="/Users/yuval.meiri", write=false)`
   - source and installed-hook read-only `rg` checks
   - focused source tests if the build or live smoke gives contradictory evidence
   - `git status --short`, `git diff --check`, obligations doctor
4. Record results in docs and Engram memory, then commit.

Explicitly excluded:

- Editing `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`.
- Editing Claude settings or settings snippets.
- Running `harness install`.
- Using `adopt_user_owned`.
- Changing public MCP request parameters, schema/storage/index behavior, ranking, `orient`,
  migration, lifecycle state, document-index behavior, or installed user-owned files.
- Inspecting T125 quarantine candidates or running M6 status/prioritize/apply/rerun.

## Validation Criteria

T133A passes only if:

- live `harness(render_adapter)` renders `.write_policy // "nudge"`,
- explicit source checks still show the source template default is `nudge`,
- no user hook/settings files were edited,
- daemon restart did not leave duplicate or dead current-daemon state,
- obligations doctor returns no open obligations after docs/memory cleanup,
- the result is captured in a focused commit.

T133A does not prove:

- installed Claude hook parity,
- Claude Code readiness,
- Codex/Gemini/Cursor adapter readiness,
- M6 migration readiness,
- lifecycle cleanup safety,
- broad ranking quality,
- `orient` hot-path completeness.

## Rollback / Stop Conditions

Stop and report before further action if:

- `cargo install` fails,
- daemon restart fails or the MCP runtime becomes unreachable,
- live render still returns `durable` after refresh,
- validation requires editing installed hooks/settings,
- validation suggests a schema/storage/index, public MCP, ranking, `orient`, migration, or lifecycle
  change.

Rollback boundary:

- If the refreshed daemon fails to serve, restore service by restarting the previously installed
  `engram` binary if available from the existing installation path or stop and ask before taking
  destructive recovery steps.
- Do not delete daemon data, RocksDB files, user hooks, settings, or generated adapters.

## Exact Approval Wording

Approve T133A: install the current `engram-cli` binary, restart the Engram daemon, and run read-only
live validation of the Claude `SessionEnd` render default; do not edit installed hooks/settings, run
`harness install`, use `adopt_user_owned`, change public MCP/schema/storage/index/ranking/`orient`/
migration/lifecycle/document-index behavior, or inspect M6 quarantine candidates.
