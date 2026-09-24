# Native strict successor Stage C2B2a — two-manifest enforcement boundary

Date: 2026-09-10
Status: enforcement frozen candidate design amendment; no implementation or execution authority

## Disposition and rejected predecessors

This amendment replaces recursive launcher self-authentication as execution permission with an
externally rooted, exact-hash, two-manifest authorization lifecycle. Candidate self-checks remain
fail-closed regression evidence: a failure blocks; success never grants execution.

The immediately preceding two-manifest epoch is rejected and preserved byte-for-byte:

```text
SHA-256 74c883fec8e075ddc20a5977fc5a36751f156e1a170554b3a414f3aca27fd0e2
lines   733
bytes   48654
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

Its direct reviews were:

```text
Codex normative review   FAIL  P0=0 P1=1 P2=2
Codex guard review       FAIL  P0=0 P1=1 P2=2
isolated Claude review   PASS  P0=0 P1=0 P2=5
aggregate gate           FAIL
```

The isolated pass cannot override either fail and no verdict transfers. Exact 74c left
`I_static` execution-authorizing without canonical identity, creation, and continuous validation,
and its blanket namespace-disjointness claim conflicted with `_AUDIT_PARENT`'s intentional mixed
input/output role. Those are enforcement-boundary failures, so 74c remains rejected raw evidence.

The earlier one-manifest/two-record epoch remains rejected and preserved:

```text
SHA-256 f1d7ddc8371c2a035f527f6d1784fe3f225cc89b3e0bac23e1616bc1f7a8ebaf
lines   690
bytes   47347
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md
state   rejected; no authority transfers
```

It froze future inputs before they existed, misplaced completion authorization, lacked durable
atomic spends, and did not close output baselines. The earlier corrected-amendment epoch also
remains rejected and preserved:

```text
SHA-256 7d0ae76f3fd721d61fc3797d3627c4cc4a99d413e7f2c39e4ae47417dc42ec80
lines   556
bytes   36410
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
state   rejected; no authority transfers
```

An older rejected epoch was recorded with SHA-256
`1cd2c842fa4fa25415b03623d71458f0377c7cbd73d0de40b1bfb0a79cd5f689`, 566 lines, and 38,215
bytes. Its raw bytes are unavailable and can confer no partial authority under any interpretation.
No property of this amendment depends on characterizing its defects. The digest and historical
review trace are provenance only, not re-reviewable evidence.

This enforcement amendment is itself only a candidate. It authorizes no source import,
compilation, AST parse, Q fill, candidate/static/build/runtime/provider/pilot execution, VM,
daemon, adapter, datastore, publication, or other runtime action until its exact final bytes and
separate design review pass every gate below.

## Exact inherited authorities and current candidates

This amendment is subordinate to these exact frozen records/reviews except only for the named
supersessions below:

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
state   historical/candidate bytes; not authorized to execute

SHA-256 eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
lines   27130
bytes   1067693
path    engram-eval/native-c2b2a-payload/build-support/controller.py
state   candidate; PREFLIGHT_AUTHORITY_SHA256 remains the zero placeholder expression

SHA-256 aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
lines   4349
bytes   160871
path    engram-eval/native-c2b2a-payload/build-support/provision.py
state   retained candidate; unchanged by this amendment
```

This amendment's exact path is:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_FROZEN_2026-09-10.md
```

Its prospective design-review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_REVIEW_2026-09-10.md
```

That review does not exist. It may be created once, no-overwrite, only after three clean same-hash
design attestations, then frozen before R0 and never modified.

## Why candidate self-proof cannot bootstrap trust

Two deliberately narrow reviews of exact b5 passed helper/matcher scopes. A broader adversarial
review found that reachable code could replace an authenticated callable's `__code__` while the
checked source body remained, and could shadow builtin `ValueError` while narrow checks passed. A
validator co-editable with its matcher, expected values, receipts, and effective objects cannot
make its own success external fact. All recursive/self-source checks remain rejection-only.

## Threat model and explicit nonclaims

The trust root is an honest, uncompromised coordinator retaining accepted exact hashes out of band.
The design assumes collision-resistant SHA-256, honest reviewed guard/bootstrap and phase guard,
an honest pinned interpreter/toolchain, an honest kernel/filesystem/durable-storage implementation,
and reviewers inspecting exact named bytes.

This unprivileged design does not defend against:

- SHA-256 collision/second-preimage or compromised hashing;
- compromised coordinator, reviewer, guard, bootstrap, phase guard, interpreter, toolchain, kernel,
  hypervisor, filesystem, storage device, administrator, dynamic loader, or shared cache;
