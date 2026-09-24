# Repeated current-candidate cross-host pilot — complete, strict portable-value gate passed

This document records the fresh reliability experiment requested after the successful
single-repetition Claude unconditional-route diagnostic. The frozen Codex retention interval,
activation, evaluation, and provider-free comparison are now complete. Teaching evidence remains
separated from the final comparative claim below.

## Frozen experiment

- Protocol:
  `evals/native_memory_pilot_v1/protocol-schema-8-unconditional-route-repeated-current.json`,
  SHA-256 `699c1d2bcbe8e03ba8676647fcad8801e648a59dea7d3bee66ecddb3424b2c97`.
- Cases: source-backed prerequisite matched and source-backed prerequisite mismatch.
- Arms: Claude and Codex native memory, Engram plus native memory, and lean Engram.
- Repetitions: three per case and arm, for 36 total lanes.
- Claude Code 2.1.234:
  `08d8700313697cbe730a25420c908a299ce52d56f0eb2cf4fac94cab5109bc57`.
- Codex 0.148.0-alpha.9:
  `6170ff5578170ee9b74ad92bfcff96e6186f41d02b60815a7c2b01ad424c754f`.
- Engram 0.2.3:
  `1bde61b2d999ec55a68e6b3cd9a09be8e4aa23c56bd5bddebe07b8ea9322a322`.
- Effective six-tool schema:
  `b704fac7da773e703dfca6bd3e9b0ffa6dcade6b259e7290315d969de3cec5b3`.
- Effective agent instructions:
  `09be365fafaf2542d137eb69384b1db1b1ac2166c1d6a71a6995cf84a27c3eac`.

The original prepared plan is preserved at
`/private/tmp/engram-native-memory-pilot-v1-unconditional-route-repeated.T5faLt/run-plan.json`,
SHA-256 `fce5c4cd1b4e9849d84edea638638196044a4d65a58404f6373ee6b14241eced`.
It must never be executed again.

## Teaching audit failure and forward recovery

The original teaching phase completed provider turns for lanes 1 and 2. Its subsequent local
incremental audit failed before lane 3 because `is_codex_empty_memory_scaffold` attempted to read
`MEMORY.md` and `memory_summary.md` before determining which Codex native-memory layout existed.
Codex 0.148 had produced its legitimate pre-consolidation layout:

- `extensions/ad_hoc/instructions.md`;
- `phase2_workspace_diff.md`;
- `raw_memories.md`.

The audit now identifies the layout before reading layout-specific files and explicitly recognizes
this evidence-empty pre-consolidation scaffold. Regression coverage proves both that the scaffold
does not count as an evidence-bearing memory and that an incomplete unknown layout is reported
without reading nonexistent files.

A provider-free execution-recovery mechanism then froze a new successor. It attests the source
plan and hash, the repaired evaluator, the exact inherited lane files, and the only recoverable
completed prefix. It preserves lanes 1 and 2, executes only lanes 3 through 36, budgets only the
remaining Claude calls, and cannot be chained with evaluation recovery.

The source correction passes all 117 `engram-eval` tests, evaluator Clippy with warnings denied,
formatting, and diff checks. Its standalone immutable evaluator is:

`/private/tmp/engram-eval-teaching-recovery-bin.bVxEzT/engram-eval`

SHA-256:

`7e39d59a7f11c74826f89190ba5f354710526770a06b043be40c52b86ac7db20`

The only active recovery plan is:

`/private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/run-plan.json`

SHA-256:

`bbc50ddf9c3ab3595b1c0ada61ef492382fd8dad1100a4ea0bafaa3b4abc6a2f`

It records 3,186,783 micro-USD of prior Claude spend, a $2.91 remaining execution allocation, and
the unchanged $7.00 cumulative ceiling. All 18 isolated Codex homes use Keychain-backed ChatGPT
login and pass the provider-free readiness gate.

### Recovery phase-scope proof

The repaired runner does not apply `completed_lane_orders` as a blanket skip list. Source review
confirms that field is used only to attest the strict completed teaching prefix and calculate the
remaining Claude call allocation. `run_native_memory_pilot` still iterates the plan's unchanged 36
lanes and delegates phase selection to `lane_run_action`:

- teaching skips only lanes whose audited lifecycle already reached Codex `awaiting_activation` or
  Claude `ready_for_evaluation`;
- activation skips non-Codex lanes and executes every Codex lane in `awaiting_activation`, so the
  inherited lane 2 is included;
