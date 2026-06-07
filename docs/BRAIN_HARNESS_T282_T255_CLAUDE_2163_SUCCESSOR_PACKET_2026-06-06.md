# Brain Harness T282 T255 Claude 2.1.163 Successor Packet - 2026-06-06

## Scope

T282 prepares a successor execution packet for the T255 native Claude prompt-bearing parity gate
after T281 proved that the original T255 packet is stale under the current installed Claude Code
version.

T282 is docs-only. It does not launch native Claude, send prompts, run `/hooks`, edit
hooks/settings/adapters, run harness install, mutate lifecycle state, run M6, change ranking or
`orient`, publish branches, delete data, roll back, send process signals, or touch user-owned
files.

## Research Question

Can Engram safely rebaseline the T255 prompt-bearing native Claude validation packet for the current
Claude Code `2.1.163` target without widening scope, combining host-label/effective-hook gates, or
claiming behavioral equivalence before a transcript proves it?

## Hypotheses

| Type | Hypothesis | Expected Evidence |
| --- | --- | --- |
| Preferred | A docs-only successor packet can make the current Claude `2.1.163` target executable for one future prompt-bearing validation while preserving T255's evidence boundaries. | Packet records fresh T281 drift evidence, new hard preflight assertions, unchanged prompt scope, and explicit non-claims. |
| Null | T255 can be reused as-is despite version drift. | Rejected by T281 because T255 hard-stops on any target/version other than `2.1.161`. |
| Simpler alternative | Explicitly defer native Claude prompt-bearing validation. | Safe but does not move the native-Claude gate toward closure. |
| Failure | The successor silently expands into T269/T270, claims `2.1.163` equivalence without evidence, or lets future execution proceed after another preflight drift. | Prevented by scope exclusions, hard stops, and postflight claim limits. |

## Consultation Synthesis

AI Council recall surfaced prior native-Claude/effective-hook and host-label boundary discussions.
A fresh three-model broadcast on 2026-06-06 agreed on the main path:

- prepare a docs-only successor packet first;
- do not execute native Claude in the same slice;
- do not combine prompt-bearing validation with T270 host-label proof;
- keep T269 effective-hook visibility separate;
- define pass/fail evidence before a future launch.

One model suggested treating `2.1.163` as a patch-level equivalent to `2.1.161`. T282 rejects that
as an overclaim. The successor packet may use the current target as a fresh baseline, but it does
not assert behavioral equivalence until a bounded execution transcript and postflight evidence
support that narrower claim.

## T281 Evidence Inputs

T281 observed:

- `/Users/yuval.meiri/.local/bin/claude` is a symlink to
  `/Users/yuval.meiri/.local/share/claude/versions/2.1.163`;
- `claude --version` returns `2.1.163 (Claude Code)`;
- the resolved target hash is
  `c7582e926e8fe459dbd9743f19ccb75500e3b455c722902d1aa587a74fb1fa7c`;
- Claude harness status and doctor both reported `ready=true`, with known split-settings,
  user-owned snippet, extra-permission, and soft lifecycle-compliance warnings;
- Engram daemon was running on port `8765`, PID `25189`;
- obligations doctor was clean;
- the 20-trace telemetry gate failed because feedback was concentrated in two intents, while the
  50-trace telemetry gate passed with 60% feedback coverage, five intents, and zero task failures;
- ambient Claude-family processes were visible, but no signal or cleanup action was sent;
- no native Claude prompt-bearing session was launched.

## Future Execution Contract

This packet authorizes only a future single-slice execution contract under the 2026-06-06 standing
authorization. The future run must re-run fresh preflight immediately before launch.

### Hard Preflight Assertions

The future execution must stop before launch unless all of these are true:

| Assertion | Required Evidence |
| --- | --- |
| CLI path | `which claude` or equivalent resolves to `/Users/yuval.meiri/.local/bin/claude` |
| Symlink target | `/Users/yuval.meiri/.local/bin/claude` resolves to `/Users/yuval.meiri/.local/share/claude/versions/2.1.163` |
| Version | `/Users/yuval.meiri/.local/bin/claude --version` returns `2.1.163 (Claude Code)` |
| Target hash | resolved target hash remains `c7582e926e8fe459dbd9743f19ccb75500e3b455c722902d1aa587a74fb1fa7c` |
| Worktree | tracked git diff is empty; only known user-owned root `AGENTS.md` may be untracked |
| Branch | branch tracks `origin/yuval.meiri/memory-os-phase0`; no pull/merge/rebase is required before the native run |
| Harness | Claude Code harness status and doctor report `ready=true`; warnings are recorded and unchanged or explained |
| Daemon | Engram daemon is running and healthy |
| Obligations | obligations doctor returns no open obligations that change the native-Claude scope |
| Telemetry | both 20-trace and 50-trace telemetry windows are recorded; failure of one window is evidence to report, not permission to broaden |
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

The future T282 execution can support the prompt-bearing native-Claude MCP-`orient` subclaim only if:

- preflight matches the `2.1.163` contract exactly;
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

T282 does not prove native Claude behavior. It only prepares a successor packet for the current
Claude target.

Even a future passing execution would not prove:

- T269 effective-hook visibility;
- T270 live Claude host-label proof;
- clean EOF semantics beyond the single observed run;
- lifecycle cleanup;
- direct legacy deprecation/deletion;
- broad cross-harness parity;
- behavioral equivalence between Claude `2.1.161` and `2.1.163`.

Those remain separate gates unless separately packeted and executed.
