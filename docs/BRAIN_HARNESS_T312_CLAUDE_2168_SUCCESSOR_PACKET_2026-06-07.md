# Brain Harness T312 Claude 2.1.168 Successor Packet - 2026-06-07

## Scope

T312 prepares a docs-only successor packet after T311 proved that the T282 native Claude
prompt-bearing packet is stale under the current installed Claude Code version.

T312 is docs-only. It does not launch native Claude, attach to running Claude processes, send
prompts, run `/hooks`, edit hooks/settings/adapters, run harness install, mutate lifecycle state,
run M6, initialize or compile the canonical vault, change ranking or `orient`, publish branches,
delete data, roll back, send process signals, or touch user-owned files.

T312 supersedes T282 only for the recorded Claude Code target/version/hash baseline. It does not
certify Claude Code `2.1.168` compatibility and does not make the current native-Claude gate
executable while process attribution and harness readiness remain unresolved.

## Research Question

Can Engram safely refresh the T282 prompt-bearing native Claude validation packet for the observed
Claude Code `2.1.168` target without launching Claude, repairing adapters, widening beta scope,
combining host-label/effective-hook gates, or claiming behavioral compatibility before a transcript
proves it?

## Hypotheses

| Type | Hypothesis | Expected Evidence |
| --- | --- | --- |
| Preferred | A docs-only successor packet can record the `2.1.168` target and preserve the T282 execution boundary while clearly hard-stopping current execution. | Packet records T311 evidence, new hard preflight assertions, adapter/process blockers, and explicit non-claims. |
| Null | T282 can still be reused despite version drift from `2.1.163` to `2.1.168`. | Rejected because T282 hard-stops on target/version/hash drift. |
| Simpler alternative | Leave T282 stale and defer native Claude proof. | Safe, but future sessions may try to execute a known-stale packet. |
| Failure | The successor silently treats `2.1.168` as supported, hides harness `ready=false`, combines T269/T270, or resets beta scope. | Prevented by scope exclusions, hard stops, and non-claims. |

## Consultation Synthesis

AI Council recall surfaced the prior T282/T269/T270 default-deny gate decisions:

- prepare docs-only successor packets before risky native execution;
- do not combine prompt-bearing validation with `/hooks` or host-label proof;
- stop before launch on version, hash, process attribution, harness, daemon, or worktree drift;
- treat model consensus as packet-design input, not proof of runtime behavior.

A fresh three-model broadcast on 2026-06-07 agreed that a docs-only T312 packet is the right
smallest production-aligned slice under the current constraints. The shared recommendation was:

- record Claude Code `2.1.168` and the observed SHA-256 as read-only evidence only;
- keep adapter drift and harness `ready=false` explicit;
- do not launch or signal existing native Claude processes;
- preserve PR/beta scope boundaries;
- state that observed binary/hash presence is not behavioral, security, or compatibility proof.

## T311 Evidence Inputs

T311/T312 read-only preflight observed:

- branch `yuval.meiri/memory-os-phase1` tracks
  `origin/yuval.meiri/memory-os-phase1`, with `HEAD...@{u}` at `0 0`;
- the tracked worktree is clean, with only root `AGENTS.md` untracked and user-owned;
- PR #3 remains open/draft/merge-clean at head
  `f2afa5b352e7febd049e5d031d077cd75dd61958`, with exact-head CI run
  `27086973233` green before this docs-only successor slice;
- `/Users/yuval.meiri/.local/bin/claude` resolves to
  `/Users/yuval.meiri/.local/share/claude/versions/2.1.168`;
- `/Users/yuval.meiri/.local/bin/claude --version` returns
  `2.1.168 (Claude Code)`;
- the resolved target SHA-256 is
  `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`;
- native Claude CLI processes are already live on `ttys001` PID `60453` and
  `ttys005` PID `311`;
- Claude Code harness status and doctor report `ready=false` because generated adapters
  `claude-memory-session-command`, `claude-end-session-command`, and
  `claude-stop-nudge-hook` have drifted from current policy;
- the same harness checks preserve known user-owned snippet, extra legacy permission, split
  settings, and effective `/hooks` caveats;
- Engram daemon status reports port `8765`, PID `75180`, spawned by
  `/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`;
- obligations doctor for project `engram` returns no open obligations and no warnings;
- telemetry windows are recorded but current confidence gates fail: the 20-trace window has 15%
  feedback coverage, and the 50-trace window has 22% feedback coverage.

No native Claude session was launched. No prompt was sent. No `/hooks` command was run. No process
signal was sent. No harness/settings/adapters were changed.

## Future Execution Contract

This packet authorizes only a future single-slice execution contract under fresh evidence. The
future run must re-run read-only preflight immediately before launch.

