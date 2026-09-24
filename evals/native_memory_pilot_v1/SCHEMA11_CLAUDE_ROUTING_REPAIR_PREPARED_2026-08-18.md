# Schema-11 Claude routing-repair diagnostic completed — 2026-08-18

This forward-only diagnostic is frozen at:

`/private/tmp/engram-native-memory-pilot-v1-claude-routing-repair.K3yvnj/run-plan.json`

Prepared run-plan SHA-256:

`083987dd34ded5daf3248ca34071d4574d7e31a1e9b06847c34e1dbb77e6d82a`

The plan contains the complete source-backed matched/mismatch case pair across Claude Code native
memory, lean Engram, and combined memory: six fixed lanes, one repetition per case/arm pair. It is
the smallest fresh-host slice that can distinguish the source-level Claude routing repair from
native-host behavior without replaying any completed schema-8 lane. There are no Codex lanes, no
isolated Codex homes, no Codex authentication prerequisite, and no activation phase.

This is a host-boundary diagnostic, not a portability experiment. Even if either Engram treatment
dominates Claude native memory, schema 11 and the report implementation prohibit a portable-value
claim because the only observed host is Claude Code. A positive result may justify a later
cross-host repeated matrix; it cannot establish cross-host portability or statistical power.

## Frozen identities and provider-free gates

- Protocol:
  `evals/native_memory_pilot_v1/protocol-schema-11-claude-routing-repair-diagnostic.json`, SHA-256
  `52ca2924dd74209910e4941d692e202791f2661ccc84a9ba48bcef14cf7af9d4`.
- Engram: `3d6b54fb0833812267c5e3d22e1e27b974004f432dfd7f8341a8db768dbbaa74`.
- Evaluator: `98001004450bcacbdd85dcf4b87233cab1980bb8d36e56b7e574dab8be73b2f1`.
- Claude Code 2.1.234:
  `08d8700313697cbe730a25420c908a299ce52d56f0eb2cf4fac94cab5109bc57`.
- Codex 0.148.0-alpha.9, attested but not used by a lane:
  `6170ff5578170ee9b74ad92bfcff96e6186f41d02b60815a7c2b01ad424c754f`.
- Generated Claude adapter:
  `506bb15eff09570d2db5bfdd55656d47e3ca951ded6d41ccc155e424035d8382`.
- Effective six-tool schema:
  `75d95bd3ee536df30243d562396cf710b828bdd97ecccdab15f97d0921bee0eb`.
- Effective agent instructions:
  `abd6bdb2d9d840c403a65cd47d85f2560b0bc592104c12ebadd2185c67046d09`.

Runtime attestation reports `verified=true`, including the effective six-tool MCP contract and its
restricted-tool and review-authority rejection probes. Provider-free authentication readiness is
`ready=true` with an empty Codex-lane list. The untouched lifecycle audit reports `invalid=false`,
six prepared lanes, zero failures, zero native-memory write attempts, and no teaching, evaluation,
or procedure-verification trace yet.

The schema-11 evaluator change passes all 114 `engram-eval` tests, focused schema/report/auth
regressions, Clippy for all evaluator targets with warnings denied, and repository formatting.
The broader source repair separately passes 271 `engram-index` tests (one model-download test
ignored), all 57 `engram-cli` tests, affected-package Clippy, and formatting. Repository
`target/debug` remains absent; the attested binaries live only in the isolated candidate target.

## Budget and authorization boundary

Prior accounted Claude spend is 2,266,166 micro-USD. The plan allocates at most $1.00 additional
Claude spend and freezes a $4.00 cumulative ceiling. Its 12 Claude teaching/evaluation calls each
receive a $0.083 turn-boundary allocation. The user's standing authorization for AI execution and
sufficient budget satisfies the provider-execution gate, while exact budget, hash, lifecycle,
trace, and outcome checks remain mandatory.

No live `.claude`, `.codex`, settings, hook, skill, or daemon installation is part of this run. The
diagnostic uses generated lane-local adapters only. Completed predecessor lanes and traces must
never be repaired, replayed, or modified.

## Completed lifecycle and immutable outputs

