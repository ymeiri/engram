# Native strict successor Stage C2B2a — two-manifest external authorization boundary

Date: 2026-09-10
Status: two-manifest frozen candidate design amendment; no implementation or execution authority

## Disposition and rejected predecessors

This amendment replaces recursive launcher self-authentication as the source of permission to
execute with an externally rooted exact-hash authorization chain. Candidate self-checks remain
fail-closed regression evidence: a failure blocks the candidate, but success never grants
execution.

The immediately preceding one-manifest/two-record epoch is rejected and preserved byte-for-byte:

```text
SHA-256 f1d7ddc8371c2a035f527f6d1784fe3f225cc89b3e0bac23e1616bc1f7a8ebaf
lines   690
bytes   47347
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

Its direct reviews were:

```text
Codex guard review       FAIL  P0=0 P1=1 P2=2
Codex normative review   FAIL  P0=0 P1=2 P2=2
isolated Claude review   PASS  P0=0 P1=0 P2=3
aggregate gate           FAIL
```

The isolated pass cannot override either fail and no verdict transfers. The f1 epoch tried to
freeze production inputs before they could exist, placed completion authorization before the
inherited pre-runtime lifecycle that produces those inputs, lacked a durable atomic at-most-one
start claim, and did not close output-parent baseline/alias transitions strongly enough. It also
blurred completion-audit authority with later provider activation/evaluation. These are lifecycle
and authorization-boundary failures, not editorial defects; f1 remains rejected raw evidence.

The earlier corrected-amendment epoch is likewise rejected and preserved byte-for-byte:

```text
SHA-256 7d0ae76f3fd721d61fc3797d3627c4cc4a99d413e7f2c39e4ae47417dc42ec80
lines   556
bytes   36410
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

That epoch was ambiguous about phase-specific manifest identity, record replay and namespace
widening, post-run authority revalidation, final-L identity, and reviewer visibility. Its bytes and
reviews cannot authorize this amendment.

An older rejected epoch was recorded with SHA-256
`1cd2c842fa4fa25415b03623d71458f0377c7cbd73d0de40b1bfb0a79cd5f689`, 566 lines, and 38,215
bytes. Its raw bytes are unavailable. Only its digest and review trace remain; they are provenance,
not re-reviewable evidence, and grant no authority. It was cyclic about manifest/bootstrap
authority, claimed an impossible complete runtime closure, required a launcher snapshot
incompatible with current b5, misstated audit-parent review visibility, and treated an internally
expected stdout as external authority.

This two-manifest amendment is itself only a candidate. It authorizes no source import,
compilation, AST parse, candidate/static/build/runtime/provider/pilot execution, Q fill, VM, daemon,
adapter, datastore, or other runtime action until its exact final bytes and separate design review
are accepted below.

## Exact inherited authorities and current candidates

This amendment is subordinate to these exact frozen records and reviews except only for the named
clauses in the supersession matrix:

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
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_FROZEN_2026-09-10.md
```

Its one prospective design-review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_REVIEW_2026-09-10.md
```

That review does not exist. It is created once by the accepted no-overwrite protocol only after
three clean same-hash design attestations, then frozen before `R0` and never modified.

## Why candidate self-proof cannot bootstrap trust

Two reviews of exact launcher `b5e50318...` passed deliberately narrow scopes. One matched six
handwritten final-helper patterns to six authentic helper bodies. The other checked matcher,
literal authentication, six-row identity/order, signature/body/lexical constraints, and final
contract call placement. They remain supporting findings about b5, not whole-program authority.

A broader adversarial review found a P0 boundary failure: later reachable code could replace an
authenticated callable's `__code__` while its checked source body stayed intact. A reachable shadow
of builtin `ValueError` could change exception classification, catch behavior, or traceback
provenance while a narrow helper check still passed. A validator co-editable with its matcher,
expected values, receipts, and effective objects cannot make its own success external fact.
Recursive/self-source proofs therefore remain rejection-only evidence.

## Threat model and explicit nonclaims

The terminal trust root is an honest, uncompromised coordinator that holds accepted reviewer state
and supplies exact E1/E2 hashes out of band. The design assumes collision-resistant SHA-256, an
honest host, honest reviewed guard/bootstrap, honest pinned interpreter, durable filesystem
semantics implementing the specified exclusive claims, and reviewers who inspect exact named bytes.

This unprivileged design does not defend against:

- a SHA-256 collision, second preimage, or compromised hash implementation;
- a compromised coordinator, reviewer, guard, bootstrap, interpreter, kernel, hypervisor, dynamic
  loader, shared cache, filesystem, administrator, or storage device that lies about durability;
- colluding reviewers or a reviewer that does not inspect the stated exact inputs;
- a same-UID debugger, injection/tracing facility, writable process memory, or equivalent host
  mechanism;
- same-UID pathname swap-and-restore during a candidate pathname-open interval;
- ambient same-UID filesystem access by malicious exact code without an OS sandbox;
- unpinned standard-library files, dyld/shared-cache components, kernel behavior, or ambient host
  TCB beyond inherited exact anchors and explicitly declared inputs; or