- colluding or negligent reviewers;
- same-UID debugger, tracing/injection, writable process memory, or equivalent host mechanism;
- same-UID pathname swap-and-restore during final L's pathname-open intervals;
- malicious same-UID precreation of a spend, acceptance, output, or review basename before the
  trusted coordinator's checked transition;
- ambient same-UID filesystem access by malicious exact code without an OS sandbox; or
- semantic correctness merely from an accepted content hash.

Owner-private paths reduce accidental exposure but do not authenticate against same UID/root.
Held descriptors authenticate guard observations under the honest-host model. Final L still
reopens pathname inputs with only FDs 0/1/2, so the same-UID interval nonclaim remains exact.
Durable spends provide honest-host at-most-one-start evidence; they do not survive a lying host or
storage stack. No detached-signature claim is made without an independently protected key.

## Complete candidate-controlled input closure and enumerated host TCB

Each phase manifest is complete for every candidate-controlled semantic input to its exact final-L
invocation. Completeness covers every repository file, writable/generated file, plugin or hook,
import-path entry, configuration, data file, executable, script, environment value, argv value,
working directory, FD identity, and directory inventory whose bytes, presence, absence, metadata,
ordering, or resolution can affect candidate behavior. Semantic review must prove the exact source
does not reach an undeclared candidate-controlled input.

Only these host components are excluded from manifest rows and treated as ambient TCB:

1. kernel/syscall/process/filesystem behavior;
2. dyld and the operating-system shared cache; and
3. each exact unpinned standard-library component named in the later frozen interpreter policy.

The standard-library exclusion is a closed named list, not a prefix, directory wildcard, import-
path allowance, or permission to omit site packages. Ambient TCB cannot launder repository,
writable, generated, plugin, import-path, configuration, or data inputs. Any such semantic input
requires an exact row. This is finite declared-input closure, not a whole-host closure claim.

Provisioner inclusion is exact: a phase manifest contains its exact row if and only if the frozen
reviewed final-L invocation references or opens it. Omission when referenced and presence when not
referenced are terminal. The identical iff-referenced rule applies to controller, runner, build
artifact, inherited anchor, directory, or any other candidate-controlled input.

## Sound two-manifest lifecycle and acyclic authority

There are exactly two create-new immutable phase manifests and two envelopes:

- `M_static`, sequence 1, contains exactly one final-L static invocation;
- `M_completion`, sequence 2, contains exactly one final-L
  `completion-audit-publish` invocation;
- `E1` binds/selects only M_static; and
- `E2` binds/selects only M_completion plus accepted earlier evidence.

There is no shared two-record manifest, third manifest, third candidate record, or third envelope.
Each manifest contains only existing independently observed facts when frozen.

The normative lifecycle is exactly:

```text
design acceptance
  -> Gate0 guard/phase-enforcement acceptance
  -> inherited Q/R0/D/L/H/R construction yielding final L and R
  -> run plan + M_static
  -> Gate1 reviews -> E1 -> S1 -> guarded final-L static once
  -> post-static reviews -> canonical I_static
  -> exact inherited pre-runtime phases with per-phase I_static pre/post receipts
  -> sealed pre-runtime-finalize
  -> M_completion -> Gate2 reviews -> E2 -> S2
  -> guarded final-L completion-audit-publish once
  -> three clean completion attestations -> atomic completion review
  -> build/source identity proposal and review
  -> provider-free Linux runner freeze and review
  -> runtime qualification and reviews
  -> every remaining inherited gate
  -> only eventually, provider activation/evaluation through unchanged pilot runner/run plan
```

Completion audit is after pre-runtime-finalize and before provider activation/evaluation. E2 grants
only completion-audit authority. It cannot directly invoke or authorize a provider, activation or
evaluation lane, pilot runner, model, network call, or budget spend.

The exact cryptographic direction is:

```text
coordinator -> expected SHA256(E1)
E1 -> {Gate0 artifact, M_static, guard/bootstrap/phase guard, run plan,
       enforcement amendment/review, Gate1 attestations, S1 path/schema/absence policy,
       I_static path/schema/bounds/absence/creation policy}
S1 -> {E1 hash, M_static hash, static phase/attempt}
static evidence + three post-static attestations -> I_static
I_static + per-phase receipts -> sealed pre-runtime-finalize
coordinator -> expected SHA256(E2)
E2 -> {same Gate0/guard/run plan/amendment/review, M_completion,
       E1/M_static/S1/static evidence, exact I_static, ordered pre-runtime receipts,
       sealed pre-runtime-finalize, Gate2 attestations, S2 policy, completion-output policy}
S2 -> {E2 hash, M_completion hash, completion phase/attempt}
completion evidence + three completion attestations -> completion review -> later inherited gates
```

