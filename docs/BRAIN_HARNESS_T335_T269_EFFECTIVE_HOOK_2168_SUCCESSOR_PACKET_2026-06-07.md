# Brain Harness T335 T269 Effective-Hook 2.1.168 Successor Packet

Date: 2026-06-07
Status: docs-only/default-deny successor packet. Not executed.

## Scope

T335 refreshes the T269 effective-hook visibility packet for the currently observed Claude Code
runtime baseline:

- CLI path: `/Users/yuval.meiri/.local/bin/claude`
- symlink target: `/Users/yuval.meiri/.local/share/claude/versions/2.1.168`
- version: `2.1.168 (Claude Code)`
- SHA-256:
  `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`

T335 is docs-only. It does not launch native Claude, attach to existing native Claude processes,
send `/hooks`, send prompts, signal processes, use Claude Bridge, run `claude -p`, edit hooks,
settings, snippets, adapters, or user-owned files, run harness install, mutate lifecycle state, run
M6/migration/quarantine actions, initialize or compile the canonical vault, change ranking or
`orient`, change public MCP/schema/storage/index/document-index behavior, publish branches, merge,
tag, release, delete data, or roll back.

T335 supersedes T269 only for the runtime target/version/hash baseline and future execution
contract. It does not supersede T269 as execution evidence, because neither T269 nor T335 has
executed a native `/hooks` run. T334's attribution hard-stop remains binding.

## Research Question

Can Engram make the effective-hook visibility gate executable for the observed Claude Code
`2.1.168` runtime without launching native Claude, weakening the T269 observation contract,
combining T312/T270, or claiming effective-hook visibility before a transcript proves it?

## Hypotheses

| Type | Hypothesis | Expected Evidence |
| --- | --- | --- |
| Preferred | A docs-only successor can carry T269's strict `/hooks` observation contract forward to the `2.1.168` runtime while preserving the T334 no-launch attribution blocker. | Packet records the new baseline, hard preflight assertions, no-launch state, and explicit non-claims. |
| Null | T269 remains executable as-is despite the `2.1.161` to `2.1.168` drift. | Rejected because T269's execution section hard-stops on drift from the `2.1.161` baseline unless fresh exact drift approval names the drift. |
| Simpler alternative | Leave T269 stale and defer effective-hook visibility. | Safe, but future sessions may try to execute a known-stale packet or waste another preflight cycle. |
| Failure | The successor implies effective-hook visibility is validated, launches native Claude, accepts ambiguous `/hooks` output, or silently authorizes T312/T270. | Prevented by scope exclusions, hard stops, and non-claims. |

## Consultation Synthesis

AI Council recall recovered prior T269/T312/T270 decisions:

- prepare docs-only successor packets before risky native execution;
- keep prompt-bearing validation, `/hooks`, and host-label proof as separate scopes unless exact
  wording explicitly combines them;
- stop before launch on version, hash, process attribution, harness, daemon, obligations, telemetry,
  or worktree drift;
- do not treat model consensus, static harness readiness, or startup guidance as runtime proof.

A fresh three-model AI Council broadcast on 2026-06-07 agreed on the smallest safe slice with one
important wording caveat:

- Claude Sonnet and Gemini recommended a docs-only T335 successor packet for the `2.1.168`
  effective-hook baseline.
- GPT recommended framing the slice as an operational successor and attribution-blocker packet, not
  as effective-hook progress.
- The synthesis adopted both points: T335 refreshes the stale T269 runtime contract but makes no
  native effective-hook visibility claim.

## Evidence Inputs

T334 read-only preflight observed:

- branch `yuval.meiri/memory-os-phase1` synced with upstream at `HEAD...@{u}` = `0 0`;
- tracked worktree clean, with only root `AGENTS.md` untracked and user-owned;
- PR #3 open/draft at head `a276a7f53735fb84425048215418acf7b92684dc`;
- hosted CI run `27099185124` failed all five jobs before workflow steps with `steps: []`,
  continuing the external GitHub Actions billing/spending-limit gate;
