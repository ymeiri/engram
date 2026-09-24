# Learned native-memory pilot v1 exact-write replacement — 2026-08-09

## Second teaching-attempt outcome

The first replacement passed attestation, all three isolated Codex login checks, and the exact
operator approval gate. Claude lane 1 then correctly observed the frozen failed command and ran the
successful replacement command, including the expected exit code and
`ATLAS_CONTEXT_PROBE_OK` marker. The teaching prompt nevertheless left the multi-field Engram write
request for the model to reconstruct. The model omitted `scope_type`, then omitted `local_path` on
its retry and mistyped the immutable receipt path. Claude Code stopped after five turns with
`error_max_budget_usd` and provider-reported cost `$0.0524307` against the `$0.050` allocation. The
runner did not execute any later lane.

This is a controlled-teaching contract failure rather than a learned-procedure failure. Engram
teaching arms now receive one machine-generated exact `memory(action=add)` argument object after
they have actually observed the failed and successful commands. The frozen request includes:

- repository remote and canonical teaching-checkout scope;
- writer harness, provider, model, surface, and actor provenance;
- the semantic context key and complete structured procedure card;
- exact prerequisites, failed invocation, successful invocation, exit code, and marker;
- the canonical pre-attested receipt path.

The agent must still execute the commands, observe the results, and submit the memory call. It may
not add, omit, rename, infer, or rewrite fields, promote or verify the candidate, or hand-write
native-memory files. This keeps the pilot focused on durable-memory transfer instead of incidental
tool-schema reconstruction.

## Provider-free adapter hardening before authentication

Before any login or provider call used the first exact-write plan, a production-adapter audit found
that the generated Codex skill and Claude command/hook still described write scope and provenance
only generically. Ordinary sessions could therefore repeat the same missing-field recovery loop
outside the controlled pilot.

The shared generated adapter guidance now states every required `memory(action=add)` identity,
scope, and writer field; names the repository-scope selector requirement; distinguishes retrieval
`scope` from write `scope_type`; and lists every structured procedure-card field. The Engram-marked
Codex and Claude project adapters were regenerated from that source. Static status verifies exact
generated-file hashes and all six agent-profile tools for both hosts. Claude settings were not
modified. The installed CLI was replaced with the rebuilt candidate, the single global daemon was
restarted, and runtime attestation reports `matched` with no warnings.

The unexecuted plan under `engram-native-memory-pilot-v1-exact-write.u4bcw7` is therefore
superseded. It consumed no authentication state and made no provider call. The final plan below
captures the hardened adapters and rebuilt Engram binary.

## Conservative budget account

- first failed attempt: `$0.0918771`, rounded up to 91,878 micro-USD;
- second failed attempt: `$0.0524307`, rounded up to 52,431 micro-USD;
- frozen prior spend: 144,309 micro-USD (`$0.144309`);
- exact-write replacement allocation: `$0.35` across six Claude calls;
- per-call runner allocation: `$0.058`;
- existing authorized cumulative ceiling: `$0.50`.

The nominal cumulative amount is `$0.494309`, leaving `$0.005691` below the authorized ceiling.
As already observed, Claude Code checks `--max-budget-usd` between API calls, so an individual
request may overshoot its turn-boundary allocation. The plan's arithmetic is conservative
accounting, not a strict provider billing ceiling.

## Fresh frozen plan

The new provider-free plan is:

`/private/tmp/engram-native-memory-pilot-v1-adapter-contract.WoyzPa/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `a9ebb318b5721ec0e380b4d5a0d8fbe43615b48dcc1457779cafc57d9ec7b1a5` |
| `protocol.snapshot.json` | `d45dd55c728448a180fb2902001f8d36251856bc68908ea598d46dc60a169ac2` |
| source `protocol.json` | `d45dd55c728448a180fb2902001f8d36251856bc68908ea598d46dc60a169ac2` |
| `target/debug/engram-eval` | `deb3cc4d085b3377b1f06407bdd9a5ad1bad788a2b7d290deb71d01c2965231f` |
| candidate and installed `engram` | `0a28565a768b6e11db026e4d5638ae491798df070affbcaada1ebae351946487` |

The plan attests Claude Code 2.1.226, Codex CLI 0.147.0-alpha.1.2, Engram 0.2.3, and the six-tool
agent MCP profile. Provider-free re-attestation passes, and all six lanes audit as pristine
`prepared`. The three fresh Codex homes report `not_logged_in`; no provider call has run against
this plan.

## Verification

```text
cargo fmt --all --check
passed

