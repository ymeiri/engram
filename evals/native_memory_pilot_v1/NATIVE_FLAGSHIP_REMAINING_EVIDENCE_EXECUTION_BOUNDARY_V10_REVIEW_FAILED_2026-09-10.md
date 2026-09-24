# V10 qualifying-review failure record

Date: 2026-09-10
Status: frozen append-only rejection evidence; no authority

## Reviewed identities

~~~text
V10
path   evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V10_FROZEN_2026-09-10.md
sha256 11c8471c0a5af4181219684ab012de25bd319f46ea9c21b620834837f3d29bec

schema11
path   evals/native_memory_pilot_v1/protocol-native-stale-safety-v2-schema-11-file-cache.json
sha256 462a64698c1f8ee692f55b5691f802821e8aa48ed4b3260fd713498031dc7693

O0 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
O1 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
O2 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
O3 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
O4 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
O5 29527306ed92c33c64f4a0ae29af88e3469fb69ca45041c20105094c0c009e1d
~~~

The digest on each line is the coordinator-observed V10/schema11 identity-pair digest. All six
pairs were identical.

## Qualifying reviews

- Semantic task "/root/v10_semantic_review": **FAIL**, P0=0, P1=4, P2=0.
- Operational task "/root/v10_operational_review": **FAIL**, P0=0, P1=5, P2=1.
- Claude bridge job "ccb_20260910184758_faf5628d", model "claude-opus-4-6",
  isolated/read-only: **PASS**, P0=0, P1=0, P2=3.

Bridge job "ccb_20260910184654_9f8bb270" failed with an empty prompt and produced no review output.
It is nonqualifying and contributes no finding, vote, or evidence.

## Nine unique blocking findings

1. **S0 — G10 imports undefined historical criteria.** A standalone boundary cannot require
   “original” criteria without an ordered canonical criteria array and deterministic predicates.
   Disposition: unresolved P1 in V10; schema12 and any execution-boundary successor must bind the
   complete closure.
2. **S1 — Missing-source and expiry boundaries are not mutually exclusive.** The missing-source
   case has no bound verification lifetime that remains strictly unexpired through the epoch.
   Disposition: unresolved P1; schema12 must reject overlap, equality, or unavailable clocks.
3. **S2 — Combined missing-source pass does not attest native contribution.** An Engram diagnosis
   plus stored-but-unused native markers can pass. Disposition: unresolved P1; typed retrieval and
   case correlation are required, while component attribution must use companion lanes.
4. **S3 — Bound identities do not enforce arm equivalence.** P/R4 lack same-host equality modulo a
   closed memory-layer delta and lack a closed cross-phase delta. Disposition: unresolved P1.
5. **O0 — The authority source and irreversible event consumption are implicit.** V10 does not
   bind a concrete Codex-host authority adapter, locator/envelope verifier, trust model, or an
   external O_EXCL replay ledger. Disposition: unresolved P1.
6. **O1 — Engine terminal receipts are not fully canonical.** Q and execution omit exhaustive
   typed completion/unknown preimages, optional encodings, enums and terminal head transitions.
   Disposition: unresolved P1.
7. **O2 — Authentication admits an unsafe copy fallback.** Cross-boot exact-path deletion retains
   ABA/reopening risk and contradicts the zero-copy/NOT_CREATED outcome. Disposition: unresolved
   P1; a later execution-boundary successor must require broker zero-copy or make Codex
   ineligible before U1.
8. **O3 — Broker client identity permits a confused deputy.** “Claim-bound client” lacks a
   concrete peer transport, one-connection lifecycle and proof that hostile tools cannot cause an
   authenticated request. Disposition: unresolved P1.
9. **O4 — Sanitized projection lacks a safe tree language.** Model-controlled path hierarchy,
   links, special files, ACL/xattr and held-FD traversal/promotion are not closed. Disposition:
   unresolved P1; V11 must bind regular-tree-v1 and its adversarial corpus.

The operational request to import historical criteria duplicates S0 and the causal-isolation and
same-host-equivalence observations duplicate S1–S3; those duplicate observations do not create
additional unique P1s.

## Nonblocking P2 dispositions

- **O5 — U1 acknowledgement excludes NOT_CREATED.** Accepted nonblocking P2; a later
  execution-boundary successor must make all six Codex dispositions exactly NOT_CREATED.
- **C0 — causal_result_precedence is orphaned.** Accepted nonblocking P2; schema12 must wire an
  exhaustive rank rule or remove the field.
- **C1 — retention_absent_signal conflicts with per-case memory_absent_signal.** Accepted
  nonblocking P2; schema12 must use one exhaustive signal mapping.
- **C2 — Docker physical-write ceiling is unenforced and unbounded.** Inapplicable-with-clause:
  retain the explicit nonclaim, add a host disk abort floor, and never report a hard physical
  Docker-write bound.

## Decision

G0 failed because nine P1 findings remain unresolved in V10. V10/schema11 are immutable rejected
evidence and grant no construction, implementation, qualification, authentication, Docker,
provider, candidate, cleanup, deletion, adapter, or pilot authority. No A0/U1 or local receipt can
be derived from this record.
