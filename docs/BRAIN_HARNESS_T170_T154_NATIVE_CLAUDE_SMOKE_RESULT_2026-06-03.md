# T170 T154 Native Claude Non-Session Smoke Result

Date: 2026-06-03

## Scope

Approved task:

```text
Approve T154 native Claude non-session smoke.
```

Executed only the two approved native Claude commands:

```text
/Users/yuval.meiri/.local/bin/claude --version
/Users/yuval.meiri/.local/bin/claude --help
```

No native prompt-bearing Claude command, interactive Claude session, `/hooks`, harness install,
settings edit, adapter edit, lifecycle mutation, migration, quarantine review command, ranking,
`orient`, schema/storage/index, public MCP, or document-index behavior change was executed.

## Research Question

Can the local Claude executable run non-session metadata/help commands without starting a Claude
session, firing lifecycle hooks, creating a handoff, creating obligations, or mutating monitored
Claude harness files?

## Hypotheses

- Preferred: `claude --version` and `claude --help` complete as metadata/help commands with no
  observed lifecycle or monitored-file mutation.
- Null: one or both commands starts a session or triggers lifecycle side effects.
- Simpler alternative: keep relying on T153/T156 static validation and defer native execution.
- Failure: command execution mutates settings, hooks, handoff, obligations, Memory OS records, or
  other project state.

## Preflight

Pre-command Memory OS cursor:

| Field | Value |
| --- | --- |
| Commit id | `019e8d88-4927-7030-975b-e86b5cf9c5ae` |
| Timestamp | `2026-06-03T13:07:23.202985Z` |

Git status before native commands:

```text
## yuval.meiri/memory-os-phase0
?? AGENTS.md
```

The untracked root `AGENTS.md` was pre-existing and was left untouched.

T153/T156 monitored hashes matched before native execution:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

Target executable preflight:

```text
lrwxr-xr-x@ 1 yuval.meiri  staff  55 Jun  3 13:30 /Users/yuval.meiri/.local/bin/claude -> /Users/yuval.meiri/.local/share/claude/versions/2.1.161
```

This is a drift from T156, which observed the symlink target at `2.1.160`. The T154 packet's
monitored T153 hash set still matched exactly. This result treats the binary target drift as an
explicitly reported uncertainty, not as evidence of hook/settings mutation.

CLI and MCP harness status/doctor before native execution reported `ready=true` with the known
warnings:

- `claude-settings-snippet` is user-owned and not overwritten by Engram.
- `settings.json` and `settings.local.json` contain extra Engram permission entries outside the
  current harness contract.
- Claude settings are split across `settings.json` and `settings.local.json`; effective hook
  behavior still requires native Claude Code `/hooks` verification.
- Lifecycle compliance remains soft and depends on the agent following policy.

Preflight hook inventory confirmed `UserPromptSubmit`, `PostToolUse`, `PostToolUseFailure`, `Stop`,
`PreCompact`, and `PostCompact` durable MCP hooks in settings, and `SessionEnd` as a command hook.

Obligations doctor before native execution:

```json
{"open":[],"warnings":[]}
```

## Approved Command Table

| Command | Expected purpose | Expected allowed state touch | Expected hook impact | Run? |
| --- | --- | --- | --- | --- |
| `/Users/yuval.meiri/.local/bin/claude --version` | Print local Claude version | Claude binary and supporting runtime files | Expected no session hooks; verify by snapshots | Yes |
| `/Users/yuval.meiri/.local/bin/claude --help` | Print local Claude help/options | Claude binary and supporting runtime files | Expected no session hooks; verify by snapshots | Yes |

## Results

### `/Users/yuval.meiri/.local/bin/claude --version`

Exit status: `0`

Output:

```text
2.1.161 (Claude Code)
```

Immediate post-state:

- monitored hashes unchanged
- git status unchanged except the pre-existing untracked `AGENTS.md`
- Memory OS `changes_since` from the pre-command cursor returned `item_count=0` and
  `commit_count=0` with trace `019e8d98-d138-70f2-b546-fe9b9b261ca7`
- obligations doctor stayed clean
- CLI harness status/doctor still reported `ready=true` with the same known warnings

### `/Users/yuval.meiri/.local/bin/claude --help`

Exit status: `0`

Output summary: printed Claude Code CLI usage, options, and command help. The help text states that
Claude Code "starts an interactive session by default" and exposes `-p/--print` for non-interactive
output, but this `--help` invocation itself exited immediately without a prompt or session.

Immediate post-state:

- monitored hashes unchanged
- git status unchanged except the pre-existing untracked `AGENTS.md`
- Memory OS `changes_since` from the pre-command cursor returned `item_count=0` and
  `commit_count=0` with trace `019e8d99-3617-7e51-be52-eb08aeed961a`
- obligations doctor stayed clean
- CLI harness status/doctor still reported `ready=true` with the same known warnings

Post-command monitored hashes:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

Final git status after native commands and before this report write:

```text
## yuval.meiri/memory-os-phase0
?? AGENTS.md
```

## Conclusion

T154 succeeded for the approved non-session native smoke: both exact commands completed with no
observed monitored-file mutation, no Memory OS item or commit creation since the pre-command cursor,
no obligation creation, and no git worktree mutation beyond the already-untracked root `AGENTS.md`.

This result does not prove effective interactive Claude Code hook behavior. `/hooks` verification,
prompt-bearing native Claude validation, and missing SessionEnd `write_policy` behavioral
verification remain explicitly deferred behind their own approval gates.

## Confounds And Unresolved Uncertainty

- The Claude binary symlink drifted from the T156-observed `2.1.160` target to `2.1.161` before
  T154 execution. The monitored T153 settings/hook hashes still matched exactly.
- The smoke monitored project git state, the three approved Claude settings/hook hashes, harness
  status/doctor, obligations doctor, and Memory OS `changes_since`. It was not a whole-home
  filesystem audit.
- Native `--help` output confirms the default `claude` command can start an interactive session when
  invoked without `--help`, `--version`, or an equivalent metadata/help mode. T154 did not authorize
  testing that path.
