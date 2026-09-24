# Schema 7 repeated wrong-scope/no-result pilot — current preparation

Date: 2026-08-19

## Purpose

The completed repeated matched/mismatch matrix proves that the unconditional procedure route can
apply a valid Atlas procedure and reject a current prerequisite mismatch. It does not prove that a
procedure learned in Atlas stays out of an unrelated repository. This successor teaches the Atlas
context-probe procedure and evaluates from Orbit across all six Codex/Claude native, combined, and
lean Engram arms for three repetitions.

Every evaluation contract requires trusted Orbit repository, project, and component identity; a
correlated read of `runbooks/deploy-worker.md` containing `ORBIT_ONLY_CANARY`; abstention; no
returned or applied `procedure-atlas-context-probe-v1`; and no amber or cobalt attempt.

## Superseded zero-execution plans

- `/private/tmp/engram-native-memory-pilot-v1-wrong-scope-final.Nr8Ac7/run-plan.json`, SHA-256
  `e1905971993d72bb4988fccdfcc3950c292bbd5d201d7ff2ad11c6ffc40736d8`, is not executable:
  its frozen Engram binary path no longer exists. Formal attestation rejects it. It must never be
  repaired or run.
- `/private/tmp/engram-native-memory-pilot-v1-wrong-scope-repeated-current.DB8Uvi/run-plan.json`,
  SHA-256 `fc95e16a42cf9233aca434d84aba926065c8579b35b2756a87b3ed5ec4546345`, exposed a preparer
  defect before any provider call. Its Atlas teaching prompts wrote the evaluation contract's Orbit
  remote into repository-scoped procedure memory, manufacturing the leak under test. It must never
  be run or repaired.

## Preparer correction

`prepare_lane` now reads `remote.origin.url` from the materialized teaching checkout and passes that
value to `teaching_prompt`. The Engram write no longer reuses the evaluation contract's expected
remote. Regression coverage proves both that the teaching prompt uses the teaching remote and that
the remote is derived from the actual teaching checkout.

Validation on the corrected source:

- `cargo test -p engram-eval`: 121 passed, 0 failed.
- `cargo fmt --all --check`: passed.
- `cargo clippy -p engram-eval --all-targets -- -D warnings`: passed.
- The repository's `target/debug` remains absent; validation used
  `/private/tmp/engram-schema7-current-check`.

## Current frozen plan

- Protocol:
  `evals/native_memory_pilot_v1/protocol-schema-7-unconditional-route-wrong-scope-repeated-current.json`
- Protocol and frozen snapshot SHA-256:
  `fafcb11112a410cd8de2ea29fb1300bb2a4023d5c0eb3e62f92f816285343d8c`
- Run plan:
  `/private/tmp/engram-native-memory-pilot-v1-wrong-scope-repeated-current-final.KLLOFc/run-plan.json`
- Run-plan SHA-256:
  `7c7a8b46fa7fe0a760097b84965c3d371e0131c126b0c8e7c9a971cbc0276458`
- Frozen evaluator SHA-256:
  `a91b125d0edc92d4cc66ac52f32f63f2b3b2a5b3aa5be3a4bfad594b2988f87d`
- Frozen Engram SHA-256:
  `1bde61b2d999ec55a68e6b3cd9a09be8e4aa23c56bd5bddebe07b8ea9322a322`
- Matrix: 18 lanes, six arms, three repetitions.
- Claude allocation: 100 cents; previously accounted flagship spend: 4,528,914 micro-USD;
  authorized flagship ceiling: 700 cents.

End-to-end inspection finds all 12 Engram teaching prompts scoped to
`git@github.com:acme/atlas.git`, zero Orbit remote occurrences in those writes, Atlas teaching paths
in all 12 writes, and Orbit evaluation checkouts in all 18 lanes. All 18 acceptance contracts
require Orbit/worker identity, abstention, an empty required-context set, and the Atlas procedure as
forbidden context.

Formal runtime attestation reports `verified=true`. The provider-free audit reports
`invalid=false`, 18 prepared lanes, zero failures, and zero native-memory write attempts. No
teaching, activation, or evaluation process has run. All nine fresh isolated Codex homes are still
awaiting Keychain-backed ChatGPT browser login; provider execution remains gated on
`check-native-memory-pilot-auth --require-ready`.

## Execution invariant

After every isolated Codex home is ready, re-attest the frozen plan and repeat the pristine audit.
Only then run teaching, enforce the retention/activation gate, and run evaluation exactly once with
`--approve-provider-execution --confirm-claude-budget-cents 100`. Never replay or repair an existing
lane. Generate the strict comparison report only after a complete, valid audit.