cargo test -p engram-eval
66 passed; 0 failed

cargo test -p engram-index harness::tests
51 passed; 0 failed

cargo clippy -p engram-index -p engram-eval --all-targets -- -D warnings
passed

cargo build -p engram-eval
passed

attest-native-memory-pilot
verified: true

audit-native-memory-pilot
6 prepared; 0 invalid

installed daemon runtime attestation
matched; 0 warnings
```

## Remaining gates

Before another teaching execution:

1. Complete ordinary Keychain-backed ChatGPT browser login for the three fresh Codex homes.
2. Confirm this plan's exact `$0.35` Claude runner allocation and acknowledge the documented
   per-call turn-boundary overshoot risk.
3. Re-run digest, attestation, pristine audit, and all-lane authentication checks.

Preparing and attesting this plan does not authorize provider execution.

## Adapter-contract plan teaching result

After the exact approval, lane 1 (`claude_engram_plus_native`) completed its teaching call in four
turns for `$0.0425264`. It executed the frozen failed and successful commands, submitted the exact
Engram write object, and returned candidate `019fe717-2fed-7cc1-af40-3a7c810e7f8d`. The trusted
evaluator independently verified the pre-attested receipt and activated that exact procedure. No
later lane ran.

The lane was nevertheless classified `invalid` because Claude Code produced no native
`MEMORY.md`. This exposed two benchmark defects:

1. Claude Code documents that auto memory does not save something every session and decides what is
   worth remembering. Its 2.1.226 host policy specifically excludes code/tool facts the agent
   discovers itself from automatic capture. Absence is therefore a legitimate product outcome for
   this learned-procedure case, not missing evaluation infrastructure.
2. The frozen Claude argv exposed only `Read` and `Bash`, while Claude's native memory is file based.
   Even when the host policy chooses capture, the normal file-edit surface must remain available.

The failed plan and trace remain immutable evidence. Its provider-reported cost is conservatively
rounded up to 42,527 micro-USD.

## Native-outcome contract

Native pilot schema 4 now separates observations from required infrastructure:

- native artifacts are recorded with `observed: true` or `observed: false`; absence after teaching
  or Codex activation is a valid measured outcome and evaluation continues;
- Engram state and exact trusted procedure verification remain required;
- missing required state, pre-existing/stale artifacts, symlinks, shell-manufactured memory, and
  zero-byte artifacts still fail integrity checks;
- Claude native and combined lanes expose `Write`/`Edit`, but `dontAsk` pre-approves edits only for
  the lane's isolated auto-memory directory. Repository, web, notebook, and subagent writes remain
  denied;
- the teaching prompt lets host-native policy decide whether and how to persist the lesson and
  forbids shell commands or repository files from emulating memory.

This preserves the benchmark question: does the host naturally retain an agent-discovered verified
procedure, and what incremental value does Engram add when it does not?

## Conservative budget account after the third attempt

- first failed attempt: 91,878 micro-USD;
- second failed attempt: 52,431 micro-USD;
- adapter-contract lane 1: 42,527 micro-USD;
- frozen predecessor spend: 186,836 micro-USD (`$0.186836`);
- proposed replacement allocation: `$0.30` across six Claude calls;
- per-call runner allocation: `$0.050`;
- existing authorized cumulative ceiling: `$0.50`.

The nominal cumulative amount is `$0.486836`, leaving `$0.013164` below the ceiling. An individual
Claude call can still overshoot its per-call allocation at a turn boundary.

## Fresh native-outcome plan

The new provider-free plan is:

`/private/tmp/engram-native-memory-pilot-v1-native-outcome-final.hZfOgq/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `35e52d187bd8ea0933d47a27318de9e82c81565806c4579456653ac86c172ffa` |
| `protocol.snapshot.json` | `18e87431a56dfb4f68461dc4ef00daa001c182678bc6ba297d37fab6927f1706` |
| source `protocol.json` | `18e87431a56dfb4f68461dc4ef00daa001c182678bc6ba297d37fab6927f1706` |
| `target/debug/engram-eval` | `44df347091ec2df40a234b1e5f4dce9fb10c2a888a94b45ce7fafedbcc8abef6` |
| candidate Engram | `0a28565a768b6e11db026e4d5638ae491798df070affbcaada1ebae351946487` |

Provider-free attestation reports `verified: true`; all six lanes are pristine `prepared` with no
failures. The three isolated Codex homes are `not_logged_in`. No provider call has run against this
plan.