Run plan embeds no manifest/envelope/acceptance future hash. It predeclares exact lifecycle, phase
identifiers, manifest/envelope/spend/I_static paths, private-parent identities, output/review paths,
bounds, and no-overwrite transitions. Manifests contain no self/E/future hash. E1/E2 bind the frozen
run-plan hash and applicable existing artifacts. E1 binds I_static path/schema/absence/creation
policy but no future inode/content/hash. E2 points only backward. No predecessor points to E2.

Envelopes use later-frozen canonical `c2b2a-external-authorization-envelope-v4`: schema, sequence,
predecessor/root, epoch, phase, manifest sequence/path/hash, Gate0 and guard identities, run-plan
path/hash, amendment/review hashes, ordered attestation/evidence hashes, spend policy, output policy,
creation policy, terminal LF. They contain no self-hash or out-of-band expected hash.

## Gate0 guard source-to-executable and enforcement acceptance

Gate0 occurs after design acceptance and before Q construction or any candidate/build/runtime
action. Three blind clean reviews, with P0=0/P1=0 and at least two harness/provider families, cover:

- native guard source, build recipe/toolchain inputs, final executable, and exact accepted source-
  to-executable binding; or interpreted guard source, pinned guard interpreter, invocation, and
  complete guard-input policy;
- bootstrap, E/manifest/spend parsers, envelope opening, coordinator hash delivery, and direct-
  parent process construction;
- I_static schema/parser/writer, trusted coordinator state, per-phase guard, inherited phase-list
  enforcement, and continuous pre/post validation;
- static/completion deadlines, raw channels, process cleanup, resource limits, audit-parent mixed-
  role state machine, external namespaces, spend/acceptance/review creation, and authority postcheck.

The coordinator seals one exact Gate0 acceptance artifact outside `_AUDIT_PARENT`, binding all
three ordered attestations and exact reviewed inputs. The writer of any artifact cannot review it;
same-gate reviewers remain blind until sealed. E1 and E2 must bind the unchanged Gate0 artifact.
Any Gate0 input change invalidates Gate0 and every later gate. Gate0 accepts enforcement machinery
only; it authorizes no Q fill, final L, child, build, runtime, or provider action by itself.

## Exact phase manifests

Both manifests use later-frozen canonical `c2b2a-phase-manifest-v2`. Each has one sequence, phase,
exact final-L invocation, complete independently observed semantic-input inventory, exact output
policy, canonical serialization/max size/metadata/hash/counts/final LF. Unknown, missing,
duplicate, reordered, optional, placeholder, wildcard, glob, prefix-derived, environment-expanded,
caller-selected, or unresolved rows are terminal.

Every file row binds sequence/role/absolute path/type/device/inode/UID/GID/mode/link count and
policy/byte count/LF count or exact `not-applicable`/SHA-256. Directory rows bind descriptor-
resolved ancestry, identity/metadata, exact ordered direct-entry inventory, and recursive inventory
only for explicitly named directories. Invocation binds interpreter identity, phase/mode/argv,
exact six-variable environment, cwd, umask, FDs 0/1/2, topology, limits/deadlines/status, input rows,
and selected filesystem state transition.

`M_static` is frozen after Gate0 and final L/R/run plan, before Gate1. It has exactly one `static`
invocation under `path-bound-b5-v1`, only existing static semantic inputs, the first complete
canonical observation of audit-parent prestate P below, and no completion/provider record. It
excludes S1 content/hash, static trace, I_static, pre-runtime evidence, M_completion, E2, S2,
completion outputs/review, qualification, and provider results. Static final L has no filesystem
output; only bounded raw FD1/FD2 captured outside `_AUDIT_PARENT` by guard.

`M_completion` cannot be drafted, templated, or frozen until canonical I_static is accepted and
every exact inherited pre-runtime phase has sealed its receipt and pre-runtime-finalize. It then
inventories every now-existing candidate-controlled semantic file/directory required by the exact
single `completion-audit-publish` invocation, including exact final L/C/R, provisioner and six
anchors iff referenced, generated/build/source inputs, import/config/data entries, and exact
`_AUDIT_PARENT` prestate P below. No future S2 content/hash, completion output/review, later
identity proposal, qualification, or provider result appears.

