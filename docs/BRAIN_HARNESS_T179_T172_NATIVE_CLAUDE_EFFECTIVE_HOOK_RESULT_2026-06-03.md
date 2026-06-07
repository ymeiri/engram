# T179 T172 Native Claude Effective-Hook Result

Date: 2026-06-03
Status: hard-stop result; effective-hook gate remains open

## Scope

The user approved the exact T172 packet:

```text
Approve T172: execute the native Claude effective-hook validation from docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md. I understand this may trigger native Claude lifecycle hook side effects caused by the single approved PTY session; observe and report those side effects only, do not clean them up or continue beyond the packet.
```

After the PTY hung on EOF, the user approved this recovery:

```text
Approve T172 recovery option 2: send Ctrl-C to the live native Claude PTY, then send EOF if it returns to the prompt, and continue with T172 postflight comparisons and the result report.
```

This run did not execute Claude Bridge, prompt-bearing Claude, another slash command, harness
install, settings or hook edits, lifecycle cleanup, M6/migration/quarantine actions, ranking or
`orient` changes, public MCP changes, schema/storage/index changes, document-index behavior
changes, deletion, rollback, force-kill, old-binary reinstall, or user-owned-file adoption.

## Research Framing

Question: can one exact native Claude Code PTY session surface the effective merged hook
configuration, while bounding and measuring any lifecycle side effects caused by startup, `/hooks`,
and exit?

| Type | Result |
| --- | --- |
| Preferred | Not supported. The session showed startup hook output, but `/hooks` did not produce visible effective-hook output and EOF did not exit. |
| Null | Still rejected. Static status and metadata remain insufficient proof of effective native Claude hook behavior. |
| Simpler alternative | Supported as the safe fallback: stop at static/read-only evidence until a new exact recovery or validation packet is approved. |
| Failure | Observed. Startup emitted lifecycle hook output, `/hooks` was inconclusive, and the process remained live after Ctrl-C plus EOF recovery. |

## Preflight Evidence

Preflight matched the T172 packet sufficiently to launch native Claude:

| Check | Result |
| --- | --- |
| Git status | Clean except pre-existing untracked root `AGENTS.md`. |
| Git diff | No tracked diff. |
| Claude binary target | `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`. |
| Claude version | `2.1.161 (Claude Code)`. |
| Harness status | `claude_code`, `ready=true`. |
| Harness doctor | `ready=true`; same warnings about user-owned snippet, legacy permissions, split settings, and soft lifecycle compliance. |
| Obligations doctor | `open=[]`, `warnings=[]`. |
| Memory changes since preflight cursor | None. |
| Native Claude process baseline | No existing `/Users/yuval.meiri/.local/bin/claude` process. |