Verification after the schema-4 correction:

```text
cargo fmt --all --check
passed

cargo test -p engram-eval
68 passed; 0 failed

cargo clippy -p engram-eval --all-targets -- -D warnings
passed

cargo build -p engram-eval
passed

attest-native-memory-pilot
verified: true

audit-native-memory-pilot
6 prepared; 0 invalid
```

Before teaching execution, complete the frozen Keychain-backed ChatGPT browser login separately for
the three new Codex homes, then approve this exact plan's `$0.30` Claude runner allocation and the
documented per-call `$0.050` turn-boundary overshoot risk. Preparation does not authorize provider
execution.

## Native-outcome teaching boundary and restart-safe runner

After all three isolated Codex homes reported `ready` and the operator approved the exact plan,
teaching lane 1 completed its controlled task and Engram write. Claude observed amber fail with exit
2, ran cobalt successfully with `ATLAS_CONTEXT_PROBE_OK`, and added candidate
`019fe73e-82f4-7570-a443-f6e7510961dc`. No Claude native `MEMORY.md` was generated, which remains a
valid measured outcome under schema 4.

Claude then returned its documented turn-boundary envelope with provider-reported cost
`$0.0534471`: `error_max_budget_usd`, `budget_exhausted`, and `end_turn`. The generic executor stopped
before trusted verification even though the provider-free audit classified the trace as passed and
the lane as `awaiting_verification`. Later lanes remained pristine `prepared`.

The runner now aligns with the audit without changing the frozen plan, provider argv, prompts, or
allocation:

- only that exact Claude terminal envelope can be accepted after trace capture;
- the nonzero exit, rounded-up provider cost (53,448 micro-USD), and boundary acceptance are recorded
  in the phase report;
- a retry can finish local cleanup and trusted verification from the immutable trace, then skip any
  completed lanes and continue the remaining preregistered lanes;
- no provider output is deleted, overwritten, repaired, or replayed;
- every other nonzero exit remains a hard failure.

Provider-free verification after the runner correction:

```text
cargo test -p engram-eval
72 passed; 0 failed

cargo clippy -p engram-eval --all-targets -- -D warnings
passed

cargo fmt --all --check
passed

cargo build -p engram-eval
passed

target/debug/engram-eval SHA-256
234a5ffe0636573afd435cf35788c86ba651e17d16b5c9af424b527c6b6ac863

run-plan.json SHA-256
35e52d187bd8ea0933d47a27318de9e82c81565806c4579456653ac86c172ffa

audit-native-memory-pilot
lane 1 awaiting_verification; lanes 2-6 prepared; 0 invalid
```

The resumed phase finalized lane 1 without a second provider call and then completed Codex lane 2's
teaching task. At Codex startup, the attested host created its standard evidence-empty memory
scaffold before activation: `raw_memories.md` said `No raw memories yet`, while `MEMORY.md` and
`memory_summary.md` explicitly described an empty memory set. The initial artifact rule treated any
pre-activation path as contamination, even though this fresh scaffold contained no learned content.

The audit now recognizes only either of the two exact evidence-empty working-tree scaffold variants
observed from the attested Codex host (or a wholly empty state) as pending and `observed: false`
until activation. Host-internal `.git` bookkeeping is excluded from the memory-content digest.
Substantive, malformed, symlinked, or pre-preparation working-tree state still fails. Freshness was
also tightened from the newest file timestamp to the oldest file timestamp, so adding one new file
can no longer conceal stale files. The frozen provider traces, plan, argv, prompts, and allocation
remain unchanged; lanes 2 and 4 now audit as `awaiting_activation`, not ready, until the
preregistered one-hour interval and activation calls.

## Completed teaching phase

The restart-safe phase completed all six matched teaching lanes with `invalid: false`:

- all six agents observed the amber failure and successful cobalt replacement;
- all four Engram lanes added exactly one structured candidate and passed trusted receipt
  verification;
- Claude native-only generated a non-empty `MEMORY.md` linking the cobalt procedure;
- Claude combined generated no native artifact, a valid non-capture outcome;
- Codex native-only and combined generated only evidence-empty scaffolds and remain
  `awaiting_activation`;
- the Codex lean Engram lane also remains `awaiting_activation` for the matched host call.

Provider-reported Claude teaching costs were `$0.0534471`, `$0.0471575`, and `$0.0313856`, totaling
`$0.1319902`. Adding all three still-unexecuted `$0.050` evaluation allocations to predecessor and
actual teaching spend yields `$0.4688262`, below the authorized `$0.50` cumulative ceiling before
any documented final-turn overshoot.

