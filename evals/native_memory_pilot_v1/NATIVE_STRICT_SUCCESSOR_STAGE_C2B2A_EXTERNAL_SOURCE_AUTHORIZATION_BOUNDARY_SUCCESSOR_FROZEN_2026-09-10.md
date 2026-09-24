# Native strict successor Stage C2B2a — external source-authorization boundary successor

Date: 2026-09-10
Status: successor frozen candidate design amendment; no implementation or execution authority

## Disposition and rejected predecessors

This amendment replaces recursive launcher self-authentication as the source of permission to
execute with one externally rooted exact-hash authorization chain. Candidate self-checks remain
fail-closed regression evidence: a failure blocks the candidate, but success never grants
execution.

The immediately preceding corrected-amendment epoch is rejected and preserved byte-for-byte as
raw historical design evidence:

```text
SHA-256 7d0ae76f3fd721d61fc3797d3627c4cc4a99d413e7f2c39e4ae47417dc42ec80
lines   556
bytes   36410
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

That epoch left phase authorization ambiguous across potentially phase-specific manifest
identities; did not make the fixed static/production record pair, deduplicated inputs, and disjoint
output namespaces one immutable reviewed object; and did not make use of the wrong record, replay,
or union-widening terminal. It did not require final post-run revalidation of the held E and every
E-bound authority artifact before acceptance. It incompletely stated run-plan/M/E acyclicity,
outer-DAG final-L identity after Q fill, and exact invalidation, called the gate-one invocation a
"b5 static run" even though the executable source must be final L, and required sharper reviewer
wording so no new review content/hash/verdict/finding can enter candidate runtime. These are the
three reviews' authority-boundary findings; the 7d bytes and reviews cannot authorize this
successor.

An older rejected epoch was recorded with SHA-256
`1cd2c842fa4fa25415b03623d71458f0377c7cbd73d0de40b1bfb0a79cd5f689`, 566 lines, and 38,215
bytes. Its raw bytes are no longer available at the recorded path because that path now contains
the preserved 7d epoch. Only the 1cd digest and sealed review trace remain. They are provenance,
not re-reviewable raw evidence, and confer no authority. The 1cd epoch was cyclic about
manifest/bootstrap authority, claimed an impossible complete runtime closure, required a launcher
snapshot incompatible with current b5, misstated review visibility in the audit parent, and treated
an internally expected stdout as external authority.

This successor amendment is itself only a candidate. It authorizes no source import, compilation,
AST parse, candidate/static/provider/pilot execution, Q fill, build, VM, daemon, adapter, datastore,
or runtime action until its exact final bytes and separate design review are accepted below.

## Exact inherited authorities

This amendment is subordinate to these exact frozen records and exact reviews except only for the
named clauses in the supersession matrix:

```text
SHA-256 8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
lines   513
bytes   34249
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md

SHA-256 352b47668388c8c8a7105e579597fcc41f0e11e202f7cb03f68b8e798b6af82e
lines   133
bytes   8512
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_REVIEW_2026-09-07.md

SHA-256 7156d2fcfc347aef901a1afc4888c6027123a1a41f03046bc7343859435b456f
lines   1566
bytes   103684
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_FROZEN_2026-09-08.md

SHA-256 9473072aaa88fc536df81495491521bd08cc04e642bffbbf33677f70bc9019b2
lines   217
bytes   13955
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_REVIEW_2026-09-08.md

SHA-256 9cd1ec0211f53e3239bd36a3eb057bbf539e956727ef5f474dc41acc5ec8473f
lines   1297
bytes   88630
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md

SHA-256 26229d625abfe557d4a41c97d3809ce2f8333d8f249bba8151aee319c73168fd
lines   182
bytes   13449
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_REVIEW_2026-09-08.md
```

The exact current source candidates remain:

```text
SHA-256 b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
lines   34415
bytes   2096688
path    evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
state   current historical/candidate bytes; not authorized to execute

SHA-256 eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
lines   27130
bytes   1067693
path    engram-eval/native-c2b2a-payload/build-support/controller.py
state   current candidate; PREFLIGHT_AUTHORITY_SHA256 is still the zero placeholder expression

SHA-256 aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
lines   4349
bytes   160871
path    engram-eval/native-c2b2a-payload/build-support/provision.py
state   current retained candidate; unchanged by this amendment
```

This successor amendment's exact path is:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md
```