- semantic correctness merely because exact bytes have an accepted hash.

Owner-only paths reduce accidental exposure; they do not authenticate against the same UID or
root. Held descriptors and pre/post validation authenticate the observed objects and durable claim
under the honest-host model, but final L still reopens pathname inputs with only FDs 0/1/2. That
candidate interval remains subject to the same-UID swap-and-restore nonclaim. No report may claim
stronger isolation.

The durable `S1`/`S2` spend records below, rather than coordinator memory alone, provide the
honest-host at-most-one-start evidence. They do not survive a dishonest filesystem or host. No
detached-signature claim is made because no independently protected signing key exists.

## Two-manifest acyclic authority chain

There are exactly two create-new immutable phase manifests and two create-new immutable envelopes:

- `M_static`, sequence 1, contains exactly one final-L static invocation;
- `M_completion`, sequence 2, contains exactly one final-L
  `completion-audit-publish` invocation;
- `E1` binds and selects only `M_static`; and
- `E2` binds and selects only `M_completion` plus E1/static/post-static/pre-runtime evidence.

There is no shared two-record manifest, no third manifest, no third invocation record, and no third
envelope in this epoch. A manifest is not a template. Each contains only facts that exist when it
is independently constructed and frozen.

The exact outer authority and lifecycle order is:

```text
accepted design
  -> reviewed guard/bootstrap + inherited Q/R0/D/L/H/R construction yielding final L and R
  -> frozen run plan + M_static
  -> three clean blind Gate1 pre-execution reviews
  -> E1
  -> durable exclusive spend S1
  -> exactly one guarded final-L static invocation
  -> direct static trace/status/authority receipt reviews
  -> canonical post-static implementation acceptance I_static
  -> unchanged inherited runner/controller/provisioner/build phases
  -> sealed pre-runtime-finalize evidence
  -> M_completion
  -> three clean blind Gate2 completion-input reviews
  -> E2
  -> durable exclusive spend S2
  -> exactly one guarded final-L completion-audit-publish invocation
  -> independent completion TSV/digest review
  -> build/source identity proposal and review
  -> provider-free Linux runner freeze and review
  -> runtime qualification and reviews
  -> every remaining inherited gate
  -> only eventually, provider activation/evaluation through the unchanged pilot runner/run plan
```

Completion audit is after sealed pre-runtime-finalize and before provider activation/evaluation.
Pre-runtime-finalize is not provider activation and no pre-runtime evidence is described as
post-activation. E2 authorizes only the one completion-audit-publish invocation. It never directly
invokes or authorizes a provider, activation lane, evaluation lane, runner call, or budget spend.

The cryptographic direction is acyclic:

```text
trusted coordinator -> exact SHA256(E1) delivered out of band
E1 -> {M_static, guard/bootstrap, run plan, amendment/review, Gate1 attestations,
       exact S1 path/schema/absence policy}
S1 -> {E1 hash, M_static hash, static phase/attempt}
static receipt + fresh reviews -> I_static
pre-runtime phases -> sealed pre-runtime-finalize evidence
trusted coordinator -> exact SHA256(E2) delivered out of band
E2 -> {M_completion, same accepted guard/bootstrap/run plan/amendment/review,
       E1, S1 and receipt, I_static, sealed pre-runtime evidence, Gate2 attestations,
       exact S2 path/schema/absence policy}
S2 -> {E2 hash, M_completion hash, completion phase/attempt}
completion trace -> independent TSV/digest review -> later inherited gates
```

The run plan contains neither manifest nor envelope hashes. It predeclares exact phase order,
manifest/envelope paths, S1/S2 paths, private parent identities, modes, fixed output basenames and
parents, bounds, and no-overwrite transitions. `M_static` and `M_completion` do not contain a run-
plan hash, any envelope hash, their own hash, or future evidence. E1/E2 each bind the already-frozen
run-plan hash and applicable manifest hash. E2 points only backward to already-frozen E1/static/
post-static/pre-runtime artifacts. No predecessor points forward to E2.

E1/E2 use later-frozen canonical schema `c2b2a-external-authorization-envelope-v3`, with fixed-
order fields for schema, sequence, predecessor hash or root sentinel, epoch, phase, manifest
sequence/path/hash, guard/bootstrap identities, run-plan path/hash, this amendment/review hashes,
ordered sealed-attestation hashes, ordered prior-evidence hashes, spend-claim parent/path/schema/
absence policy, output policy, creation policy, and terminal LF. Neither has a self-hash or embeds
the out-of-band hash parameter used to open it. E1 has sequence 1/root/static and no future evidence.
E2 has sequence 2/predecessor E1/completion and complete already-existing prerequisite evidence.

The bootstrap invocation is later frozen as one exact argv, environment, cwd, umask, and FD grammar
with one lowercase-64-hex coordinator-supplied E hash in one fixed position. The bootstrap
stable-opens and authenticates that E, then every E-bound artifact and applicable manifest before
creating its spend claim. Candidate values can block later, but cannot flow backward into a
manifest, envelope, claim precondition, or coordinator expectation.

## Exact phase manifests

