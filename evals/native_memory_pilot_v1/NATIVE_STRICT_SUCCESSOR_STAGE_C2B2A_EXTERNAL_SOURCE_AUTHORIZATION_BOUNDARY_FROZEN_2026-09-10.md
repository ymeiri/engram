# Native strict successor Stage C2B2a — external source-authorization boundary

Date: 2026-09-10
Status: corrected frozen candidate design amendment; no implementation or execution authority

## Disposition and rejected predecessor

This amendment replaces recursive launcher self-authentication as the source of permission to
execute with one externally rooted exact-hash authorization chain. Candidate self-checks remain
fail-closed regression evidence: a failure blocks the candidate, but success never grants
execution.

The preceding amendment epoch is rejected historical design evidence:

```text
SHA-256 1cd2c842fa4fa25415b03623d71458f0377c7cbd73d0de40b1bfb0a79cd5f689
lines   566
bytes   38215
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

That epoch was cyclic about manifest/bootstrap authority, claimed an impossible complete runtime
closure, required a launcher snapshot incompatible with current b5, misstated review visibility in
the audit parent, and treated an internally expected stdout as external authority. Its identity and
any review of it cannot authorize these corrected bytes.

This corrected amendment is itself only a candidate. It authorizes no source import, compilation,
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

This amendment's exact path is:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
```

Its one prospective design-review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_REVIEW_2026-09-10.md
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

Let `E` be one canonical external authorization envelope and `M` one canonical declared-input
manifest for a single epoch and phase. The authority chain is exactly:

```text
terminal trusted coordinator
  -> exact SHA256(E) delivered out of band
  -> stable-opened, held, canonical E
  -> {SHA256(M), guard/bootstrap identity, SHA256(run plan),
      amendment/review hashes, phase/epoch, sealed reviewer-attestation hashes}
  -> stable-opened, held, canonical M
  -> declared candidate/runtime inputs and exact candidate invocation/bounds
```

The terminal coordinator and its out-of-band delivery are the trust root under the honest-host
assumption. The expected `SHA256(E)` is never read from a caller file, candidate output, M, E, run
plan, environment, provider output, or candidate-derived record.

`E` has exact schema `c2b2a-external-authorization-envelope-v1`. Its later frozen canonical grammar
has fixed-order fields for schema, envelope sequence, predecessor-envelope hash or exact root
sentinel, authorization epoch, phase, manifest path/hash, guard/bootstrap identities, run-plan
path/hash, this amendment path/hash, its design-review path/hash, ordered sealed-attestation
path/hashes, optional external-oracle path/hash, creation policy, and terminal LF. It contains no
self-hash and cannot bind the value of the out-of-band `E`-hash parameter used to open it.

`E` is intentionally excluded from `M`. `M` excludes the run plan, guard/bootstrap, this external
boundary amendment and design review, gate attestations, and every object containing an `M` or `E`
hash. The sole genre exception is the inherited exact six amendment/review runtime anchors: b5 is
already specified to read those files, so M inventories them under the distinct role
`inherited-runtime-anchor`, not as external-envelope authority. The new amendment/review are not in
that six-anchor set.

If `E` binds the run-plan hash, the run plan must not embed an `E` or `M` hash. It may predeclare
paths, phases, modes, output namespaces, and append-only successor creation, but cannot choose
expected envelope/manifest identities. The bootstrap invocation is later frozen as one exact argv,
environment, cwd, umask, and FD grammar containing exactly one lowercase-64-hex out-of-band
`E`-hash parameter in one fixed position. The grammar, not E, fixes that parameter's shape and
location; E cannot bind its value. Only the terminal coordinator supplies it.

The bootstrap first authenticates held `E` against that parameter, parses canonical E, then uses E
to authenticate the guard/bootstrap, run plan, amendment/review, attestations, and held `M`. M is
parsed only after its exact E-bound hash is proven. Candidate values can block later, but never
flow backward into E, M, or the coordinator's expected values.

## Canonical finite declared-input manifest

`M` has exact schema `c2b2a-declared-input-manifest-v1`. Its path, maximum size, exact canonical
serialization, metadata, SHA-256, LF count, byte count, and final LF are frozen later. It has no
self-hash. Fixed-order, sequence-numbered records reject unknown, missing, duplicate, reordered,
optional, wildcard, prefix-derived, environment-expanded, or caller-selected fields.

M is finite per mode. It contains only:

1. every immutable file the reviewed candidate is specified to open in that mode, including final
   launcher `L`, final controller `C`, provisioner where applicable, final combined record `R`, the
   inherited exact six runtime anchors, and any other exact inherited candidate input;
