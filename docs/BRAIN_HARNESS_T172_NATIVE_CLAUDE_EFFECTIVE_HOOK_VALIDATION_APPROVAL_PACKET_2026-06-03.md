# T172 Native Claude Effective-Hook Validation Approval Packet

Date: 2026-06-03

## Scope

This is a docs-only/default-deny approval packet. It prepares a future exact approval for native
Claude Code effective-hook validation. It does not execute native Claude, Claude Bridge, `/hooks`,
prompt-bearing Claude, harness install, settings or hook edits, lifecycle cleanup, Memory OS write
APIs, M6/migration/quarantine actions, ranking/`orient` changes, public MCP changes,
schema/storage/index changes, document-index behavior changes, deletion, rollback, force-kill,
old-binary reinstall, or user-owned-file adoption.

The future execution is intentionally narrower than full Claude behavior validation. It is only
meant to answer whether native Claude Code shows the effective merged hook configuration for this
repository, and whether starting native Claude, sending one `/hooks` slash command, and exiting
causes observable hook side effects.

## Current Evidence

- T170 executed only:
  - `/Users/yuval.meiri/.local/bin/claude --version`
  - `/Users/yuval.meiri/.local/bin/claude --help`
- T170 recorded `2.1.161 (Claude Code)` and no observed monitored mutation from those metadata
  commands.
- Fresh read-only `engram harness status --harness claude-code` and
  `engram harness doctor --harness claude-code` still report `ready=true`.
- Current Claude Code warnings remain:
  - user-owned `claude-settings-snippet`,
  - extra legacy Engram permission entries in both settings files,
  - split `settings.json` / `settings.local.json`,
  - effective hook configuration requires native Claude Code `/hooks` verification,
  - lifecycle compliance remains soft and depends on the agent following policy.
- Current monitored hashes:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `68e6b524b5505b66419631df3991e5f56985acc5272a490993eb50a47e230e9e` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

- Current Claude binary target:
  `/Users/yuval.meiri/.local/bin/claude -> /Users/yuval.meiri/.local/share/claude/versions/2.1.161`
- Fresh Memory OS cursor:
  `019e8d9f-fd4f-75d3-9899-154596aab93a` at
  `2026-06-03T13:20:38.620478Z`.
- Fresh obligations doctor reports `open=[]` and `warnings=[]`.
- Git status remains clean except the pre-existing untracked root `AGENTS.md`.
- Project-local Claude files exist under `/Users/yuval.meiri/projects/engram/.claude`, so future
  pre/post monitoring must include both user-level and project-level Claude paths.

## Research Question

Can one exact native Claude Code PTY session surface the effective merged hook configuration, while
bounding and measuring any lifecycle side effects caused by startup, `/hooks`, and exit?

## Hypotheses

| Hypothesis | Expected Evidence |
| --- | --- |
| Preferred | One native PTY session with only `/hooks` plus EOF captures effective hook configuration and produces no unapproved monitored mutation. |
| Null | Native metadata/status evidence is enough; no `/hooks` validation is needed. |
| Simpler alternative | Continue static inspection only and defer native effective-hook proof. |
| Failure | Startup, `/hooks`, or exit triggers hidden writes, hangs, prompts for broader interaction, or requires settings edits to inspect effective hooks. |

The null hypothesis is rejected by T171: split settings and installed hook registrations are not
proof of effective native Claude behavior. The simpler alternative remains safe, but it does not
close the effective-hook completion gate.

## AI Council Synthesis

Prior AI Council guidance for T153 warned that native Claude Code execution is not inherently
read-only because startup and exit can trigger lifecycle hooks and hidden writes. The T172 Council
critique agreed that the future slice can be appropriately narrow if it is a single PTY session
with one `/hooks` input, immediate exit, complete pre/post comparisons, no retries, and no cleanup.

The useful additions from the critique are:

- hard-stop on binary or monitored-hash drift before native execution,
- monitor both user-level and project-level Claude paths,
- capture process state and exit code,
- treat timeout or inconclusive `/hooks` output as a stop, not a reason to probe further,
- explicitly forbid retrying with additional slash commands, natural-language prompts, Claude
  Bridge, settings edits, or cleanup.

