# Brain Harness T129 Claude Session-End Handoff Root Cause

Date: 2026-06-02
Status: Completed as docs-only root-cause packet
Scope: Read-only source, harness-doctor, handoff, AI Council, and Claude Bridge critique attempt

T129 investigated the T128 handoff-continuity failure without changing code, installed hooks,
Claude settings, adapters, ranking, `orient`, public MCP parameters, schema/storage/index behavior,
document-index behavior, migration state, lifecycle state, or candidate files.

The root cause is a write-policy mismatch at the Claude Code session-end path. The Engram harness
service writes `SessionEnd` handoffs only when the event write policy is `durable`, but the
generated Claude session-end shell hook defaults a missing hook-input `write_policy` to `durable`.
That means a surrounding bridge task can be read-only while the actual session-end hook still sends
`write_policy=durable` to the daemon and writes a low-information rolling handoff.

## Research Question

Why did Claude Code session-end automation write stub handoffs during a bridge job launched with
`write=false`, and what is the smallest safe next slice under the current approval gates?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The bridge read-only flag does not reach the generated session-end hook input, so the hook's missing-policy default to `durable` authorizes the daemon-side handoff write. |
| Null | The write came from unrelated handoff or memory lifecycle code, so changing the session-end hook policy would not address the observed failure. |
| Simpler alternative | Treat the T128 failure as only documentation noise and keep relying on current-plan memory plus Codex handoff restoration. |
| Failure | The investigation crosses into unapproved hook/settings/adapter writes, `orient` changes, ranking changes, migration work, lifecycle mutation, or candidate inspection. |

## Measurement

Read-only evidence collected on 2026-06-02:

- Startup `orient` trace `019e8793-4814-7930-879e-46926698d793` surfaced the harness-write gate,
  M6 gate, and current-plan memory `019e8791-70dc-7361-a187-0d19cd3647c1`.
- Direct current-plan search trace `019e8793-6cae-7183-bfa2-82b887138ec1` returned current-plan
  memory `019e8791-70dc-7361-a187-0d19cd3647c1` first, but still returned stale
  repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` below it.
- `engram-index/src/harness.rs` computes `write_durable` solely from
  `event.write_policy == "durable"` at lines 466-470.
- The same file writes the `SessionEnd` rolling handoff only under `if write_durable` at
  lines 659-684.
- `engram-index/src/harness.rs` generates MCP hook handlers with explicit
  `write_policy: "durable"` for normal MCP hook events at lines 1858-1888.
- `engram-index/src/harness.rs` generates the command-style `claude_session_end_hook` with
  `WRITE_POLICY=$(... '.write_policy // "durable"')` at line 2085, then passes that value to
  `harness(action="hook_event", hook_event_name="SessionEnd")` at lines 2121-2127.
- `engram-index/src/handoff.rs` treats every non-dry-run rolling handoff update as a new active
  handoff that supersedes the previous handoff at lines 78-120.
- Live `harness(action="doctor", harness="claude_code", root="/Users/yuval.meiri")` returned
  `ready=false`: required generated hook files are installed, but Claude settings still lack
  required `SessionStart:startup|resume|compact` and `SessionEnd` registrations, and both settings
  files still contain legacy Engram permission drift.
- Live `handoff(action="get", project="engram")` after the T129 Claude Bridge critique timeout
  still returned restored Codex handoff `019e8791-a5de-7461-bcd0-4b9de3cdfb55`, so the isolated
  critique attempt did not create another observed handoff overwrite.
- AI Council recall found prior relevant discipline: keep prepare-handoff changes strict and do
  not treat hook/harness readiness as `orient` approval.
- AI Council broadcast reached 3/3 model agreement that the smallest safe next slice is a
  docs-only root-cause and approval packet. The models agreed that changing the generated
  `SessionEnd` default away from `durable` is likely the first eventual fix, but it is a hook
  behavior change and therefore needs explicit approval.
- Claude Bridge critique through the isolated harness timed out after 120 seconds. No Claude result
  was used as evidence.

## Root Cause

The observed T128 write is explained by a local policy boundary mismatch:

1. The bridge task can be configured `write=false`, but that setting is outside the generated
   Claude session-end hook input seen by `engram-session-end.sh`.
2. The generated session-end hook defaults missing `write_policy` to `durable`.
3. The daemon-side harness tool receives `write_policy=durable`.
4. `handle_hook_event` correctly treats `durable` as authorization to write.
5. The rolling handoff service writes a new active handoff and marks the previous handoff as
   superseded.

This is not an `orient` retrieval failure. It is also not proof that Claude Code harness readiness
is healthy: doctor still reports `ready=false`, and the observed write path shows write-path risk
rather than readiness.

## Completion Matrix Delta

| Area | T129 state | Evidence |
| --- | --- | --- |
| Current-plan retrieval | Validated for tested Codex and Claude Code startup prompts | T127/T128 traces return active current-plan memory first for lean `orient` and broad continuation search. |
| Rolling handoff continuity | Risky cross-harness | T128 showed Claude session-end stubs can supersede rich handoffs; T129 explains the likely write-policy path. |
| Claude Code harness readiness | Still not ready | Doctor reports installed generated files but missing `SessionStart` and `SessionEnd` settings registrations. |
| M6 migration | Still gated | T125 quarantine inspection and all status/prioritize/apply/rerun decisions remain unapproved. |
| `orient` hot path | Preserved | T129 made no retrieval, ranking, payload, or hot-path change. |
| Proposed fix | Not implemented | Changing session-end hook default or API guards requires exact approval. |

## Recommended Approval Packet

Preferred next implementation slice after explicit approval:

`Approve T130: change the generated Claude Code SessionEnd hook template so missing hook input write_policy defaults to non-durable/nudge instead of durable; add focused tests proving missing write_policy does not write a handoff, explicit durable still writes, and installed/rendered hook output matches the new default; do not edit installed user hooks or settings, do not run harness install, do not change public MCP parameters, schema/storage/index behavior, ranking, orient, migration, or lifecycle state.`

Secondary follow-up after T130, if needed:

`Approve T131: improve read-only Claude harness doctor/status diagnostics for installed hook files with missing settings registrations; do not install adapters, edit settings, adopt user-owned files, change hook behavior, or change memory/ranking/migration/orient behavior.`

## Validation

This is a docs-only evidence slice. Validation is limited to:

- read-only source inspection with file/line evidence;
- read-only Engram `harness(doctor)` and `handoff(get)`;
- AI Council recall and broadcast;
- Claude Bridge isolated critique attempt, which timed out and produced no result;
- exact-source documentation updates;
- `git diff --check` before commit.

No T125 quarantine candidate files, migration status/prioritize/apply/rerun, candidate decisions,
document indexing, lifecycle writes, hook/settings/adapter writes, user-owned file adoption,
ranking changes, `orient` expansion, public MCP/schema/storage/index behavior changes, or
document-index behavior changes were run.
