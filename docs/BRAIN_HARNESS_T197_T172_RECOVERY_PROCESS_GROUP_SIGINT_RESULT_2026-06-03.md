# T197 T172 Recovery Process-Group SIGINT Result

Date: 2026-06-03
Status: cleanup result; live native Claude process resolved; effective-hook gate remains open

## Scope

The user approved this recovery operation:

```text
Approve T172 recovery option 2: send Ctrl-C to the live native Claude PTY, then send EOF if it returns to the prompt, and continue with T172 postflight comparisons and the result report.
```

This result records the bounded recovery after T179, T185, and T190 left the native Claude process
PID `49349` live. Because the original Codex PTY session handle was no longer available after
context compaction, Codex could not send Ctrl-C through `write_stdin`. Fresh process evidence showed
the native Claude PTY foreground process group was `49349`, so Codex sent one foreground-process-
group `SIGINT`:

```text
kill -INT -49349
```

That is the process-group equivalent of Ctrl-C for the live PTY. EOF was not sent because the
process group exited and there was no returned prompt to receive EOF.

This run did not launch native Claude, run Claude Bridge, run `/hooks`, send natural-language input,
send EOF, send a second signal, force-kill, edit hooks/settings/adapters, run harness install,
mutate lifecycle or migration state, change ranking/`orient`, public MCP, schema/storage/index, or
document-index behavior, delete, roll back, reinstall binaries, or touch user-owned files.

## Research Framing

Question: can the live native Claude process left by T172 be resolved through the approved
Ctrl-C-style recovery path, while measuring any lifecycle side effects?

| Type | Result |
| --- | --- |
| Preferred | Supported for cleanup. One foreground-process-group interrupt exited PID `49349` and its PTY children. |
| Null | Rejected. Leaving the process live would preserve an unresolved native validation side effect. |
| Simpler alternative | No longer needed for this gate. The process is no longer live. |
| Failure | Partially observed as a side effect: native Claude exit triggered a Claude Code SessionEnd handoff MemoryItem. |

## Preflight Evidence

| Check | Result |
| --- | --- |
| Startup `orient` | Lean trace `019e8ea4-dd01-7ab3-912c-49bbfffc5a03`; current-plan memory `019e8ea2-b1d7-7c31-8a6b-5afaf473bd0f` returned first. |
| Memory cursor | `019e8ea2-b1fd-7ba3-95a6-ba775153eae8` at `2026-06-03T18:00:29.476306Z`. |
| Obligations | `detect` dry-run returned no candidates; `doctor` returned `open=[]`, `warnings=[]`. |
| Git status | Branch `yuval.meiri/memory-os-phase0`; clean except pre-existing untracked root `AGENTS.md`. |
| Latest commit | `66d1616 Record T196 design preference retrieval recheck`. |
| Native Claude process | PID `49349`, PGID `49349`, TPGID `49349`, state `Ss+`, TTY `ttys000`, command `/Users/yuval.meiri/.local/bin/claude`. |
| PTY children | PIDs `49389`, `49391`, and `49546` were in PGID `49349`. |
| Claude binary | `2.1.161 (Claude Code)`; target `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`. |
| App terminal | `read_thread_terminal` reported no app terminal session attached to this thread. |
| Harness status/doctor | `claude_code`, `ready=true`; warnings unchanged from T179/T185/T190. |

Representative monitored hashes matched T172/T179/T185/T186/T190 evidence:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |
| `/Users/yuval.meiri/.local/bin/claude` | `5b4dc79eab05f9756c252c71deb339efa4429dffc1967dd8392cf87fcde4867f` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-session-end.sh` | `f614a59a4d226f262100aada54836160a0740538e264837b6959177638edd5d7` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-session-start.sh` | `c49c516aa30604cb87841d368e830275aa05355c27a359664876ac742350b27f` |
| `/Users/yuval.meiri/projects/engram/.claude/hooks/engram-stop-nudge.sh` | `66ecbae5279f08a8e0d6ff52bd69e2e9b8b7dd4993c5753074196e03111d9f85` |
| `/Users/yuval.meiri/projects/engram/.claude/settings.json` | `93a6d5c289121a4c43cda65f6729d0f5135ae3cc4733d1396fb126cbe2bc68bd` |
| `/Users/yuval.meiri/projects/engram/.claude/settings.local.json` | `e2bb188f7fc346750f6ebaee0632694da603c48402974d4dd8b2b8fd1c4daaf6` |