2. the pinned Python executable identity used for the candidate invocation; and
3. the exact candidate argv, six-variable environment, cwd, umask, FDs 0/1/2, process topology,
   phase/mode, resource limits, direct-stream ceilings, deadline policy, status policy, and declared
   output namespace.

Every file entry binds fixed sequence, role, absolute path, file type, UID, GID, mode, link policy,
byte count, LF count or `not-applicable`, lowercase SHA-256, and the exact inherited metadata policy.
Every invocation field is literal and bounded. The manifest identity is constructed from independent
stable host reads, never by importing, compiling, parsing, invoking, or asking the candidate.
Candidate-reported identities and self-contract receipts are ignored.

M does not attempt an impossible inventory of every OS object the process could theoretically read.
The OS/kernel, dyld/shared cache, ambient same-UID filesystem, and unpinned standard library beyond
inherited anchors remain host TCB/nonclaims. Current design has no OS sandbox. Candidate
nonvisibility means E/M/new review/guard/gate data are not supplied through candidate argv,
environment, stdin, FDs, or named source constants, and semantic review proves the exact candidate
contains no undeclared read. It does not mean the filesystem is technically inaccessible.

M is create-new and immutable. It is never overwritten, repaired, normalized, or updated. Any
authoritative input or invocation change requires a new M identity and a successor E. M bytes,
path, or hash are never passed to the candidate.

Static stdout is not an expected-value field in M. The guard directly captures bounded raw stdout,
stderr, and wait/resource facts. B5's internally constructed expected stdout remains a
rejection-only regression check. Gate-two reviewers judge the direct trace. A source-independent
external oracle is optional only if separately frozen and bound by E; it is not required and may
not be derived from candidate source, vectors, receipts, or output.

## Audit-parent and runtime-anchor visibility

The inherited exact six amendment/review runtime anchors remain candidate-readable inputs in their
unchanged order, roles, hashes, and scope. This amendment adds no seventh runtime anchor and does
not remove or relabel any of the six.

This new amendment and its one design review are not runtime anchors and b5 does not read their
contents. Their mandated paths nevertheless reside inside b5's existing `_AUDIT_PARENT`. Their
bounded basename/type/device/inode metadata therefore legitimately enters
`audit_parent_snapshot_hex` and transitively `R0/D/L/H/R`. This paragraph expressly supersedes any
earlier claim in this amendment family that no new review-file metadata can affect Q/H construction.
Only the audit-parent metadata projection enters; their content, verdicts, findings, and hashes do
not become Q/H or runtime inputs.

The design review record must be created by the accepted no-overwrite protocol, frozen, and
accepted before `R0` construction. It is never modified, replaced, removed, or recreated afterward.
The amendment itself is likewise immutable at that point.

All E files, M, guard/bootstrap implementation records, raw reviewer reports, sealed gate-one and
gate-two attestations, guard envelopes, and optional oracle records must live outside
`_AUDIT_PARENT`. Their predeclared append-only creation cannot change its snapshot. Raw reviewer
content and verdicts never enter Q/H, combined-record content, static vectors, provider input,
payload, or runtime.

## Exact narrow supersession matrix

Only these named sections and permission meanings are superseded. All unlisted clauses remain.

