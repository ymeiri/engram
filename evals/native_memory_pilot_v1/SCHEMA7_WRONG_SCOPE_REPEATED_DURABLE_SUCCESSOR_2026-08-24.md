# Schema 7 repeated wrong-scope pilot — durable successor

Updated: 2026-08-26T20:22:52Z

## Purpose

Continue the corrected 18-lane, six-arm, three-repetition wrong-repository-scope/no-result
experiment without depending on `/private/tmp` for authoritative plans, binaries, isolated host
state, or traces. The expired August 19 plan executed no provider calls and must not be repaired or
reconstructed.

## Current state

- Repository: `git@github.com:ymeiri/engram.git`
- Git HEAD: `4e2b4c1e0047633bbc0a640c1966e60774925f82`
- The pre-existing dirty and untracked worktree is preserved. Nothing was staged or committed.
- The repository's `target/debug` remains absent.
- Teaching started once. Lane 1 completed one Claude provider call; no other lane has run.
- The disposable validation target was removed after explicit user confirmation: `cargo clean`
  removed 20,026 files and 14.0 GiB.

The corrected protocol remains:

`evals/native_memory_pilot_v1/protocol-schema-7-unconditional-route-wrong-scope-repeated-current.json`

SHA-256:

`fafcb11112a410cd8de2ea29fb1300bb2a4023d5c0eb3e62f92f816285343d8c`

## Provider-free validation

Validation used the disposable target directory
`/private/tmp/engram-native-memory-durable-successor-build-20260824-01`:

- `cargo fmt --all --check`: passed.
- `cargo test -p engram-eval`: 121 passed, 0 failed.
- `cargo clippy -p engram-eval --all-targets -- -D warnings`: passed.
- `cargo build -p engram-eval -p engram-cli`: passed.

The regression `teaching_repository_remote_comes_from_the_teaching_checkout` passed as part of the
suite.

## Durable frozen binaries

Private artifact root, mode `0700`:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/wrong-scope-repeated-durable-successor-20260824-01`

| Artifact | SHA-256 |
| --- | --- |
| `bin/engram` | `2122a4d9356d26e0d519ed7a672c85d9a74109ff0e2e35393fa9f8e2626a8388` |
| `bin/engram-eval` | `e355387c40a818cfb9886cd5f5123bcc06d61d6448ec8c5489291b4cf1e39aa9` |

Both executable files have mode `0700`. After preparation, the durable root occupies approximately
406 MiB.

## Prepared successor

The provider-free preparation command completed successfully. The authoritative plan is:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/wrong-scope-repeated-durable-successor-20260824-01/run/run-plan.json`

Run-plan SHA-256:

`93fa920da74672ffabe05c503080f715d93cd278dcb04c35e5f60bff98952058`

The frozen host executables are:

| Host | Version | SHA-256 |
| --- | --- | --- |
| Codex | `codex-cli 0.149.0-alpha.4.1` | `09db9560f6f9dec139d3324254fb3c8fdbad5ecce1d8c794113dc15294f6aefd` |
| Claude Code | `2.1.241` | `1495eb7c42d3b4451f5f1cd38b6d498d22a4a38c802bc2be5c1cf1795e64820d` |

Provider-free gates after preparation:

- formal plan and runtime attestation: `verified=true`;
- matrix: 18 lanes, all six arms, three repetitions, nine Codex and nine Claude lanes;
- lifecycle audit: `invalid=false`, 18 `prepared`, zero failures, zero native-memory write
  attempts, and zero trace/output/verification files;
- authentication audit: `provider_free=true`, `ready=false`, all nine Codex lanes
  `not_logged_in`;
- no lane-local `auth.json` exists;
- all 12 Engram teaching prompts contain the Atlas remote and none contains the Orbit remote;
- all 18 evaluation working directories are the Orbit worker checkout;
- acceptance requires Orbit/worker identity, `abstain`, no required context key, and forbids
  `procedure-atlas-context-probe-v1`;
- the installed live daemon is separately attested as matched and writable, with
  26,951,806,976 bytes available against a 19,893,251,686-byte reserve.

The pilot candidate's isolated six-tool MCP contract is frozen in the plan. The installed live
daemon has a different executable and tool-schema hash; it is not substituted for the frozen
candidate.

## Authentication-ready gate

Fresh isolated Keychain-backed ChatGPT browser login completed sequentially for Codex lanes 2, 4,
6, 8, 10, 12, 14, 16, and 18. The frozen provider-free checker with `--require-ready` reports
`ready=true` and all nine lanes `ready`. No lane-local `auth.json` exists.

The immediately repeated hard gate reports:

- exact plan and binary hashes unchanged;
- formal attestation `verified=true`, including restricted-tool and reviewer-authority rejection;
- lifecycle audit `invalid=false`, all 18 lanes `prepared`, zero failures, zero native-memory write
  attempts, and zero trace/output/verification files;
- installed live daemon separately `running=true`, `ready=true`, storage `ready`, and runtime
  attestation `matched`;
- 25,913,384,960 bytes available against a 19,893,251,686-byte disk reserve;
- repository `target/debug` absent.

## Teaching interruption and exact recovery

Lane 1 completed successfully at the provider boundary and produced teaching trace SHA-256
`50cb9d95cf147121f7758c2998f4c89ba2c068621ab5bdf7f87f1ff848f69327`. Claude observed the
required failed and successful commands and added exactly one Engram procedure candidate with ID
`01a0337d-0593-7ef1-af45-83541ccbd414`. The candidate review file has SHA-256
`0f11fb1f0c1f3916ab8dc4ad5ba3dca8eabadfed38cc2b235d22e79b943b946b`.

