# T186 T185 Native Claude SIGINT Cleanup Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval to resolve the live native Claude Code process left by
T172/T179 and still present after T185. It does not send input to Claude, signal or kill a process,
launch native Claude, run Claude Bridge, run `/hooks`, use prompt-bearing Claude, edit
hooks/settings/adapters, run harness install, mutate lifecycle or migration state, change
ranking/`orient`, change public MCP/schema/storage/index or document-index behavior, delete files,
roll back, reinstall binaries, or touch user-owned files.

The future approved slice is intentionally only one process-level `SIGINT` cleanup attempt plus
post-recovery comparison. It is not a second effective-hook validation attempt and must not try to
recover `/hooks` output or interact with native Claude's UI.

## Current Evidence

- T179 is committed as `602f1a1` and records the exact-approved T172 native Claude session.
- T180 is committed as `9e6f78f` and prepared a default-deny one-Ctrl-C PTY cleanup packet.
- T185 is committed as `07c0ce2` and records that the re-approved T172 recovery option 2 rerun sent
  exactly one Ctrl-C byte to `/dev/ttys000`; PID `49349` remained live.
- T185 did not send EOF because EOF was conditional on Claude returning to a prompt, and after
  context compaction there was no reliable transcript handle.
- Fresh process evidence shows PID `49349`, PPID `70787`, state `Ss+`, TTY `ttys000`, command
  `/Users/yuval.meiri/.local/bin/claude`, elapsed `01:37:02`.
- Fresh Claude version is `2.1.161 (Claude Code)`.
- Fresh Claude symlink target is
  `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`.
- Fresh git status is clean except pre-existing untracked root `AGENTS.md`.
- Fresh `harness(status)` and `harness(doctor)` for `claude_code` report `ready=true`, with the
  known warnings about user-owned snippet, legacy permissions, split settings, effective hook
  verification, and soft lifecycle compliance.
- Fresh obligations doctor reports `open=[]` and `warnings=[]`.
- Fresh Memory OS `changes_since` from the latest current-plan cursor returned `item_count=0` and
  `commit_count=0`, trace `019e8e66-5c7c-7730-a187-943eadd9fe38`.

Fresh user-level hashes:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |
| `/Users/yuval.meiri/.local/bin/claude` | `5b4dc79eab05f9756c252c71deb339efa4429dffc1967dd8392cf87fcde4867f` |

Fresh project-local hashes:

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

## Completion Matrix Delta

| Area | State After T186 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Live native Claude process | Unresolved; exact cleanup approval prepared | PID `49349` remains live after T179 and T185 | Needs exact T186 approval before any process-level signal. |
| Effective-hook visibility | Still open | T179 saw startup hook output but no visible `/hooks` configuration | T186 cannot close this gate. |
| Native prompt-bearing behavior | Unproven | No natural-language native Claude prompt was sent | Requires a separate approval/eval, if still needed. |
| `orient` hot path | Preserved | No source or runtime behavior change in this packet | Do not expand payload or ranking as part of cleanup. |
| M6/migration | Still high-risk and gated | No M6 action in this packet | Needs reviewed candidates, dry-run evidence, rollback plan, and explicit approval. |
| Document indexing | Not run | T181/T184 remain separate exact-file indexing gates | Generic continuation is not indexing approval. |

## Research Question

Can Engram safely resolve the T172/T179/T185 live native Claude side effect by sending exactly one
process-level `SIGINT` to PID `49349`, then measuring whether any SessionEnd or other lifecycle side
effects occur, without UI input, broader signals, force-kill, or configuration mutation?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | One `kill -INT 49349` exits the native Claude process, after which read-only postflight shows no unapproved git, Claude-file, Memory OS, obligation, or harness drift except any lifecycle side effects directly attributable to that exit. |
| Null | Leaving PID `49349` live is acceptable and no recovery is needed. |
| Simpler alternative | Stop with T185 as the hard-stop result and leave PID resolution to the user outside Engram work. |
| Failure | `SIGINT` does not exit the process, exit triggers unexpected writes, or attribution becomes unclear enough that `SIGTERM`, `SIGKILL`, UI input, cleanup, or a new native session would be required. |