Teaching ran exactly once from `2026-08-18T15:56:01+03:00` to
`2026-08-18T15:59:23+03:00`; evaluation ran exactly once from
`2026-08-18T16:01:49+03:00` to `2026-08-18T16:04:06+03:00`. All 12 provider calls exited 0,
none used a recovered trace or a turn-boundary budget exit, and no lane was repaired or replayed.
All four Engram candidates passed exact trusted verification before evaluation. Claude Code
generated substantive native memory in both native-only lanes and in neither combined lane.

The immutable report identities are:

- teaching runner: `314f79042b1a68d7fbada886df548f7bd825e69e6df30d48d9fc1e0860ddbf46`;
- evaluation runner: `db8d74e12a3b05ecd05dec4f0d571bfa668bcd6ba4f854568210902db3b99e69`;
- completion audit: `0410c5a5e3a6684975aa0f0afb830a6da419a6596a6f48308cf4b3287b226a5d`;
- comparison report: `aa4103801285148b23c247876f2fc4ff9378d700e156731a30f004ee6ca1648a`;
- strict comparison output:
  `4ac560d881957566d56852a1e3098f551879cf2d95c45aee69680489a5c8f42a`, exiting 1 as required
  because portable incremental value was not observed.

The final runtime attestation remains `verified=true`. The completion audit is `complete=true`,
`invalid=false`, with six `evaluation_complete` lanes, zero integrity failures, and zero
native-memory write attempts. `all_acceptance_passed=false` is an outcome, not an integrity defect.

Teaching cost 260,488 micro-USD and evaluation cost 163,229 micro-USD. The diagnostic therefore
used 423,717 micro-USD and brings cumulative accounted Claude spend to 2,689,883 micro-USD, below
the frozen 4,000,000-micro-USD ceiling.

## Outcome

The fresh diagnostic does **not** validate the Claude routing repair. Every layer passes zero of
two full contracts. Claude native memory gets both task outcomes right: it executes cobalt in the
matched checkout and abstains in the mismatch checkout. Lean Engram and combined memory each get
only the mismatch outcome right and incorrectly abstain in the matched checkout. Both treatments
therefore regress outcome correctness, first action, procedure revalidation, and successful
execution relative to native memory. The comparison signal is `not_observed`, and the report also
enforces the independent single-host limitation.

| Lane | Layer | Case | Full pass | Outcome correct | Procedure revalidated | Observed behavior |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | both | mismatch | no | yes | no | oriented, inspected only the remote, then abstained |
| 2 | native | matched | no | yes | yes | ran cobalt successfully; missed identity/context evidence fields |
| 3 | Engram | mismatch | no | yes | no | used `memory(list)` instead of `procedure_match`, then abstained |
| 4 | both | matched | no | no | no | stopped after orientation and incorrectly abstained |
| 5 | native | mismatch | no | yes | yes | safely abstained; missed identity/source evidence fields |
| 6 | Engram | matched | no | no | no | stopped after orientation and incorrectly abstained |

Individual Engram results remain below the existing 8,192-byte reference ceiling: the maximum is
3,200 bytes. Treatment interactions use one or two Engram calls and return 3,178–3,593 aggregate
bytes. This supplementary telemetry does not alter acceptance.

## Trace-grounded root cause and next boundary

The earlier project-ambiguity defect is no longer the decisive failure: all four treatment lanes
resolve the `atlas` repository and `queue-worker` component, and the orientation response explicitly
says project ambiguity does not block repository-local procedure matching. The remaining trigger is
too lexical and conditional.

The generated adapter and orientation response require `procedure_match` when the user asks for an
“earlier, previous, remembered, or learned” procedure. The frozen task instead says to handle the
context probe “using durable procedure memory.” In lanes 4 and 6, Claude concludes that no specific
learned procedure was requested and stops after `orient`. In lane 3, Claude says it will check
procedures but calls `memory(action=list)`; the empty list is then treated as proof that no procedure
exists. Lane 1 supplies a shortened prompt to orientation and also never reaches matching. None of
the treatment lanes calls the verified matcher with the exact user request.

The next provider-free repair should remove host-side intent classification from this critical
path: after orientation, every actionable repository task must perform one bounded repository-local
`procedure_match` with the full exact user request before command exploration, regardless of
whether the user says “remembered” or “learned.” `memory(list)` must be stated as not substituting
for procedure matching. Project ambiguity continues to block only project/task-scoped memory. This
is a source/test change first; it does not authorize a live adapter installation or replay of this
completed diagnostic.
