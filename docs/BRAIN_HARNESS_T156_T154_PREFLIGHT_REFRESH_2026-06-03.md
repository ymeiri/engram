# T156 T154 Preflight Refresh

Date: 2026-06-03

## Status

Read-only/static preflight refresh complete. No native Claude command was executed. This slice did
not run `claude --version`, `claude --help`, Claude Bridge, Claude `/hooks`, prompt-bearing Claude,
interactive Claude, lifecycle hooks, harness install, settings edit, M6/migration/quarantine
action, lifecycle cleanup, ranking, `orient`, schema/storage/index, public MCP, document-index,
deletion, rollback, force-kill, or user-owned adoption.

T154 remains default-deny. This report refreshes the static evidence needed before T154 execution;
it is not T154 approval and does not prove native Claude behavior.

## Research Question

Can the T154 non-session Claude smoke preflight be refreshed from static/read-only evidence without
starting a Claude process or crossing any remaining approval gate?

## Hypotheses

Preferred hypothesis: monitored Claude settings and hook files still match T153, Claude Code
harness status remains `ready=true` with the same effective-settings caveats, and the Claude binary
target can be identified without executing it.

Null hypothesis: static preflight evidence has drifted from T153, so T154 should pause until the
approval packet is refreshed or the drift is explained.

Simpler alternative: rely on T153 only and ask for T154 approval without refreshing pre-state.

Failure hypothesis: the preflight accidentally starts Claude, mutates local state, edits hooks or
settings, or treats static evidence as native behavioral proof.

## Measurement

- Re-ran Engram `orient` and direct search for the current plan, gates, user design philosophy, and
  recent risks.
- Read the current architecture, implementation-plan, research-method, `ORIENT_CONTRACT`, T152,
  T153, T154, and T155 docs before choosing the slice.
- Checked `git status --short` and the latest commit.
- Recomputed SHA-256 hashes for the monitored Claude settings and SessionEnd hook files.
- Ran read-only `harness(action="status", harness="claude_code")` and
  `harness(action="doctor", harness="claude_code")`.
- Parsed `/Users/yuval.meiri/.claude/settings.json` and
  `/Users/yuval.meiri/.claude/settings.local.json` as JSON.
- Read static hook/settings declarations and `write_policy` values with `rg`.
- Inspected `/Users/yuval.meiri/.local/bin/claude` with `readlink`, `stat`, `file`, and target-file
  hashing only. No Claude command was invoked.

## Evidence Snapshot

Repository:

- `git status --short` shows only unrelated untracked root `AGENTS.md`.
- Latest commit before this report was `b396df1 Record T155 completion gate audit`.

Monitored Claude file hashes:

| File | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

These match the T153 static preflight hashes.

Claude executable target:

| Path | Evidence |
| --- | --- |
| `/Users/yuval.meiri/.local/bin/claude` | Symlink to `/Users/yuval.meiri/.local/share/claude/versions/2.1.160`; `file` reports Mach-O 64-bit executable arm64. |
| `/Users/yuval.meiri/.local/share/claude/versions/2.1.160` | `-rwxr-xr-x`, owner `yuval.meiri:staff`, size `217429920`, SHA-256 `6c9069a9ee0e7b9b6ee43d006c3402e66815e19f87ac4313330cf03f83611968`; `file` reports Mach-O 64-bit executable arm64. |

Claude settings and hook inventory:

- Both Claude settings files parse as JSON.
- `settings.local.json` declares `SessionStart`, `UserPromptSubmit`, `PostToolUse`,
  `PostToolUseFailure`, `Stop`, `PreCompact`, `PostCompact`, and `SessionEnd`.
- `settings.json` also declares those hook classes.
- Both settings files still contain explicit `"write_policy": "durable"` values for
  `UserPromptSubmit`, `PostToolUse`, `PostToolUseFailure`, `Stop`, `PreCompact`, and
  `PostCompact`.
- The installed SessionEnd command hook still defaults omitted hook input to non-durable nudge:

```text
WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')
```

Claude Code harness status:

| Check | Result |
| --- | --- |
| `harness(status, claude_code)` | `ready=true` |
| `harness(doctor, claude_code)` | `ready=true` |

Doctor/status warnings remain:

- user-owned Claude settings snippet is preserved and will not be overwritten;
- `settings.json` and `settings.local.json` still contain extra legacy Engram permission entries;
- Engram Claude settings are split across `settings.json` and `settings.local.json`;
- effective hook configuration still needs Claude Code `/hooks` validation, which is not approved
  under T154 or this preflight refresh;
- lifecycle compliance remains soft and depends on the agent following policy.

## Decision

T156 keeps T154 executable only after exact approval, but the static preflight state is fresh:

- monitored hashes did not drift from T153;
- the Claude binary target is identified as `2.1.160` without execution;
- Claude Code harness status/doctor remain `ready=true`;
- the installed SessionEnd command-hook default remains `nudge`;
- explicit durable policies remain present in other lifecycle hook inputs;
- effective Claude hook behavior and native side effects remain unproven.

The next product-moving step is still the exact T154 approval phrase:

```text
Approve T154 native Claude non-session smoke.
```

Without that exact approval, do not run `/Users/yuval.meiri/.local/bin/claude --version`,
`/Users/yuval.meiri/.local/bin/claude --help`, any Claude Bridge command, Claude `/hooks`,
prompt-bearing Claude, interactive Claude, lifecycle writes, M6/migration/quarantine actions,
harness install, settings/hooks/adapters edits, ranking/`orient` changes, schema/storage/index
changes, public MCP changes, document-index behavior changes, deletion, rollback, force-kill, or
user-owned adoption.