Both manifests are create-new immutable files outside `_AUDIT_PARENT`. They are never overwritten,
repaired, normalized, or updated. A phase/source/input change follows exact invalidation; it is not
represented by an in-place edit or placeholder.

## Canonical `I_static` and continuous pre-runtime enforcement

`I_static` uses exact schema `c2b2a-static-implementation-acceptance-v1`. It is the sole
coordinator-issued implementation acceptance for the exact ordered inherited pre-runtime phase
list. It is necessary but never sufficient: each listed phase also requires its unchanged inherited
gate, exact per-phase durable replay control, authority precheck, execution receipt, and postcheck.
I_static authorizes no final-L invocation, completion audit, provider, activation, evaluation,
pilot, unlisted phase, reordered phase, retry, or source/input change.

E1 and the run plan predeclare I_static's exact owner-private external parent path and held
device/inode/UID/GID/mode/link/ancestry policy, exact basename, schema, maximum LF/byte bounds,
required absence, requested `0600`, and create-new protocol. Neither binds future I_static content,
inode, or hash. The run plan enumerates the real inherited pre-runtime phase identifiers in one
literal fixed order; no class name, prefix, wildcard, optional row, or caller-selected list is
allowed.

Canonical I_static content is fixed-order and length-delimited. It binds:

1. schema, authorization epoch, exact `pass` status, and exact scope
   `inherited-pre-runtime-phases-only`;
2. run-plan path/hash plus the exact ordered pre-runtime phase list and its canonical digest;
3. one canonical length-delimited Gate1 input-set digest;
4. full role/path/type/device/inode/UID/GID/mode/nlink/size/count/SHA-256 rows for E1, M_static, S1,
   static guard envelope, static coordinator receipt, raw FD1, and raw FD2; and
5. exactly three ordered post-static attestations in roles `semantic`, `guard`, `adversarial`, each
   with exact metadata/hash and P0=0/P1=0.

It contains no self-hash, future E2/M_completion/S2 hash, future phase receipt, provider result, or
candidate-derived expected identity. The coordinator alone writes it after directly accepting all
three sealed post-static attestations and all bound evidence. Coordinator retains expected
`SHA256(I_static)` only in trusted state and later gives it to reviewed phase guards out of band.

The single-writer creation protocol is exact: stable-open/hold parent no-follow; verify identity,
ancestry, owner-private metadata, exact basename absence, and no case/normalization/path/hardlink/
inode alias; `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)`; require regular same-parent-device file,
exact UID/GID/mode `0600`, nlink 1, size 0, and stable inode; bounded full canonical write; file
fsync; close; reopen read-only no-follow; require pathname/FD same object; bounded EOF readback;
canonical parse/hash/count; repeated file/parent fstat; close; parent-directory fsync. Existing or
partially created I_static poisons the epoch forever and is never repaired/deleted/reused.

Immediately before and after every literal inherited pre-runtime phase, coordinator and reviewed
phase guard stable-open and hold I_static plus every I_static-bound artifact, validate against the
coordinator's expected I_static hash and canonical rows, and record a canonical equality receipt.
The postcheck occurs after descendants drain/reap and before that phase can seal success or the next
phase can start. I_static/hash/reviews/receipts are never passed to a candidate phase. E2 later binds
the exact I_static and complete ordered per-phase receipts.

If any inherited pre-runtime phase lacks crash-durable at-most-one/replay control for its exact
start, the implementation is terminal and unimplementable under this amendment. Coordinator memory
or an in-memory flag is insufficient. No phase may run until its existing or separately accepted
durable control is proven without weakening inherited no-replay semantics.

## Atomic durable spend claims `S1` and `S2`

S1/S2 live outside `_AUDIT_PARENT` in distinct owner-private parents. Run plan predeclares, and the
applicable E binds, exact held parent identity, basename, schema, mode, and absence policy, but not
future claim hash. Claim content may bind the already-existing E hash without a cycle.

Immediately before child creation, bootstrap/guard stable-opens the private parent no-follow and
verifies identity/ancestry/no aliases. It proves exact basename absence and calls descriptor-
relative `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)`. The initial FD must be regular, on the same
device as held parent, exact UID/GID, mode `0600`, nlink 1, size 0, and stable inode by repeated
fstat; parent is also repeatedly fstat'ed. Any mismatch is terminal before write.

