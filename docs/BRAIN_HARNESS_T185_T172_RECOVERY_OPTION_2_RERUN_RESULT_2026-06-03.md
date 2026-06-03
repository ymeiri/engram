# T185 T172 Recovery Option 2 Rerun Result

Date: 2026-06-03
Status: hard-stop recovery result; live native Claude process remains open

## Scope

The user re-approved this operation:

```text
Approve T172 recovery option 2: send Ctrl-C to the live native Claude PTY, then send EOF if it returns to the prompt, and continue with T172 postflight comparisons and the result report.
```

This result records only the bounded recovery attempt after the already-committed T179 hard-stop
report. It did not run Claude Bridge, launch a new native Claude session, send a slash command,
send natural-language input, use prompt-bearing Claude, edit hooks/settings/adapters, run harness
install, mutate lifecycle or migration state, change ranking/`orient`, change public
MCP/schema/storage/index/document-index behavior, delete, roll back, force-kill, reinstall a
binary, or touch user-owned files.

## Research Framing

Question: can the live native Claude process left by T172/T179 be resolved by the newly approved
Ctrl-C plus conditional EOF recovery path, without unapproved side effects?

| Type | Result |
| --- | --- |
| Preferred | Not supported. Ctrl-C was sent to the live PTY, but PID `49349` remained live. |
| Null | Rejected for the Brain Harness goal. The live process remains an unresolved validation side effect. |
| Simpler alternative | Supported as the safe fallback: stop after one bounded Ctrl-C attempt because prompt-return evidence was unavailable. |
| Failure | Observed. The process did not exit, and the transcript/prompt state could not be verified after context compaction. |

## Preflight Evidence

Preflight matched the prior T179/T180 state closely enough to attempt the approved Ctrl-C:

| Check | Result |
| --- | --- |
| Git status | Branch `yuval.meiri/memory-os-phase0`; clean except pre-existing untracked root `AGENTS.md`. |
| Git diff | No tracked diff. |
| Latest commit | `e285bd6 Record T184 document index packet`. |
| Native Claude process | PID `49349`, PPID `70787`, state `Ss+`, TTY `ttys000`, command `/Users/yuval.meiri/.local/bin/claude`. |
| Claude version | `2.1.161 (Claude Code)`. |
| Claude symlink target | `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`. |
| Harness status/doctor | `claude_code`, `ready=true`; warnings unchanged from T179/T180. |
| Obligations doctor | `open=[]`, `warnings=[]`. |
| Memory changes since current cursor | `item_count=0`, `commit_count=0`, trace `019e8e61-affc-7611-b17d-fbdd7385e183`. |
| Pre-recovery cursor | `019e8e5e-a5b4-72f0-9079-72128db8e738` at `2026-06-03T16:47:07.268429Z`. |

User-level hashes matched T179/T180:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |
| `/Users/yuval.meiri/.local/bin/claude` | `5b4dc79eab05f9756c252c71deb339efa4429dffc1967dd8392cf87fcde4867f` |

Project-local Claude hashes matched T179/T180:

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

## Recovery Attempt

The only recovery input sent was one Ctrl-C byte to the live PTY device for the existing native
Claude process:

```text
printf '\003' > /dev/ttys000
```

No process-level `kill` signal was used. No slash command, natural-language input, or second
Ctrl-C was sent.

EOF was not sent. After context compaction, the original interactive transcript handle was no
longer available. `read_thread_terminal` reported no app terminal session attached to this thread,
and Computer Use could not inspect the Codex app. Because the approval allowed EOF only if Claude
returned to a prompt, and no reliable prompt-state evidence was available, sending EOF would have
required inference rather than evidence.

## Postflight Evidence

| Check | Result |
| --- | --- |
| Native Claude process | PID `49349` remained live as `/Users/yuval.meiri/.local/bin/claude`, state `Ss+`, TTY `ttys000`. |
| Git status | Still clean except pre-existing untracked root `AGENTS.md`. |
| Git diff | No tracked diff. |
| Claude version/target | Still `2.1.161 (Claude Code)` and `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`. |
| User-level Claude hashes | Unchanged from preflight. |
| Project-local Claude hashes | Unchanged from preflight. |
| Harness status/doctor | Still `ready=true`; warnings unchanged. |
| Memory changes since pre-recovery cursor | `item_count=0`, `commit_count=0`, trace `019e8e62-ad36-7872-b253-7e566a93e37a`. |
| Post-recovery cursor | `019e8e5e-a5b4-72f0-9079-72128db8e738` at `2026-06-03T16:48:12.095489Z`. |
| Obligations doctor | `open=[]`, `warnings=[]`. |

## Side Effects And Attribution

| Side Effect | Attribution | Notes |
| --- | --- | --- |
| One Ctrl-C byte sent to `/dev/ttys000` | Caused by this approved recovery attempt | This was the only native Claude input sent in T185. |
| PID `49349` remained live | Caused or exposed by this recovery attempt | The recovery did not resolve the process. |
| No EOF sent | Bounded by evidence requirement | Prompt-return state could not be verified after compaction. |
| Git, monitored Claude files, Memory OS, obligations, and harness readiness unchanged | No observed mutation | Postflight matched preflight. |

## Decision

T185 does not close the live-process cleanup gate. It also does not close the T172 effective-hook
visibility gate, prompt-bearing native Claude behavior, missing SessionEnd `write_policy`
behavior, M6 migration readiness, lifecycle cleanup, ranking/`orient`, public MCP,
schema/storage/index, document-index behavior, deletion, rollback, force-kill, old-binary
reinstall, or user-owned-file changes.

The next step still requires an exact new approval packet or explicit user instruction for how to
resolve PID `49349`. Any future cleanup should state whether EOF without visible prompt evidence,
a second Ctrl-C, process-level signal, force-kill, or user/manual intervention is allowed.