Both manifests use later-frozen canonical schema `c2b2a-phase-manifest-v1`. Each has one fixed
sequence, one exact phase, one exact final-L invocation record, one independently read deduplicated
input inventory, one exact output policy, fixed canonical serialization, maximum size, metadata,
SHA-256, LF/byte counts, and terminal LF. Unknown, missing, duplicate, reordered, optional,
placeholder, wildcard, glob, prefix-derived, environment-expanded, caller-selected, or unresolved
rows are terminal.

Every file row binds sequence, role, absolute path, file type, device, inode, UID, GID, mode, link
policy/count, byte count, LF count or exact `not-applicable`, lowercase SHA-256, and inherited
metadata policy. Every directory row binds exact absolute path, descriptor-resolved ancestry,
device/inode, UID/GID/mode/link policy, exact ordered direct-entry inventory, and entry metadata
required by the selected phase. Each invocation binds literal executable/interpreter identity,
phase, mode, argv, six-variable environment, cwd, umask, visible FDs 0/1/2, direct-parent topology,
limits, deadlines, status policy, input rows, output parent/basenames, and absence/transition policy.

Provisioner inclusion is exact, never "where applicable": a phase manifest contains the
provisioner row if and only if the frozen reviewed final-L invocation for that phase references or
opens the exact provisioner path. If referenced, omission is terminal. If not referenced, presence
is terminal. The same iff-referenced rule applies to every controller, combined-record, inherited
anchor, runner, build artifact, directory, and other selected-phase input.

### `M_static` sequence 1

`M_static` is constructed and frozen before Gate1, after final L/R and guard/run plan are frozen.
It contains exactly one invocation named `static`, profile `path-bound-b5-v1`, and no completion or
provider record. Its inventory contains only now-existing exact inputs required by that reviewed
static invocation. It excludes every future static trace, S1 content/hash, I_static, pre-runtime
artifact, M_completion, E2, S2, completion output/review, runner qualification, and provider result.

Static candidate output is exactly none: final L may create, modify, rename, link, or remove no file
or directory. Its only outputs are bounded FD 1 and FD 2 bytes captured separately by the guard in
an external owner-only trace namespace outside `_AUDIT_PARENT`. A closed exact filesystem baseline
for both selected and unselected namespaces is validated before and after.

### `M_completion` sequence 2

`M_completion` cannot be constructed, partially drafted, templated, or frozen until I_static is
accepted and the unchanged inherited runner/controller/provisioner/build chain has successfully
sealed `pre-runtime-finalize`. It is then constructed exactly once from independent stable reads of
every now-existing file and directory required by the reviewed final-L
`completion-audit-publish` invocation.

Its single invocation is named exactly `completion-audit-publish`, profile
`path-bound-b5-v1`. Its complete inventory includes final L/C/R and each exact inherited anchor or
provisioner iff referenced, all exact sealed pre-runtime files consumed by final L, exact build and
source identities consumed by final L, exact parent directories and their closed baselines, and
every other now-existing selected input. Each is represented by literal closed path, role,
inventory, hash/count/metadata. No future completion output, S2 content/hash, completion-review
content, later build/source proposal, runner qualification, or provider result appears in it.

Final-L completion filesystem output is exactly two create-new regular files inside
`_AUDIT_PARENT`: the exact predeclared completion TSV basename and exact predeclared digest
basename. Both must be absent in the exact parent identity before S2 and before child creation.
Their exact formats, maximum sizes, owner/group/mode, no-link policy, fsync/publication order, and
closed two-file transition are frozen in M_completion and the run plan. No directory, third file,
temporary residue, alternate basename, overwrite, or mutation is allowed.

The independent completion-review file is not a final-L output and is not in M_completion. Its
exact path inside `_AUDIT_PARENT` is predeclared by the run plan, absent throughout the child and
guard postcheck, and created no-overwrite by independent reviewers only after final L has exited,
all descendants are reaped, E2 authority postcheck succeeds, and the two outputs are sealed. It
cannot retroactively enter Q/R0/D/L/H/R or candidate runtime.

Both manifests are create-new and immutable. Neither is overwritten, repaired, normalized, or
updated. Any manifest/input/invocation change follows the invalidation rules below; it is never
represented by a placeholder or in-place correction.

## Atomic durable spend claims `S1` and `S2`

S1 and S2 live outside `_AUDIT_PARENT` in distinct frozen owner-only private parents. Their exact
descriptor-resolved parent identities and exact basenames are predeclared in the run plan and bound
by the applicable E creation policy. E binds the claim path, parent identity, canonical schema,
mode, and required absence, but not a future claim hash. After E exists, the claim canonically binds
that E hash without a cycle.

Immediately before any candidate child creation, direct bootstrap/guard must:

1. stable-open the frozen private parent directory no-follow and verify exact device/inode,
   UID/GID/mode/link and ancestry policy;
2. prove the exact claim basename absent, with no symlink, case-normalized alias, ancestry alias,
   hardlink, or inode alias in either phase namespace;
3. descriptor-relatively create it using `O_CREAT|O_EXCL|O_NOFOLLOW` and requested mode `0600`;
4. write canonical fixed-order content containing schema, epoch, exact E hash, exact manifest hash,
   manifest sequence, record/phase, attempt `1`, creation instant, and terminal LF;