Guard writes bounded canonical content with schema/epoch/E hash/manifest hash/manifest sequence/
record/phase/attempt `1`/creation instant/final LF, requires full write and file fsync, closes,
reopens read-only no-follow, uses `fstatat` no-follow on the basename, proves reopened FD/path are
the same original inode and parent device, exact canonical size/metadata, reads bounded EOF,
parses/hashes/counts, repeats file+parent fstat, closes, and fsyncs the parent. Child starts only
after complete readback and parent-fsync durability.

Successful O_EXCL creation is the live logical spend. A crash before durability starts no child but
abandons the epoch even if the directory entry later disappears. Once full file/readback/parent
fsync completes, the durable claim blocks post-crash replay. On any success/failure/crash/timeout/
cancel/partial start/publication, a surviving zero/partial/complete entry is retained forever and
never deleted, truncated, renamed, repaired, normalized, recycled, or recreated. Claim proves at
most one start, not completion; direct child/cleanup/authority receipts prove completion.

Guard/coordinator receipts bind absence, initial fstats, create/write/fsync/reopen/readback/path-
same-object facts, parent sync, creation-before-child ordering, child start, and immutable poststate.
E2 requires exact valid S1, its receipt, and I_static. S2 is postchecked and enters completion
evidence; E2 binds only its path/schema/absence policy because S2 does not yet exist.

## Exact `_AUDIT_PARENT` mixed-role state machine

`_AUDIT_PARENT` is one intentional mixed-role direct-parent namespace and the sole exception to
blanket input/output parent disjointness. Guard holds one no-follow directory FD throughout each
relevant observation/child/postcheck interval and proves the same parent device/inode/UID/GID/mode/
link/ancestry identity. Literal parent sharing is authorized only through the following state
machine; leaf aliasing is never authorized.

Let P be the complete canonical prestate of every direct entry, C the two final-L completion output
leaves, and Rv the one completion-review leaf:

```text
P  = every exact pre-existing direct entry, all present and immutable
C  = {exact completion TSV basename, exact completion digest basename}
Rv = {exact completion-review basename}

before completion child:  P present and unchanged; C union Rv absent
child transition:          P -> P union C
reviewer transition:       P union C -> P union C union Rv
```

P is first frozen canonically from the held parent FD in M_static after final L/R construction and
before Gate1. M_completion later performs a fresh independent held-FD observation and must reproduce
the identical complete P bytes and digest; any intervening delta is terminal. Every direct entry has
exact basename bytes, file type, device/inode, UID/GID/mode/nlink, size, SHA-256, and byte/LF counts.
Only explicitly named directories are recursively inventoried, with closed ordered descendants;
unlisted recursion is forbidden. Every P input leaf is protected immutable from the M_static
observation through static, every pre-runtime phase, completion child, and reviewer transition.

C and Rv each have exact byte basenames and absent prestates. Across P/C/Rv and all external
namespaces there may be no symlink, case-fold/case-normalization, alternate-path/ancestry, hardlink,
or device/inode alias. The child transition permits exactly two create-new regular files, TSV then
digest under later-frozen publication protocol. It permits no P mutation/deletion, directory
mutation, temporary entry/residue, overwrite, rename from elsewhere, third file, or other delta.
The reviewer transition permits exactly Rv creation and no other delta.

All E/M/S/I_static/Gate0/design/Gate1/post-static/Gate2 raw/sealed review, guard, receipt, trace,
and FD-capture namespaces remain outside `_AUDIT_PARENT`, with strict parent/ancestry disjointness
from it and each other. Static final L has no filesystem output. Its only output is bounded raw FD1/
FD2 captured externally. Before/after static, audit parent and every unselected namespace remain
byte/metadata identical.

## Audit-parent historical metadata

The exact six inherited runtime anchors remain candidate-readable in unchanged order/roles/hashes;
there is no seventh. The three rejected amendment basenames are protected pre-existing P entries:

```text
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_FROZEN_2026-09-10.md
```

Their prospective review paths remain absent, uncreated, and unauthorized:

```text
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_REVIEW_2026-09-10.md
```

Among amendment/design-review metadata, only this
`...TWO_MANIFEST_ENFORCEMENT_FROZEN_2026-09-10.md` and eventual exact
`...TWO_MANIFEST_ENFORCEMENT_REVIEW_2026-09-10.md` are newly frozen before R0. They are not runtime
anchors and final L never reads their contents. Only their basename/type/device/inode metadata
enters `audit_parent_snapshot_hex` and transitively R0/D/L/H/R. Content/hash/verdict/findings do not.