Its one prospective design-review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_REVIEW_2026-09-10.md
```

That review record does not yet exist. It is created once, before `R0`, only after three clean
same-hash design attestations; it is then frozen and never modified.

## Why candidate self-proof cannot bootstrap trust

Two independent reviews of exact launcher `b5e50318...` passed their deliberately narrow scopes.
One verified the six handwritten final-helper patterns against the six authentic helper bodies.
The other verified the matcher, literal authentication, six-row identity/order,
signature/body/lexical checks, and checked final-contract call placement. Those remain useful
findings about exact b5 bytes; they were not whole-program authorization verdicts.

A broader adversarial review found a P0 boundary failure. A later reachable path could replace an
authenticated callable's `__code__`, leaving its checked source body intact while changing the
effective runtime object. A separately reachable shadow of builtin `ValueError` could change
exception classification, catch behavior, or traceback provenance while the narrowly checked
helper still matched. The scoped passes and P0 do not conflict: they concern different closure.

A validator co-editable with the source it validates can be altered together with its matcher,
expected values, receipt, or effective objects. It cannot make its own success an external fact.
Every recursive source proof in b5 therefore remains rejection-only evidence.

## Threat model and explicit nonclaims

The terminal trust root is an honest, uncompromised trusted coordinator holding accepted reviewer
state and supplying one exact envelope hash through an out-of-band channel. The design also assumes
collision-resistant SHA-256, an honest host, honest guard/bootstrap, honest pinned interpreter, and
reviewers who inspect the named exact bytes rather than candidate claims.

This unprivileged design does not defend against:

- a SHA-256 collision, second preimage, or compromised hash implementation;
- a compromised coordinator, reviewer, guard, bootstrap, interpreter, kernel, hypervisor, dynamic
  loader, shared cache, filesystem, or administrator;
- colluding reviewers, or a reviewer who does not inspect the stated exact inputs;
- a same-UID debugger, injection/tracing facility, writable process memory, or equivalent host
  mechanism;
- same-UID pathname swap-and-restore between accepted pre-run and post-run observations;
- ambient same-UID filesystem access by malicious exact code in the absence of an OS sandbox;
- unpinned standard-library files, dyld/shared-cache components, kernel behavior, or other ambient
  host TCB beyond the inherited exact anchors and explicitly declared inputs; or
- semantic correctness merely because exact bytes have an accepted hash.

Owner-only files and directories reduce accidental exposure; they are not authentication against
the same UID or root. Pre/post path validation proves only the boundary observations, not what a
malicious same-UID adversary exposed during the interval.

No detached-signature claim is made because no independently protected signing key exists in this
authority. Under the inherited honest-host assumption, coordinator delivery of an accepted exact
envelope hash is sufficient. Optional signature provenance requires a future accepted external-key
bootstrap; a key stored beside candidate data adds no authority.

## Acyclic external authority chain

There is exactly one canonical immutable declared-input manifest `M` for this authorization epoch
and exactly two canonical external authorization envelopes, `E1` and `E2`. M contains a shared
deduplicated input table followed by two fixed-order invocation records: record 1 is `static` and
record 2 is `production`. Both envelopes bind the same exact M hash. E1 selects only record 1; E2
selects only record 2 and binds the accepted E1/static evidence. No second manifest exists.

The authority chain is exactly:

```text
terminal trusted coordinator
  -> exact SHA256(E1) delivered out of band
  -> stable-opened, held, canonical E1 selecting M record 1 (static)
  -> {one SHA256(M), guard/bootstrap identity, SHA256(run plan),
      successor-amendment/review hashes, epoch, static phase/profile,
      sealed gate-one reviewer-attestation hashes}
  -> stable-opened, held, canonical M
  -> deduplicated declared-input table + fixed invocation records [static, production]
  -> exactly one guarded static invocation using record 1
  -> E1-bound immutable static trace and gate-two attestations
  -> exact SHA256(E2) delivered out of band
  -> stable-opened, held, canonical E2 selecting unchanged M record 2 (production)
  -> {same SHA256(M), same guard/bootstrap and run-plan identities,
      same successor-amendment/review hashes, E1 hash, static-evidence hashes,
      epoch, production phase/profile, sealed gate-two reviewer-attestation hashes}
  -> exactly one guarded production invocation using record 2
