# T162 Telemetry Coverage Follow-Through

Date: 2026-06-03
Status: complete as docs-only / telemetry-only evidence

## Scope

T162 records a non-gated telemetry follow-through after T161 showed real-session feedback coverage
below the confidence gate. It does not change source behavior, installed harness files, lifecycle
state, ranking, `orient`, public MCP parameters, schema/storage/index behavior, document-index
behavior, M6/migration/quarantine state, native Claude execution, Claude Bridge execution,
deletion, rollback, force-kill, old-binary install state, or user-owned files.

## Research Framing

Question: can non-gated telemetry feedback follow-through close the current real-session eval
confidence gate without crossing exact approval gates?

Preferred hypothesis: feedback on freshly assessed startup and approval-gate retrieval traces will
move the current 50-trace sample to a passing confidence gate and preserve retrieval failures as
missing-context evidence.

Null hypothesis: coverage remains below threshold, or feedback breadth remains below the required
three intents.

Simpler alternative: stop at the T161 audit and wait for approval-gated work.

Failure hypothesis: treating a telemetry threshold pass as migration approval, or hiding that exact
approval-packet searches are noisy.

Measurement was defined before implementation as the Memory OS real-session eval gate: at least
20 traces, at least 10 feedback records, feedback coverage at least 50%, feedback across at least
three intents, and at least one outcome feedback record.

## Evidence

Initial eval at `2026-06-03T11:04:30.824392Z` had 50 traces, 17 feedback records, 34% coverage,
two intents with feedback, and `confidence_gate.passed=false`.

Six feedback records were submitted for the fresh startup traces:

- Orient trace `019e8d27-3ee6-72b1-8211-e24f8efa142f`.
- Current-plan search trace `019e8d27-7452-75d0-b52a-a4b25521b473`.
- Architecture/completion search trace `019e8d27-7513-7e21-ba23-2c65a373fcb2`.
- Memory OS / gate search trace `019e8d27-75d0-7a72-94b5-c1e6fb3ff660`.
- User design philosophy search trace `019e8d27-7690-7411-b99d-181341d7938f`.
- Recent failures / risks search trace `019e8d27-774b-7ba1-889e-3b1223d6a82f`.

Intermediate eval at `2026-06-03T11:05:00.841898Z` had 23 feedback records, 46% coverage, three
intents with feedback, and still failed only the coverage threshold.

Four verify-decision searches were then run for remaining exact approval gates and received
missing-context feedback because they did not reliably surface the packet docs:

- T154 native Claude gate trace `019e8d28-ace5-7eb1-a316-90eca6773b86`.
- T160 wrong-scope lifecycle gate trace `019e8d28-adbe-7bb3-a840-776e26f27b60`.
- T157 stale current-plan lifecycle gate trace `019e8d28-ae87-7600-a384-a99fb9e2389b`.
- T125/T158 M6 quarantine inspection gate trace `019e8d28-af74-7612-b781-74816acb8f25`.

Final eval at `2026-06-03T11:05:36.690308Z` had 50 traces, 25 feedback records, 50% coverage,
four intents with feedback, and `confidence_gate.passed=true`. A fresh post-compaction eval at
`2026-06-03T11:07:33.347585Z` still passed with the same 25 of 50 feedback coverage. The gate
still reports `requires_user_approval=true`.

The verify-decision intent is noisy in this sample: average usefulness 2.25, average noise 4.75,
and four missing-context records. `rg` against the repo docs, not retrieval ranking, remains the
authority for exact approval phrases until a separately approved document visibility or ranking
slice exists.

## Completion Matrix Delta

| Area | Status after T162 | Evidence | Residual risk |
| --- | --- | --- | --- |
| Telemetry evidence quality | Current 50-trace sample passes | 25/50 feedback coverage, four intents, gate passed | Threshold pass is fragile in a sliding window |
| Current-plan orientation | Healthy for active continuation | T161/T162 orient traces return T161 first | Stale current-plan memory still appears lower until T157 |
| Exact approval packet retrieval | Risky / noisy | Four verify-decision missing-context feedback records | Exact phrases must be read from packet files |
| T135 harness repair | Done / consumed | T152 implementation and T161 duplicate audit | Duplicate approvals must not reopen harness writes |
| Native Claude and effective hooks | Missing / exact-gated | T154 packet | No native Claude process without exact approval |
| Lifecycle cleanup | Missing / exact-gated | T157, T159, T160 packets | No archive or `lint apply_safe` without exact approval |
| M6 migration/quarantine | High-risk / exact-gated | T158/T125 packet | No quarantine inspection, candidate decision, apply, cleanup, or deletion without approval |

## Decision

T162 closes the immediate telemetry coverage gap in the current 50-trace window and records the
approval-packet retrieval noise as explicit feedback. It does not complete the Brain Harness goal,
does not authorize M6 write-apply or quarantine inspection, and does not reduce the exact approval
requirements for T154, T157, T159, T160, or T125/T158.