The completion TSV/digest are C. Rv is absent through child/guard postcheck and is created only by
the trusted completion-review protocol below. No later byte flows backward into Q/R0/D/L/H/R.

No new review content/hash/verdict/finding enters candidate argv/environment/stdin/FDs/source,
Q/H/R, static vectors, provider input/payload, or runtime. Sole exceptions are unchanged inherited
review documents among the six runtime anchors and only declared audit-parent basename/type/device/
inode metadata for this amendment/eventual design review.

## Deadlines and resource containment

All clocks are monotonic and nested; no child, descendant, cleanup, or postcheck starts a replacement
outer clock or extends its ancestor deadline.

- Static child deadline is exactly 120 seconds from child start.
- Static outer-guard deadline is exactly 180 seconds from authority-precheck start, including E/
  artifact validation, S1 durability, child execution, FD capture, descendant cleanup/reap, input/
  namespace checks, and external-authority postcheck.
- Completion scanner deadline is exactly 900 seconds from scanner start.
- Final-L completion whole-operation deadline is exactly 1100 seconds from final L's first trusted
  entry, including scanner start/wait/capture/publication/cleanup and its internal final checks.
- Completion outer-guard deadline is exactly 1200 seconds from authority-precheck start, including
  E/artifact validation, S2 durability, final-L operation, raw capture, cleanup/reap, mixed-parent
  transition check, and external-authority postcheck.

Each descendant deadline is capped at its remaining ancestor budget and can never lie beyond outer
deadline. Cleanup/reap/session-empty verification and postcheck must finish inside outer deadline.
Any timeout, clock anomaly, deadline overrun, incomplete drain/reap, or late postcheck makes success
impossible. The spend remains consumed and replay is forbidden. Exact CPU, address-space, file-size,
process/FD, and direct-stream ceilings are frozen with Gate0/phase manifests later and may refine but
never weaken inherited bounds.

## Direct guard channels and external-authority postcheck

Guard is final L's direct parent under `path-bound-b5-v1`. Pinned Python executes repository
`_LAUNCHER` with exact phase argv, six-variable environment, cwd `/var/empty`, umask `0077`, and
only FDs 0/1/2 visible. FD0 is inherited read-only `/dev/null`; FD1/FD2 are distinct nonseekable
guard-owned capture pipes. No source/guard/E/M/S/I/review FD passes to final L.

Raw stdout/stderr remain distinct bounded external evidence. A third coordinator-owned channel
carries canonical `c2b2a-guard-observation-envelope-v4`: epoch/phase, E/manifest/Gate0/spend,
invocation, authority precheck origin/deadline, child/process-group/session facts, nested deadline/
resource/overflow facts, wait/cleanup/reap facts, raw stream hashes/counts, selected/unselected
state transition, authority receipts, failures, and terminal validity.

Before dispatch, bootstrap/guard and coordinator stable-open/hold selected E and every E-bound
artifact: manifest, Gate0, guard/bootstrap/phase-guard identities, run plan, amendment/review,
attestations, ordered evidence, and optional oracle. Regular files and directory ancestors remain
held through postcheck. For candidate path inputs, guard-held descriptors authenticate guard
observations but final L's reopen remains under same-UID interval nonclaim. If any required held-
descriptor policy cannot be implemented, the epoch is terminal; there is no fallback.

Pre/post receipts include every role/path/type/device/inode/UID/GID/mode/nlink/size/hash/count and
repeated fstat, held-E pre/post identity/hash, E-bound inventory digests, coordinator expected-E
match, spend facts, exact invocation once, deadline/resource status, selected transition, unselected
unchanged, and `authority_revalidation_valid`. Acceptance seals only after child/descendants drain/
reap and every E-bound artifact and parent/leaf state revalidates. Any missing/extra/reordered/
rebound/mutated/unreadable/unvalidated object is terminal. Raw evidence remains; success does not.

## Review attestations and exact gates

Reviews create no-overwrite canonical attestations outside `_AUDIT_PARENT`, with schema/epoch/gate/
phase/role, exact ordered input hashes, harness/provider/model/session, UTC start/end, P0/P1/P2,
`pass|fail`, and sealed findings digest. Pass requires P0=0/P1=0. Artifact writers cannot review
their artifact. Same-gate reviewers are blind until sealed. Every three-review gate uses at least
two harness/provider families and one family supplies at most two. Correlated training/host/design
blind spots remain an explicit nonclaim.

The gates are:

1. **Design acceptance.** Three clean same-hash reviews accept this design only and allow creation
   of its exact design-review file before R0.
