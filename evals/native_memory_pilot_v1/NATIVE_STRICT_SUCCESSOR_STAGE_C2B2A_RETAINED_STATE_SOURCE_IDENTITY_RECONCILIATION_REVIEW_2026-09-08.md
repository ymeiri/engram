# Native strict successor Stage C2B2a — retained-state source-identity reconciliation review

## Disposition

The exact retained-state source-identity reconciliation below is accepted as a narrow authority
repair, subject to separate exact-byte acceptance of this review record:

```text
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_FROZEN_2026-09-08.md
SHA-256 7156d2fcfc347aef901a1afc4888c6027123a1a41f03046bc7343859435b456f
lines   1566
bytes   103684
ending  one final LF; no BOM, CR, or NUL
```

Three independent technical reviewers each inspected those exact final bytes after all earlier
votes had been invalidated by edits or adversarial correction. Their final outcomes are:

| Final independent reviewer | P0 | P1 | P2 | Verdict |
| --- | ---: | ---: | ---: | --- |
| `/root/launcher_checkpoint_review_a` | 0 | 0 | 0 | clean |
| `/root/launcher_checkpoint_review_b` | 0 | 0 | 0 | clean |
| `/root/launcher_checkpoint_review_a/reconciliation_exact_review_child` | 0 | 0 | 0 | clean |

These are independent agent inspection results. No reviewer is a human or manual authority, no
agent review can manufacture human authority, and no verdict transfers to another path or byte
identity. This record is itself only a candidate preflight-only non-input audit record until its
exact final bytes are separately reviewed and accepted. It is never a runtime or scanner trust
anchor and contains no self-SHA claim or placeholder.

Acceptance of the reconciliation and, separately, this record authorizes only the next bounded
implementation and exact-source review work. It authorizes no combined record, static execution,
build, controller or launcher runtime, provider, VM, pilot, payload, completion, or evaluation.

## Controlling accepted inputs and retained source identities

The final reviews revalidated the reconciliation against these immediately controlling accepted
records:

```text
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
SHA-256 8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
lines   513
bytes   34249

path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md
SHA-256 9cd1ec0211f53e3239bd36a3eb057bbf539e956727ef5f474dc41acc5ec8473f
lines   1297
bytes   88630

path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_REVIEW_2026-09-08.md
SHA-256 26229d625abfe557d4a41c97d3809ce2f8333d8f249bba8151aee319c73168fd
lines   182
bytes   13449
```

All authority inherited through those records remains controlling except for the reconciliation's
explicit narrow supersessions. The exact retained source candidates examined as context were:

```text
path    evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
SHA-256 810051d1990df5875fa0dce6d5b1007ebc221c85c0a7e04050c60e33d782818e
lines   12158
bytes   603827
state   candidate only; IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256 is the 64-zero placeholder

path    engram-eval/native-c2b2a-payload/build-support/controller.py
SHA-256 b11ff52f4f4dc105dc9c146bfc07ee381826b295ecd5aaf7e730acf18487d60c
lines   27118
bytes   1067143
state   candidate only; PREFLIGHT_AUTHORITY_SHA256 is the 64-zero placeholder expression

path    engram-eval/native-c2b2a-payload/build-support/provision.py
SHA-256 aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
lines   4349
bytes   160871
state   retained candidate; no provisioner change is authorized by this reconciliation
```

The launcher and controller candidates are not final source identities, and neither candidate
digest may substitute for `H`, `S`, `A`, `HC`, or another value in the accepted finite
construction. No controller or launcher semantic digest was finalized during these reviews.

## Candidate ratchet

No vote, finding, or hash transferred across a revision. Every predecessor below is rejected
history and authorizes nothing:

| Rejected SHA-256 | Lines | Bytes | Final disposition |
| --- | ---: | ---: | --- |
| `d98a7ef0cc6c12815d44b5ca641c0df199b202194d9f1de37632880d1f9fa6f0` | 760 | 43915 | Rejected: its single source-analysis epoch, proof transport, role derivation, staged digest checks, and finite construction order were incomplete. |
| `bb6371804dfd723b350340803a9e6fc5298204fa309dc706e98d9122b99d4d30` | 1122 | 70215 | Rejected: construction improperly claimed final-only facts, and proof/vector/record allocation bounds were not closed before serialization. |
| `ef0eb9472465c4e0441c14636573834b689e6c936887bf7f2ab6038f72725ca7` | 1309 | 83538 | Rejected: pre-substitution guards, actual-`H` timing, Q-only source roles, and component-first vector sizing were not jointly sound. |
| `17b77b31d4103ebc4cc7b6bb94d3e5fc86fa253fef5c469340a00a503d396b05` | 1414 | 92559 | Rejected: the over-1-MiB direct case could not traverse the inherited generic 1-MiB argument decoder. |
| `2b13c02dd6be1c6d1482cbe22fda1ceac0ca61e716821e6e05003d6bd8d8ac60` | 1506 | 99568 | Rejected: the raw authority grammar could not canonically carry the required empty `expected_hex` for `ValueError`. |

The rejected identities are retained solely to prevent accidental vote transfer or regression to a
superseded design. A clean observation of one rejected revision does not weaken its later union
finding and cannot satisfy any current gate.

The distinct current reviewed identity is:

```text
SHA-256 7156d2fcfc347aef901a1afc4888c6027123a1a41f03046bc7343859435b456f
lines   1566
bytes   103684
status  accepted reconciliation amendment; three independent P0=0/P1=0/P2=0 final reviews
```

That identity is not a predecessor row and is not part of the rejected history above. Its exact
acceptance is the disposition recorded by this review record, subject to this record's own separate
exact-byte acceptance.

## Final independent findings

All three final reviewers found the following exact clauses closed in the accepted reconciliation:

1. **Dual source identity and finite DAG.** The raw launcher digest `H` remains only in the sole
   projected `launcher_source_sha256` field. The tagged semantic digest `S` is computed from exact
   `Q(L)`, which zeroes only the sole semantic-digest literal. The construction is finite and
   ordered through `U -> A -> C -> HC`, `L0 -> S`, proof/vector/record construction, `R0 -> D`,
   `D -> L -> H -> R`, followed by authoritative reconstruction. There is no fixed point, refill,
   retry, record self-hash, or result cycle.
2. **Two source-analysis epochs.** Construction performs one stable, non-authoritative read and AST
   parse of final `C` and final-form `L0`. Authoritative verification later performs one independent
   stable read and parse of final `C` and final `L`, normalizes only a deep-copied sole digest
   constant, rebuilds all carried facts, and requires exact equality plus `Q(L) == L0`.
3. **Q-invariant rows and final binding.** The 26 row recipes serialize only Q-invariant facts.
   Pure non-authoritative `G_D` and `G_H` guards discard unsafe candidates before substitution or
   fill but accept nothing. Final-binding gates independently replay those predicates, bind actual
   `D`, `H`, final `L`, and `R`, and dominate proof-witness, static, and preflight success without
   adding row or vector bytes.
4. **Injective proof transport.** Four exact scalar fields plus 26 indexed row fields carry the
   canonical 27-row proof record. Each of the 26 source vectors maps injectively to its accepted
   proof-row bytes, with exact counts, rows, byte lengths, SHA-256 values, def-use, dominators,
   exits, and controller/launcher reconstruction checked independently.
5. **Roles and global gate ownership.** The exact ordered 26-entry role table uses only
   `controller-source`, `launcher-source`, and `cross-source`. Roles classify only Q-invariant row
   recipes. Transition-guard and final-gate ownership is proven separately by one global source
   predicate, so a row cannot classify or authorize its own final gate.
6. **Component-first caps.** All 200 non-source component tuples and 26 source prefixes are frozen
   and sized arithmetically before serialization. Every proof row, canonical value, complete
   226-vector definition stream, static expected output, and projected record is bounded before
   nested hex allocation. No cap is raised.
7. **Sole compact direct fixture.** Global vector index 186 alone carries the exact 47-byte
   descriptor `c2b2a-retained-state-over-1mib-bytes-fixture-v1`. Only after complete authority,
   vector, proof, and case validation may the launcher materialize one exact 1,048,577-byte builtin
   payload and pass it unchanged to the extracted retained-state parser. Only that target's own
   `ValueError` can satisfy the case; the expanded bytes never enter authority, record, proof,
   vector definitions, or static stdout.