The null is not acceptable for the full Brain Harness goal because a live native Claude process is
an unresolved validation side effect. The simpler alternative remains safe but keeps the cleanup
gate open.

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T186: execute the T185 native Claude live-process SIGINT cleanup packet from docs/BRAIN_HARNESS_T186_T185_NATIVE_CLAUDE_SIGINT_CLEANUP_APPROVAL_PACKET_2026-06-03.md. After fresh matching read-only git/process/Claude version/target/hash/harness/Memory OS/obligation evidence and no intervening writes, send exactly one process-level SIGINT using `kill -INT 49349` to PID 49349 if it is still the same `/Users/yuval.meiri/.local/bin/claude` process, then run read-only postflight comparisons and write the result report. Do not send PTY input, EOF, Ctrl-C bytes, slash commands, natural-language prompts, any signal other than the single SIGINT, force-kill, launch native Claude, run Claude Bridge, edit hooks/settings/adapters, run harness install, mutate lifecycle or migration state, change ranking/orient/public MCP/schema/storage/index/document-index behavior, delete, roll back, reinstall binaries, or touch user-owned files.
```

Shorter approval, generic continuation, T172/T180 approval, or T174 approval must not be treated as
T186 approval.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before sending `SIGINT`:

- `git status --short --branch`
- `git diff --stat`
- `git log --oneline -8`
- process snapshot for PID `49349`
- Claude version and symlink target checks
- user-level and project-local Claude file hash snapshots
- `harness(status)` and `harness(doctor)` for `claude_code`
- `obligations(action=doctor)`
- Memory OS cursor and `changes_since` from the latest current-plan cursor

### Single Recovery Signal

Allowed only if preflight still matches this packet and no intervening writes occurred:

```text
kill -INT 49349
```

Wait only long enough to observe whether the process exits or remains live.

No PTY input, EOF, Ctrl-C byte, second `SIGINT`, other signal, slash command, natural-language
prompt, force-kill, or new Claude launch is authorized.

### Post-Recovery Read-Only Snapshots

Allowed after the single `SIGINT`:

- Re-run the same git, process, Claude version/target, hash, harness, Memory OS, and obligations
  snapshots.
- Compare pre/post state.
- Attribute observed deltas as:
  - caused by the exact `SIGINT` recovery,
  - ambient/unattributed,
  - unclear.
- Write one result report under `docs/`.
- Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Commit only those intended documentation files.
- Capture current-plan memory after the commit.
- Submit telemetry feedback for assessed retrieval traces.

## Explicitly Forbidden

T186 does not authorize:

- any native Claude UI input, including EOF, Ctrl-C bytes, `/hooks`, another slash command, or
  prompt-bearing input;
- any signal except the single approved `SIGINT`;
- a second `SIGINT`, `SIGTERM`, `SIGKILL`, Activity Monitor, terminal automation, or force-kill
  fallback;
- launching native Claude or Claude Bridge;
- `claude -p` or any other prompt-bearing Claude invocation;
- editing hooks, settings, adapters, snippets, or user-owned files;
- `engram harness install`, `adopt_user_owned`, or generated harness writes;
- manual `harness(action=hook_event)` calls;
- lifecycle cleanup, archive, resolve, skip, handoff update, or other Memory OS write APIs during
  recovery except the final post-commit current-plan capture;
- M6/migration/quarantine status, prioritize, apply, rerun, candidate decisions, or candidate
  inspection;
- ranking/`orient`, public MCP, schema/storage/index, or document-index behavior changes;
- deletion, rollback, old-binary reinstall, binary/source refresh, service/PATH/auth
  configuration, or user-owned-file adoption.

## Hard Stops

Do not send `SIGINT` if:

- PID `49349` is no longer the same `/Users/yuval.meiri/.local/bin/claude` process;
- git status has unexpected tracked changes;
- Claude binary target or monitored hashes drift before recovery;
- harness readiness changes before recovery;
- obligations doctor reports an open obligation that changes the scope;
- Memory OS shows intervening writes after the preflight cursor;
- the user approval is ambiguous or does not match the exact T186 scope.

Stop immediately and report without cleanup if:

- `SIGINT` does not exit the process;
- any observed output suggests a write-capable action beyond normal exit;
- postflight shows tracked git changes, monitored Claude hash drift, unexpected Memory OS writes,
  obligation changes, harness readiness drift, or unclear side effects;
- continuing would require another signal, PTY input, force-kill, another native session, or
  cleanup.

## Measurement

The result, if approved and executed, must record:

- pre/post PID state and command;
- exact signal sent;
- whether SessionEnd or other lifecycle output appeared if visible through available logs;
- pre/post git status;
- pre/post user-level and project-local Claude hashes;
- pre/post Memory OS cursor and `changes_since`;
- pre/post obligations doctor;
- pre/post harness status/doctor;
- any side effects with attribution and evidence.

## Completion Criteria

T186 execution can be marked complete only if it produces a committed result report and
implementation-plan note. It may close the live-process cleanup gate if PID `49349` exits and
postflight evidence is clean or attributable. It cannot close the T172 effective-hook visibility
gate, prompt-bearing native Claude behavior, missing SessionEnd `write_policy` behavior, M6
migration readiness, lifecycle cleanup, ranking/`orient`, public MCP, schema/storage/index,
document-index behavior, deletion, rollback, or user-owned-file changes.
