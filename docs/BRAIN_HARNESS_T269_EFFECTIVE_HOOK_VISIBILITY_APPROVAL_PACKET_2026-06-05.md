# Brain Harness T269 Effective-Hook Visibility Approval Packet

Date: 2026-06-05
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval for one native Claude Code effective-hook visibility
revalidation. It exists because T172/T179 attempted `/hooks`, saw native startup guidance, did not
capture visible effective-hook configuration, and left a live process until T197 resolved it with a
process-group `SIGINT`. T254 then kept effective-hook visibility open, while T255 prepared a
separate prompt-bearing MCP-`orient` validation packet that intentionally excludes `/hooks`.

T269 does not execute native Claude, Claude Bridge, `/hooks`, prompt-bearing Claude, harness
install, settings or hook edits, lifecycle cleanup, Memory OS archive, `lint apply_safe`,
M6/migration/quarantine actions, canonical vault actions, ranking/`orient` changes, public MCP
changes, schema/storage/index changes, document-index behavior changes, branch publication,
deletion, rollback, force-kill, old-binary reinstall, or user-owned-file adoption.

## Current Evidence

- T152/T153/T156/T254 show generated Claude Code adapter/readiness checks are green but static:
  Engram can validate installed/generated files and settings structure, not native runtime
  effective hook behavior.
- `engram-index/src/harness.rs` warns when Claude settings are split across
  `settings.json` and `settings.local.json`, and tells operators to verify effective hook
  configuration with Claude Code `/hooks`.
- The same source renders required Claude hook registrations for `SessionStart`, durable MCP hook
  events, and command-style `SessionEnd`.
- The installed/generated `SessionEnd` command hook defaults missing hook input `write_policy` to
  `nudge`, while most MCP hook registrations intentionally carry explicit
  `write_policy="durable"`.
- T172/T179 proved native startup guidance can appear, but `/hooks` did not produce visible
  effective configuration output in the captured PTY window.
- T197 proved the T172 live-process side effect can be resolved with one process-group `SIGINT`,
  and that native Claude exit can write a `SessionEnd` handoff MemoryItem.
- T255 is prepared for prompt-bearing MCP-`orient` behavior only. It does not authorize `/hooks`.
- T267 is prepared for canonical vault init/compile only. It does not authorize hook validation.
- T210/T250 keep M6 blocked on human dispositions or explicit deferral; T269 does not change that.

## Research Question

Can a future exact native Claude Code session produce falsifiable effective-hook visibility
evidence, or a clean bounded failure, without repeating T172's ambiguous output and live-process
side effect?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A new default-deny packet can make the future `/hooks` probe falsifiable by defining the visible observation channel, pass/fail rubric, and T197-style cleanup before launch. |
| Null | T172 already showed `/hooks` is not a useful evidence source, so another `/hooks` packet adds risk without reducing uncertainty. |
| Simpler alternative | Leave effective-hook visibility deferred and run only T255 prompt-bearing validation later. |
| Failure | The packet repeats T172 by accepting ambiguous output, requiring extra prompts or commands, leaving a process live, or treating SessionEnd side effects as cleanup permission. |

## Consultation Synthesis

AI Council recall recovered the T172 rule: native effective-hook validation is acceptable only as a
default-deny, single-session probe with complete pre/post snapshots, hard stops, no retries, and no
cleanup broadening.

A fresh AI Council broadcast on 2026-06-05 agreed that a T269 docs-only packet is useful if it does
not merely retry `/hooks`. The useful constraints were:

- define the expected visible output before execution;
- treat absence of hook output as a clean failed measurement, not a reason to probe further;
- pre-authorize process-group cleanup because T197 proved that path and T172 did not;
- keep T269 separate from T255 prompt-bearing validation, M6, lifecycle cleanup, and vault work;
- avoid claiming more than effective-hook visibility from one native session.

Claude Bridge read-only critique warned that "visible" must be operationally defined, hook
registration and hook execution must not be conflated, the observation channel must be named before
launch, and no-live-process PID accounting must be a completion condition rather than an advisory
cleanup note.

Model consensus is not proof. It only shaped the future packet boundaries.

## Observation Contract

The future execution's only observation channel for effective-hook visibility is the captured native
Claude PTY transcript. The run can pass the effective-hook visibility check only if the visible
post-`/hooks` output directly shows effective hook configuration for this repository and includes
all of these classes or an equivalent unambiguous effective-configuration table:

- `SessionStart`
- `UserPromptSubmit`
- `PostToolUse`
- `PostToolUseFailure`
- `Stop`
- `PreCompact`
- `PostCompact`
- `SessionEnd`