AI Council agreement is not proof. It only sharpens the measurement and stop conditions.

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T172: execute the native Claude effective-hook validation from docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md. I understand this may trigger native Claude lifecycle hook side effects caused by the single approved PTY session; observe and report those side effects only, do not clean them up or continue beyond the packet.
```

Shorter approvals should not be treated as approval to expand this packet.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before native execution:

- `git status --short --branch`
- `git diff --stat`
- `git log --oneline -8`
- `/Users/yuval.meiri/.local/bin/engram harness status --harness claude-code`
- `/Users/yuval.meiri/.local/bin/engram harness doctor --harness claude-code`
- Memory OS `memory(action=cursor)`.
- Obligations `obligations(action=doctor)`.
- Hash and file inventory snapshots for:
  - `/Users/yuval.meiri/.claude/settings.json`
  - `/Users/yuval.meiri/.claude/settings.local.json`
  - `/Users/yuval.meiri/.claude/hooks`
  - `/Users/yuval.meiri/.claude/commands`
  - `/Users/yuval.meiri/.claude/engram-settings-snippet.json`
  - `/Users/yuval.meiri/projects/engram/.claude`
  - `/Users/yuval.meiri/.local/bin/claude`
  - `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`
- Read-only Claude process snapshot for orphan-process comparison.

### Single Native PTY Session

Allowed execution if preflight matches this packet:

1. Launch exactly `/Users/yuval.meiri/.local/bin/claude` in a PTY from
   `/Users/yuval.meiri/projects/engram`.
2. Send exactly one input line:

   ```text
   /hooks
   ```

3. Capture the visible output.
4. Exit using EOF/Ctrl-D.
5. Wait only long enough to observe clean exit or a timeout.

No natural-language prompt may be sent. No second slash command may be sent. No retry is authorized.

### Postflight Read-Only Snapshots

Allowed after the PTY session:

- Re-run the same git, harness, hash, inventory, process, Memory OS cursor, Memory OS
  `changes_since`, and obligations doctor snapshots.
- Compare pre/post state.
- Attribute each observed delta as one of:
  - caused by the exact native session,
  - ambient/unattributed,
  - unclear.
- Write one result report under `docs/`.
- Update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Commit only those intended documentation files.
- Capture a new current-plan memory after the commit.
- Submit telemetry feedback for assessed `orient`/`search` retrievals.

## Explicitly Forbidden

This packet does not authorize:

- prompt-bearing natural-language input to native Claude,
- any slash command except the single literal `/hooks`,
- Claude Bridge,
- `claude -p` or any prompt-bearing non-interactive Claude invocation,
- editing installed hooks, settings, adapters, snippets, or user-owned files,
- `engram harness install`, `adopt_user_owned`, or generated harness writes,
- manual `harness(action=hook_event)` calls,
- lifecycle cleanup, archive, resolve, skip, handoff update, or other Memory OS write APIs during
  the validation window except the final post-commit current-plan capture,
- M6/migration/quarantine status, prioritize, apply, rerun, candidate decisions, or candidate
  inspection,
- ranking/`orient`, public MCP, schema/storage/index, or document-index behavior changes,
- deletion, rollback, force-kill, old-binary reinstall, binary/source refresh, service/PATH/auth
  configuration, or user-owned-file adoption,
- retries, fallback probes, or broader commands if `/hooks` is unavailable or inconclusive.

## Hard Stops

Do not launch native Claude if any preflight condition is true:

- the Claude binary target is not
  `/Users/yuval.meiri/.local/share/claude/versions/2.1.161`,
- any monitored hash above differs before execution,
- git status has unexpected tracked changes,
- harness status is not `ready=true`,
- obligations doctor reports an open obligation that changes the scope,
- required monitoring paths are missing in a way that prevents comparison.

After launch, stop immediately and report without cleanup if:

- Claude requests natural-language input or attempts normal prompt-bearing continuation before
  `/hooks` can be captured,
- any input beyond `/hooks` and EOF would be required,
- `/hooks` is unavailable, inconclusive, or requires another command to inspect effective hooks,
- the PTY hangs or does not exit after EOF within the chosen timeout,
- a hook visibly writes or starts acting outside the approved observation boundary,
- postflight shows tracked git changes, Claude settings/hook/config drift, new unexpected files,
  Memory OS writes, obligation changes, harness readiness drift, or orphan Claude processes,
- attribution is unclear and continuing would require another native session or cleanup.

If a hard stop leaves a live process and graceful EOF has failed, do not force-kill under this
packet. Pause and ask the user for explicit next instructions.

## Measurement

Minimum result evidence:

- exact native Claude binary target and version at execution time,
- exact PTY transcript or a sanitized summary with enough detail to prove only `/hooks` and EOF
  were sent,
- captured effective hook configuration or the exact reason it could not be captured,
- pre/post hashes and inventories for user-level and project-level Claude paths,
- pre/post git status,
- pre/post Memory OS cursor and `changes_since` result,
- pre/post obligations doctor,
- pre/post harness status/doctor,
- process exit code or timeout state,
- list of observed side effects, with attribution and IDs/paths where available.

## Completion Criteria

T172 execution can be marked complete only if it produces a committed result report and current-plan
memory. A passing result closes only the effective-hook visibility gate for the single approved
native session. It does not prove prompt-bearing native Claude behavior, broad lifecycle compliance,
M6 migration readiness, or missing SessionEnd `write_policy` behavioral semantics unless those facts
are directly observed and bounded by the approved session.