- evaluation executes every lane in `ready_for_evaluation`, independent of whether its teaching
  evidence was inherited or created by the successor.

Before each execution it requires the phase report and provider targets to be absent, and after
each lane it runs an incremental audit and requires the exact next lifecycle state. Activation also
checks the one-hour interval from each lane's own teaching-trace modification time. The final phase
postconditions require all lanes ready after activation and all lanes complete after evaluation.
Therefore the recovery preserves a teaching prefix without reducing activation or evaluation
coverage and will stop instead of replaying a lane whose target already exists.

The frozen activation phase contains 18 Codex calls with a one-hour minimum idle interval and the
exact response marker `NATIVE_MEMORY_ACTIVATION_COMPLETE`. Native and combined arms enable native
memory generation while keeping memory use disabled for the activation turn; lean Engram arms keep
native memory disabled. This preserves the host's normal consolidation boundary without injecting
the learned lesson into the activation prompt.

Native capture remains an observed outcome, not a required success condition. After a passing
activation trace, a recognized evidence-empty scaffold or an absent optional artifact passes as
`observed=false`; substantive provider-created memory passes as `observed=true`. Files predating
pilot preparation, premature substantive state, symlinks, or trace-visible shell writes to native
memory remain integrity failures. The evaluator therefore measures whether Codex chose to retain
the lesson without manufacturing retention or invalidating legitimate non-capture.

### Evaluation-isolation proof

The unchanged recovery plan has 36 distinct evaluation working directories, one per lane. It also
has 18 distinct Codex homes, 18 distinct Claude configuration roots, and 24 distinct Engram
homes/projects for the 24 Engram-bearing lanes. No native or Engram state is shared across arms,
cases, or repetitions.

Matched evaluations use each lane's independently created arbitrary-name moved checkout; mismatch
evaluations use its independently created legacy checkout. Codex runs with ignored user config and
rules, strict frozen config, and a read-only sandbox. Native and combined Codex evaluations set
`features.memories=true`, `memories.generate_memories=false`, and `memories.use_memories=true`;
lean Engram sets all three false. Claude native and combined lanes have isolated auto-memory
enabled, while lean Engram disables it. Only Engram-bearing lanes receive their lane-specific
six-tool MCP server, project name, home, and generated adapter.

This design gives every lane the same frozen task and acceptance contract while varying only its
declared memory layer. A result can therefore be attributed to that lane's host/layer behavior
without importing another repetition's memory or the developer's live harness configuration.

### Acceptance anti-forgery proof

The agent's structured output is not sufficient to pass acceptance. Under this schema, repository
remote must appear in a correlated host tool result and normalize to the frozen Atlas remote.
Engram-bearing identity must come from a traced Engram orientation result for the exact evaluation
cwd. Native-only project/component claims must match structured output and be corroborated by the
frozen target, excerpt, and SHA-256 of the exact checkout evidence read.

Source-backed prerequisite state is also trace-derived. Engram lanes must contain the applicable
call/result-correlated condition observation from `procedure_match`; native lanes must perform the
exact checkout read whose output and hash match the frozen source. In the matched case, context
application requires both a trace-returned applicable procedure and a successful correlated host
command. Evidence citation requires the pre-attested receipt path in a host tool result. The first
attempt, command, exit status, and success marker must all belong to the same host invocation.

For mismatch, `abstained=true` passes only when the trace contains zero relevant procedure
attempts, the exact current prerequisite evidence was inspected, and the forbidden procedure was
neither returned nor applied. Consequently, self-reported identity, evidence, success, or
abstention cannot manufacture a passing result without the corresponding trusted host boundary.

### Recovery budget-envelope proof

The 291-cent confirmation is one frozen residual runner envelope, not a new allocation for each
phase. The source plan had 36 Claude calls: 18 teaching and 18 evaluation. Recovery inherited one
completed Claude teaching call, leaving exactly 35 calls. Every remaining frozen Claude teaching
and evaluation argv has `--max-budget-usd 0.083`, so their summed maximum-argument envelope is
$2.905, rounded upward to the plan's $2.91 confirmation. Activation invokes only Codex and consumes
none of that Claude envelope.

Recovered teaching used 17 of those calls and reported 680,495 micro-USD, well below its 1,411,000
micro-USD summed per-call maximum. The remaining 18 evaluation argv sum to a $1.494 maximum before
the explicitly acknowledged possibility of an individual end-of-turn overshoot. Even if every
evaluation call reached that frozen maximum, accounted spend would be 5,361,278 micro-USD before
such overshoot headroom, below the unchanged 7,000,000-micro-USD authorized ceiling. Each phase
requires the same exact 291-cent confirmation only because the runner verifies the immutable plan
contract on every entry; it does not add that amount to spend a second time.