Monitored user-level hashes matched the T172 packet:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.local/bin/claude` | `5b4dc79eab05f9756c252c71deb339efa4429dffc1967dd8392cf87fcde4867f` |

Project-local Claude hashes were captured for comparison:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/projects/engram/.claude/commands/engram-end-session.md` | `688af0b6ec43764f37635ab234d0dd3bb1c472f28db8c6f0fddc411182d889f0` |
| `/Users/yuval.meiri/projects/engram/.claude/commands/engram-memory-session.md` | `6e12ba4416fe5d5a8b07d193e53db9e3bf2b6a70c5fa89f9a4e9257ed5eaaab4` |
| `/Users/yuval.meiri/projects/engram/.claude/commands/engram-resume-session.md` | `90cdf6b33a24c1d8db0f33202dc5cc43dd0c11edb128271d91ad982f48d2a83d` |
| `/Users/yuval.meiri/projects/engram/.claude/engram-settings-snippet.json` | `70dc8934bf11f5a25b31174c0f29697ae3cd17b91a51ce3557d72e7981b034d2` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-session-end.sh` | `f614a59a4d226f262100aada54836160a0740538e264837b6959177638edd5d7` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-session-start.sh` | `c49c516aa30604cb87841d368e830275aa05355c27a359664876ac742350b27f` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-stop-nudge.sh` | `66ecbae5279f08a8e0d6ff52bd69e2e9b8b7dd4993c5753074196e03111d9f85` |
| `/Users/yuval.meiri/projects/engram/.claude/settings.json` | `93a6d5c289121a4c43cda65f6729d0f5135ae3cc4733d1396fb126cbe2bc68bd` |
| `/Users/yuval.meiri/projects/engram/.claude/settings.local.json` | `e2bb188f7fc346750f6ebaee0632694da603c48402974d4dd8b2b8fd1c4daaf6` |

## Native PTY Transcript Summary

The only native Claude executable launched was:

```text
/Users/yuval.meiri/.local/bin/claude
```

The session started in `/Users/yuval.meiri/projects/engram`.

Observed startup output included Claude Code `v2.1.161`, model `Sonnet 4.6`, two setup issues, and
two visible `SessionStart:startup` blocks containing Engram activation text. This confirms native
startup hooks can emit lifecycle guidance during the approved PTY session.

The only slash-command input sent before recovery was:

```text
/hooks
```

No natural-language prompt was sent. No second slash command was sent. `/hooks` did not produce
visible effective-hook configuration output within the wait window. EOF/Ctrl-D did not exit.

After user-approved recovery, Ctrl-C returned control to the Claude UI and displayed:

```text
Press Ctrl-C again to exit
```

EOF/Ctrl-D then displayed:

```text
Press Ctrl-D again to exit
```

A second EOF/Ctrl-D still left the process live. No further input was sent.

## Postflight Evidence

Postflight comparisons were read-only and run while the native Claude process remained live.

| Check | Result |
| --- | --- |
| Git status | Clean except pre-existing untracked root `AGENTS.md`. |
| Git diff | No tracked diff. |
| Harness status | `claude_code`, `ready=true`; warnings unchanged. |
| Harness doctor | `ready=true`; warnings unchanged. |
| Obligations doctor | `open=[]`, `warnings=[]`. |
| Memory changes since preflight cursor | `item_count=0`, `commit_count=0`, trace `019e8e4a-35a6-7fe1-8efa-d76c021313bc`. |
| Memory cursor after postflight | `019e8e03-6c6f-7331-8309-1c45e14550e9` at `2026-06-03T16:21:28.624273Z`. |
| Native Claude process state | PID `49349`, command `/Users/yuval.meiri/.local/bin/claude`, still live. |

Postflight user-level hashes were unchanged:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.local/bin/claude` | `5b4dc79eab05f9756c252c71deb339efa4429dffc1967dd8392cf87fcde4867f` |

Postflight project-local hashes were unchanged from preflight.

## Side Effects And Attribution

| Side Effect | Attribution | Notes |
| --- | --- | --- |
| Visible `SessionStart:startup` Engram activation output | Caused by the exact native session | Observed twice in the PTY output. |
| No visible `/hooks` effective configuration | Caused or exposed by the exact native session | `/hooks` was accepted as input but did not produce hook configuration before the wait window ended. |
| Live native Claude process PID `49349` after Ctrl-C plus EOF recovery | Caused by the exact native session | Recovery option 2 did not exit the PTY. No force-kill or second Ctrl-C was run. |
| Git, monitored Claude files, Memory OS, obligations, and harness readiness | No observed mutation | Postflight read-only comparisons matched preflight. |

## Decision

T172 executed the approved native session far enough to prove two useful facts:

1. Native Claude startup does run visible Engram lifecycle guidance in this repository.
2. The exact `/hooks` probe did not provide effective-hook configuration evidence in this run and
   left a live process after EOF-based recovery.

T172 did not close the effective-hook visibility gate. It also does not prove prompt-bearing native
Claude behavior, missing SessionEnd `write_policy` behavioral semantics, M6 migration readiness, or
broad lifecycle compliance.

The next step requires explicit user approval for live-process cleanup or a new bounded native
Claude validation/recovery packet. Until PID `49349` is resolved, no further native Claude probing
should run.
