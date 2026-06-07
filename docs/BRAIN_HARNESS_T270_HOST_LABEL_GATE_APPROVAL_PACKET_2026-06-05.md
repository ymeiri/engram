# Brain Harness T270 Host External-Session Label Gate

Date: 2026-06-05
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet narrows the remaining host external-session label gate after T263/T265:

- Codex Desktop host labeling is live-validated for the installed runtime.
- Claude Code host labeling has source and installed-runtime support through guarded
  `CLAUDE_CODE_SESSION_ID`, but no live native Claude Code run has proved that stored Engram
  traces receive the expected `claude-code://sessions/{id}` label.
- Gemini CLI host labeling remains deferred because T264 found no documented MCP-subprocess
  session-id environment contract to implement against.

T270 does not execute native Claude, Gemini CLI, Claude Bridge, Gemini Bridge, T255, T269,
`/hooks`, harness install, hook/settings/adapter edits, lifecycle cleanup, Memory OS archive,
`lint apply_safe`, M6/migration/quarantine actions, canonical vault writes, ranking or `orient`
changes, public MCP changes, schema/storage/index changes, document-index behavior changes, branch
publication, deletion, rollback, old-binary reinstall, or user-owned-file adoption.

## Research Question

Can Engram close the host-label completion gate by defining exact live Claude Code proof criteria
and explicitly deferring Gemini host-label fallback until a real host-session contract exists,
without implementing guessed labels or expanding the hot path?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only/default-deny packet can make the remaining host-label evidence gap executable: future native Claude proof is falsifiable, while Gemini remains default-deny until a documented subprocess session contract exists. |
| Null | T255 or T269 already covers host-label validation implicitly, so another packet adds no value. |
| Simpler alternative | Leave the matrix wording unchanged and wait for exact execution of T255/T269. |
| Failure | The packet overclaims host-label completion, treats simulated Claude evidence as live proof, lets T255 approval silently broaden into host-label validation, or invents a Gemini env variable. |

## Evidence Inputs

- T200 added explicit CLI caller support for `--external-session-id` on the relevant CLI paths.
- T217/T221/T223/T230/T242 established and refreshed `ENGRAM_EXTERNAL_SESSION_ID` fallback behavior
  for MCP and CLI trace-producing paths.
- T262 added guarded Codex Desktop `CODEX_THREAD_ID` fallback after explicit labels and
  `ENGRAM_EXTERNAL_SESSION_ID`.
- T263 installed that source and live-validated Codex Desktop `orient` trace labeling plus feedback
  inheritance.
- T264 added guarded Claude Code fallback after explicit labels and `ENGRAM_EXTERNAL_SESSION_ID` and
  before Codex fallback: `CLAUDE_CODE_SESSION_ID` is used only when `CLAUDECODE=1` and the value is a
  short safe token.
- T264 also recorded that checked Gemini CLI docs exposed resume/list-session behavior and
  configuration environment variables but no documented MCP-subprocess session-id environment
  contract.
- T265 installed the T264 source, preserved live Codex labeling, and ran only a simulated
  Claude+inherited-Codex installed-CLI smoke. That smoke does not expose the stored
  `external_session_id`, so it is not live Claude label proof.
- T255 prepares a prompt-bearing native Claude MCP-`orient` packet and T269 prepares an
  effective-hook visibility packet. Neither has been executed, and neither silently authorizes the
  other scope.

## Consultation Synthesis

AI Council recall found no stored decision for this exact T270 packet. A fresh three-model AI
Council broadcast on 2026-06-05 agreed that the next safe repo-local slice is a docs-only,
default-deny host-label packet:

- require live native Claude evidence before claiming Claude Code labels are validated;
- let T270 evidence piggyback on T255 only when exact approval names both scopes;
- record Gemini as deferred/no-contract rather than guessing `GEMINI_SESSION_ID` or any similar
  environment variable;
- avoid claiming that source/runtime support, simulated CLI smoke output, or model consensus is
  proof of native host labeling.

Claude Bridge read-only critique timed out. Treat that timeout as a consultation confound, not as
evidence for or against the packet.

## Current Resolver Contract

The current CLI and MCP resolver order is:

1. Explicit request value or CLI `--external-session-id`.
2. `ENGRAM_EXTERNAL_SESSION_ID`.
3. Guarded Claude Code session ID:
   `CLAUDECODE=1` plus safe `CLAUDE_CODE_SESSION_ID` becomes
   `claude-code://sessions/{id}`.