- Claude path, target, version, and SHA-256 matched the `2.1.168` baseline above;
- installed Claude Code `harness status` and `harness doctor` both reported `ready=true`;
- snippet-only install dry-run reported `planned=[]`;
- daemon status reported port `8765`, PID `75180`, spawned by
  `/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`;
- obligations doctor returned no open obligations and no warnings;
- canonical vault status was count-aligned after closeout at 2,544 generated files, zero user
  files, and 2,544 expected generated files;
- `real_session_eval(project=engram, limit=50)` remained below the confidence gate because
  feedback coverage was about `0.32`;
- native Claude CLI processes were already live on `ttys001` PID `60453` and `ttys005` PID `311`,
  making new-session attribution ambiguous.

T334 did not launch native Claude, send `/hooks`, send a prompt, signal a process, use Claude
Bridge, or mutate settings/adapters.

## Observation Contract

T335 preserves T269's observation contract. The future execution's only observation channel for
effective-hook visibility is the captured native Claude PTY transcript after one literal `/hooks`
input.

The run can pass only if visible post-`/hooks` output directly shows effective hook configuration
for this repository and includes all of these classes, or an equivalent unambiguous
effective-configuration table:

- `SessionStart`
- `UserPromptSubmit`
- `PostToolUse`
- `PostToolUseFailure`
- `Stop`
- `PreCompact`
- `PostCompact`
- `SessionEnd`

The output must show enough handler detail to distinguish Engram hook registrations from a generic
hook list, such as Engram command paths, the `engram` MCP server/tool handler, or equivalent Claude
Code effective-configuration text.

These are not passing evidence:

- startup guidance alone;
- static file contents alone;
- `harness doctor ready=true`;
- absence of errors;
- an interactive menu that would require another command, cursor movement, selection, or prompt;
- a partial screen without event names;
- a SessionEnd handoff write;
- any trace or output produced by Codex, Claude Bridge, simulated Claude env, or a shell-only
  command.

If `/hooks` produces no visible configuration, only a menu, only startup guidance, or any output
that cannot be tied to the effective merged configuration, the future execution must record a
failed or inconclusive measurement and stop after cleanup/postflight.

## Proposed Approval Wording

Use this exact approval if the next slice should execute this successor packet:

```text
Approve T335: execute the native Claude effective-hook visibility revalidation from docs/BRAIN_HARNESS_T335_T269_EFFECTIVE_HOOK_2168_SUCCESSOR_PACKET_2026-06-07.md. I understand this may trigger native Claude lifecycle hook side effects, including SessionEnd handoff writes. Run exactly the packet's fresh preflight, one native Claude PTY session, one literal `/hooks` command, one EOF exit attempt, pre-authorized process-group SIGINT cleanup if EOF or the session hangs, and postflight comparisons. Treat missing, menu-only, startup-only, Codex-labeled, simulated, or inconclusive `/hooks` output as a failed measurement, not permission to send more input. Do not send natural-language prompts, run T312/T255 prompt-bearing validation, run T270 host-label validation, edit hooks/settings/adapters, run harness install, use adopt_user_owned, mutate lifecycle outside observed hook side effects, run M6/migration/quarantine, initialize or compile the canonical vault, change ranking/orient/public MCP/schema/storage/index/document-index behavior, publish branches, merge, tag, release, delete, rollback, force-kill beyond the packet, reinstall old binaries, or touch user-owned files.
```

Shorter approvals, generic continuation, or approvals naming only T312, T269, T270, M6, lifecycle,
or beta release work are not authorization to execute T335.

## Future Execution Contract

The future run must re-run read-only preflight immediately before launch. It must stop before
launch unless all hard preflight assertions pass.

### Hard Preflight Assertions