## Recovery Action

Codex sent exactly one interrupt to the live PTY foreground process group:

```text
kill -INT -49349
```

After a two-second wait, `ps -o pid,ppid,pgid,tpgid,stat,tty,command -p 49349,49389,49391,49546`
returned no process rows. EOF was not sent because the target process group had exited.

## Postflight Evidence

| Check | Result |
| --- | --- |
| Native Claude process | PID `49349` no longer exists; no `/Users/yuval.meiri/.local/bin/claude` process remains on `ttys000`. |
| Claude binary | Still `2.1.161 (Claude Code)` and target `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`. |
| Git status | Still clean except pre-existing untracked root `AGENTS.md`. |
| Harness status/doctor | Still `ready=true`; warnings unchanged. |
| Obligations doctor | `open=[]`, `warnings=[]`. |
| Lint read-only | Known wrong-scope and superseded-active findings remain; no `apply_safe` was run. |
| Monitored hashes | User-level and project-local Claude hashes matched preflight. |
| Telemetry | Current 50-trace window has no task failures or bad-memory-use reports, but feedback coverage dropped below the confidence gate after new unscored traces. |

Memory OS `changes_since` from the pre-recovery cursor returned one item and no commits:

| Field | Value |
| --- | --- |
| Trace | `019e8ea6-26e7-7d91-a9fb-768d62e1c078` |
| Item count | `1` |
| Commit count | `0` |
| Item | `019e8ea5-663e-7152-b346-9c5ab7ddc93b` |
| Kind | `handoff` |
| Writer | `claude_code` / `anthropic` / `claude-code` |
| Created at | `2026-06-03T18:01:04.830306Z` |
| Supersedes | `019e8e9d-1e08-76d1-ab53-3c7f63ca0baa` |

The handoff content identifies the native Claude session:

```text
Session: b8b93a45-7d72-4699-b320-cbce503c3a86
CWD: /Users/yuval.meiri/projects/engram
Reason: other
Transcript: /Users/yuval.meiri/.claude/projects/-Users-yuval-meiri-projects-engram/b8b93a45-7d72-4699-b320-cbce503c3a86.jsonl
```

## Side Effects And Attribution

| Side Effect | Attribution | Notes |
| --- | --- | --- |
| PID `49349` and PTY children exited | Caused by approved recovery | This resolves the live native Claude cleanup gate. |
| MemoryItem `019e8ea5-663e-7152-b346-9c5ab7ddc93b` was written | Caused by native Claude SessionEnd lifecycle side effect from exit | This was observed and reported only; no lifecycle cleanup or archive was run. |
| No EOF sent | Bounded by observed state | The process exited, so the conditional EOF path was not applicable. |
| Monitored Claude settings/hooks/config hashes unchanged | No observed config mutation | Hashes matched preflight. |
| Harness readiness unchanged | No observed readiness drift | Known warnings remain. |

## Decision

T197 closes the live native-Claude process cleanup gate: PID `49349` is gone and postflight evidence
shows no tracked git changes, no monitored Claude file drift, no obligation drift, and unchanged
harness readiness.

T197 does not close the T172 effective-hook visibility gate. The approved native `/hooks` session
still did not produce visible effective-hook configuration evidence. It also does not prove
prompt-bearing native Claude behavior, missing SessionEnd `write_policy` behavior, M6 migration
readiness, lifecycle cleanup, ranking/`orient`, public MCP, schema/storage/index, document-index
behavior, deletion, rollback, old-binary reinstall, or user-owned-file changes.

The newly written SessionEnd handoff is an observed lifecycle side effect. It should remain active
unless a separate lifecycle archive packet is prepared and explicitly approved.