## Teaching outcome

Recovered teaching ran exactly once from Unix millisecond `1787128393168` through
`1787130079329`. The recovery runner contains exactly lane orders 3 through 36; all 34 provider
processes exited 0, none used a recovered trace, and none accepted a turn-boundary budget exit.
The 17 recovered Claude calls cost 680,495 micro-USD. Together with the 3,186,783 micro-USD frozen
prior spend, cumulative accounted Claude spend is 3,867,278 micro-USD, below the $7.00 ceiling.

Teaching runner:

`/private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/runner-teaching.json`

SHA-256:

`850d214ab6b2b0a45d8cedaa2fa20fe380bcc96ef9994644a826061fbaec1801`

The post-teaching provider-free audit reports:

- `invalid=false`;
- zero failures;
- all 18 Claude lanes `ready_for_evaluation`;
- all 18 Codex lanes `awaiting_activation`;
- every required Engram procedure verification passed;
- no evaluation trace or outcome exists yet.

Runtime attestation remains `verified=true`, including the exact host and Engram hashes, the six
effective MCP tools, restricted-tool rejection, and reviewer-authority rejection. All 18 Codex
authentication homes remain ready. Repository `target/debug` is absent.

### Disk-reserve preflight

At 2026-08-19 12:17 +03:00 the data filesystem had 33,612,783,616 bytes available. Engram's own
persistent-store policy required 19,893,251,686 bytes, leaving a 13,719,531,930-byte margin
(12.78 GiB). The preserved 36-lane source tree occupied 893,852 KiB and the compact recovery-plan
directory occupied 768 KiB. No trace or user-owned file was deleted to obtain this margin.

This measurement is evidence for readiness, not a waiver: the frozen runner recomputes Engram's
disk-headroom policy before provider execution in each phase and must stop if the live margin falls
below the required reserve.

At 2026-08-19 12:45 +03:00, after the validation below, the previously confirmed cache cleanup
removed only the rebuildable, unreferenced Cargo targets `engram-schema10-check`,
`engram-schema9-check`, `engram-schema8-check`, and `engram-schema8-postrun-check` from
`/private/tmp`. Available space rose to 69,692,313,600 bytes, leaving a 49,799,061,914-byte margin
above the unchanged reserve. The recovery plan, evaluator, and Engram executable hashes remained
exact, and the full provider-free runtime attestation returned `verified=true` afterward.

### Phase-target absence preflight

At 2026-08-19 12:33 +03:00 a provider-free pass derived the retention deadline independently for
all 18 Codex lanes from each lane's audited teaching-trace modification time. Lane 2 was the
earliest eligible at `1787130434993` (12:07:14 +03:00); lane 36 was the latest at
`1787133677666` (13:01:17 +03:00). Because activation must cover every Codex lane, the latest
deadline controls the phase.

Source review then mapped the exact non-replay set enforced by `validate_phase_targets_absent`,
including provider traces, stderr, structured agent output, and Engram-cleanup stdout/stderr. At
12:35 +03:00, all 60 activation targets, all 156 evaluation targets, and both phase-report targets
were absent. This is direct evidence that neither remaining phase has already begun and that the
runner's non-replay target preconditions remain satisfiable. The checks must still be repeated
immediately before execution.

The current source now includes
`phase_target_validation_refuses_every_provider_output_class`, a focused regression that creates
each activation/evaluation target class in turn and proves the runner refuses to overwrite it.
That focused test passed, the complete 118-test Engram-eval suite passed, strict all-target
Engram-eval Clippy passed with warnings denied, formatting passed, and repository `target/debug`
remained absent. This test-only hardening does not alter the already-frozen evaluator executable
used by the current pilot.

## Retention boundary and next exact phase

The latest Codex teaching trace has modification time `1787130077666`. The frozen one-hour
activation boundary is therefore `1787133677666`, or 2026-08-19 13:01:17 +03:00. No activation may
run before that instant.

After the boundary, and only if plan/binary hashes, runtime attestation, Codex authentication,
provider-free audit, disk reserve, and untouched traces remain pristine:

1. Run activation exactly once with the frozen recovery plan and the confirmed $2.91 allocation.
2. Require every lane to be ready and re-run all provider-free integrity gates.
3. Run evaluation exactly once with the same plan and allocation.
4. Require a complete, valid audit, then generate the normal and strict comparison reports.
5. Inspect matched/mismatch source observations, canonical identity, first action, command,
   execution or abstention, evidence, leakage, stale influence, cost, tokens, latency, and packet
   telemetry before updating the product claim.

The audit and comparison commands currently emit JSON to stdout; they do not create
`completion-audit.json` or `comparison-report.json` automatically. Evidence must not cite those
paths unless a later explicit capture actually creates them.

## Preregistered verdict and measurement contract

The matched case requires canonical Atlas remote and `queue-worker` component evidence, the
`procedure-atlas-context-probe-v1` context key, the tracked `toolchain.toml` observation
`tools.version = 3`, first action `run_verified_procedure`, no amber invocation, one successful
cobalt invocation, exit code 0, and `ATLAS_CONTEXT_PROBE_OK`.

The mismatch case requires the same canonical repository/component identity, first action
`inspect_procedure_prerequisites`, the tracked `toolchain.toml` observation `tools.version = 2`,
safe abstention, exclusion of `procedure-atlas-context-probe-v1` from applicable context, and zero
amber or cobalt procedure attempts. Both cases bind the expected evidence files and hashes in each
lane's frozen acceptance contract; agent prose alone cannot satisfy them.

The reporter compares each Engram treatment with native memory within the same host, case, and
repetition. Its preregistered unweighted outcome metrics are full acceptance, outcome and identity
correctness, first-action correctness, context application and handling, evidence citation,
procedure revalidation, first-procedure-attempt correctness, successful execution, expected exit
and marker observation, repeated failures, abstention, and correct abstention. Pareto dominance
is evaluated pair by pair, not from aggregate totals: every host/case/repetition treatment pair
must be non-regressive on its applicable directional metrics, and at least one pair must strictly
improve. Aggregate cancellation therefore cannot hide a repetition regression. A complete delta
for this matrix must contain six matched pairs per host and treatment. Raw `context_applied` remains
reported as an aggregate, while dominance uses `context_handling_correct` because correct raw
application is true for execution but false for safe abstention. Portable incremental value
requires the same treatment—lean Engram or combined—to satisfy the pairwise dominance rule
across both Codex and Claude Code; a partial, mixed-treatment, or single-host improvement is
insufficient.

Trace telemetry is supplementary and cannot change acceptance or Pareto classification. For each
completed lane the report separately records host-reported and runner-observed duration, input and
output tokens, cache and reasoning-token fields when the host supplies them, Engram tool-call count,
aggregate Engram result bytes, maximum individual result bytes, and result bytes by tool. Final
interpretation must report these overheads honestly and must not turn a favorable task result into
an unregistered packet-size or latency claim.

## Exact provider-free post-run extraction

After evaluation, the following read-only commands provide the compact evidence needed for the
claim update. They intentionally consume evaluator stdout instead of naming nonexistent report
files.

Lifecycle and integrity:

```bash
set -o pipefail
/private/tmp/engram-eval-teaching-recovery-bin.bVxEzT/engram-eval \
  audit-native-memory-pilot \
  --plan /private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/run-plan.json \
  --require-complete \
| jq '{pilot_id, complete, invalid, ready_for_evaluation, all_acceptance_passed,
       lane_failure_count: ([.lanes[].failures[]] | length),
       phases: (.lanes | sort_by(.phase) | group_by(.phase)
       | map({phase: .[0].phase, count: length})),
       lane_failures: [.lanes[] | select((.failures | length) > 0)
       | {order, arm, phase, failures, native_memory_write_attempts}]}'
```

Preregistered aggregates and matched Pareto deltas:

```bash
/private/tmp/engram-eval-teaching-recovery-bin.bVxEzT/engram-eval \
  report-native-memory-pilot \
  --plan /private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/run-plan.json \
| jq '{pilot_id, complete, invalid, all_acceptance_passed,
       portable_incremental_value_observed, incremental_value_signal,
       layer_aggregates, matched_deltas, claim_limitations,
       failed_lanes: [.lane_outcomes[] | select(.metrics.passed == false)
       | {order, case_id, repetition, arm, host, memory_layer, outcome_failures}]}'
```

Supplementary telemetry by host and memory layer:

```bash
/private/tmp/engram-eval-teaching-recovery-bin.bVxEzT/engram-eval \
  report-native-memory-pilot \
  --plan /private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/run-plan.json \
| jq '[.lane_outcomes[] | select(.telemetry != null)]
       | sort_by(.host, .memory_layer) | group_by([.host, .memory_layer])
       | map({host: .[0].host, memory_layer: .[0].memory_layer, lanes: length,
              input_tokens: ([.[].telemetry.input_tokens // 0] | add),
              output_tokens: ([.[].telemetry.output_tokens // 0] | add),
              reasoning_output_tokens:
                ([.[].telemetry.reasoning_output_tokens // 0] | add),
              engram_tool_calls: ([.[].telemetry.engram_tool_calls] | add),
              engram_result_bytes: ([.[].telemetry.engram_result_bytes] | add),
              max_engram_result_bytes:
                ([.[].telemetry.engram_max_result_bytes] | max),
              runner_provider_duration_ms:
                ([.[].telemetry.runner_provider_duration_ms // 0] | add),
              max_runner_provider_duration_ms:
                ([.[].telemetry.runner_provider_duration_ms // 0] | max)})'
```

Provider execution accounting:

```bash
jq '{phase, lane_count: (.lanes | length),
     exit_codes: ([.lanes[].exit_code] | sort | group_by(.)
       | map({exit_code: .[0], count: length})),
     claude_calls: ([.lanes[] | select(.host == "claude_code")] | length),
     claude_cost_microusd:
       ([.lanes[].provider_reported_cost_microusd // 0] | add),
     recovered_traces:
       ([.lanes[] | select(.recovered_from_existing_trace == true)] | length)}' \
  /private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/runner-evaluation.json
```

The strict reporter must also run once. Exit 0 proves its frozen portable-value gate; exit 1 is a
valid negative result that must be reported, not repaired:

```bash
/private/tmp/engram-eval-teaching-recovery-bin.bVxEzT/engram-eval \
  report-native-memory-pilot \
  --plan /private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/run-plan.json \
  --require-portable-incremental-value
```

## Final activation and evaluation outcome

The retention gate passed at 2026-08-19 13:01:17 +03:00. Immediately before execution, the plan,
evaluator, Engram executable, teaching receipt, and both evidence documents matched their frozen
SHA-256 values. Runtime attestation was `verified=true`; all 18 isolated Codex homes were ready;
the audit was `invalid=false` with 18 Claude lanes `ready_for_evaluation`, 18 Codex lanes
`awaiting_activation`, and zero failures. All 60 activation targets, 156 evaluation targets, and
two phase reports were absent. The live disk margin was 49,594,245,530 bytes above Engram's own
reserve, and repository `target/debug` was absent.

Activation ran exactly once. Its 18 Codex calls all exited 0, no trace was recovered or replayed,
and every lane became `ready_for_evaluation`. The activation report is
`/private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/runner-activation.json`,
SHA-256 `da8d1280404fff9213a800cf1ebf4a05a90c784b2c69f49e0f9561b26a6c258a`.
Post-activation audit, attestation, authentication, hash, disk, and non-replay checks all passed;
all 156 evaluation targets and the evaluation report were still absent.

Evaluation then ran exactly once. All 36 provider calls exited 0, no trace was recovered or
replayed, and all 36 lanes reached `evaluation_complete`. The evaluation report is
`/private/tmp/engram-native-memory-pilot-v1-teaching-recovery-final.LAdnE1/runner-evaluation.json`,
SHA-256 `6c342db9263141c107bcef9ebff041f59c798e56071161111286efcdc1fb9581`.
The final audit is `complete=true`, `invalid=false`, with zero lifecycle/integrity failures. Its
`all_acceptance_passed=false` value is expected for a comparative experiment: all 12 native-only
controls failed at least one full-contract criterion, while all 24 Engram treatments passed.

The preregistered layer results are:

| Host | Layer | Full contracts | Outcome correct | Identity correct | Correct abstentions |
| --- | --- | ---: | ---: | ---: | ---: |
| Claude Code | native | 0/6 | 5/6 | 0/6 | 2/3 |
| Claude Code | lean Engram | 6/6 | 6/6 | 6/6 | 3/3 |
| Claude Code | Engram + native | 6/6 | 6/6 | 6/6 | 3/3 |
| Codex | native | 0/6 | 3/6 | 0/6 | 3/3 |
| Codex | lean Engram | 6/6 | 6/6 | 6/6 | 3/3 |
| Codex | Engram + native | 6/6 | 6/6 | 6/6 | 3/3 |