```

The terminal coordinator and its out-of-band delivery are the trust root under the honest-host
assumption. Neither expected `SHA256(E1)` nor expected `SHA256(E2)` is read from a caller file,
candidate output, M, either E, run plan, environment, provider output, or candidate-derived record.

E1 and E2 use exact schema `c2b2a-external-authorization-envelope-v2`. Their later frozen canonical
grammar has fixed-order fields for schema, envelope sequence, predecessor-envelope hash or exact
root sentinel, authorization epoch, selected record index/name, phase/profile, manifest path/hash,
guard/bootstrap identities, run-plan path/hash, this successor amendment path/hash, its
design-review path/hash, ordered sealed-attestation path/hashes, selected evidence path/hashes,
optional external-oracle path/hash, creation policy, and terminal LF. E1 has sequence 1, root
sentinel, selected record `1/static`, and no static-evidence fields. E2 has sequence 2, predecessor
`SHA256(E1)`, selected record `2/production`, and the complete immutable E1-bound static-evidence
inventory. Neither envelope contains a self-hash or binds the out-of-band E-hash parameter used to
open it.

E1 and E2 are intentionally excluded from M. M excludes the run plan, guard/bootstrap, this
successor amendment and design review, gate attestations, static evidence, and every object
containing an M, E1, or E2 hash. The sole genre exception is the inherited exact six
amendment/review runtime anchors: final L is already specified to read those files, so M inventories
them under distinct role `inherited-runtime-anchor`, not as external-envelope authority. The new
successor amendment/review are not in that six-anchor set.

The run plan, M, E1, and E2 are acyclic by construction. The run plan never embeds an E1, E2, or M
hash. It predeclares exactly the two phases, fixed record identifiers, modes, mutually disjoint
output namespaces, bounds, and append-only successor creation. M does not embed the run plan or any
envelope/attestation/evidence hash; it binds only its independently read input table and the two
literal invocation records. E1 and E2 each bind the already-frozen run-plan hash and same M hash.
E2 may point backward to E1 and E1-bound evidence; no object in that predecessor set points to E2.

The bootstrap invocation is later frozen as one exact argv, environment, cwd, umask, and FD grammar
containing exactly one lowercase-64-hex out-of-band E-hash parameter in one fixed position. The
grammar, not an envelope, fixes that parameter's shape and location. Only the terminal coordinator
supplies it. The bootstrap authenticates the held selected envelope, then authenticates every
E-bound artifact and held M before dispatch. Candidate values can block later, but never flow
backward into M, E1, E2, or coordinator expectations.

## One canonical manifest and two fixed invocation records

`M` has exact schema `c2b2a-declared-input-manifest-v2`. Its path, maximum size, exact canonical
serialization, metadata, SHA-256, LF count, byte count, and final LF are frozen before gate one. It
has no self-hash. Fixed-order, sequence-numbered records reject unknown, missing, duplicate,
reordered, optional, wildcard, prefix-derived, environment-expanded, or caller-selected fields.

M is one indivisible reviewed object with exactly these parts in order:

1. one deduplicated immutable-input table containing every file required by either invocation;
2. invocation record 1, named exactly `static`, profile `path-bound-b5-v1`, with its literal phase,
   mode, argv, six-variable environment, cwd, umask, FDs, topology, bounds, and static-only output
   namespace; and
3. invocation record 2, named exactly `production`, profile `path-bound-b5-v1`, with its literal
   phase, mode, argv, six-variable environment, cwd, umask, FDs, topology, bounds, and
   production-only output namespace.

The two output namespaces are explicitly disjoint and individually closed. Their union is not an
authorized namespace. A phase/profile mismatch, use of the other record, synthesized third record,
record replay, repeated use, skipped sequence, union-widened output check, output outside the
selected namespace, or mutation/reordering/re-encoding of any record or input-table row is terminal.
E1 can select only record 1 and exactly once; E2 can select only record 2 and exactly once after a
valid E1-bound static trace. Neither envelope nor caller can override a record field.

The deduplicated table includes final launcher `L`, final controller `C`, provisioner where
applicable, final combined record `R`, the inherited exact six runtime anchors, every other exact
inherited candidate input used by either record, and the pinned Python executable identity. A path
shared by both records appears once. Each invocation record refers only to fixed table sequence
numbers; per-record input subsets are closed literal lists, and their union cannot widen either
invocation's readable-input claim.

Every file entry binds fixed sequence, role, absolute path, file type, UID, GID, mode, link policy,
byte count, LF count or `not-applicable`, lowercase SHA-256, and exact inherited metadata policy.
Every invocation field is literal and bounded. M is constructed from independent stable host reads,
never by importing, compiling, parsing, invoking, or asking the candidate. Candidate-reported
identities and self-contract receipts are ignored.

M does not attempt an impossible inventory of every OS object the process could theoretically read.
The OS/kernel, dyld/shared cache, ambient same-UID filesystem, and unpinned standard library beyond
inherited anchors remain host TCB/nonclaims. Current design has no OS sandbox. Candidate
nonvisibility means E1/E2/M/new review/guard/gate data are not supplied through candidate argv,
environment, stdin, FDs, or named source constants, and semantic review proves the exact candidate
contains no undeclared read. It does not mean the filesystem is technically inaccessible.

M is create-new and immutable. It is never overwritten, repaired, normalized, or updated between
gates. Gate-one reviews cover the entire M, including both records, while authorizing only record 1.
Gate-two reviews the identical M bytes and evidence from record 1, while E2 authorizes only record
2. Any authoritative input, either invocation record, or M metadata change terminates the epoch;
continuation requires a separately named successor M plus new E1/E2 and reviews from gate one. M
bytes, path, and hash are never passed to the candidate.

Static stdout is not an expected-value field in M. The guard directly captures bounded raw stdout,
stderr, and wait/resource facts. Final L's internally constructed expected stdout remains a
rejection-only regression check. Gate-two reviewers judge the direct trace. A source-independent
external oracle is optional only if separately frozen and bound by E1/E2; it is not required and
may not be derived from candidate source, vectors, receipts, or output.

## Audit-parent and runtime-anchor visibility

The inherited exact six amendment/review runtime anchors remain candidate-readable inputs in their
unchanged order, roles, hashes, and scope. This amendment adds no seventh runtime anchor and does
not remove or relabel any of the six.

This successor amendment and its one successor design review are not runtime anchors and final L
does not read their contents. Their mandated paths nevertheless reside inside final L's existing
`_AUDIT_PARENT`. Their bounded basename/type/device/inode metadata therefore legitimately enters
`audit_parent_snapshot_hex` and transitively `R0/D/L/H/R`. This paragraph expressly supersedes any
earlier claim in this amendment family that no new review-file metadata can affect Q/H
construction. Only this already-declared audit-parent metadata projection for the exact successor
amendment and exact successor design-review basenames enters; their content, verdicts, findings,
and hashes do not become Q/H or candidate runtime inputs.

The successor design-review record must be created by the accepted no-overwrite protocol, frozen,
and accepted before `R0` construction. It is never modified, replaced, removed, or recreated
afterward. The successor amendment itself is likewise immutable before R0. These two exact paths,
not the rejected 7d amendment's prospective review path, are the metadata entries frozen before R0.

All E files, M, guard/bootstrap implementation records, raw reviewer reports, sealed gate-one and
gate-two attestations, guard envelopes, optional oracle records, and static/production traces must
live outside `_AUDIT_PARENT`. Their predeclared append-only creation cannot change its snapshot.
No new reviewer content, content hash, verdict, count, findings digest, attestation, or raw report
enters candidate argv, environment, stdin, FDs, source, Q/H, combined-record content, static
vectors, provider input, payload, or runtime. The only exceptions are (a) the unchanged inherited
review documents among the fixed six candidate-readable anchors and (b) only the declared
basename/type/device/inode metadata projection for this successor amendment and eventual successor
design-review file. Sealed new review hashes are E-bound guard/coordinator authority only; they are
never candidate-readable authority.

## Exact narrow supersession matrix

Only these named sections and permission meanings are superseded. All unlisted clauses remain.

| Inherited frozen document | Exact named section(s) | Corrected narrow successor meaning |
| --- | --- | --- |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md` | **Finite implementation-preflight authority** | One-hole projection and finite construction remain. Embedded semantic digest, self-hash comparison, self-stable-read, and self-contract receipts are blocking regression checks, never execution permission. E1/E2/M/guard/review gates provide permission. |
| same | **Closed preflight-only static test** | Vector semantics, isolation intent, output/status/deadline bounds, and fail-closed behavior remain. AST, matcher, purity, call-graph, def-use, bootstrap, Q250, and self-contract success cannot authorize their own effective objects. Static runs only through gate one, final L, M record 1, and `path-bound-b5-v1`. |
| same | **Closed production launcher role** | Scanner, capture, publication, deadlines, validation, and no-repair semantics remain. The accepted guard is the direct parent and invokes pinned Python against original repository `_LAUNCHER`; final L's internal authority receipt is rejection-only. No source snapshot/FD is claimed. |
| same | **Outer-launcher completion witness** | The trusted coordinator/reviewer directly starts and observes the accepted guard; the guard directly parents final L and captures its raw streams/status/resources. A distinct guard envelope reports direct facts. The reviewer no longer directly starts final L without the guard. Existing descendant/session cleanup and publication-success requirements remain. |
| same | **Implementation-preflight additions and sole early-execution exception** | Self-source obligations remain blocking regression evidence. The early exception requires accepted E1/M/guard plus gate one; it is not self-activating. Non-source prohibitions and later prerequisites remain. |
| same | **Acceptance boundary** | Source/record self-consistency is necessary but insufficient. Gate one permits only one bounded guarded static invocation; fresh gate two and E2 are required for production eligibility. |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_FROZEN_2026-09-08.md` | **Symbols and exact domains**, **Launcher-independent authority first**, **Exact launcher normalization `Q(L)`** | Exact domains and finite equations remain construction evidence. Expected execution identities are independently listed in M; candidate derivation of S/D/H cannot grant permission. |
| same | **Two exact source-analysis epochs** | Both epochs are non-authoritative regression analyses. Their reads, ASTs, copies, equality, and receipts may block but cannot authenticate their own implementation/effective objects. |
| same | **Semantic identity in the 26 proof rows**, **Canonical proof-row record**, **Exact combined-authority transport**, **Exact source-row to static-vector mapping** | All 26 rows, W0/W1, F0/F1, normalized facts, spans, roles, digests, four scalar fields, 26 indexed fields, and source-derived static expected values remain bounded diagnostic transport only. |
| same | **Q-invariant static proof and expected-output independence**, **Exact construction and final revalidation**, **Fail-closed source and projection witnesses** | Q-invariance, transition guards, matcher/pattern recursion, self-contract receipts, mutations, and replay retain blocking semantics only. Success never grants execution. |
| same | **Narrow supersessions**, **Untouched authority**, **Non-expansion and execution prohibition**, **Acceptance gates** | Only permission authority and the audit-parent metadata clarification change. The complete DAG, sole raw H field, non-source authority, runtime anchors, prohibitions, and non-expansion remain exact. |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md` | **Implementation-preflight and launcher obligations**, **Non-executing authority and adapter source proof** | The 26 source cases, three-codec extraction, adapter/authority AST facts, callsite/dominance evidence, and launcher/controller agreement remain rejection-capable regression evidence, not permission authority. |
| same | **Deterministic tests and mutation witnesses** | Every source mutation, including any current or future 250-case/Q250 self-source inventory, is quarantined regression evidence. Non-source carrier fixtures and later runtime qualification keep inherited meaning. |
| same | **Acceptance boundary** | Source review and codec/static results remain necessary but require the corrected external chain and two-step gate. |

