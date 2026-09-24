# Schema-8 repeated source matrix prepared — 2026-08-18

The three-repetition matched/mismatch reliability plan is frozen at:

`/private/tmp/engram-native-memory-pilot-v1-source-repeated-final.GoFlUi/run-plan.json`

Run-plan SHA-256:

`7a763b94d0a66f15e26909d00e9b234ec035331339343614f573ec3a14acec17`

The protocol is
`protocol-schema-8-source-backed-matched-mismatch-repeated-forward.json`, SHA-256
`fd583a41f230af3946207cf853f909819da5e30db3cbf9e2dcc508b41748d8e7`.
It repeats both the moved-checkout source-matched execution case and the legacy-checkout
source-mismatched abstention case three times across native memory, lean Engram, and combined
memory on Codex and Claude Code.

The 36 fixed lanes contain every case/arm/repetition triplet exactly once:

- 12 lanes per repetition;
- 18 matched and 18 mismatched lanes;
- 18 Codex and 18 Claude Code lanes;
- 18 `matched` and 18 `mismatched` frozen source observations;
- only the two expected source hashes from the single-repetition matrix.

The run order rotates the existing counterbalanced 12-lane order between repetitions. It does not
reuse a host home, Engram project, fixture tree, native-memory directory, trace, or output across
lanes.

The immutable evaluator is
`/private/tmp/engram-forward-safety-final.INjO9A/engram-eval`, SHA-256
`fa030da18c9935eaa429a9e6f90b3dfe3a5f792d1cd2d5d08ba241facda6a9e5`.
Exact runtime re-attestation reports `verified=true`. The untouched provider-free lifecycle audit
reports `invalid=false`, 36 `prepared` lanes, zero failures, and zero native-memory write attempts.
All 107 evaluator tests, Clippy with warnings denied, formatting, protocol uniqueness, and frozen
source-status/hash checks pass.

The plan conservatively accounts for the full allocations of the single-repetition source matrix,
missing-source slice, and wrong-scope/no-result slice: prior spend is 3,752,912 micro-USD. It
allocates at most $3.00 additional Claude spend under a $7.00 cumulative ceiling. Its 36 Claude
teaching/evaluation calls each receive a $0.083 turn-boundary allocation.

No provider phase or login-status command ran for this repeated plan. Its 18 Codex homes were
freshly created during provider-free preparation and will require an explicit authentication plan
before execution. Do not begin this larger run until the earlier single-repetition source matrix is
complete and its result justifies the added provider and operator burden.
