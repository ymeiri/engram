# Brain Harness T254 Native Claude Harness Parity Gate

Date: 2026-06-04
Status: completed docs-only/static scoping. No native Claude launch, Claude Bridge write, hook or
settings edit, harness install, lifecycle archive, `lint apply_safe`, M6/migration/quarantine
action, ranking/`orient` change, public MCP change, schema/storage/index/document-index behavior
change, runtime refresh, deletion, rollback, force-kill, branch reconciliation, legacy
simplification, or user-owned-file change was executed.

## Scope

T254 scopes the remaining native-Claude/harness parity gap after T253. It uses committed reports,
fresh read-only harness doctor checks, static source inspection, AI Council consultation, Claude
Bridge critique, and one read-only `memory(get)` check on the T197 SessionEnd side-effect item.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

What is the smallest safe next step that advances full native-Claude/harness parity without
overclaiming and without running native Claude or changing hooks/settings under broad workflow
permission?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Static evidence can narrow the remaining parity gap and define the next exact live-validation packet without running native Claude now. | Supported. |
| Null | Static evidence cannot reduce uncertainty; only a new native Claude run can move the gate. | Partially rejected. Static evidence cannot prove behavior, but it does identify which claims remain runtime-only. |
| Simpler alternative | Record a docs-only matrix note and avoid source/consultation work. | Rejected. The next packet would be too easy to overbroaden without source and side-effect evidence. |
| Failure | T254 accidentally treats adapter readiness, T170 metadata smoke, or T253 telemetry as full native-Claude parity. | Avoided. Those remain bounded evidence only. |

## Evidence Re-Read

- T152 reports generated harness adapter readiness after the approved T135 repair:
  `ready=true` for generic, Claude Code, Codex, Gemini CLI, and Cursor. Claude Code still warns
  about a user-owned snippet, split settings files, extra legacy permissions, and soft lifecycle
  compliance.
- T170 ran only native Claude metadata/help commands. It observed no monitored mutation, but it
  does not prove interactive hook behavior.
- T172/T179 launched one approved native Claude PTY. Startup Engram guidance appeared, `/hooks`
  did not produce visible effective-hook configuration output, EOF did not exit, and the run
  stopped without further probing.
- T197 resolved the live native Claude process through one process-group SIGINT and observed a
  SessionEnd handoff side effect.
- T198/T200 show core `external_session_id` storage, pass-through, reporting, and direct CLI caller
  support exist, while live host-session labels still depend on real caller adoption.
- T214 already corrected the matrix from "harnesses are not ready" to "adapter readiness is green,
  behavioral caveats remain."
- T253 keeps M6, lifecycle cleanup, full native-Claude/harness behavior, and branch
  synchronization open.

## Fresh Static Evidence

Fresh read-only `harness(action="doctor")` checks on 2026-06-04 again reported `ready=true` for
generic, Claude Code, Codex, Gemini CLI, and Cursor. Claude Code warnings remained materially the
same: user-owned `claude-settings-snippet`, extra legacy Engram permissions in settings, split
`settings.json` and `settings.local.json`, and soft lifecycle compliance.

Static source inspection found:

- `engram-index/src/harness.rs` computes Claude readiness by checking generated adapter files and
  the presence of required permissions/hooks across `settings.json` and `settings.local.json`.
  This is a configuration-readiness check; it does not prove native Claude will display effective
  hooks or execute every hook path.
- `SessionStart` and `SessionEnd` are rendered as command hooks. Most other Claude hook events are
  rendered as MCP tool hooks with `write_policy="durable"`.
- The rendered `SessionEnd` command hook defaults missing hook input
  `write_policy` to `"nudge"`, while explicit durable hook events still write handoffs.
- Focused tests in `engram-index/src/harness.rs` cover missing `SessionEnd` `write_policy` not
  writing a handoff, explicit durable writing one, and installed/rendered hook output containing
  the nudge default.
- There is no repo-local native-Claude effective-hook parser that can be exercised as a safe
  substitute for Claude Code's own `/hooks` runtime behavior. The observed `/hooks` gap remains
  runtime-owned by Claude Code, not proved by static Engram source.

