# T173 Telemetry And Stale Approval Follow-Through

Date: 2026-06-03

## Scope

Docs-only and telemetry-only follow-through after the user re-approved T125 and T154.

This slice did not rerun T125, rerun T154, execute native Claude, execute Claude Bridge, run
Claude `/hooks`, run prompt-bearing Claude, run harness install, edit settings/hooks/adapters,
run M6 status/prioritize/apply/rerun, make candidate decisions, inspect additional quarantine or
review files, mutate lifecycle state, run `lint apply_safe`, change ranking or `orient`, change
public MCP/schema/storage/index/document-index behavior, delete anything, roll back, force-kill,
reinstall an old binary, or touch user-owned files.

The only Engram writes were telemetry feedback records for assessable retrieval traces. The only
repo writes are this report and the matching implementation-plan note.

## Research Question

After committed T125/T154 execution and the T171 telemetry confidence regression, can Engram avoid
duplicating already-completed approved work while recording honest retrieval feedback and restoring
the current 50-trace telemetry confidence gate?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | T125 and T154 are already complete in committed result docs, and feedback on this turn's assessable retrievals can move the current telemetry window back above threshold without hiding retrieval noise. | Supported. T169/T170 are committed, and the post-feedback gate passes at 60% coverage. |
| Null | The new approvals require rerunning T125/T154 or the telemetry window remains below threshold. | Not supported. Rerun would duplicate prior committed execution; post-feedback eval passes. |
| Simpler alternative | Treat the duplicate approvals as final status only and skip telemetry follow-through. | Rejected because T171 made telemetry confidence an explicit remaining gate and current traces were assessable. |
| Failure | Feedback is artificial, masks missing-context retrieval failures, or is misread as M6/native-Claude approval. | Not observed. Feedback included missing-context/noise records, and this report preserves remaining gates. |

## Evidence

Startup and direct retrieval:

- Lean `orient` trace `019e8dab-ef0f-7b81-8560-b57491d3d440` surfaced the harness/hook gate,
  M6 gate, T172 next gate, user design preference, and research-method rule.
- Direct current-plan search trace `019e8dac-49aa-7351-94f0-f579bfe72b5e` found T154/T158 packet
  evidence and T172, but did not surface the completed T169/T170 result reports.
- Direct implementation-plan search trace `019e8dac-4b15-7982-89ae-c83d4dd44517` similarly
  surfaced packet evidence and old handoffs, requiring repo docs for authoritative completed
  status.
- Direct preference search trace `019e8dac-4bc6-7272-b135-da437a18f604` returned the reviewed
  Ousterhout/evidence-over-confidence preference.
- Direct recent-risk search trace `019e8dac-4c7c-74c2-944d-2948dc596382` was noisy, ranking stale
  rolling handoffs above current execution state.
- AI Council `recall_decision` for this exact T125/T154/M6/native-Claude scope returned no prior
  matching consultation results. No new model consultation was needed for this docs/telemetry
  follow-through.

Repo and docs evidence:

- Git log shows `3dfc23d Record T125 quarantine inspection`.
- Git log shows `ebe835d Record T154 native Claude smoke`.
- `docs/BRAIN_HARNESS_T169_T125_QUARANTINE_INSPECTION_REPORT_2026-06-03.md` records T125 complete
  for quarantine candidate files 0010-0011 and explicitly forbids treating that as candidate
  decision or migration apply evidence.
- `docs/BRAIN_HARNESS_T170_T154_NATIVE_CLAUDE_SMOKE_RESULT_2026-06-03.md` records T154 complete
  for only `claude --version` and `claude --help`, with no proof of interactive, prompt-bearing,
  `/hooks`, or missing `write_policy` behavior.
- `docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md`
  remains the next native-Claude approval gate and was not approved by the duplicate T125/T154
  wording.

## Telemetry Result

Baseline before this turn's feedback:

| Metric | Value |
| --- | --- |
| Generated at | `2026-06-03T13:30:01.167955Z` |
| Trace window | 50 |
| Feedback traces | 24 |
| Feedback coverage | 48% |
| Feedback-bearing intents | 5 |
| Bad memory used | 0 |
| Gate | Failed: needed at least 50% feedback coverage |

Feedback submitted for this turn:

| Trace | Assessment |
| --- | --- |
| `019e8dab-ef0f-7b81-8560-b57491d3d440` | Useful startup orient; used five relevant rule/decision/preference items. |
| `019e8dac-49aa-7351-94f0-f579bfe72b5e` | Useful but noisy current-plan search; missing completed T169/T170 result reports. |
| `019e8dac-4a62-7a71-a7f1-1b7af4aedc07` | Useful for stable architecture context; missing latest execution-state docs. |
| `019e8dac-4b15-7982-89ae-c83d4dd44517` | Useful for locating gate docs; noisy old handoffs and missing completed-state context. |
| `019e8dac-4bc6-7272-b135-da437a18f604` | Clean reviewed preference retrieval. |
| `019e8dac-4c7c-74c2-944d-2948dc596382` | Noisy recent-risk retrieval; file/docs evidence was required. |

Post-feedback eval:

| Metric | Value |
| --- | --- |
| Generated at | `2026-06-03T13:30:49.377492Z` |
| Trace window | 50 |
| Feedback traces | 30 |
| Feedback coverage | 60% |
| Feedback-bearing intents | 5 |
| Bad memory used | 0 |
| Missing-context count | 8 |
| Gate | Passed |

## Completion Matrix Delta

| Area | State After T173 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T125 quarantine inspection | Already complete; not rerun | T169 report; commit `3dfc23d` | Candidate decisions and M6 status/dry-run/apply remain separate |
| T154 native Claude non-session smoke | Already complete; not rerun | T170 report; commit `ebe835d` | Does not prove interactive, prompt-bearing, `/hooks`, or missing-policy behavior |
| Telemetry confidence | Current sliding window passes | 30/50 feedback traces, 60% coverage, five intents, zero bad-memory use | Sliding-window weak signal; keep feedback discipline |
| Retrieval quality around stale approvals | Mixed/noisy | Current searches missed completed result reports and required repo docs | Preserve missing-context feedback; do not broaden ranking without a focused approved slice |
| Effective Claude hooks | Still gated | T172 packet exists and was not approved by T125/T154 wording | Requires exact T172 approval before one native `/hooks` PTY session |
| M6 migration completion | Still gated | T169 closes inspection only | Needs reviewed decisions, dry-run/apply plan, rollback evidence, and explicit approval |

## Decision

The latest T125 and T154 approvals are stale relative to repo state: both approved tasks were
already executed and committed. Rerunning either would add risk without evidence gain.

T173 restores the current telemetry confidence gate, but this is not a production-completion proof
and not migration readiness. The full Brain Harness goal remains incomplete until the remaining
approval-controlled gates are closed or explicitly deferred with evidence.

The next product-moving gate remains the exact T172 approval phrase for one bounded native Claude
effective-hook validation session. M6 candidate decisions and migration dry-run/apply planning are
still separate gates.