5. require a complete bounded write, `fsync` the file, close the writer, reopen read-only no-follow,
   read back bounded EOF bytes, independently hash/count/parse them, repeat `fstat`, close, and
   `fsync` the held parent directory; and
6. only after every check succeeds, fork/posix_spawn/exec the candidate child.

An already-existing claim is terminal before child creation. Any create, write, fsync, reopen,
readback, hash, metadata, or parent-sync failure is terminal and no child starts. Once exclusive
creation succeeds, even a zero-byte or partial claim permanently spends that phase attempt. S1/S2
are retained forever on success, failure, crash, timeout, cancellation, partial start, or partial
publication. They are never deleted, truncated, renamed, repaired, normalized, recycled, or
recreated. A spent claim is at-most-one-start evidence, not completion evidence; completion also
requires the direct child/cleanup/authority receipt.

The guard envelope and coordinator receipt bind claim role/path, pre-absence proof, create result,
content bytes/hash, device/inode/UID/GID/mode/link/size, readback result, file and parent fsync facts,
creation-before-child ordering, child-start fact, and immutable postcheck. E2 additionally requires
the exact valid S1, its receipt, and I_static. S2 is revalidated after completion and included in
completion evidence, but E2 cannot bind S2's future content hash.

No same-phase retry is possible in this epoch. Any desire to try again after S1 or S2 exclusive
creation requires a separately named successor amendment/plan/manifest/envelope/claim namespace
and all reviews from the applicable first gate. No completed or partially started lane is replayed
or repaired.

## Identity-level namespace disjointness

Every authority, trace, spend, input, and output namespace has one fixed descriptor-resolved parent
identity plus exact basename inventory. Preflight rejects symlinked ancestry, case-normalization or
case-fold aliases, `..`/alternate spelling, hardlink aliases, duplicate device/inode identities,
parent/child ancestry overlap, or any selected/unselected namespace intersection. Literal string
difference alone is insufficient.

E1/E2, M_static/M_completion, S1/S2, guard/bootstrap records, design/Gate1/post-static/Gate2 raw
and sealed reviewer artifacts, raw FD captures, guard envelopes, receipts, and I_static all live
outside `_AUDIT_PARENT`. Static has no filesystem output; its FD1/FD2 capture paths are external and
disjoint from all candidate paths. Completion's only final-L outputs are the exact TSV and digest
inside `_AUDIT_PARENT`. The later independent completion-review file is the sole post-child review
placement exception and is reviewer-created there only after child/guard completion.

Before each child, the guard records the exact selected and unselected namespace baselines. After
drain/reap it proves the selected transition exactly and the entire unselected baseline unchanged.
It cannot validate only the union of allowed outputs. Any unexpected entry, alias, inode reuse,
selected/unselected mutation, or parent identity change is terminal.

## Audit-parent and runtime-anchor visibility

The inherited exact six amendment/review runtime anchors remain candidate-readable inputs in their
unchanged order, roles, hashes, and scope. This amendment adds no seventh runtime anchor and does
not remove or relabel any of the six.

The two preserved rejected amendment basenames are pre-existing `_AUDIT_PARENT` baseline entries:

```text
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md
```

Their exact basename/type/device/inode metadata is included as pre-existing baseline metadata
before R0. Their prospective review paths are absent, remain uncreated, and are unauthorized:

```text
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_REVIEW_2026-09-10.md
```

Among amendment/design-review metadata, only this `...TWO_MANIFEST_FROZEN_2026-09-10.md`
amendment and its eventual exact `...TWO_MANIFEST_REVIEW_2026-09-10.md` design review are newly
frozen before R0. They are not runtime anchors and final L does not read their contents. Only their
declared basename/type/device/inode metadata enters `audit_parent_snapshot_hex` and transitively
R0/D/L/H/R. Their content, hashes, verdicts, and findings do not enter candidate runtime. Inherited
output basenames and their required absent prestates retain their separate inherited treatment.

The later completion TSV and digest are the exact M_completion-selected output transition. Their
independent completion-review path is absent during R0 construction and candidate execution, then
created only after the completion guard has sealed success. That reviewer-created post-child file
is a forward lifecycle artifact; no later bytes flow backward into Q/R0/D/L/H/R.

No new reviewer content, content hash, verdict, count, findings digest, attestation, or raw report
enters final-L argv, environment, stdin, FDs, source, Q/H, combined-record content, static vectors,
provider input, payload, or candidate runtime. The sole exceptions are (a) unchanged inherited
review documents among the fixed six runtime anchors and (b) only the declared audit-parent
basename/type/device/inode metadata for this amendment and eventual design review. Sealed new
review authority is consumed only by bootstrap/guard/coordinator.

## Exact narrow supersession matrix

Only these named sections and permission meanings are superseded. All unlisted clauses remain.