| Assertion | Required Evidence |
| --- | --- |
| CLI path | `which claude` or equivalent resolves to `/Users/yuval.meiri/.local/bin/claude` |
| Symlink target | `/Users/yuval.meiri/.local/bin/claude` resolves to `/Users/yuval.meiri/.local/share/claude/versions/2.1.168` |
| Version | `/Users/yuval.meiri/.local/bin/claude --version` returns `2.1.168 (Claude Code)` |
| Target hash | resolved target hash remains `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78` |
| Worktree | tracked git diff is empty; only known user-owned root `AGENTS.md` may be untracked |
| Branch | branch tracks the intended phase-1 branch; no pull, merge, or rebase is required before the native run |
| Harness | Claude Code harness status and doctor report `ready=true`; warnings are recorded and unchanged or explained |
| Daemon | Engram daemon is running and healthy |
| Obligations | obligations doctor returns no open obligations that change the native-Claude scope |
| Telemetry | 20-trace and 50-trace telemetry windows are recorded; confidence failure is evidence to report, not permission to broaden scope |
| Processes | existing native Claude or Claude-family processes are listed; if any process would make attribution ambiguous, stop before launch |
| Monitoring | user/project Claude path hashes and inventories are captured before launch |

Any path, version, hash, harness, daemon, obligations, process, or worktree mismatch is a hard stop.
Do not adapt the packet during execution.

### Single Native PTY Session

If every preflight assertion passes, the future execution may launch exactly one PTY session:

```text
/Users/yuval.meiri/.local/bin/claude
```

from:

```text
/Users/yuval.meiri/projects/engram
```

It may send exactly one input line:

```text
/hooks
```

Then it must capture visible output until the observation contract is satisfied, the output is
clearly inconclusive, or a bounded wait expires. It must send exactly one EOF/Ctrl-D exit attempt.
If EOF does not exit within the chosen timeout, it may send exactly one process-group `SIGINT` to
the foreground process group recorded for this PTY:

```text
kill -INT -<PGID>
```

If the process remains live after that one process-group `SIGINT`, stop and report. Do not send
another signal, `SIGTERM`, `SIGKILL`, second EOF, second Ctrl-C, another slash command, a
natural-language prompt, or any follow-up command.

### Postflight Evidence

The future execution must re-run the same read-only snapshots and compare:

- git status, tracked diff, and branch/upstream state;
- Claude binary path, target, version, and hash;
- harness status and doctor;
- daemon status;
- process snapshot;
- monitored user/project Claude path hashes and inventories;
- Memory OS cursor and `changes_since`;
- telemetry windows;
- obligations doctor.

Each delta must be attributed as expected from the exact native session, caused by the exact native
session but unexpected, ambient/unattributed, or unclear.

## Success Criteria

A future T335 execution can support only the effective-hook visibility subclaim if:

- preflight matches the `2.1.168` contract exactly;
- there are no attribution-confusing native Claude processes before launch;
- the transcript shows one native Claude session, one `/hooks` input, one EOF attempt, and at most
  one process-group `SIGINT`;
- visible output satisfies the observation contract;
- postflight shows no unexpected tracked git diff, monitored config drift, obligation drift,
  daemon drift, or orphan native process;
- a committed result report, indexed docs, current-plan memory, handoff update, telemetry feedback,
  and vault refresh are completed.

If any criterion is missing or ambiguous, report the result as failed or inconclusive.

## Non-Claims

T335 does not prove native Claude behavior. It records a docs-only successor packet for the
effective-hook visibility gate under the observed Claude Code `2.1.168` target.

T335 does not claim:

- effective-hook visibility is validated;
- `/hooks` will produce a useful effective-configuration report;
- Claude Code `2.1.168` is behaviorally equivalent to `2.1.161`;
- native Claude prompt-bearing MCP behavior;
- T312/T255 execution;
- T270 live Claude host-label proof;
- clean EOF semantics;
- lifecycle cleanup;
- direct legacy deprecation/deletion;
- M6 write-apply confidence;
- hosted CI success;
- beta release-owner approval;
- production/GA Brain Harness completion.

Those remain separate gates unless separately packeted and executed.