2. **Gate0.** Three clean blind enforcement reviews seal the Gate0 artifact described above. No
   candidate action is authorized.
3. **Gate1 static eligibility.** Three new clean blind reviews cover Gate0, final L/C/R,
   provisioner iff referenced, complete M_static semantic inputs, M_static, run plan, guard,
   S1/I_static creation policies, P baseline, external namespaces, deadlines, and inherited gates.
   Coordinator then creates E1. E1 permits only S1 and one guarded static final-L invocation.
4. **Post-static implementation acceptance.** After successful guard postcheck, exactly three new
   clean sealed attestations in fixed semantic/guard/adversarial order review E1/M_static/S1, raw
   FD1/FD2, guard envelope, coordinator receipt, deadline/resource/cleanup facts, zero filesystem
   output, unchanged P/unselected namespaces, and semantic result. Coordinator alone then creates
   canonical I_static. Without it no pre-runtime phase starts.
5. **Pre-runtime-finalize.** Each exact ordered inherited phase validates I_static and bound
   artifacts before/after, uses proven crash-durable replay control, satisfies inherited gates, and
   emits a sealed equality/execution receipt. Only complete ordered success seals pre-runtime-finalize.
6. **Gate2 completion eligibility.** Three new clean blind reviews cover unchanged Gate0/guard/run
   plan/final L, exact M_completion and all semantic inputs, E1/S1/static evidence/I_static, every
   phase receipt/pre-runtime-finalize, S2 policy, exact P/C/Rv state machine, completion deadlines,
   and inherited gates. Coordinator creates E2. E2 permits only S2 and one guarded
   completion-audit-publish invocation.
7. **Completion attestations and Rv.** After valid E2 postcheck and sealed C, exactly three clean
   blind completion attestations in semantic/guard/adversarial order review TSV/digest bytes,
   hashes/semantics/source observations, E2/S2/receipts, deadlines, exact P->P union C transition,
   and inherited completion criteria. Only then coordinator may create Rv.

Completion review does not authorize providers. It is followed exactly by build/source identity
proposal and review, provider-free Linux runner freeze and review, runtime qualification and
reviews, every remaining inherited gate, and only eventually provider activation/evaluation via
the unchanged pilot runner. E2 is never provider authority or a provider input.

## Trusted completion-review creation

Rv is exactly one canonical completion-review regular file inside `_AUDIT_PARENT`, never a child
output. Its exact schema/content/bounds are frozen before Gate2. Only the trusted coordinator is an
authorized writer, after direct acceptance of exactly three sealed clean completion attestations.
The protocol excludes concurrent authorized writers; malicious same-UID precreation remains the
explicit nonclaim above.

Coordinator holds the same audit-parent FD and proves exact P union C state, Rv absence/no aliases,
then calls `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)`. Initial FD must be regular/same-parent-device/
exact UID/GID/mode0600/nlink1/size0/stable inode. Coordinator performs bounded full canonical write,
file fsync, close, read-only no-follow reopen, pathname/FD same-object proof, bounded EOF readback,
canonical parse/hash/count, repeated file/parent fstat, close, and parent fsync. Only exact
P union C -> P union C union Rv is accepted.

Existing/partial/aliased Rv or any other delta poisons the epoch; no second writer, repair, delete,
overwrite, normalization, or retry is permitted. Rv binds exact three attestations and C evidence,
contains no self/future hash, and remains immutable. Its success is necessary but not sufficient
for the later build/source, Linux-runner, qualification, and provider gates.

## Preserved construction DAG and narrow supersession

The inherited construction remains exact:

```text
U -> A -> C -> HC -> L0/Q -> S -> W0/F0 -> components/B/V/O
  -> R0 -> D -> G_D -> L -> H -> G_H -> R
  -> authoritative W1/F1/replay
```

`Q(L)=L0`, `P(R)=R0`; H is raw only in sole `launcher_source_sha256` removed by P. Six anchors stay
ordered. Every node keeps finite no-iteration/no-repair blocking semantics, never permission.
Final L after Q has a new identity distinct from b5; no b5 review/hash transfers.

Only permission authority, Gate0/I_static enforcement, two-manifest lifecycle, durable spends,
mixed audit-parent transitions, deadline scopes, and trusted Rv creation supersede their matching
inherited clauses. All 26 rows, W0/W1, F0/F1, Q invariance, carrier 75-row/three-edge/29-root/
26-parent/ten-absence/six-observation facts, codecs, dominance, descriptor capture, historical/graph
replay, runtime anchors, non-source gates, no-writer/no-repair/no-replay rules, and non-expansion
remain unchanged. Recursive AST/matcher/source/vector receipts are blocking regression evidence only.