8. **Canonical empty expected bytes.** Raw authority values remain nonempty except for byte-exact
   `static_vector.` plus six ASCII digits plus `.expected_hex` keys. Empty bytes are canonical only
   for `ValueError`; return vectors require nonempty expected bytes, `args_hex` is always nonempty,
   and the bidirectional invariant dominates definition comparison, materialization, dispatch, and
   static success.
9. **Closed inventories and output.** The exact inventories remain 19 directly extracted codec
   cases, 26 non-executing source cases, and 45 total reconciliation cases within 226 total static
   vectors. Static stdout remains exactly 229 LF and contains no literal `S`, raw-`H` reference, or
   proof fact; only the accepted definition and expected-output digests may depend transitively on
   proof rows.
10. **Bounded audit records and unchanged runtime anchors.** The preflight-only non-input audit
    list expands from exactly 20 to exactly 22 by appending the reconciliation and, after separate
    acceptance, this review in fixed order. The six runtime trust anchors remain byte-for-byte and
    position-for-position unchanged. Neither new record is a child, gate, scanner, completion,
    launcher-runtime, build, provider, VM, pilot, payload, or runtime input.
11. **Non-expansion.** The reconciliation changes only source identity, static proof transport, and
    bounded preflight review authority needed for the retained-state carrier. It adds no provider,
    VM, pilot, connector, network, authentication, datastore, daemon, recovery, user-data,
    provisioner, payload, build, or runtime authority.

## Review activity and side effects

The three named reviewers performed independent read-only exact-byte inspection and adversarial
cross-checking. Earlier provisional votes were invalidated whenever a later finding or edit changed
the applicable bytes; each final `P0=0/P1=0/P2=0` outcome applies only to the exact reconciliation
identity recorded above. The review record reports those outcomes without treating an agent as a
human reviewer or manual authority.

Review and authoring activity was limited to reading Markdown and relevant source text, SHA-256 and
LF/byte counts, bounded text searches, arithmetic checks, filesystem metadata checks, and creation
of this Markdown record. Read-only Git queries may have refreshed Git metadata. No reconciliation,
accepted authority, controller, launcher, provisioner, or other source bytes were edited while
creating this record.

No controller, launcher, provisioner, extracted FunctionDef, source mutation, static vector,
combined record, compiler, Cargo command, build, completion scanner, provider, VM, pilot, payload,
or runtime mode was imported, compiled, or executed. No combined-record evidence, final controller
authority digest, final launcher semantic digest, static proof result, runtime result, provider
result, VM result, pilot result, or evaluation result was produced. Repository `target/debug`
remained absent.

## Remaining gates

1. Review and accept this record by its exact external path, SHA-256, LF count, byte count, final
   LF, and clean independent outcomes. Any edit restarts that record-review ratchet.
2. Append the accepted reconciliation amendment path and this accepted review-record path, in that
   order, to the exact bounded non-input audit-record list, yielding exactly 22 entries before any
   implementation begins. The unchanged six runtime anchors remain separate.
3. With one editor, implement the accepted reconciliation and retained-state amendment and construct
   the complete bounded controller, launcher, and combined-record candidate through the exact finite
   `U/A/C/HC/L0/Q/S/W0/F0/components/V/O/R0/D/G_D/L/H/G_H/R/P` sequence plus authoritative
   `W1/F1` reconstruction and every final-binding gate, obtaining the final `C`, `L`, and `R` bytes.
   Do not widen the provisioner or another scope.
4. Only after those final bytes exist, obtain fresh independent exact-source and full-ratchet
   reviews of every final controller, launcher, and combined-record byte and every proof surface,
   transport, cap, dispatcher, dominance edge, and success-exit dependency. Any edit restarts the
   applicable exact-byte review ratchet; no candidate hash or prior verdict transfers.
5. Only after those reviews are accepted may the combined preflight run, and it must receive its
   own separate acceptance.
6. Only after combined-preflight acceptance may the separately authorized static proof run, and it
   too must receive its own separate acceptance.
7. Only after all applicable preflight and static gates pass may the specifically authorized
   runtime, provider, VM, pilot, and evaluation stages proceed.

Implementation acceptance, build/source identity, production completion, runtime qualification,
C2B2a evaluation, native-memory pilot acceptance, and the Engram flagship goal all remain separate
future gates. This record satisfies none of them.