### Hard Preflight Assertions

The future execution must stop before launch unless all of these are true:

| Assertion | Required Evidence |
| --- | --- |
| CLI path | `which claude` or equivalent resolves to `/Users/yuval.meiri/.local/bin/claude` |
| Symlink target | `/Users/yuval.meiri/.local/bin/claude` resolves to `/Users/yuval.meiri/.local/share/claude/versions/2.1.168` |
| Version | `/Users/yuval.meiri/.local/bin/claude --version` returns `2.1.168 (Claude Code)` |
| Target hash | resolved target hash remains `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78` |
| Worktree | tracked git diff is empty; only known user-owned root `AGENTS.md` may be untracked |
| Branch | branch tracks the intended phase-1 branch; no pull/merge/rebase is required before the native run |
| Harness | Claude Code harness status and doctor report `ready=true`; warnings are recorded and unchanged or explained |
| Daemon | Engram daemon is running and healthy |
| Obligations | obligations doctor returns no open obligations that change the native-Claude scope |
| Telemetry | both 20-trace and 50-trace telemetry windows are recorded; failure is evidence to report, not permission to broaden |
| Processes | existing native Claude or Claude-family processes are listed; if any process would make attribution ambiguous, stop before launch |
| Monitoring | user/project Claude path hashes and inventories are captured before launch |

Any path, version, hash, harness, daemon, obligations, process, or worktree mismatch is a hard stop.
Do not adapt the packet during execution.

### Single Native PTY Session

If preflight passes, the future execution may launch exactly one PTY session:

```text
/Users/yuval.meiri/.local/bin/claude
```

from:

```text
/Users/yuval.meiri/projects/engram
```

It may send exactly one natural-language prompt:

```text
Read-only Engram harness validation. Use only the Engram MCP orient tool with project "engram", cwd "/Users/yuval.meiri/projects/engram", intent "plan_work", response_shape "lean", and agent "claude_code". Do not edit files or run shell commands. Then answer with exactly two lines:
ORIENT_TRACE_ID: <trace_id>
TOP_ITEM_ID: <first brain_loop top_items id, or none>
```

Then it must attempt one EOF/Ctrl-D exit. If EOF does not exit within the chosen timeout, it may
send exactly one process-group SIGINT equivalent to Ctrl-C:

```text
kill -INT -<PGID>
```

If the process remains live after that one SIGINT, stop and report. Do not retry, send another
prompt, run a slash command, use Claude Bridge, run `claude -p`, inspect `/hooks`, force-kill, or
modify files.

### Postflight Evidence

The future execution must re-run the same read-only snapshots and compare:

- git status and tracked diff;
- branch/upstream state;
- Claude binary path, target, version, and hash;
- harness status and doctor;
- daemon status;
- process snapshot;
- monitored user/project Claude path hashes and inventories;
- Memory OS cursor and `changes_since`;
- telemetry windows;
- obligations doctor.

Each delta must be attributed as expected from the native session, caused but unexpected,
ambient/unattributed, or unclear.

## Success Criteria

A future T312 execution can support only the prompt-bearing native-Claude MCP-`orient` subclaim if:

- preflight matches the `2.1.168` contract exactly;
- Claude Code harness status and doctor are `ready=true` before launch;
- the transcript shows one native Claude session, one prompt, one EOF, and at most one SIGINT;
- startup Engram guidance is visible or its absence is explicitly reported;
- native Claude uses the live Engram MCP `orient` path;
- the output includes a parseable `ORIENT_TRACE_ID` and `TOP_ITEM_ID`;
- the trace is retrievable through telemetry or Memory OS tooling;
- postflight shows no unexpected tracked git diff, monitored config drift, obligation drift,
  daemon drift, or orphan process;
- a committed result report, indexed docs, current-plan memory, handoff update, telemetry feedback,
  and vault refresh are completed.

If any criterion is missing or ambiguous, report the result as failed or inconclusive.

## Non-Claims

T312 does not prove native Claude behavior. It records a docs-only successor packet and a current
hard-stop state for Claude Code `2.1.168`.

T312 does not claim:

- Claude Code `2.1.168` is supported or compatible with Engram;
- the observed SHA-256 is a security trust chain or signature verification;
- generated Claude adapters are current;
- Claude Code harness readiness is restored;
- T269 effective-hook visibility;
- T270 live Claude host-label proof;
- clean EOF semantics;
- lifecycle cleanup;
- direct legacy deprecation/deletion;
- broad cross-harness parity;
- production/GA Brain Harness completion;
- any change to the supported local/Codex beta scope.

Those remain separate gates unless separately packeted and executed.