| Inherited frozen document | Exact named section(s) | Two-manifest successor meaning |
| --- | --- | --- |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md` | **Finite implementation-preflight authority** | One-hole projection and finite construction remain. Embedded semantic digest, self-hash comparison, self-stable-read, and self-contract receipts are blocking regression checks, never execution permission. Applicable E/manifest/spend/guard/review gates provide permission. |
| same | **Closed preflight-only static test** | Vector semantics, isolation intent, output/status/deadline bounds, and fail-closed behavior remain. Static runs only once through Gate1/E1/S1, final L, M_static, and `path-bound-b5-v1`; it has no filesystem output. |
| same | **Closed production launcher role** | The production-like operation at this boundary is completion-audit-publish after sealed pre-runtime-finalize, not provider activation. Scanner/capture/publication/deadline/no-repair semantics remain. E2/S2 authorize only final-L completion audit. |
| same | **Outer-launcher completion witness** | Coordinator starts reviewed guard; guard directly parents final L and captures raw streams/status/resources. Guard and coordinator produce direct receipts. No reviewer starts final L without guard. |
| same | **Implementation-preflight additions and sole early-execution exception** | Self-source obligations remain blocking regression evidence. The early exception requires accepted E1/M_static/S1/guard/Gate1; later completion requires I_static, sealed pre-runtime-finalize, E2/M_completion/S2/Gate2. |
| same | **Acceptance boundary** | Gate1 permits one static invocation. Post-static I_static gates pre-runtime work. Gate2 permits one completion-audit-publish invocation. Neither gate permits providers. |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_FROZEN_2026-09-08.md` | **Symbols and exact domains**, **Launcher-independent authority first**, **Exact launcher normalization `Q(L)`** | Domains/equations remain construction evidence. Final expected identities are independently inventoried in the applicable phase manifest. Candidate S/D/H derivation cannot grant permission. |
| same | **Two exact source-analysis epochs** | Both epochs are rejection-only regression analyses. Their reads, ASTs, copies, equality, and receipts cannot authenticate their own effective objects. |
| same | **Semantic identity in the 26 proof rows**, **Canonical proof-row record**, **Exact combined-authority transport**, **Exact source-row to static-vector mapping** | All rows, W0/W1, F0/F1, normalized facts, spans, roles, digests, scalar/index fields, and source-derived expectations remain bounded diagnostic transport only. |
| same | **Q-invariant static proof and expected-output independence**, **Exact construction and final revalidation**, **Fail-closed source and projection witnesses** | Q-invariance, transition guards, matcher/pattern recursion, self-contract receipts, mutations, and replay remain blocking only. Success never grants execution. |
| same | **Narrow supersessions**, **Untouched authority**, **Non-expansion and execution prohibition**, **Acceptance gates** | Only external permission, two-manifest lifecycle, audit-parent transitions, and durable spends change. Complete DAG, sole raw H field, six anchors, prohibitions, no-repair, and non-expansion remain. |
| `NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md` | **Implementation-preflight and launcher obligations**, **Non-executing authority and adapter source proof** | Source cases, codec extraction, adapter/authority AST facts, callsite/dominance evidence, and launcher/controller agreement remain rejection-capable, not permission authority. |
| same | **Deterministic tests and mutation witnesses** | Every source mutation/Q250 inventory is quarantined regression evidence. Non-source carrier fixtures and later runtime qualification retain inherited meaning. |
| same | **Acceptance boundary** | Source review and codec/static results remain necessary but require applicable external envelope/manifest/spend and lifecycle gates. |

The carrier's 75-row table, three edges, 29 roots, 26 parent entries, ten absences, six
observations, producer commit, consumer dominance, descriptor-relative capture, historical/graph
replay, bounds, no-writer schedule, and no-repair/no-replay rules remain unchanged.

## Complete preserved construction DAG and outer lifecycle

The complete inherited construction remains exactly:

```text
U -> A -> C -> HC -> L0/Q -> S -> W0/F0 -> components/B/V/O
  -> R0 -> D -> G_D -> L -> H -> G_H -> R
  -> authoritative W1/F1/replay
```

The exact invariants remain `Q(L) = L0` and `P(R) = R0`. H occurs raw only in the sole
`launcher_source_sha256` field removed by P. The exact six runtime anchors stay ordered; there is
no seventh. Provider output is never authority.

`L` is the final launcher after one-hole Q construction. If Q is filled, its identity is not b5
`b5e50318...`. B5 remains precursor evidence; final L receives its own exact hash/stable reads and
independent reviews. No b5 identity or review transfers.

The acyclic outer lifecycle is:

```text
inherited Q/R0/D/L/H/R
  -> {M_static, run plan}
  -> Gate1 reviews -> E1 -> S1 -> static trace/receipt
  -> post-static reviews -> I_static
  -> inherited controller/provisioner/build chain -> sealed pre-runtime-finalize
  -> M_completion -> Gate2 reviews -> E2 -> S2
  -> completion TSV/digest -> independent completion review
  -> build/source identity proposal+review
  -> provider-free Linux runner freeze/review
  -> runtime qualification+reviews
  -> remaining inherited gates
  -> eventual provider activation/evaluation via unchanged pilot runner
```