4. Guarded Codex Desktop thread ID:
   a Codex host marker plus safe `CODEX_THREAD_ID` becomes `codex://threads/{id}`.
5. No external-session label.

This packet does not change that contract.

## Claude Code Live Label Proof Contract

Future native Claude Code host-label validation can pass only if one bounded live native run shows
all of the following:

1. The run is launched from native Claude Code, not simulated CLI env, Claude Bridge, Codex, or a
   shell-only command.
2. The Engram MCP call that creates the trace is made by that native Claude session.
3. The returned `orient` or `search` trace ID is captured.
4. A postflight telemetry read for that exact trace shows
   `external_session_id="claude-code://sessions/{safe_id}"`.
5. The `{safe_id}` value is attributable to the guarded Claude Code path, not to an explicit
   request label, `ENGRAM_EXTERNAL_SESSION_ID`, inherited Codex fallback, or a test fixture.
6. Feedback submitted for that trace without an explicit feedback label inherits the same trace
   label, or the run records why feedback inheritance was not in scope.

These are not passing evidence:

- installed CLI help or source inspection alone;
- the T265 simulated Claude+Codex CLI smoke;
- a trace label supplied explicitly in the MCP request;
- `ENGRAM_EXTERNAL_SESSION_ID` labels;
- Codex labels observed during a Claude-related test;
- `harness(action="doctor")` readiness;
- Claude startup guidance;
- `/hooks` output;
- SessionEnd handoff writes;
- model consensus or documentation without a matching live trace.

If any proof condition is unavailable, ambiguous, or contradicted by postflight telemetry, the
future execution must report a failed or inconclusive measurement and stop without fallback probes.

## Relationship To T255 And T269

T270 may be executed as a separate exact native Claude validation packet, or it may be combined
with T255's prompt-bearing native Claude `orient` run only if the exact approval text names both
T255 and T270 and permits the extra postflight telemetry trace read and feedback inheritance check.

T255 alone does not authorize T270 host-label validation. T270 alone does not authorize T255 prompt
content unless the future packet states the exact prompt and native run boundaries.

T269 is separate. `/hooks` output can help effective-hook visibility but does not prove
external-session labeling. T270 does not authorize `/hooks`.

## Gemini CLI Decision

Gemini host labeling is deferred as default-deny/no-contract:

- no Gemini CLI fallback is implemented in this packet;
- no guessed environment names or URI schemes are introduced;
- explicit request labels and `ENGRAM_EXTERNAL_SESSION_ID` remain the supported Gemini-compatible
  routes when a caller can provide a stable label;
- a future Gemini host-label slice must first identify a documented, stable host-session contract
  for MCP subprocesses or equivalent live runtime evidence.

This is not a claim that Gemini CLI can never support host labels. It is a claim that current
Engram evidence does not justify implementing an inferred Gemini label source.

## Proposed Approval Wording

Use this exact approval if the next slice should execute only the native Claude host-label
validation described here:

```text
Approve T270: execute the native Claude host external-session label validation from docs/BRAIN_HARNESS_T270_HOST_LABEL_GATE_APPROVAL_PACKET_2026-06-05.md. Run one bounded native Claude Code session that creates one Engram trace through the live MCP path, then perform only the packet's postflight telemetry trace and feedback-inheritance checks for that exact trace. Treat missing, Codex-labeled, explicit-label, ENGRAM_EXTERNAL_SESSION_ID-labeled, simulated, or inconclusive evidence as a failed measurement. Do not run T255 unless this approval also names T255, do not run T269 or /hooks, do not edit hooks/settings/adapters, run harness install, use adopt_user_owned, mutate lifecycle, run M6/migration/quarantine, initialize or compile the canonical vault, change ranking/orient/public MCP/schema/storage/index/document-index behavior, publish branches, delete, rollback, reinstall old binaries, force-kill beyond a separately approved native-session cleanup path, or touch user-owned files.
```

Shorter approvals, generic continuation, or approvals naming only T255/T269/T267/M6/lifecycle work
are not authorization to execute T270.

## Completion Impact

T270 does not complete host external-session labeling. It changes the gate from "live
Claude/Gemini labels unproved" to:

- Codex Desktop: live-validated.
- Claude Code: source/runtime path implemented; live native proof packet defined but not executed.
- Gemini CLI: deferred/no-contract until documented host-session evidence exists.

The broader goal still remains incomplete on M6 dispositions/deferral, lifecycle cleanup/deferral,
prompt-bearing native Claude, effective-hook visibility, canonical vault execution, and any remote
publication policy the user wants.