The carrier's 75-row table, three edges, 29 roots, 26 parent entries, ten absences, six
observations, producer commit, consumer dominance, descriptor-relative capture, historical/graph
replay, bounds, no-writer schedule, and no-repair/no-replay rules remain unchanged.

## Complete preserved construction DAG

The complete inherited construction is preserved exactly:

```text
U -> A -> C -> HC -> L0/Q -> S -> W0/F0 -> components/B/V/O
  -> R0 -> D -> G_D -> L -> H -> G_H -> R
  -> authoritative W1/F1/replay
```

The exact invariants remain `Q(L) = L0` and `P(R) = R0`. `H` occurs raw only in the sole
`launcher_source_sha256` record field removed by P. The inherited exact six runtime anchors remain
unchanged and ordered; there is no seventh. Provider output is never authority.

`L` in this DAG is the final launcher produced after the one-hole Q construction. It is not
identified by the precursor b5 hash when Q is filled. B5 `b5e50318...` remains historical/candidate
input to construction and review; final L receives its own exact hash, its own stable reads, the
sole final-launcher M row, and all gate reviews. No b5 identity or review transfers to final L.

Every node and edge retains its inherited finite, no-iteration, no-repair construction and blocking
semantics. `G_D`, `G_H`, authoritative `W1/F1`, and replay must reject a mismatch. But no DAG node,
source proof, vector, receipt, or provider result grants execution; external E1/E2/M identity and
the applicable guard/review gate do.