No later manifest/envelope/spend/review/trace/I_static/pre-runtime/completion/provider byte flows
backward into Q/R0/D/L/H/R.

## Exact invalidation and retry boundaries

- Any Gate0 input, guard/bootstrap/phase guard, interpreter/toolchain, run plan, final L/Q/R,
  controller/provisioner iff referenced, inherited anchor, common bound, P pre-R0 entry, this
  amendment/review, or M_static change invalidates from Gate0/Gate1 as applicable. No review transfers.
- Static failure/mismatch stops before I_static. Accepted E1/S1/static/I_static mutation invalidates
  post-static acceptance; S1 stays spent, so static retry requires separately named successor.
- I_static/expected hash/ordered phase-list/bound artifact/pre/post phase receipt mismatch stops the
  current phase and every later phase. If replay controls are absent or ambiguous, implementation is
  unimplementable. No in-place I_static or receipt repair exists.
- Post-static-only pre-runtime input/evidence change with common Gate1 inputs unchanged invalidates
  from the affected phase/pre-runtime-finalize/Gate2, not historical static evidence. M_completion
  cannot be edited; continuation uses a separately named later-phase successor and new reviews.
- M_completion/input/P/C/Rv baseline/Gate2/E2 change invalidates Gate2. Once S2 is created it stays
  spent; completion retry requires separately named successor M_completion/E2/S2 namespace.
- Phase mismatch, alternate invocation, replay/second start, missing/duplicate/reordered row,
  placeholder/wildcard, P/C/Rv or external alias, unexpected delta/temp/overwrite/delete, deadline,
  cleanup, or external-authority failure is terminal even if a content hash matches.

Modification/removal/truncation/replacement/rename/recreation of any prior append-only object is
terminal. Only exact predeclared create-new forward transitions are allowed. S1/S2/partial artifacts
remain forever. On failure execute nothing further; preserve bytes/traces/partial outputs and never
delete, repair, normalize, overwrite, or replay. Partial completion requires separately reviewed
recovery freeze. User-owned worktree changes remain preserved.

## Preconditions and acceptance boundary

Before any candidate/static/build/runtime/provider action:

1. freeze/review this amendment with three clean same-hash reviews and at least two families;
2. create/freeze its exact design review; freeze enforcement amendment/review metadata and protected
   7d/f1/74c P entries before R0 while predecessor prospective review paths stay absent;
3. implement/review Gate0 machinery and seal Gate0;
4. review b5/eventual final L, construct inherited Q/R0/D/L/H/R once, freeze final L/C/R, and keep
   all recursive checks blocking-only;
5. freeze run plan/M_static, pass Gate1, create E1, durably spend S1, run static once inside exact
   deadlines, complete postchecks/reviews, and atomically create I_static;
6. continuously enforce I_static through each crash-durable inherited pre-runtime phase and seal
   ordered pre-runtime-finalize receipts;
7. construct M_completion from exact current semantic inputs/P, pass Gate2, create E2, durably spend
   S2, run completion audit once inside exact deadlines, and seal exact P->P union C;
8. obtain three completion attestations and atomically create Rv as the sole reviewer transition;
9. complete build/source identity review, provider-free Linux runner review, runtime qualification,
   and all remaining inherited gates before eventual unchanged-pilot provider execution; and
10. throughout require pristine identities, positive disk reserve, absent `target/debug`, combined
    output absent before sole inherited create-new publication, and every no-replay/no-repair gate.

Design acceptance alone accepts no guard, Gate0, manifest, envelope, spend, I_static, source, Q,
static, pre-runtime, completion, Rv, provider run, pilot, or flagship goal.

## Current exact disposition

This amendment adds no provider/model/budget/VM/pilot lane/adapter/daemon/datastore/connector/
network/authentication/product behavior/user data/runtime anchor/recovery mode/source mutation. It
authorizes no stage, commit, publication, live setting change, or `target/debug` creation.

Its prospective review does not exist. There is no execution authority, Gate0 artifact, Q fill,
combined output, manifest, envelope, spend, I_static, pre-runtime run, completion output/review, or
controller/provisioner change. B5 `b5e50318...`, controller `eefa6eec...`, and provisioner
`aabe6b17...` remain unexecuted candidate bytes. Rejected 7d/f1/74c remain immutable evidence.
