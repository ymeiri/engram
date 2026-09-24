# Native strict successor Stage C2B3 — correction-journey research review

Date: 2026-09-06 (Asia/Jerusalem)

## Disposition

The C2B3 correction-journey research record is review-safe as non-authorizing sequencing and
future-freeze input. It is not a C2B3 design freeze, implementation, acceptance record, mutation
authority or evidence of flagship completion.

The exact reviewed record is:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B3_CORRECTION_JOURNEY_RESEARCH_2026-09-06.md
SHA-256 7f7ddd980b398077bcefb456798636b2e8d6f6f7375da2c1ee48237746bb2805
```

Three independent reviewers verified that SHA-256 before and after their final exact-source audits,
made no edits and found no remaining issue at any tracked severity:

| Review | P0 | P1 | P2 | Verdict |
| --- | ---: | ---: | ---: | --- |
| inherited C2A/C2B1/C2B2 contracts | 0 | 0 | 0 | review-safe as research only |
| correction/deletion lifecycle | 0 | 0 | 0 | review-safe as research only |
| adversarial witness and causal authority | 0 | 0 | 0 | review-safe as research only |

All pinned source hashes embedded in the research record matched during the final reviews.

## Audit ratchet

Each audit round held the record byte-for-byte stable. Findings were consolidated only after every
reviewer of that hash had finished, then all reviewers were rerun against the new hash.

| Research SHA-256 | Inherited P0/P1/P2 | Lifecycle P0/P1/P2 | Adversarial P0/P1/P2 |
| --- | --- | --- | --- |
| `39ffc4df4493cb04332ca5055ca2bb1e29e59f01cdfd8fd984bb8d4bbbe89e08` | 0/3/2 | 0/4/4 | 0/5/3 |
| `55160652771afacb805fc9a27bdd4945b0bfadd067dba775602d6a3ecad613c9` | 0/0/4 | 0/2/3 | 0/3/3 |
| `9a31a7d0840c6844e46e73db32acbf95c6f1e8a6adaefcbf7d0869d5fe18f77a` | 0/2/0 | 0/4/2 | 0/1/0 |
| `6819e48192d34e51387d267a9b7ac7c80ffd16b883dec5eb057bfe52adefda31` | 0/1/0 | 0/1/1 | 0/1/1 |
| `cdb17a5c3f41b540c1044821755b4f9caa3c5d84c3a83027a0cb85eba681871a` | 0/2/1 | 0/0/1 | 0/2/0 |
| `7f7ddd980b398077bcefb456798636b2e8d6f6f7375da2c1ee48237746bb2805` | 0/0/0 | 0/0/0 | 0/0/0 |

The ratchet closed these material attack paths:

- promotion of a replayable C2B2 structural receipt into live C2B3 authority;
- omission of the initial-frame-derived expected-generation witness, which could let a different
  but C2A-valid generation become its own baseline;
- cross-journey replay through a marker keyed by journey ID instead of store generation/proposal;
- idempotent `AlreadyApplied` being mislabeled as causal success;
- a journal being made durable before the action frame whose digest it records;
- early or duplicate semantic reopen and action handles surviving until process death;
- no forensic post-read authority after timeout, malformed output, EOF or executor death;
- an applied forensic observation upgrading an uncertain action into success;
- overlapping post-state classes and incomplete action/state outcome coverage;
- contradictory cleanup of a deliberately retained host deny marker;
- cleanup failures that could not reach a truthful terminal state;
- early, reusable or caller-supplied late liveness challenges;
- whole-state equality being inferred from incomparable fresh-key MACs;
- evaluator-only SQL being presented as proof of production Engram behavior; and
- overclaiming authenticated human intent or reviewer authority.

## Reviewed direction

The future C2B3 freeze may cover exactly one pending-to-applied correction journey. Its proposed
proof composes:

1. an atomic host deny marker keyed across journeys by exact store generation and proposal;
2. a live C2B2-prefix handoff before receipt minting or cleanup;
3. exact comparison of the writer-derived expected generation with the same pre-acquisition later
   consumed by C2A;
4. one long-lived semantic authority retaining C2A's move-only pending binding and a private complete
   nine-table witness;
5. one phase-gated production `MemoryService::apply_correction` child, with a live release probe;
6. a closed `AppliedNow`/`AlreadyApplied`/rejection/uncertainty result contract specified by the
   freeze before production implementation changes;
7. one observation-bound post-read permit whose uncertainty can never be upgraded;
8. exact selected-row transition plus unchanged complete unrelated raw state;
9. late host liveness binding; and
10. consuming cleanup with truthful partial-failure results and stable guest/private-disk absence.

The production action choice deliberately requires a separately frozen and reaccepted C2B2 delta
for the internally constructed local Rocks `Surreal<Any>` action origin. Pre/post collectors remain
on the direct-Core boundary. Daemon, MCP, caller endpoints, live user stores and remote engines stay
outside the slice.

## Deletion boundary

Confirmed forget remains a separate later successor. Current source has blockers that C2A and C2B3
cannot honestly cover:

- an unlocked obsolete row can retain `snapshot_digest = NONE` after its pending replacement is
  forgotten;
- cleanup is sequential and can race a new reference without exclusive writer authority;
- cleanup targets do not include every deleted proposal ID;
- `knowledge_commit`, `brain_harness_trace`, `agent_feedback`, derived results and vault files lie
  outside C2A's three-table deletion checks;
- vault refresh failure occurs after the retry receipt is removed;
- forgetting a third referenced memory can leave an applied correction pair internally stale; and
- exact archive/forget request IDs do not establish strict project/repository/task identity or
  authenticated human intent.

No correction, deletion or live-store operation was performed by this research or review.

## Sequencing and current executable gate

The mandatory order remains:

1. finish and accept C2A;
2. implement and accept the frozen C2B1 slice;
3. freeze, review, implement and accept C2B2;
4. freeze, review, implement and accept C2B3 from this research; and
5. separately freeze and prove deletion propagation.

C2A remains one frozen integration test from acceptance. Its exact design and implementation hashes
were revalidated after the C2B3 audits:

```text
C2A frozen design
  5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc

engram-eval/src/native_successor_semantic.rs
  4da0ce228b5127587a72f65d19efd58ddbc7d6e7d00dc98a41117ff18dbce4b5

engram-eval/src/lib.rs
  2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74

Cargo.lock
  8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e
```

At the final review-state check, `/private/tmp` had 1,322,132 KiB available versus the mandatory
19,427,004 KiB reserve. Repository `target/debug` was absent. The verified directory
`/private/tmp/engram-claude-host-action-target-20260904-01` remained untouched and occupied
20,781,808 KiB. No build or test was attempted below the reserve; that is enforcement of the frozen
gate, not acceptance evidence.

## Nonclaims

This review proves only that the named research record survived three independent exact-source
audits without remaining P0, P1 or P2 findings. It does not prove or authorize C2A acceptance,
C2B1/C2B2/C2B3 implementation, correction or deletion execution, production API changes,
persistence, crash durability, adapter behavior, provider behavior, authenticated human identity,
live-store safety, complete erasure or flagship completion.