The local trusted verifier then stopped before verification with:

`trusted evaluator expected exactly one matching procedure candidate, found 0`

Root cause: the candidate is correctly repository-scoped to the Atlas teaching checkout, but the
selector incorrectly compared it with the Orbit evaluation remote. That is intentional in this
wrong-repository-scope experiment. The provider call, candidate, cleanup outputs, and source plan
remain unchanged, and lane 1 is `awaiting_verification`; lanes 2–18 remain `prepared`.

The evaluator repair:

- resolves repository scope from the attested teaching checkout while retaining exact task,
  command, failure-signature, prerequisite, verification, status, and candidate-ID checks;
- permits recovery only for one exact successful Claude teaching trace, an existing candidate
  review, absent verification output, a strict completed provider prefix, and distinct teaching
  and evaluation remotes;
- hash-attests every inherited lane-1 trace/review/cleanup file and subtracts the completed Claude
  call from successor budget accounting;
- never replays lane 1.

Validation of the repair used the release profile without creating `target/debug`:

- `cargo fmt --all --check`: passed;
- `cargo test -p engram-eval --release`: 122 passed, 0 failed;
- `cargo clippy -p engram-eval --release --all-targets -- -D warnings`: passed;
- `cargo build -p engram-eval --release`: passed.

The durable recovery root is:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/wrong-scope-repeated-teaching-scope-recovery-20260824-01`

Repaired evaluator SHA-256:

`a430e205648cb2e4c21124da79f0a397257bd1235a8ed71c241a2819c3fd6bdd`

Recovery plan:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/wrong-scope-repeated-teaching-scope-recovery-20260824-01/run/run-plan.json`

Recovery plan SHA-256:

`dfec6b3db45a4a686f53ae33626f42147ba3cc9884baf893f4a06b5e70ab34c6`

The successor accounts `4,575,535` micro-USD of prior Claude spend, authorizes 17 remaining Claude
calls with a 94-cent allocation under the unchanged 700-cent cumulative ceiling, and declares lane
1 as a completed provider call that must not be replayed. Formal attestation is `verified=true`,
all nine isolated Codex lanes are Keychain-backed and `ready`, and the provider-free lifecycle audit
is pristine: `invalid=false`, lane 1 `awaiting_verification`, and lanes 2–18 `prepared`.

## Current phase

The recovery is complete and immutable. Disk cleanup left approximately 170 GiB free. The exact
plan, protocol, evaluator, Engram binary, runtime MCP contract, and all nine Keychain-backed Codex
logins re-attested before activation and again at the activation/evaluation boundary.

The recovery teaching phase completed across all 18 lanes. Lane 1 reports
`recovered_from_existing_trace=true`, exit code 0, and its original provider trace SHA-256 remains
`50cb9d95cf147121f7758c2998f4c89ba2c068621ab5bdf7f87f1ff848f69327`; it was not replayed.
Teaching report SHA-256 is
`357a119d7c75de8508f6990462c2ca6a4ab8b7cf218474a0d84b44ad97226a36`.

The latest Codex teaching boundary was `2026-08-26T22:02:22+03:00`. Activation began only after
the frozen one-hour retention deadline. All nine activation calls exited 0; none was recovered or
replayed. Activation report SHA-256 is
`1a6d3f96f7064a346bb6534f6d42e79dd327dd154a6f6bf9d04dc8fde6fdbd88`.

The exact evaluation phase then ran once. All 18 processes exited 0; none was recovered or
replayed. Evaluation report SHA-256 is
`505baa2d5f854866db3cd92e49f4f338d38c330c64e3ef86bc44f5b844b25cba`.
The final provider-free audit is `complete=true`, `invalid=false`, with 18
`evaluation_complete` lanes, zero lifecycle failures, and zero native-memory write attempts.

The outcome is a valid negative comparison:

- 18/18 lanes correctly abstained;
- 18/18 avoided applying the Atlas procedure or attempting either Atlas command;
- 0/18 passed the full identity/evidence contract;
- no lane read `runbooks/deploy-worker.md` or observed `ORBIT_ONLY_CANARY`;
- every treatment/native matched delta is zero;
- the strict report returns `incremental_value_signal=not_observed` and rejects portable
  incremental value.

The recovery plan's prior `4,575,535` micro-USD already includes inherited lane 1. New recovery
teaching spend was `364,818` micro-USD and evaluation spend was `305,354` micro-USD, for
`5,245,707` cumulative accounted micro-USD under the `7,000,000` ceiling.

The durable comparison report is:

`evals/native_memory_pilot_v1/SCHEMA7_WRONG_SCOPE_REPEATED_RESULTS_2026-08-26.md`

SHA-256:

`e330f5997c3ed009e29043c57b44c9ee17d5d9960d38f809701a94031c485a66`

Never rerun teaching, activation, or evaluation for this plan. The next forward-only slice should
close the host-boundary evidence loop after a safe no-result: absence of remembered guidance is not
itself sufficient task context.

## Continuity invariant

At preparation, authentication-ready, teaching-complete, retention-ready, activation-complete,
evaluation-complete, and reporting boundaries, update this record and the active Engram handoff
with the exact durable paths, hashes, audit state, accounted spend, and next command. Never store
credentials. Never replay, repair, or overwrite a completed or invalid lane.