## Guard/bootstrap trust and source binding

E1/E2, not M, bind the guard/bootstrap identity and run-plan hash. The trusted coordinator also
holds the accepted bootstrap identity needed to start the chain; an envelope cannot bootstrap the
code used to authenticate itself.

A native guard requires exact source, build recipe/toolchain inputs, final executable hash, and an
accepted source-to-executable binding. An interpreted guard requires exact interpreted-source hash,
pinned guard-interpreter identity, isolated invocation, and complete declared guard-input policy.
Merely hashing source while executing an unrelated binary is insufficient. All guard design,
source, executable/interpreter, and bootstrap identities receive separate review before use.

The bootstrap stable-opens the selected E with no-follow descriptor rules, checks the
coordinator-supplied hash, canonical grammar, type/owner/mode/size, and repeated fstat on a held
descriptor. It authenticates the same M and guard before dispatch. The guard never imports
candidate code or asks it for expected identity.

For each selected-record M path, the guard uses descriptor-relative traversal where required,
read-only `open` with `O_NOFOLLOW`, held-descriptor `fstat`, bounded EOF read, independent
SHA-256/counts, and repeated fstat. It validates before candidate creation and after complete
drain/reap. It verifies mutation only for declared inputs and the selected record's declared output
namespace; it never substitutes the union of the two namespaces or claims a global no-mutation
proof over the repository, home, or filesystem.

The exact later guard freeze defines separately: child-execution interval, child deadline, cleanup
grace, process/session enumeration interval, whole-guard deadline, and which inherited deadline
each refines without weakening it. No prose-level inferred deadline is executable authority.

## Two execution profiles

### Normative current profile: `path-bound-b5-v1`

This is the sole profile compatible with the current launcher topology and final L derived from it.
The accepted guard is final L's direct parent. It executes the M-pinned Python against the original
repository `_LAUNCHER` pathname with the selected record's exact mode argv, six-variable
environment, cwd `/var/empty`, incoming umask `0077`, and only FDs 0, 1, and 2 visible to final L.
FD 0 is the inherited read-only `/dev/null` identity; FDs 1 and 2 are distinct nonseekable
guard-owned capture pipes. No launcher snapshot, `/dev/fd` source, inherited source FD, guard FD,
E FD, M FD, or review FD is claimed or passed.

Immediately before child creation, the guard stable-opens and hashes original `_LAUNCHER` and each
selected-record fixed-path controller/provisioner/combined-record/runtime input against M, then
closes every guard-only descriptor from the child. Final L's inherited internal stable rereads
remain mandatory. After final L and every permitted descendant are drained and reaped, the guard
repeats full selected-record M validation, the selected output-namespace check, and the complete
E-bound final authority revalidation defined below.

Every pathname-open interval, including final L's own source, controller, provisioner,
combined-record, and inherited-anchor reads, is covered only by the inherited same-UID
swap-and-restore nonclaim. Pre/post equality detects ordinary drift but does not prove interval
immutability against a malicious same UID. No review or report may state a stronger claim.

This same profile supports static and production only as distinct fixed records after their
separate gates. In production, final L retains its exact sole scanner-child, capture, publication,
deadline, process-group, and cleanup semantics. The guard remains the direct parent of final L and
verifies only record 2's declared output namespace; it does not become the completion scanner or
publication authority.

### Optional future profile: `fd-snapshot-v1`