The output must also show enough handler detail to distinguish Engram hook registrations from a
generic hook list, such as Engram command paths, the `engram` MCP server/tool handler, or equivalent
Claude Code effective-configuration text.

These are not passing evidence:

- startup guidance alone;
- static file contents alone;
- `harness doctor ready=true`;
- absence of errors;
- an interactive menu that would require another command, cursor movement, selection, or prompt;
- a partial screen without event names;
- a SessionEnd handoff write.

If `/hooks` produces no visible configuration, only a menu, only startup guidance, or any output
that cannot be tied to the effective merged configuration, the future execution must record a
failed/inconclusive measurement and stop after cleanup/postflight.

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T269: execute the native Claude effective-hook visibility revalidation from docs/BRAIN_HARNESS_T269_EFFECTIVE_HOOK_VISIBILITY_APPROVAL_PACKET_2026-06-05.md. I understand this may trigger native Claude lifecycle hook side effects, including SessionEnd handoff writes. Run exactly the packet's preflight, one native Claude PTY session, one `/hooks` command, one EOF exit attempt, pre-authorized process-group SIGINT cleanup if EOF or the session hangs, and postflight comparisons. Treat missing or inconclusive `/hooks` output as a failed measurement, not permission to send more input. Do not send natural-language prompts, run T255, edit hooks/settings/adapters, run harness install, use adopt_user_owned, mutate lifecycle outside observed hook side effects, run M6/migration/quarantine, initialize or compile the canonical vault, change ranking/orient/public MCP/schema/storage/index/document-index behavior, publish branches, delete, rollback, force-kill beyond the packet, reinstall old binaries, or touch user-owned files.
```

Shorter approvals, generic continuation, or approval bundled with T255/T267/M6/lifecycle work are
not authorization to execute T269.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before native execution:

- `git status --short --branch`
- `git diff --stat`
- `git log --oneline -8`
- read-only branch/upstream status; do not run `git pull`, rebase, merge, checkout, push, or set
  upstream
- `/Users/yuval.meiri/.local/bin/engram harness status --harness claude-code`
- `/Users/yuval.meiri/.local/bin/engram harness doctor --harness claude-code`
- `/Users/yuval.meiri/.local/bin/engram daemon status`
- Memory OS `memory(action=cursor)`
- Obligations `obligations(action=doctor)`
- Telemetry `real_session_eval` for `limit=20` and `limit=50`
- Read-only native Claude process snapshot proving no existing process for this executable/session
  path
- Claude binary target and version check
- Hash and inventory snapshots for:
  - `/Users/yuval.meiri/.claude/settings.json`
  - `/Users/yuval.meiri/.claude/settings.local.json`
  - `/Users/yuval.meiri/.claude/hooks`
  - `/Users/yuval.meiri/.claude/commands`
  - `/Users/yuval.meiri/.claude/engram-settings-snippet.json`
  - `/Users/yuval.meiri/projects/engram/.claude`
  - `/Users/yuval.meiri/.local/bin/claude`
  - the resolved `/Users/yuval.meiri/.local/share/claude/versions/...` target

If the Claude binary target/version differs from the T179/T197/T255 baseline `2.1.161`, stop and
write a packet-drift report instead of launching native Claude, unless the user gives a fresh exact
approval that names the drift.

### Single Native PTY Session

Allowed execution if preflight matches this packet:

1. Launch exactly `/Users/yuval.meiri/.local/bin/claude` in a PTY from
   `/Users/yuval.meiri/projects/engram`.
2. Record the spawned PID, process group, TTY, and command before sending input.
3. Send exactly one input line:

   ```text
   /hooks
   ```

4. Capture the visible output until either the observation contract is satisfied, the output is
   clearly inconclusive, or a bounded wait expires.
5. Send exactly one EOF/Ctrl-D exit attempt.
6. If EOF does not exit within the chosen timeout, send exactly one process-group `SIGINT` to the
   foreground process group recorded for this PTY, equivalent to:

   ```text
   kill -INT -<PGID>
   ```

7. If the process remains live after that one process-group `SIGINT`, stop and report. Do not send
   another signal, `SIGTERM`, `SIGKILL`, second EOF, second Ctrl-C, another slash command, or a
   natural-language prompt.

No natural-language prompt is authorized. No `/hooks` retry is authorized. No T255 prompt-bearing
validation is authorized.

### Postflight Read-Only Snapshots

Allowed after the PTY session:

- Re-run the same git, branch, harness, daemon, hash, inventory, process, Memory OS cursor,
  Memory OS `changes_since`, telemetry, and obligations doctor snapshots.
- Compare pre/post state.
- Attribute each observed delta as one of:
  - expected from the exact native session,
  - caused by the exact native session but not expected,
  - ambient/unattributed,
  - unclear.
- Write one result report under `docs/`.
- Update `docs/BRAIN_HARNESS_ARCHITECTURE.md` and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` with the result note.
- Index exact changed docs.
- Resolve document obligations if any are created.
- Commit only intended documentation files.
- Capture a new current-plan memory after the commit.
- Submit telemetry feedback for assessed `orient`/`search` retrievals.