The T197 SessionEnd side-effect item `019e8ea5-663e-7152-b346-9c5ab7ddc93b` is now
`status=superseded` by a later rolling handoff, with evidence recorded on 2026-06-04. It is still
important evidence that native Claude exit can write Memory OS state, but it is not currently an
active unpacketized lifecycle target.

## Consultation Synthesis

AI Council recall resurfaced the T172 rule: native effective-hook validation is acceptable only as
a default-deny, single-session probe with full pre/post snapshots, hard stops, no retries, and no
cleanup broadening.

A fresh AI Council broadcast converged on a docs/static gap-audit slice. The models agreed not to
launch native Claude now. They also agreed that any future live packet must define exact commands,
side-effect expectations, cleanup authorization, and what cannot be claimed from the result.

Claude Bridge agreed with the docs/static direction and added three constraints:

- do not treat `ready=true` as evidence that native hooks execute;
- make the T197 SessionEnd side effect explicit in future planning;
- pre-authorize the cleanup path in any future live packet instead of discovering recovery steps
  after EOF hangs.

## Completion Matrix Delta

| Area | State After T254 | Remaining Gate |
| --- | --- | --- |
| Generated harness adapter readiness | Green in fresh read-only doctor checks | None for adapter presence/readiness. |
| Native Claude non-session commands | Passed in T170 | Does not prove interactive behavior. |
| Native startup guidance | Observed in T179 | Single-session evidence only. |
| Effective hook visibility | Still open | `/hooks` was inconclusive; needs a new exact live packet or official/runtime evidence. |
| Prompt-bearing native Claude behavior | Still unproved | Needs exact live validation with pre-authorized side-effect handling. |
| `SessionEnd` handoff side effects | Proven possible | Future live packet must measure writes and cleanup; T197 side-effect handoff is superseded, not active. |
| External-session host labels | Core support exists; live host adoption incomplete | Requires real caller/host label adoption, not static docs. |
| Lifecycle cleanup | Still incomplete | T234/T247/T248 remain exact/default-deny; no `lint apply_safe`. |
| M6 migration | Still blocked | T210A/T210B human dispositions or explicit deferral rationale/evidence. |
| Branch synchronization | Still unresolved | Explicit branch reconciliation strategy before pull/rebase/merge. |

## Future Packet Shape

The next native-Claude/harness parity execution should be a new default-deny approval packet, not a
continuation under broad workflow permission. It should measure one live behavior class at a time.

Recommended next live packet, if this gate is prioritized:

```text
Approve T255: execute the native Claude prompt-bearing harness parity validation from docs/BRAIN_HARNESS_T255_NATIVE_CLAUDE_PROMPT_BEARING_PARITY_APPROVAL_PACKET_2026-06-04.md. I understand this may trigger Claude Code lifecycle hook side effects, including Memory OS handoff writes. Run exactly the packet's preflight, one native Claude session, bounded prompt, bounded exit path, pre-authorized process-group SIGINT cleanup if EOF hangs, and postflight comparisons. Do not edit hooks/settings/adapters, run harness install, use adopt_user_owned, mutate lifecycle outside observed hook side effects, run M6/migration/quarantine, change ranking/orient/public MCP/schema/storage/index/document-index behavior, reconcile branches, delete, rollback, force-kill beyond the packet, reinstall old binaries, or touch user-owned files.
```

That packet should specify:

- exact branch state and git preflight required before execution;
- exact Claude binary target/version and monitored hash preflight;
- exact native command and maximum inputs;
- whether a minimal natural-language prompt is allowed and what it must prove;
- exact expected `SessionStart`, prompt/hook, `Stop`, and `SessionEnd` evidence;
- exact telemetry window parameters and trace/feedback scoring expectations;
- exact Memory OS cursor and `changes_since` checks;
- exact process cleanup path if EOF hangs, including whether process-group SIGINT is allowed;
- explicit classification of expected handoff writes versus unexpected lifecycle mutations.

## What T254 Does Not Claim

T254 does not claim full native-Claude/harness parity, effective hook visibility, clean EOF
semantics, prompt-bearing native Claude behavior, host-label adoption, M6 readiness, lifecycle
cleanup, branch synchronization, or stable telemetry confidence beyond the observed windows.