Every lean-Engram/native and combined/native comparison contained all six matched
host/case/repetition pairs. Each of the four comparisons was non-regressive on every applicable
preregistered directional metric and strictly improved at least one pair. Both normal and strict
reporters therefore return `portable_incremental_value_observed=true` with signal
`engram_and_combined_across_hosts`; the strict reporter exits 0. In the 12 matched Engram lanes,
the agents resolved Atlas and `queue-worker`, revalidated the tracked `toolchain.toml` source and
attested receipt, returned and applied `procedure-atlas-context-probe-v1`, invoked cobalt first,
and observed exit 0 plus `ATLAS_CONTEXT_PROBE_OK`. In the 12 mismatch Engram lanes, the agents
resolved the same repository/component, inspected the version-2 source, kept the procedure out of
applicable context, attempted neither amber nor cobalt, and abstained. No treatment lane failed.

Telemetry remains descriptive because this run froze no numeric host-boundary target. Across each
six-lane group, observed input tokens were 373,544/314,581/412,018 for Claude
native/Engram/combined and 1,073,252/751,265/814,498 for Codex. Runner-observed provider time was
148,098/170,132/162,014 ms for Claude and 262,391/268,306/257,610 ms for Codex. Engram generated
12-13 tool calls per six-lane treatment, 53,687-55,963 aggregate result bytes, and a maximum single
result of 7,720 bytes. These observations do not establish a general latency, token, or packet-size
bound. This matrix also contains no registered scope-leakage or stale-influence metric; those
claims remain outside this result rather than being inferred as zero.

Evaluation used 18 Claude calls and reported 661,636 micro-USD. Together with 3,186,783 micro-USD
of frozen prior spend and 680,495 micro-USD for recovered teaching, total accounted spend is
4,528,914 micro-USD under the 7,000,000 micro-USD authorized ceiling. Final provider-free checks
again found all 18 Codex homes ready and runtime attestation `verified=true`; the plan remained at
SHA-256 `bbc50ddf9c3ab3595b1c0ada61ef492382fd8dad1100a4ea0bafaa3b4abc6a2f`, repository
`target/debug` remained absent, and no live adapter, settings, hook, daemon, or harness installation
was changed.

## Coverage exclusions

Even a complete, valid, strict-positive result has a deliberately narrow meaning. This matrix:

- compares native, lean Engram, and combined memory, but has no instructions-only arm;
- keeps both cases inside Atlas, so prerequisite mismatch is not a wrong-project or cross-project
  leakage test;
- verifies remote and component in fixed fixtures but does not exercise task identity, conflicting
  project candidates, or an ambiguity that should block;
- does not natively run missing-source, expired/stale procedure, unrelated no-result,
  compaction/resume, correction/deletion, or secret-canary scenarios;
- records packet, token, and latency telemetry descriptively but freezes no numeric host-boundary
  target for this run;
- repeats on one machine with one Codex build, one Claude build, one Engram build, and one fixture
  family, rather than across independent machines, versions, repositories, or operators;
- exercises isolated generated adapters, not already-running live harnesses, and contains no Cursor
  lane.

Those boundaries must remain explicit in the final report and flagship completion audit. Separate
provider-free or prepared scenarios are not substitutes for completed native evidence, and this run
cannot by itself prove the full no-cross-project, ambiguity, bounded-packet, stale-memory,
correction/deletion, secret-persistence, live-installation, or multi-harness completion criteria.

## Current claim boundary

The full flagship goal remains incomplete, but this phase now proves the central narrow product
claim it was designed to test: across three repetitions of matched execution and prerequisite
mismatch on both Codex and Claude Code, lean Engram and Engram plus native memory each passed all
12 full contracts and Pareto-dominated native memory in every preregistered matched pair. Native
memory alone passed none of its 12 full contracts. This is strong evidence that the portable,
verified procedure layer adds value across both tested hosts and that native memory does not make
Engram redundant for this journey.

The claim remains limited to one machine, one repository family, two procedure-applicability
cases, isolated generated adapters, and the frozen versions listed above. It does not prove the
instructions-only comparison, cross-project isolation, missing-source or expiry behavior,
task/ambiguity resolution, compaction/resume, correction/deletion propagation, secret persistence,
bounded host-visible overhead, Cursor support, already-running live-harness behavior, or
independent-machine/operator reproducibility.

No live adapter, skill, settings, hook, daemon, or harness installation was changed. No lane may be
repaired or replayed, and no user-owned worktree change may be staged or committed as part of this
experiment.