This profile is not authorized for current b5 or final L. It requires a new launcher source epoch
whose exact argv and reads accept held immutable input FDs and never reopen original paths.

A later accepted design must distinguish source-file and snapshot-file metadata policies. The
guard opens the accepted source read-only, creates a fresh owner-only no-overwrite snapshot with a
writer FD, copies exact bounded bytes, fsyncs, closes the writer FD, and descriptor-relatively
reopens the snapshot `O_RDONLY|O_NOFOLLOW`. It fstats and rehashes using `pread`, or rewinds and
proves the exact offset before interpreter consumption. If unlinked while held, it validates the
exact link-count transition and platform `/dev/fd` behavior; otherwise it freezes the private-path
claim. No writable FD is inherited. Controller, provisioner, and record FDs require the same rules.

Any move to this profile changes launcher/guard/invocation authority, invalidates all current
launcher reviews, M, E1/E2, Q/H evidence, and guarded traces, and begins a new exact epoch.

## Direct guard observation channels

The guard is the direct parent and captures raw child stdout and stderr as distinct bounded byte
streams. Those raw streams remain separate, preserved evidence; they are not embedded in or
reframed as candidate output.

A third distinct coordinator-owned channel carries one canonical guard envelope with later-frozen
schema `c2b2a-guard-observation-envelope-v2`. Its fixed fields include epoch/phase, selected
E sequence/hash, M identity, selected record index/name, invocation identity, input precheck result,
child PID/start/PGID/SID facts, monotonic start/deadline/finish facts, resource-limit and overflow
facts, complete wait status, cleanup/reap/session-empty facts, raw child stdout byte count/hash, raw
child stderr byte count/hash, selected declared-output transition, postcheck result, authority
precheck receipt, authority postcheck receipt, failures, and terminal validity.

The guard envelope frames facts only; it does not contain raw child streams or an internally
expected stdout. The trusted coordinator directly captures the two raw streams, guard envelope,
and guard wait status under separately frozen bounds. A candidate `valid=true`, self-hash, or
expected-output match cannot replace those observations.

## Mandatory pre/post authority revalidation

Before dispatch, bootstrap/guard and coordinator independently establish an authority precheck
over the held selected E and every artifact E binds: M, guard/bootstrap source and executable or
interpreter identities, run plan, this successor amendment, successor design review, every listed
sealed attestation, every selected prior-envelope/static-evidence artifact, and any external oracle.
Each artifact is stable-opened no-follow and held where the later platform freeze permits; the
precheck records fixed role/path, type, device, inode, UID, GID, mode, link count, size, SHA-256,
byte/LF counts, and repeated-fstat result. The coordinator separately records the supplied E hash,
held-E observed hash, envelope sequence, selected record, and accepted guard identity.

After child and permitted descendants are completely drained and reaped, and after selected-input
and output-namespace postchecks, the guard and coordinator repeat those observations from the held
descriptors and stable path bindings. The final receipt has exact pre/post rows for every E-bound
artifact, `held_e_pre_sha256`, `held_e_post_sha256`, `held_e_pre_fstat`, `held_e_post_fstat`,
`e_bound_pre_inventory_sha256`, `e_bound_post_inventory_sha256`, per-row equality bits,
`selected_record_consumed_once`, `other_record_consumed_zero`, `selected_namespace_only`,
`coordinator_expected_e_sha256_match`, and terminal `authority_revalidation_valid`.

Acceptance sealing occurs only after both guard and coordinator validate exact pre/post equality,
the held E still matches the out-of-band coordinator value, every E field still resolves to the
same held accepted artifact, M is byte-identical, the selected record was consumed exactly once,
the other record was unused, and all path/type/owner/mode/link/size policies still hold. Any absent,
extra, reordered, rebound, mutated, unreadable, or un-revalidated E-bound artifact is terminal. The
raw streams, guard envelope, receipt, and coordinator result are preserved, but no success trace or
gate evidence is sealed. This does not strengthen the same-UID interval nonclaim.

For E1, the accepted post-run receipt becomes part of immutable E1-bound static evidence reviewed
at gate two. For E2, the post-run receipt is required before production acceptance; E2 grants only
eligibility and cannot predeclare its own successful outcome.

## Sealed review attestations and independence

Each review produces one create-new canonical attestation with exact design-level schema
`c2b2a-external-review-attestation-v2` and fields in this order:

```text
schema
authorization_epoch
gate
phase
review_role
exact_input_hashes
harness
provider
model
session
started_at_utc
finished_at_utc
p0_count
p1_count
p2_count
verdict
findings_digest
```

`exact_input_hashes` is a fixed-order, role-tagged inventory of every file/envelope/manifest/guard/
trace input actually reviewed. Times are canonical UTC instants. Counts are canonical nonnegative
integers. Verdict is exactly `pass` or `fail`, with `pass` requiring `P0=0/P1=0`; P2 remains
reported. `findings_digest` hashes the complete sealed raw findings stored outside `_AUDIT_PARENT`.
The attestation contains no self-hash, future E hash, peer verdict, or candidate-derived expected
identity. The trusted coordinator ingests the direct harness report, creates/accepts the sealed
artifact, hashes it independently, and later lists its hash in the applicable E.

