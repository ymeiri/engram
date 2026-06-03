# T153 Post-T152 Claude Static Preflight

Date: 2026-06-03

## Status

Static preflight complete. No native Claude Code process, Claude Bridge process, lifecycle hook,
harness install, settings edit, M6/migration/quarantine action, lifecycle cleanup, ranking,
`orient`, schema/storage/index, public MCP, document-index, deletion, rollback, force-kill, or
user-owned adoption action was executed.

This slice used static file inspection plus read-only Engram harness status/doctor commands to
separate what is already proven after T152 from what still requires a native Claude validation gate.

## Research Question

After the approved T152/T135 harness repair, what can be validated safely by static inspection
before running native Claude Code or Claude Bridge, given the prior SessionEnd side-effect risk?

## Hypotheses

Preferred hypothesis: static inspection confirms that the installed Claude SessionEnd command hook
defaults missing hook input `write_policy` to `nudge`, while harness status/doctor still identify
the remaining effective-settings warnings that need native validation.

Null hypothesis: static inspection finds drift from T152, such as a missing SessionEnd hook, invalid
`settings.local.json`, non-executable hook, or `ready=false` harness status.

Simpler alternative: stop after T152 and ask for native Claude validation approval without a static
preflight.

Failure hypothesis: treating static inspection as behavioral proof would hide that native Claude
execution can still create lifecycle side effects or hidden local writes.

## Measurement

- Re-ran Engram `orient` and direct search for the current post-T152 plan.
- Read the current architecture, implementation-plan, T152 result, research-method, and
  `ORIENT_CONTRACT` excerpts.
- Rechecked `git status --short` and recent commits.
- Asked AI Council for approval-packet critique after `recall_decision`; did not use Claude Bridge.
- Captured SHA-256 hashes for:
  - `/Users/yuval.meiri/.claude/settings.json`
  - `/Users/yuval.meiri/.claude/settings.local.json`
  - `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`
- Parsed `/Users/yuval.meiri/.claude/settings.local.json` with `python3 -m json.tool`.
- Checked that `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` is executable.
- Used `rg` to statically inspect `write_policy`, `durable`, `SessionStart`, and `SessionEnd`
  occurrences in Claude settings and the installed SessionEnd hook.
- Ran read-only `engram harness status --json` and `engram harness doctor --json` for generic,
  Codex, Gemini CLI, Cursor, and Claude Code.

## Static Evidence

Current repo state:

- `git status --short` shows only unrelated untracked root `AGENTS.md`.
- Latest commit is `5f577e5 Record T152 harness repair validation`.

Static file hashes:

| File | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

Static hook evidence:

```text
WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')
```

`settings.local.json` parses as valid JSON, and the installed SessionEnd hook is executable.

Static settings evidence:

- Both `settings.json` and `settings.local.json` contain `SessionStart` and `SessionEnd`
  declarations.
- Existing Claude settings still contain explicit `"write_policy": "durable"` values for other
  lifecycle hook inputs.
- Therefore T153 can prove only the installed command-hook default for missing SessionEnd
  `write_policy`, not that all effective Claude lifecycle hooks are non-durable.

Harness status evidence:

| Harness | `status` | `doctor` warnings |
| --- | --- | --- |
| `generic` | `ready=true` | Soft lifecycle compliance warning |
| `codex` | `ready=true` | Soft lifecycle compliance warning |
| `gemini_cli` | `ready=true` | Soft lifecycle compliance warning |
| `cursor` | `ready=true` | Soft lifecycle compliance warning |
| `claude_code` | `ready=true` | User-owned snippet preserved, extra legacy Engram permissions in Claude settings, split `settings.json` / `settings.local.json`, verify effective hooks with Claude Code `/hooks`, soft lifecycle compliance warning |

## AI Council Synthesis

AI Council agreed that native Claude Code or Claude Bridge execution is not inherently read-only.
The main blind spot is the observer effect: starting and exiting Claude can trigger SessionEnd or
other lifecycle behavior. The Council recommended separating static inspection from native
execution, defining pre/post state snapshots, treating hidden writes as possible side effects, and
verifying missing `write_policy` by static inspection unless a later exact approval allows
behavioral testing.

Claude Bridge was intentionally not used for this critique because post-repair Claude behavior is
the thing still under validation, and prior read-only Claude Bridge critique caused SessionEnd stub
handoffs.

## Decision

T153 validates the static post-T152 state:

- The generated local harnesses remain installed and report `ready=true`.
- The installed Claude SessionEnd command hook defaults omitted hook input `write_policy` to
  `nudge`.
- Claude Code still needs native effective-hook validation because settings are split and existing
  settings contain explicit durable hook policies outside the SessionEnd command hook.

T153 does not prove native Claude Code or Claude Bridge behavioral parity. It does not prove that
running Claude creates no lifecycle handoffs, hidden config/cache writes, telemetry writes, or
server-side side effects.

## Completion Matrix

| Area | State | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| Local generated adapter readiness | Validated | Fresh CLI status/doctor `ready=true` for all five harnesses | Ongoing soft compliance |
| Claude SessionEnd missing-policy default | Statically validated | Installed hook line defaults to `nudge` | Behavioral validation if approved |
| Claude settings validity | Partially validated | `settings.local.json` parses; settings hashes captured | Effective native `/hooks` behavior |
| Explicit durable policies | Identified | `rg` found explicit durable values in Claude settings | Decide whether they are expected or need a later exact cleanup |
| Claude Bridge side-effect risk | Still open | T150 side-effect evidence plus no post-repair native run | T154 approval before any native run |
| M6/migration/quarantine | Still review-gated | No action in T153 | Separate explicit approval |
| Lifecycle cleanup | Still gated | No archive/supersede/apply-safe run | Separate exact lifecycle approval |

## Next Action

Use `docs/BRAIN_HARNESS_T154_NATIVE_CLAUDE_VALIDATION_APPROVAL_PACKET_2026-06-03.md` as the next
approval gate before any native Claude execution. T154 is intentionally limited to a non-session
`claude --version` / `claude --help` smoke and does not authorize Claude Bridge, prompt-bearing
Claude execution, interactive Claude sessions, or Claude `/hooks`.