| Inherited frozen document | Exact named section(s) | Corrected narrow successor meaning |
| --- | --- | --- |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md` | **Finite implementation-preflight authority** | One-hole projection and finite construction remain. Embedded semantic digest, self-hash comparison, self-stable-read, and self-contract receipts are blocking regression checks, never execution permission. E/M/guard/review gates provide permission. |
| same | **Closed preflight-only static test** | Vector semantics, isolation intent, output/status/deadline bounds, and fail-closed behavior remain. AST, matcher, purity, call-graph, def-use, bootstrap, Q250, and self-contract success cannot authorize their own effective objects. Static runs only through gate one and `path-bound-b5-v1`. |
| same | **Closed production launcher role** | Scanner, capture, publication, deadlines, validation, and no-repair semantics remain. For current b5, the accepted guard is the direct parent and invokes pinned Python against original repository `_LAUNCHER`; b5's internal authority receipt is rejection-only. No source snapshot/FD is claimed. |
| same | **Outer-launcher completion witness** | The trusted coordinator/reviewer directly starts and observes the accepted guard; the guard directly parents b5 and captures b5 raw streams/status/resources. A distinct guard envelope reports direct facts. The reviewer no longer directly starts b5 without the guard. Existing descendant/session cleanup and publication-success requirements remain. |
| same | **Implementation-preflight additions and sole early-execution exception** | Self-source obligations remain blocking regression evidence. The early exception requires accepted E/M/guard plus gate one; it is not self-activating. Non-source prohibitions and later prerequisites remain. |
| same | **Acceptance boundary** | Source/record self-consistency is necessary but insufficient. Gate one permits only bounded guarded static behavior; fresh gate two is required for production eligibility. |
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

Every node and edge retains its inherited finite, no-iteration, no-repair construction and blocking
semantics. `G_D`, `G_H`, authoritative `W1/F1`, and replay must reject a mismatch. But no DAG node,
source proof, vector, receipt, or provider result grants execution; external E/M identity and the
applicable guard/review gate do.

## Guard/bootstrap trust and source binding

E, not M, binds the guard/bootstrap identity and run-plan hash. The trusted coordinator also holds
the accepted bootstrap identity needed to start the chain; E cannot bootstrap the code used to
authenticate E.

A native guard requires exact source, build recipe/toolchain inputs, final executable hash, and an
accepted source-to-executable binding. An interpreted guard requires exact interpreted-source hash,
pinned guard-interpreter identity, isolated invocation, and complete declared guard-input policy.
Merely hashing source while executing an unrelated binary is insufficient. All guard design,
source, executable/interpreter, and bootstrap identities receive separate review before use.

The bootstrap stable-opens E with no-follow descriptor rules, checks the coordinator-supplied hash,
canonical grammar, type/owner/mode/size, and repeated fstat on a held descriptor. It authenticates M
and the guard before dispatch. The guard never imports candidate code or asks it for expected
identity.

For each M-declared path, the guard uses descriptor-relative traversal where required, read-only
`open` with `O_NOFOLLOW`, held-descriptor `fstat`, bounded EOF read, independent SHA-256/counts, and
repeated fstat. It validates before candidate creation and after complete drain/reap. It verifies
mutation only for declared inputs and the declared output namespace; it never claims a global
no-mutation proof over the repository, home, or filesystem.

The exact later guard freeze defines separately: child-execution interval, child deadline, cleanup
grace, process/session enumeration interval, whole-guard deadline, and which inherited deadline
each refines without weakening it. No prose-level inferred deadline is executable authority.

## Two execution profiles

### Normative current profile: `path-bound-b5-v1`

This is the sole profile compatible with current b5 topology. The accepted guard is b5's direct
parent. It executes the M-pinned Python against the original repository `_LAUNCHER` pathname with
the exact b5 mode argv, six-variable environment, cwd `/var/empty`, incoming umask `0077`, and only
FDs 0, 1, and 2 visible to b5. FD 0 is the inherited read-only `/dev/null` identity; FDs 1 and 2 are
distinct nonseekable guard-owned capture pipes. No launcher snapshot, `/dev/fd` source, inherited
source FD, guard FD, E FD, M FD, or review FD is claimed or passed.

Immediately before child creation, the guard stable-opens and hashes original `_LAUNCHER` and each
declared fixed-path controller/provisioner/combined-record/runtime input against M, then closes every
guard-only descriptor from the child. B5's inherited internal stable rereads remain mandatory.
After b5 and every permitted descendant are drained and reaped, the guard repeats full M validation
and declared output-namespace checks.

Every pathname-open interval, including b5's own source, controller, provisioner, combined-record,
and inherited anchor reads, is covered only by the inherited same-UID swap-and-restore nonclaim.
Pre/post equality detects ordinary drift but does not prove interval immutability against a
malicious same UID. No review or report may state a stronger claim.

This same profile can support static and production modes only after their separate gates. In
production, b5 retains its exact sole scanner-child, capture, publication, deadline, process-group,
and cleanup semantics. The guard remains the direct parent of b5 and verifies the declared output
namespace; it does not become the completion scanner or publication authority.

### Optional future profile: `fd-snapshot-v1`

This profile is not authorized for b5. It requires a new launcher source epoch whose exact argv and
reads accept held immutable input FDs and never reopen the original paths.

A later accepted design must distinguish source-file and snapshot-file metadata policies. The
guard opens the accepted source read-only, creates a fresh owner-only no-overwrite snapshot with a
writer FD, copies exact bounded bytes, fsyncs, closes the writer FD, and descriptor-relatively
reopens the snapshot `O_RDONLY|O_NOFOLLOW`. It fstats and rehashes using `pread`, or rewinds and
proves the exact offset before interpreter consumption. If unlinked while held, it validates the
exact link-count transition and platform `/dev/fd` behavior; otherwise it freezes the private-path
claim. No writable FD is inherited. Controller, provisioner, and record FDs require the same rules.

Any move to this profile changes launcher/guard/invocation authority, invalidates all b5 reviews,
M, E, Q/H evidence, and guarded traces, and begins a new exact epoch.

## Direct guard observation channels

The guard is the direct parent and captures raw child stdout and stderr as distinct bounded byte
streams. Those raw streams remain separate, preserved evidence; they are not embedded in or
reframed as candidate output.

A third distinct coordinator-owned channel carries one canonical guard envelope with later-frozen
schema `c2b2a-guard-observation-envelope-v1`. Its fixed fields include epoch/phase, E and M
identifiers observed by the guard, invocation identity, input precheck result, child PID/start/
PGID/SID facts, monotonic start/deadline/finish facts, resource-limit and overflow facts, complete
wait status, cleanup/reap/session-empty facts, raw child stdout byte count/hash, raw child stderr
byte count/hash, declared-output transition, postcheck result, failures, and terminal validity.

The guard envelope frames facts only; it does not contain raw child streams or an internally
expected stdout. The trusted coordinator directly captures the two raw streams, guard envelope,
and guard wait status under separately frozen bounds. A candidate `valid=true`, self-hash, or
expected-output match cannot replace those observations.

## Sealed review attestations and independence

Each review produces one create-new canonical attestation with exact design-level schema
`c2b2a-external-review-attestation-v1` and fields in this order:

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
artifact, hashes it independently, and later lists its hash in E.

The writer of an artifact cannot review that artifact. Same-gate reviewers remain blind to peer
reports until all their reports are sealed. Every three-review gate uses at least two distinct
harness/provider families; one family may supply at most two reviews. This reduces a shared harness
failure but does not prove statistical independence: models/providers can share training data,
implementation ideas, host state, and blind spots. That correlated-model limitation remains an
explicit nonclaim.

Reviewer roles are semantic correctness, guard/TOCTOU/topology, and broad adversarial closure. The
semantic reviewer checks effective-object mutation, `__code__`, rebinding, alias/reflection,
`ValueError` and builtin shadowing, and faithful evaluation. The guard reviewer checks E/M
acyclicity, source-to-executable binding, path-bound claims, FDs, raw channels, wait/resources,
deadlines, cleanup, and declared outputs. The adversarial reviewer attempts stale/substituted E/M,
omitted input, alternate interpreter, matcher co-edit, source-vector self-fulfillment, output
forgery, undeclared read, audit-parent drift, and review leakage.

## Append-only phase envelopes and the two-step gate

The run plan predeclares outside `_AUDIT_PARENT` the no-overwrite namespaces for design, gate-one,
and gate-two attestations and envelope successors, without embedding any E/M hash. Every E after
the first contains its predecessor E hash and fixed increasing sequence. Prior E, M, reports,
attestations, traces, and raw streams remain immutable.

Creation of predeclared gate-two reports and a successor E after successful gate one is an expected
append-only transition and does not invalidate gate one. Modification, replacement, truncation,
renaming, removal, or recreation of any prior object is terminal and invalidates the successor.

The gates are:

1. **Gate one — guarded static eligibility.** Three blind, sealed, clean same-hash non-executing
   reviews cover final L/C/provisioner/R, M, path-bound guard/bootstrap, run plan, this amendment/
   review, and every declared input. The coordinator constructs successor E binding their
   attestations and supplies its hash out of band. Gate one permits exactly one bounded b5 static
   run through `path-bound-b5-v1`; it permits no production/provider/activation/evaluation.
2. **Gate two — production eligibility.** Three fresh blind, sealed, clean same-hash reviews cover
   the unchanged gate-one inputs plus direct raw streams, guard envelope, status/resource facts,
   and semantic result. The coordinator appends their attestations, creates a successor E, and
   supplies its hash out of band. Gate two only makes the already-defined production phase eligible
   under every inherited gate; it does not prove production success.

Any authoritative source/input change starts a new M and review epoch. A failing candidate self-test
or source proof blocks the gate even though it cannot grant one. Reviewer artifacts never become
candidate inputs, except the unchanged inherited six anchors already named above.

## Preconditions and finite transition

Before any candidate/provider/static/production execution:

1. freeze this corrected amendment and obtain three clean same-hash design attestations;
2. create once, freeze, and independently accept its exact design review inside `_AUDIT_PARENT`;
3. freeze the amendment/review metadata before `R0` and never mutate either path;
4. design/implement/freeze the tiny guard/bootstrap outside `_AUDIT_PARENT`, including exact
   source-to-executable or interpreted-source binding, path-bound topology, envelope grammars,
   direct channels, resource/deadline scopes, and output policy;
5. independently review exact b5 for semantic correctness, guard/TOCTOU compatibility, and broad
   adversarial closure, treating the two scoped passes only as supporting evidence;
6. optionally, after a candidate hash and at least one blind review are sealed, run one bounded AST
   inventory as quarantined omission evidence; it cannot grant permission and is stale on edit;
7. construct exactly once the complete inherited DAG, including Q/H, freeze final sources and R,
   and require every internal blocking check;
8. freeze finite M from independent stable reads and freeze the run plan without E/M hashes;
9. require exact pristine hashes/metadata, positive disk reserve, absent repository `target/debug`,
   absent combined output before its sole create-new publication, and every inherited auth/
   no-replay/no-repair/phase-order gate;
10. complete gate one, run the guarded static phase exactly once, then complete gate two; and
11. only after gate two and all inherited gates, run already-authorized activation/evaluation
    phases without replaying or repairing a completed lane.

No raw future reviewer output enters the DAG, R, candidate source, static vectors, provider prompt,
payload, or runtime. Preconstruction audit-parent metadata for this amendment/design review is the
sole new permitted review-related effect on `R0/D/L/H/R`.

## Exact b5 disposition

Exact `b5e50318...` remains an unexecuted current candidate. External hashing can distinguish those
bytes from hypothetical modified bytes without another recursive self-matcher, but a hash does not
establish benign semantics. Exact semantic review must still exclude `__code__` replacement,
`ValueError`/builtin shadowing, malicious effective-object mutation, alternate paths, and unfaithful
evaluation behavior.

The b5-compatible implementation can potentially serve both static and production only through
`path-bound-b5-v1`, under the narrow same-UID pathname nonclaim, after exact semantic reviews and
complete Q/H construction. If Q construction fills the inherited 64-byte interval, final `L` has a
new exact hash: the b5 hash remains precursor evidence, and final L must be independently entered in
M and pass both gates. No b5 review transfers across that byte change. B5 self-tests remain
defense-in-depth only.

## Acceptance and invalidation

This amendment is acceptable only if three independent same-hash design reviews return
`P0=0/P1=0`, use at least two harness/provider families, cover every correction here, and are bound
in the one accepted design review record. Acceptance confirms design only; it does not accept a
guard, E, M, source, Q fill, static result, provider run, production phase, pilot, or flagship goal.

Acceptance must confirm:

- the acyclic coordinator -> E -> M chain and the exact exclusions preventing loops;
- the finite per-mode declared-input closure and explicit ambient-host TCB/nonclaims;
- the unchanged inherited six anchors and permitted audit-parent metadata effect;
- the full preserved DAG and rejection-only status of all recursive/self-source evidence;
- current `path-bound-b5-v1`, only FDs 0/1/2, direct guard parenthood, and the same-UID limitation;
- optional `fd-snapshot-v1` only through a new source epoch with read-only-FD rules;
- distinct raw streams and canonical guard envelope, without an internal external-output oracle;
- sealed blind attestation provenance, minimum family diversity, and correlated-model nonclaim;
- append-only gate successors and the exact two-step authorization boundary; and
- no raw future reviewer content in Q/H/runtime.

Any change to an accepted amendment/review, E, M, run plan, guard/bootstrap, interpreter, launcher,
controller, provisioner, R, inherited anchor, invocation, bound, declared output, attestation, or
prior trace invalidates the applicable epoch. Path/type/owner/mode/size/metadata drift is terminal
where bound even if content hash matches. Modification/removal/replacement of a prior append-only
object is terminal; only predeclared creation of a successor report/E is allowed.

On failure, execute nothing further, preserve exact rejected bytes and traces, and do not overwrite,
delete, repair, normalize, or replay a partial/completed output. Continuing requires a separately
named successor, new M/E, and all reviews from the first invalidated gate. Partial production
publication still requires a separately reviewed recovery freeze. User-owned worktree changes are
preserved.

## Non-expansion and current exact disposition

This amendment adds no provider, model, budget, VM, pilot lane, adapter, daemon, datastore,
connector, network, authentication, product behavior, user data, runtime anchor, recovery mode, or
source mutation. It authorizes no stage, commit, publication, live setting change, or
`target/debug` creation.

The design becomes frozen only after this corrected file's exact hash is reviewed and its one
design review is accepted. At present there is no execution authorization, no Q fill, no combined
record publication, and no controller or provisioner change. Launcher `b5e50318...`, controller
`eefa6eec...`, and provisioner `aabe6b17...` remain non-executed candidate bytes.