The final Codex teaching trace was written at 2026-08-09T19:33:30+0300. Activation must not start
before 2026-08-09T20:33:30+0300. After that deadline, re-attest the plan and Keychain status, run the
activation phase, require every lane to become `ready_for_evaluation`, and only then start fresh
evaluation.

## Activation result and rejected evaluation boundary

After the frozen one-hour retention interval, provider-free digest, executable, runtime-contract,
authentication, and lifecycle checks all passed. The approved activation phase then completed the
three Codex lanes with exit code 0. Codex produced no evidence-bearing native memory beyond its
fresh empty scaffolds; this remained a valid `observed: false` product outcome. All six lanes
subsequently audited `ready_for_evaluation` with no failures.

The approved evaluation invocation stopped before its first model request. Claude Code 2.1.226
rejected the inline output schema locally:

```text
Error: --json-schema is not a valid JSON Schema: no schema with key or ref
"https://json-schema.org/draft/2020-12/schema"
```

Lane 1's evaluation trace is an immutable zero-byte file and its private stderr contains exactly
that diagnostic. There is no terminal provider event, structured result, evaluation report, or
provider-reported evaluation cost. The source plan therefore remains `invalid`; no outcome
comparison is admissible and no source lane will be repaired or replayed.

The evaluator's shared output schema now omits the optional draft-2020-12 meta-schema declaration
and stays within the object/array/string/boolean/null subset accepted by both frozen hosts. The
change affects only host-side structured-output validation, not the task prompt, required fields,
acceptance contract, memory state, or preregistered metrics.

## Evaluation-only recovery plan

The provider-free recovery command accepts only the exact source failure above. It hashes and
references the rejected plan, trace, and stderr; revalidates the source digest, executable and MCP
attestations, lifecycle, and exact failure shape on every use; preserves every teaching,
activation, memory, fixture, adapter, and acceptance artifact by reference; creates fresh private
evaluation destinations for all six lanes; and rewrites only Claude's `--json-schema` value and
Codex's `--output-schema` path. A recovery plan cannot run teaching or activation, cannot be
chained, and cannot overwrite a provider artifact.

The frozen successor is:

`/private/tmp/engram-native-memory-pilot-v1-evaluation-recovery.6IaEPh/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| recovery `run-plan.json` | `b9c261bdaf6303b3ef284716d1b5ca9179c6975404679cb1c2461c5c1e415d98` |
| portable `agent-output.schema.json` | `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60` |
| rejected source `run-plan.json` | `35e52d187bd8ea0933d47a27318de9e82c81565806c4579456653ac86c172ffa` |
| rejected empty trace | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| rejected stderr | `d8a1e758a0885d376f7a2457d829c53f27040ef929246fb4e28105670fb743de` |
| recovery-builder `target/debug/engram-eval` | `78e4a138e5b101a35545189a70b58eec57afc400bb0f86346a3e45f67b401b1c` |
| attested Engram executable | `0a28565a768b6e11db026e4d5638ae491798df070affbcaada1ebae351946487` |

Provider-free attestation reports `verified: true`, all three isolated Codex homes report `ready`,
and all six recovery lanes audit `ready_for_evaluation` with `invalid: false`. The recovery root is
mode `0700`; its plan and schema are mode `0600`.

The successor accounts the source plan's 186,836 micro-USD predecessor spend plus its three
rounded teaching costs (53,448, 47,158, and 31,386 micro-USD), for 318,828 micro-USD of prior spend.
Its remaining three Claude evaluation calls retain the frozen `$0.050` per-call boundary and require
a new exact `$0.15` allocation. The nominal total is 468,828 micro-USD, below the already authorized
`$0.50` cumulative ceiling. Preparation and provider-free validation do not authorize execution;
the recovery evaluation requires exact plan-specific approval and acknowledgement that an
individual call may overshoot its turn-boundary allocation.

Verification after the recovery implementation:

```text
cargo fmt --all --check
passed

cargo test -p engram-eval
74 passed; 0 failed

cargo clippy -p engram-eval --all-targets -- -D warnings
passed

cargo build -p engram-eval
passed

attest-native-memory-pilot (recovery plan)
verified: true

check-native-memory-pilot-auth --require-ready (recovery plan)
ready: true

audit-native-memory-pilot --require-ready (recovery plan)
6 ready_for_evaluation; 0 invalid
```