No M_static, run-plan, review, E, spend, trace, I_static, pre-runtime, M_completion, completion, or
later provider byte flows backward into Q/R0/D/L/H/R. Every inherited DAG node keeps finite,
no-iteration, no-repair construction and blocking semantics. `G_D`, `G_H`, authoritative W1/F1,
and replay reject mismatch but cannot grant execution.

## Guard/bootstrap trust and path-bound profile

E1/E2, not a phase manifest, bind guard/bootstrap and run-plan identity. Coordinator independently
holds the accepted bootstrap identity used to authenticate an E. A native guard requires exact
source, build recipe/toolchain inputs, executable hash, and accepted source-to-executable binding.
An interpreted guard requires exact source hash, pinned guard-interpreter identity, isolated
invocation, and complete declared guard-input policy. Hashing source while executing another binary
is insufficient. Guard design/source/executable/interpreter/bootstrap receive separate reviews.

The sole current profile is `path-bound-b5-v1`, compatible with current topology and final L
derived from it. Guard is final L's direct parent. It executes the applicable manifest-pinned Python
against original repository `_LAUNCHER` with exact phase argv, six-variable environment, cwd
`/var/empty`, incoming umask `0077`, and only FDs 0/1/2 visible to final L. FD0 is inherited
read-only `/dev/null`; FD1/FD2 are distinct nonseekable guard-owned pipes. No source, guard, E,
manifest, review, or other FD is inherited.

Before spend/child creation the guard stable-opens and hashes every applicable phase-manifest input
and directory baseline. It keeps guard-side descriptors held across the child where specified,
without passing them. Held descriptors authenticate guard observations; because final L reopens
pathnames, the same-UID interval nonclaim still applies exactly as stated above. After drain/reap,
the guard repeats applicable input, selected-output, unselected-namespace, spend, and E-bound
authority validation.

Every final-L pathname-open interval is covered only by that same-UID nonclaim. Pre/post equality
detects ordinary drift, not malicious interval swap-and-restore. No report may claim stronger.

Optional future `fd-snapshot-v1` is unauthorized. It requires a new launcher epoch accepting held
read-only immutable FDs and never reopening source paths, plus separately reviewed copy/fsync/
writer-close/read-only-reopen/pread/offset/link-count rules. Moving profiles invalidates launcher,
guard, phase manifests, envelopes, Q/H evidence, spends, and traces.

## Direct guard channels and external-authority postcheck

Guard captures raw child stdout and stderr in distinct bounded streams outside `_AUDIT_PARENT`.
They remain separate preserved evidence, not reframed candidate output. A third coordinator-owned
channel carries canonical `c2b2a-guard-observation-envelope-v3` facts: epoch/phase, E/manifest,
spend claim, invocation, input precheck, child PID/start/PGID/SID, monotonic deadlines, resource/
overflow facts, wait status, cleanup/reap/session-empty facts, raw stream byte counts/hashes,
selected/unselected filesystem transition, authority pre/post receipts, failures, and validity.

Before dispatch, bootstrap/guard and coordinator independently establish an authority precheck
over held selected E and every E-bound artifact: applicable manifest, guard/bootstrap source and
executable/interpreter, run plan, this amendment/review, sealed attestations, ordered prior evidence,
and optional oracle. Every regular authority file is stable-opened no-follow and held through
postcheck; directory ancestors are descriptor-resolved and held. For path-bound candidate inputs,
the guard may hold its own descriptors but final L's independent reopen remains within the same-UID
nonclaim. A required artifact that cannot meet its specified held-descriptor policy makes the epoch
unimplementable and terminal; there is no silent qualifier or fallback.

Precheck records per role/path type, device/inode, UID/GID/mode/link, size, hash, byte/LF counts,
and repeated fstat, plus supplied E hash, held-E hash, sequence/phase, manifest identity, spend
absence, and guard identity. After child and descendants drain/reap and namespace postchecks,
guard/coordinator repeat observations from held descriptors and stable bindings before acceptance.

The canonical receipt contains per-row pre/post facts and equality, held-E pre/post hashes/fstats,
E-bound pre/post inventory digests, spend prestate/content/readback/fsync/poststate, child-start
ordering, selected invocation consumed once, no other phase invocation, selected namespace exact,
unselected namespace unchanged, coordinator E-hash match, and terminal
`authority_revalidation_valid`.

Acceptance sealing occurs only after exact authority equality, held E still matching the out-of-
band value, every E field resolving to the same held artifact, applicable manifest byte-identical,
spend immutable, exact invocation consumed once, and all path/metadata policies holding. Any absent,
extra, reordered, rebound, mutated, unreadable, or un-revalidated object is terminal. Raw evidence
is preserved but success is not sealed. This does not strengthen the same-UID nonclaim.

## Review attestations, gates, and lifecycle acceptance

Every review creates one no-overwrite canonical `c2b2a-external-review-attestation-v3` outside
`_AUDIT_PARENT`, with fixed schema/epoch/gate/phase/role, exact ordered input hashes, harness,
provider, model, session, canonical UTC start/end, P0/P1/P2 counts, `pass|fail`, and complete sealed
findings digest. Pass requires P0=0/P1=0; P2 remains reported. Attestations contain no self-hash,
future E hash, peer verdict, or candidate-derived expected identity.

