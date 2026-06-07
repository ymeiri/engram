# T255 Native Claude Prompt-Bearing Parity Approval Packet

Date: 2026-06-04
Status: docs-only/default-deny approval packet. This packet has not been executed.

## Scope

This packet prepares a future exact approval for one native Claude Code prompt-bearing validation
run in the Engram repository. It is intentionally narrower than full native-Claude/harness parity:
it tests whether a native prompt-bearing Claude session can receive Engram startup guidance, use the
live Engram MCP `orient` tool on request, produce a bounded answer, and exit or be cleaned up under
a pre-authorized recovery path while all side effects are measured.

This packet does not execute native Claude, Claude Bridge, `/hooks`, harness install, hook/settings
edits, lifecycle cleanup, Memory OS archive, `lint apply_safe`, M6/migration/quarantine actions,
ranking/`orient` changes, public MCP changes, schema/storage/index changes, document-index behavior
changes, branch reconciliation, deletion, rollback, force-kill, old-binary reinstall, or
user-owned-file adoption.

## Research Question

Can one exact native Claude Code prompt-bearing session use Engram's live MCP `orient` path and exit
under bounded cleanup rules, while making all hook, Memory OS, telemetry, process, file, and git
side effects visible?

## Hypotheses

| Type | Hypothesis | Expected Evidence |
| --- | --- | --- |
| Preferred | Native Claude receives Engram startup guidance, follows a bounded read-only prompt, calls `orient`, returns the trace/top item, and exits without unapproved mutation. | PTY transcript shows startup guidance and one prompt; postflight shows no tracked git or monitored config drift; Memory OS changes are limited to expected traces and any explicitly observed hook side effects. |
| Null | T170/T179/T197 evidence is sufficient; no prompt-bearing validation is needed. | Rejected by T254 because prompt-bearing native Claude behavior remains unproved. |
| Simpler alternative | Keep T255 docs-only and defer all native execution. | Safe, but does not close the prompt-bearing parity gate. |
| Failure | The session hangs, requests broader input/permissions, cannot use MCP `orient`, writes unexpected memory/obligations/handoff state, changes monitored files, or requires cleanup beyond the pre-authorized path. | Stop, report, and do not retry or broaden. |

## Proposed Approval Wording

Use this exact approval if the next slice should execute:

```text
Approve T255: execute the native Claude prompt-bearing harness parity validation from docs/BRAIN_HARNESS_T255_NATIVE_CLAUDE_PROMPT_BEARING_PARITY_APPROVAL_PACKET_2026-06-04.md. I understand this may trigger Claude Code lifecycle hook side effects, including Memory OS handoff writes. Run exactly the packet's preflight, one native Claude session, bounded prompt, bounded exit path, pre-authorized process-group SIGINT cleanup if EOF hangs, and postflight comparisons. Do not edit hooks/settings/adapters, run harness install, use adopt_user_owned, mutate lifecycle outside observed hook side effects, run M6/migration/quarantine, change ranking/orient/public MCP/schema/storage/index/document-index behavior, reconcile branches, delete, rollback, force-kill beyond the packet, reinstall old binaries, or touch user-owned files.
```

Shorter or broader approvals should not be treated as authorization to execute this packet.

## If Approved: Authorized Operations

### Preflight Read-Only Snapshots

Allowed before native execution:

- `git status --short --branch`
- `git diff --stat`
- `git log --oneline -8`
- read-only branch/upstream status; do not run `git pull`, `git fetch`, rebase, merge, or checkout
- `/Users/yuval.meiri/.local/bin/engram harness status --harness claude-code`
- `/Users/yuval.meiri/.local/bin/engram harness doctor --harness claude-code`
- `/Users/yuval.meiri/.local/bin/engram daemon status`
- Memory OS `memory(action=cursor)`
- Obligations `obligations(action=doctor)`
- Telemetry `real_session_eval` for both `limit=20` and `limit=50`
- Read-only Claude process snapshot for orphan-process comparison
- Hash and inventory snapshots for:
  - `/Users/yuval.meiri/.claude/settings.json`
  - `/Users/yuval.meiri/.claude/settings.local.json`
  - `/Users/yuval.meiri/.claude/hooks`
  - `/Users/yuval.meiri/.claude/commands`
  - `/Users/yuval.meiri/.claude/engram-settings-snippet.json`
  - `/Users/yuval.meiri/projects/engram/.claude`
  - `/Users/yuval.meiri/.local/bin/claude`
  - the resolved `/Users/yuval.meiri/.local/share/claude/versions/...` target

Preflight must record the exact Claude binary target and version. If the target differs from the
T170/T179/T197 baseline `2.1.161`, stop and write a packet-drift report instead of launching
native Claude.

### Single Native PTY Session