## Explicitly Forbidden

T269 does not authorize:

- any natural-language prompt to native Claude;
- any slash command except the single literal `/hooks`;
- T255 prompt-bearing MCP-`orient` validation;
- Claude Bridge execution as part of the future T269 run;
- non-interactive `claude -p`;
- editing installed hooks, settings, adapters, snippets, or user-owned files;
- `engram harness install`, `adopt_user_owned`, generated harness writes, or manual
  `harness(action=hook_event)` calls;
- lifecycle cleanup, archive, resolve, skip, handoff update, or other Memory OS writes during the
  validation window except hook-caused side effects and final post-commit current-plan capture;
- M6/migration/quarantine status, prioritize, apply, export, rerun, candidate decisions, or
  candidate inspection;
- canonical vault init/compile/status writes;
- ranking/`orient`, public MCP, schema/storage/index, or document-index behavior changes;
- branch reconciliation, pull, fetch, merge, rebase, checkout, push, upstream config, or PR
  publication;
- deletion, rollback, force-kill beyond the one process-group `SIGINT`, old-binary reinstall,
  binary/source refresh, service/PATH/auth configuration, or user-owned-file adoption;
- retries or fallback probes if `/hooks` is unavailable, output is inconclusive, EOF hangs beyond
  the single process-group `SIGINT`, or attribution is unclear.

## Hard Stops

Do not launch native Claude if any preflight condition is true:

- the Claude binary target/version differs from the recorded `2.1.161` baseline without fresh
  exact drift approval;
- any monitored hash differs unexpectedly before execution;
- git status has unexpected tracked changes;
- branch state is not recorded clearly enough to interpret the run;
- an existing native Claude process is live for the same executable/session path;
- Engram daemon is unhealthy and would make hook/MCP behavior uninterpretable;
- harness status/doctor is not `ready=true`;
- obligations doctor reports an open obligation that changes the scope;
- telemetry preflight cannot be recorded;
- required monitoring paths are missing in a way that prevents comparison.

After launch, stop immediately and report without broadening if:

- Claude requests setup/auth/config changes, extra permissions, or broader interaction;
- the startup transcript cannot be captured;
- the exact `/hooks` line cannot be sent without another interaction;
- `/hooks` is unavailable, inconclusive, only opens a menu, or requires another command to inspect
  effective hooks;
- any input beyond `/hooks` and one EOF would be required before cleanup;
- Claude attempts file edits, shell commands, branch operations, hook/settings changes, lifecycle
  cleanup, M6/migration/quarantine actions, vault actions, or out-of-scope work;
- EOF hangs and the single process-group `SIGINT` does not end the process;
- postflight shows tracked git changes, monitored Claude config drift, unexpected Memory OS writes,
  obligation drift, harness readiness drift, daemon drift, or orphan Claude processes;
- attribution is unclear and continuing would require another native session, another cleanup
  action, or user-owned-file inspection beyond this packet.

## Measurement

Minimum future result evidence:

- exact native Claude binary target and version at execution time;
- exact branch/worktree state at execution time;
- exact PID, process group, TTY, command, and process-tree evidence;
- exact PTY transcript or sanitized summary proving only `/hooks`, one EOF, and at most one
  process-group `SIGINT` were sent;
- whether native startup Engram guidance was visible;
- whether `/hooks` output met the observation contract;
- if not, the exact failed/inconclusive reason;
- pre/post hashes and inventories for user-level and project-local Claude paths;
- pre/post git status;
- pre/post Memory OS cursor and `changes_since` result;
- pre/post obligations doctor;
- pre/post harness status/doctor;
- pre/post daemon status;
- pre/post telemetry window reports;
- process exit code or timeout/SIGINT state;
- list of observed side effects, including Memory OS item IDs and whether they were expected.

## Completion Criteria For Future Execution

T269 execution can be marked complete only if it produces a committed result report, indexed docs,
resolved obligations, post-commit current-plan memory, telemetry feedback for material traces, and
a clear statement of whether effective-hook visibility is now supported or still open.

A passing future T269 run would close only the effective-hook visibility gate for one native Claude
session in this repository. It would not prove prompt-bearing native Claude MCP behavior, clean EOF
semantics, host external-session adoption, lifecycle cleanup, M6 migration, canonical vault
readiness, remote publication, or full cross-harness parity.