The writer of an artifact cannot review that artifact. Same-gate reviewers remain blind to peer
reports until all their reports are sealed. Every three-review gate uses at least two distinct
harness/provider families; one family may supply at most two reviews. This reduces a shared harness
failure but does not prove statistical independence: models/providers can share training data,
implementation ideas, host state, and blind spots. That correlated-model limitation remains an
explicit nonclaim.

Reviewer roles are semantic correctness, guard/TOCTOU/topology, and broad adversarial closure. The
semantic reviewer checks effective-object mutation, `__code__`, rebinding, alias/reflection,
`ValueError` and builtin shadowing, and faithful evaluation. The guard reviewer checks E1/E2/M
acyclicity, source-to-executable binding, fixed record selection, path-bound claims, FDs, raw
channels, pre/post authority receipts, wait/resources, deadlines, cleanup, and selected outputs.
The adversarial reviewer attempts stale/substituted E/M, alternate record, record replay,
union-widened namespace, omitted input, alternate interpreter, matcher co-edit, source-vector
self-fulfillment, output forgery, undeclared read, audit-parent drift, review leakage, and
post-run authority substitution.

No new reviewer content, hash, verdict, count, or finding becomes candidate-readable or enters the
candidate runtime. The unchanged inherited review docs within the fixed six runtime anchors remain
the only review-content exception. The only other exception is the previously declared
basename/type/device/inode audit-parent metadata projection for this successor amendment and its
eventual design-review file. E-bound sealed attestation hashes are consumed solely by the trusted
bootstrap/guard/coordinator and are never passed to final L.

## Append-only phase envelopes and the two-step gate

The run plan predeclares outside `_AUDIT_PARENT` the no-overwrite namespaces for design, gate-one,
and gate-two attestations, E1/E2, record-specific traces, and record-specific outputs, without
embedding any E or M hash. M, once frozen, contains the exact static-then-production record pair and
deduplicated input table. E1 and E2 both bind that same M.

Prior E, M, reports, attestations, traces, raw streams, and receipts remain immutable. Creation of
predeclared gate-two reports and E2 after successful gate one is an expected append-only transition
and does not invalidate gate one. Modification, replacement, truncation, renaming, removal, or
recreation of any prior object is terminal and invalidates the successor.

The gates are:

1. **Gate one — guarded static eligibility.** Three blind, sealed, clean same-hash non-executing
   reviews cover final L/C/provisioner/R, the entire immutable M including both fixed invocation
   records and deduplicated input table, path-bound guard/bootstrap, run plan, this successor
   amendment/review, and every declared input. The coordinator constructs E1 binding their
   attestations and supplies its hash out of band. Gate one and E1 permit exactly one bounded final
   L static invocation through M record 1 and `path-bound-b5-v1`; they permit no production,
   provider, activation, or evaluation.
2. **Gate two — production eligibility.** Three fresh blind, sealed, clean same-hash reviews cover
   the byte-identical whole M and all unchanged gate-one inputs plus E1, direct raw streams, guard
   envelope, E1 authority pre/post receipt, status/resource facts, selected static-output
   transitions, and semantic result. The coordinator appends their attestations, creates E2 binding
   the same M plus E1-bound evidence, and supplies E2's hash out of band. Gate two and E2 permit
   exactly one production invocation through M record 2 and `path-bound-b5-v1`, subject to every
   inherited gate; they do not prove production success.

Phase/profile mismatch, using the other record, any replay or second consumption, widened union of
record inputs or outputs, mutable/replaced M, inconsistent E/M/run-plan selection, missing E1
evidence, or any pre/post authority mismatch is terminal. A failing candidate self-test or source
proof blocks the gate even though it cannot grant one.

## Preconditions and finite transition

Before any candidate/provider/static/production execution:

1. freeze this successor amendment and obtain three clean same-hash design attestations;
2. create once, freeze, and independently accept its exact successor design review inside
   `_AUDIT_PARENT`;
3. freeze this successor amendment and successor-review basename/type/device/inode metadata before
   `R0`, then never mutate, replace, remove, or recreate either path;
4. design/implement/freeze the tiny guard/bootstrap outside `_AUDIT_PARENT`, including exact
   source-to-executable or interpreted-source binding, path-bound topology, envelope grammars,
   direct channels, resource/deadline scopes, record-specific output policy, and final E-bound
   authority revalidation;
5. independently review exact b5 and eventual final L for semantic correctness,
   guard/TOCTOU compatibility, and broad adversarial closure, treating the two scoped b5 passes
   only as supporting evidence and transferring no b5 identity to final L;
6. optionally, after a candidate hash and at least one blind review are sealed, run one bounded AST
   inventory as quarantined omission evidence; it cannot grant permission and is stale on edit;
7. construct exactly once the complete inherited DAG, including Q/H, freeze final L/C/R, and
   require every internal blocking check;
8. freeze exactly one complete M from independent stable reads, with deduplicated inputs and fixed
   records `[1/static, 2/production]`, and freeze the run plan without E/M hashes;