Allowed execution if preflight matches this packet:

1. Launch exactly `/Users/yuval.meiri/.local/bin/claude` in a PTY from
   `/Users/yuval.meiri/projects/engram`.
2. Send exactly one natural-language prompt:

   ```text
   Read-only Engram harness validation. Use only the Engram MCP orient tool with project "engram", cwd "/Users/yuval.meiri/projects/engram", intent "plan_work", response_shape "lean", and agent "claude_code". Do not edit files or run shell commands. Then answer with exactly two lines:
   ORIENT_TRACE_ID: <trace_id>
   TOP_ITEM_ID: <first brain_loop top_items id, or none>
   ```

3. Capture the visible output.
4. Exit using one EOF/Ctrl-D.
5. If EOF does not exit within the chosen timeout, identify the foreground process group for the
   native Claude PTY and send exactly one process-group SIGINT equivalent to Ctrl-C:

   ```text
   kill -INT -<PGID>
   ```

6. If the process remains live after that one process-group SIGINT, stop and report. Do not send a
   second signal, force-kill, EOF retry, slash command, natural-language prompt, or cleanup action.

No `/hooks` command is authorized in this packet. No second prompt is authorized. No retry is
authorized.

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
- Resolve document obligations.
- Commit only intended documentation files.
- Capture a new current-plan memory after the commit.
- Submit telemetry feedback for assessed `orient`/`search` retrievals.

## Explicitly Forbidden

This packet does not authorize:

- any slash command, including `/hooks`;
- more than one natural-language prompt;
- Claude Bridge;
- non-interactive `claude -p`;
- editing installed hooks, settings, adapters, snippets, or user-owned files;
- `engram harness install`, `adopt_user_owned`, generated harness writes, or manual
  `harness(action=hook_event)` calls;
- lifecycle cleanup, archive, resolve, skip, handoff update, or other Memory OS writes during the
  validation window except hook-caused side effects and final post-commit current-plan capture;
- M6/migration/quarantine status, prioritize, apply, export, rerun, candidate decisions, or
  candidate inspection;
- ranking/`orient`, public MCP, schema/storage/index, or document-index behavior changes;
- git branch reconciliation, pull, fetch, merge, rebase, checkout, or push;
- deletion, rollback, force-kill, old-binary reinstall, binary/source refresh, service/PATH/auth
  configuration, or user-owned-file adoption;
- retries or fallback probes if MCP orient is unavailable, output is inconclusive, EOF hangs beyond
  the single SIGINT recovery, or attribution is unclear.

## Hard Stops

Do not launch native Claude if any preflight condition is true:

- Claude binary target/version differs from the recorded `2.1.161` baseline;
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
- the startup transcript lacks enough evidence to identify whether Engram guidance appeared;
- the exact prompt cannot be sent without additional interaction;
- Claude cannot call the Engram MCP `orient` tool within the wait window;
- Claude attempts file edits, shell commands, branch operations, hook/settings changes, lifecycle
  cleanup, M6/migration/quarantine actions, or other out-of-scope work;
- the answer is inconclusive and another prompt would be required;
- EOF hangs and the single process-group SIGINT does not end the process;
- postflight shows tracked git changes, monitored Claude config drift, unexpected Memory OS writes,
  obligation drift, harness readiness drift, daemon drift, or orphan Claude processes;
- attribution is unclear and continuing would require another native session, another cleanup
  action, or user-owned-file inspection beyond this packet.

## Measurement

Minimum result evidence:

- exact native Claude binary target and version at execution time;
- exact branch/worktree state at execution time;
- exact PTY transcript or sanitized summary proving only one prompt, one EOF, and at most one
  process-group SIGINT were sent;
- whether native startup Engram guidance was visible;
- whether native Claude used MCP `orient` and returned a trace ID;
- `orient` top item ID and whether it matched the active current-plan memory at execution time;
- pre/post hashes and inventories for user-level and project-local Claude paths;
- pre/post git status;
- pre/post Memory OS cursor and `changes_since` result;
- pre/post obligations doctor;
- pre/post harness status/doctor;
- pre/post daemon status;
- pre/post telemetry window reports;
- process exit code or timeout/SIGINT state;
- list of observed side effects, including Memory OS item IDs and whether they were expected.

## Completion Criteria

T255 execution can be marked complete only if it produces a committed result report, indexed docs,
resolved obligations, post-commit current-plan memory, telemetry feedback for material traces, and
a clear statement of which subclaims are now supported or still open.

T255 can support only the prompt-bearing native-Claude MCP-orient subclaim. It cannot by itself
close effective-hook visibility, clean EOF semantics, host external-session adoption, lifecycle
cleanup, M6 migration, branch synchronization, or full cross-harness parity.