Artifact writers cannot review their artifact. Same-gate reviewers remain blind to peers until all
reports are sealed. Each three-review gate uses at least two distinct harness/provider families;
one family supplies at most two. This reduces but does not eliminate correlated training,
implementation, host-state, or conceptual blind spots.

Review roles remain semantic correctness, guard/TOCTOU/topology, and broad adversarial closure.
They explicitly test effective-object mutation/`__code__`/rebinding/builtin shadowing, E/manifest/
run-plan acyclicity, source-to-executable binding, path/FD/channel/deadline/cleanup facts, spend
atomicity/durability, stale/substituted authority, matcher co-edit, self-fulfilled vectors, output
forgery, undeclared reads, aliasing, namespace widening, review leakage, and post-run substitution.

The exact gates are:

1. **Design acceptance.** Three clean blind same-hash reviews accept only this amendment design.
   Their consolidated TWO_MANIFEST_REVIEW file is created/frozen before R0. No execution follows
   from design acceptance.
2. **Gate1 pre-execution static eligibility.** Three fresh clean blind reviews cover final L/C/R,
   provisioner exactly iff referenced, every M_static input, exact M_static, run plan,
   guard/bootstrap binding, this amendment/review, S1 policy, namespaces, and every inherited
   prerequisite. Only then coordinator creates E1 and supplies its hash out of band. E1 permits
   only spending S1 and one guarded final-L static invocation.
3. **Post-static implementation acceptance.** After successful guard postcheck, three fresh direct
   reviews inspect immutable E1/S1, spend and authority receipts, raw FD1/FD2, guard status/resources,
   zero filesystem output, unselected baseline, semantic result, and unchanged Gate1 inputs. The
   coordinator creates canonical no-overwrite `I_static` outside `_AUDIT_PARENT` binding all review
   attestations and exact evidence. Without I_static, no inherited pre-runtime phase starts.
4. **Sealed pre-runtime-finalize.** Only after I_static, unchanged inherited runner/controller/
   provisioner/build phases run through every inherited gate and seal their exact pre-runtime-finalize
   evidence. These are not provider activation/evaluation. Failure preserves traces and stops.
5. **Gate2 completion-audit eligibility.** After M_completion is independently frozen, three fresh
   clean blind reviews cover its exact single completion invocation and every literal input/
   directory baseline, E1/S1/receipt/I_static, sealed pre-runtime evidence, unchanged guard/run plan/
   final L, S2 policy, exact TSV/digest transition, and unselected namespaces. Only then coordinator
   creates E2 and supplies its hash out of band. E2 permits only spending S2 and one guarded final-L
   completion-audit-publish invocation.
6. **Independent completion review.** After successful E2 postcheck and sealed TSV/digest, fresh
   independent reviewers directly validate the exact TSV/digest bytes, hashes, semantics, source
   observations, receipts, namespaces, and inherited completion criteria. Only then may the exact
   no-overwrite completion-review file be created inside `_AUDIT_PARENT`.

Completion review does not authorize providers. It is followed in order by build/source identity
proposal and review, provider-free Linux runner freeze and review, runtime qualification and
reviews, and every remaining inherited gate. Only after all of them may the unchanged pilot runner
eventually perform already-authorized provider activation/evaluation. E2 is never passed to final
provider lanes and grants them no authority.

No new reviewer content/hash/verdict/finding enters candidate runtime. The only review-content
exception is the unchanged inherited review documents among the exact six runtime anchors; the
only metadata exception is the declared basename/type/device/inode projection for this amendment
and eventual design review. Completion review is post-child and cannot become a candidate input.

## Exact invalidation and retry boundaries

Invalidation is exact and fail closed:

- Any shared source/identity change to guard/bootstrap, interpreter, run plan, final L/Q/R,
  controller, provisioner iff referenced, inherited anchor, shared bound, audit-parent pre-R0
  baseline, this amendment/review, or M_static invalidates from Gate1. It requires a new successor
  design epoch and no prior review transfers.
- A static trace/status/receipt or post-static review mismatch stops before I_static. Mutation of
  accepted E1/S1/static evidence/I_static invalidates from post-static acceptance; S1 remains spent,
  so repeating static requires a separately named successor and fresh Gate1.
- A post-static-only pre-runtime input/evidence change, with all shared Gate1 inputs and I_static
  unchanged, invalidates from sealed pre-runtime-finalize/Gate2 rather than retroactively changing
  the historical static result. M_completion cannot be edited; continuation requires a separately
  named successor later-phase epoch and fresh Gate2 authority.
- Any M_completion row, inventory, metadata, invocation, output baseline, Gate2 attestation, or E2
  change invalidates from Gate2. If S2 has been created, it remains spent and any completion retry
  requires a separately named successor with fresh M_completion/E2/S2 namespace and reviews.
- Any phase mismatch, alternate invocation, replay, second start, missing/duplicate/reordered row,
  placeholder/wildcard, selected/unselected alias, union-widened namespace, output outside exact
  policy, post-run authority failure, path rebinding, or prior-object replacement is terminal even
  when a content hash matches.