9. require exact pristine hashes/metadata, positive disk reserve, absent repository `target/debug`,
   absent combined output before its sole create-new publication, and every inherited auth/
   no-replay/no-repair/phase-order gate;
10. complete gate one, create E1, run final L through guarded static record 1 exactly once, require
    full E1-bound post-run authority receipt, then complete gate two and create E2; and
11. only through E2 and production record 2, and after every inherited gate, run the
    already-authorized activation/evaluation phases without replaying or repairing a completed lane.

No new raw or sealed reviewer output enters the DAG, R, candidate source, static vectors, provider
prompt, payload, or candidate runtime. The unchanged inherited six anchors and preconstruction
audit-parent basename/type/device/inode metadata for this successor amendment/design review are the
sole exceptions exactly defined above.

## Exact b5 and final-L disposition

Exact `b5e50318...` remains an unexecuted current candidate. External hashing can distinguish those
bytes from hypothetical modified bytes without another recursive self-matcher, but a hash does not
establish benign semantics. Exact semantic review must still exclude `__code__` replacement,
`ValueError`/builtin shadowing, malicious effective-object mutation, alternate paths, and unfaithful
evaluation behavior.

If Q construction fills the inherited 64-byte interval, final `L` necessarily has a new exact hash.
The b5 hash remains precursor evidence only. Final L must be independently entered in the sole M
input table, referenced by both fixed invocation records, and pass gate-one and gate-two reviews.
No b5 review transfers across that byte change. B5 self-tests remain defense-in-depth only. Final L
can serve both fixed records only through `path-bound-b5-v1`, under the narrow same-UID pathname
nonclaim and after complete Q/H construction.

## Acceptance and exact invalidation

This successor amendment is acceptable only if three independent same-hash design reviews return
`P0=0/P1=0`, use at least two harness/provider families, cover every correction here, and are bound
in the one accepted successor design-review record. Acceptance confirms design only; it does not
accept a guard, E1/E2, M, source, Q fill, static result, provider run, production phase, pilot, or
flagship goal.

Acceptance must confirm:

- the acyclic coordinator -> E1 -> one M -> static evidence -> E2 chain and exclusions preventing
  loops;
- one immutable whole M with a deduplicated input table and fixed records `[static, production]`;
- E1 selecting only static once, E2 selecting only production once, and terminal rejection of
  record mismatch, replay, mutation, or union-widened namespaces;
- the finite declared-input closure and explicit ambient-host TCB/nonclaims;
- the unchanged inherited six anchors and sole permitted successor audit-parent metadata effect;
- the full preserved DAG, final L distinct from b5 after Q fill, and rejection-only status of every
  recursive/self-source proof;
- current `path-bound-b5-v1`, only FDs 0/1/2, direct guard parenthood, and same-UID limitation;
- optional `fd-snapshot-v1` only through a new source epoch with read-only-FD rules;
- distinct raw streams and canonical guard envelope, without an internal external-output oracle;
- mandatory pre/post revalidation of held E and every E-bound artifact before acceptance sealing;
- sealed blind attestation provenance, minimum family diversity, and correlated-model nonclaim;
- append-only E1/E2 transition and exact two-step authorization boundary; and
- no new reviewer content/hash/verdict/findings in candidate runtime beyond the two exact exceptions.

Invalidation is exact and fail closed. Any byte, path, type, device, inode, owner, group, mode, link,
size, count, ordering, schema, or bound change to an accepted successor amendment/review, E1, E2,
M, run plan, guard/bootstrap, interpreter, final L, controller, provisioner, R, inherited anchor,
input-table row, either invocation record, declared input/output namespace, sealed attestation,
prior trace, or E1-bound evidence invalidates the applicable epoch. A phase/profile mismatch,
selection of the other record, record replay, second execution, skipped sequence, union-widened
namespace, post-run E/E-bound revalidation failure, or path rebinding is terminal even if a content
hash happens to match. Modification/removal/replacement of a prior append-only object is terminal;
only the exactly predeclared no-overwrite creation of gate-two artifacts and E2 is allowed.

On failure, execute nothing further, preserve exact rejected bytes and traces, and do not overwrite,
delete, repair, normalize, or replay a partial/completed output. Continuing requires a separately
named successor M/E epoch and all reviews from the first invalidated gate. Partial production
publication still requires a separately reviewed recovery freeze. User-owned worktree changes are
preserved.

## Non-expansion and current exact disposition

This amendment adds no provider, model, budget, VM, pilot lane, adapter, daemon, datastore,
connector, network, authentication, product behavior, user data, runtime anchor, recovery mode, or
source mutation. It authorizes no stage, commit, publication, live setting change, or
`target/debug` creation.

The design becomes frozen only after this successor file's exact hash is reviewed and its one
successor design review is accepted. At present there is no execution authorization, no Q fill, no
combined record publication, and no controller or provisioner change. Launcher `b5e50318...`,
controller `eefa6eec...`, and provisioner `aabe6b17...` remain non-executed candidate bytes.