Modification, removal, truncation, replacement, renaming, or recreation of an append-only prior
object is terminal. Only exact predeclared no-overwrite forward creation is permitted. S1/S2 are
never removed. On failure execute nothing further, preserve bytes/traces/partial outputs, and never
delete, repair, normalize, overwrite, or replay them. Partial completion publication requires a
separately reviewed recovery freeze. User-owned worktree changes remain preserved.

## Preconditions and finite transition

Before any candidate/static/build/runtime/provider execution:

1. freeze this amendment and obtain three clean same-hash design attestations using at least two
   harness/provider families;
2. create/freeze/accept its exact TWO_MANIFEST_REVIEW file, then freeze only this amendment/review's
   new audit-parent metadata with the pre-existing 7d/f1 baselines before R0;
3. freeze/review guard/bootstrap, source-to-executable binding, envelopes/manifests/spend grammars,
   path-bound topology, channels, resources/deadlines, namespaces, and authority postcheck;
4. review exact b5 and eventual final L for semantic, guard/TOCTOU, and broad adversarial closure;
5. construct once the complete inherited Q/R0/D/L/H/R DAG, freeze final L/C/R, and require every
   internal blocking check without treating it as permission;
6. freeze run plan and M_static from independent stable reads, require pristine metadata, positive
   disk reserve, absent `target/debug`, absent combined output before its sole inherited create-new
   publication, and every inherited auth/no-replay/no-repair/phase-order gate;
7. pass Gate1, create E1, atomically spend S1 before child, run final-L static once, complete
   authority postcheck, pass post-static reviews, and create I_static;
8. only then run unchanged inherited runner/controller/provisioner/build phases through sealed
   pre-runtime-finalize; construct M_completion only after every required input exists;
9. pass Gate2, create E2, atomically spend S2 before child, run final-L
   completion-audit-publish once, complete authority/namespace postchecks, and independently review
   exact TSV/digest before creating the completion-review file; and
10. subsequently complete build/source identity proposal+review, provider-free Linux runner
    freeze/review, runtime qualification+reviews, and every remaining inherited gate before any
    eventual provider activation/evaluation through the unchanged pilot runner.

No raw/sealed future reviewer output enters the DAG, R, candidate source, static vectors, provider
prompt, payload, or candidate runtime. No step above is authorized by this candidate document now.

## Exact b5 and final-L disposition

Exact `b5e50318...` remains an unexecuted candidate. External hashing distinguishes bytes but does
not establish benign semantics. Review must exclude `__code__` replacement, builtin/`ValueError`
shadowing, effective-object mutation, alternate paths, and unfaithful evaluation.

If Q fills the inherited 64-byte interval, final L has a new exact hash. B5 is precursor evidence
only. Final L must be independently entered in each applicable phase manifest and pass all reviews.
No b5 review transfers across that change. Self-tests remain defense in depth. Final L serves the
two lifecycle invocations only via distinct phase manifests and envelopes under
`path-bound-b5-v1`; it never receives both authorities at once.

## Acceptance boundary and current exact disposition

This amendment is acceptable only if three blind same-hash design reviews return P0=0/P1=0, use at
least two harness/provider families, cover all corrections, and are bound in the one accepted
TWO_MANIFEST_REVIEW record. Acceptance confirms design only; it accepts no guard, manifest,
envelope, spend, source, Q fill, static/build/runtime/provider result, completion audit, pilot, or
flagship goal.

Acceptance must confirm:

- two phase-correct manifests, E1/static and E2/completion only, with no future placeholders;
- exact lifecycle through I_static and sealed pre-runtime-finalize before M_completion/Gate2;
- E2 authorizes completion audit only, followed by all later inherited gates before providers;
- durable exclusive S1/S2 creation and permanent retention on every outcome;
- identity-level namespace disjointness, no static file output, and exact completion TSV/digest;
- preserved 7d/f1 baseline metadata, absent predecessor review paths, and only new two-manifest
  amendment/review metadata frozen before R0;
- complete inherited DAG with no later bytes flowing backward and final L distinct from b5;
- exact iff-referenced provisioner/input policy and closed independent inventories;
- direct guard parenthood, only FDs 0/1/2, path-bound same-UID nonclaim, raw channels, and complete
  E-bound pre/post authority revalidation;
- six inherited anchors, reviewer diversity/correlation nonclaim, no review leakage, and all
  inherited no-replay/no-repair gates; and
- exact later-gate invalidation without retroactive authority or same-claim retry.

This amendment adds no provider, model, budget, VM, pilot lane, adapter, daemon, datastore,
connector, network, authentication, product behavior, user data, runtime anchor, recovery mode, or
source mutation. It authorizes no stage, commit, publication, live setting change, or
`target/debug` creation.

At present its prospective review does not exist. There is no execution authorization, Q fill,
combined-record publication, spend claim, manifest, envelope, or controller/provisioner change.
Launcher `b5e50318...`, controller `eefa6eec...`, and provisioner `aabe6b17...` remain unexecuted
candidate bytes. The rejected 7d and f1 files remain immutable historical evidence.
