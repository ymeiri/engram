# Native strict successor Stage C2B2a — two-manifest enforcement successor

Date: 2026-09-10
Status: enforcement-successor frozen candidate design; no implementation or execution authority

## Disposition and rejected predecessors

This amendment replaces recursive launcher self-authentication as permission with an externally
rooted, exact-hash, two-manifest authorization lifecycle. Candidate self-checks remain fail-closed
regression evidence: failure blocks; success never grants execution.

The immediately preceding enforcement epoch is rejected and preserved byte-for-byte:

```text
SHA-256 bed960697d497a8b721e53a7d9bad7a084c9a1e169339c290a05cf2ee9446bd6
lines   677
bytes   41736
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_FROZEN_2026-09-10.md
state   rejected; no review, implementation, test, or execution authority transfers
```

Its direct reviews were:

```text
Codex normative review   FAIL  P0=0 P1=2 P2=2
Codex guard review       FAIL  P0=0 P1=4 P2=3
isolated Claude review   FAIL  P0=0 P1=2 P2=4
aggregate gate           FAIL
```

No verdict transfers. Exact bed960 left Gate0 artifact identity/creation incomplete, retained an
ambiguous guard-build branch, under-specified transitive authority closure and expected-hash
delivery, froze future completion-review content too early, and left clock/P/spend-restart details
insufficiently enforceable. Those are authority-boundary failures; bed960 remains rejected raw
evidence.

Earlier rejected raw evidence remains immutable:

```text
SHA-256 74c883fec8e075ddc20a5977fc5a36751f156e1a170554b3a414f3aca27fd0e2
lines   733
bytes   48654
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_FROZEN_2026-09-10.md

SHA-256 f1d7ddc8371c2a035f527f6d1784fe3f225cc89b3e0bac23e1616bc1f7a8ebaf
lines   690
bytes   47347
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_FROZEN_2026-09-10.md

SHA-256 7d0ae76f3fd721d61fc3797d3627c4cc4a99d413e7f2c39e4ae47417dc42ec80
lines   556
bytes   36410
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_FROZEN_2026-09-10.md

state   all rejected; no authority transfers
```

An older rejected epoch was recorded as SHA-256
`1cd2c842fa4fa25415b03623d71458f0377c7cbd73d0de40b1bfb0a79cd5f689`, 566 lines, 38,215
bytes. Its raw bytes are unavailable and confer no partial authority under any interpretation. No
property here depends on characterizing its defects; its digest/review trace are provenance only.

This successor is itself only a candidate. It authorizes no source import, compilation, AST parse,
Q fill, candidate/static/build/runtime/provider/pilot execution, VM, daemon, adapter, datastore,
publication, or setting change until its exact bytes and separate design review pass every gate.

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

Current candidate identities remain:

```text
SHA-256 b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
lines   34415
bytes   2096688
path    evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
state   precursor candidate; not authorized to execute

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

This successor path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_SUCCESSOR_FROZEN_2026-09-10.md
```

Its prospective consolidated design-review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_SUCCESSOR_REVIEW_2026-09-10.md
```

It is absent. It may be created once, no-overwrite, only after three clean same-hash external design
attestations, then frozen before R0 and never modified.

## Self-proof limitation and threat model

Narrow b5 reviews matched helper/matcher scopes, but broader review proved reachable `__code__`
replacement and builtin `ValueError` shadowing could preserve checked source while changing
effective behavior. A co-editable validator cannot make its own success external fact. Every
recursive source/matcher/vector/self-contract receipt is rejection-only.

The trust root is an honest coordinator retaining accepted expected hashes out of band. Assumptions
are collision-resistant SHA-256; honest reviewed interpreted guard/bootstrap/phase guard; honest
pinned interpreter; honest kernel/filesystem/durable storage; and reviewers inspecting exact bytes.

This unprivileged design does not defend against compromised trust-root components, colluding or
negligent reviewers, same-UID debugging/injection/path swap-and-restore, malicious same-UID
precreation, a lying storage stack, or ambient same-UID filesystem access without an OS sandbox.
Owner-private paths are not authentication against same UID/root. Guard-held descriptors prove
guard observations, while final L's path reopens remain under the same-UID interval nonclaim. No
detached-signature or whole-host-closure claim is made.

## Frozen interpreted runtime and complete semantic-input closure

There is no native guard/build alternative. The sole enforcement runtime is an interpreted guard
with exact source, exact pinned interpreter executable identity, exact invocation, and an exact
closed `c2b2a-interpreter-tcb-policy-v1`. That policy is created/frozen no later than Gate0 and is
an explicit exact input to Gate0, Gate1, M_static, M_completion, E1, and E2.

The interpreter policy canonically binds its schema/path/type/device/inode/UID/GID/mode/nlink/size/
hash/counts, the pinned interpreter executable row, loader/shared-cache classification, import
isolation flags, empty/closed site-package and plugin policy, and the exact ordered list of unpinned
standard-library module/path identities treated as host TCB. The list is literal and finite: no
prefix, directory wildcard, import-path allowance, optional row, or caller expansion. Any policy or
interpreter drift invalidates Gate0 and all later gates.

Each phase manifest completely inventories every candidate-controlled semantic input to its exact
final-L invocation: repository, writable/generated, plugin/hook, import-path, config, data,
executable/script, environment, argv, cwd, FD, presence/absence, metadata, ordering, and directory
resolution. Ambient TCB cannot launder any of those. Only kernel/syscall/filesystem behavior,
dyld/shared cache, and the exact policy-listed unpinned stdlib identities are excluded from rows.
Semantic review must prove no undeclared read. This is finite declared closure, not whole-host closure.

Provisioner/controller/runner/build/anchor/directory inclusion follows one exact rule: a manifest
contains the row if and only if the reviewed invocation references or opens it. Omission when
referenced or presence when not referenced is terminal.

## Sound two-manifest lifecycle

Exactly two immutable manifests and two envelopes exist:

- `M_static`, sequence 1, one final-L static invocation;
- `M_completion`, sequence 2, one final-L `completion-audit-publish` invocation;
- `E1`, selecting only M_static; and
- `E2`, selecting only M_completion plus prior accepted evidence.

There is no shared manifest, third record, third manifest, or third envelope. The lifecycle is:

```text
design acceptance
  -> atomic Gate0 interpreted-enforcement acceptance
  -> inherited Q/R0/D/L/H/R construction yielding final L and R
  -> run plan + M_static -> Gate1 -> E1 -> S1 -> final-L static once
  -> three post-static attestations -> atomic I_static
  -> exact pre-runtime phases with continuous I_static receipts -> pre-runtime-finalize
  -> M_completion -> Gate2 -> E2 -> S2 -> completion-audit-publish once
  -> three completion attestations -> atomic completion review Rv
  -> build/source identity proposal+review
  -> provider-free Linux runner freeze+review
  -> runtime qualification+reviews
  -> every remaining inherited gate
  -> only eventually provider activation/evaluation through unchanged pilot runner/run plan
```

Completion audit occurs after pre-runtime-finalize and before provider work. E2 is completion-audit
authority only. It cannot invoke/authorize a provider, activation/evaluation lane, model/network
call, pilot runner, or budget spend.

## Acyclic envelopes and full transitive authority closure

Run plan embeds no future manifest/envelope/acceptance hash. It predeclares exact lifecycle, phase
identifiers, paths/parents/basenames, schemas, bounds, deadlines, and no-overwrite transitions.
Manifests contain no self/E/future hash. E1/E2 contain no self-hash or caller-supplied expected hash.

Each envelope uses canonical `c2b2a-external-authorization-envelope-v5` and contains a complete
recursively expanded authority closure, not only top-level content hashes. The closure is a fixed-
order sequence of canonical preimage rows. Each row binds rank, role, absolute path, file type,
device/inode, UID/GID/mode/nlink/size, byte/LF counts, content SHA-256, and a fixed ordered list of
lower-rank referenced row indexes. Its final `row_preimage_sha256` is computed over the exact
length-delimited serialization of all preceding row fields, expressly excluding that digest field;
no row hashes bytes containing its own digest. Every referenced authority artifact is represented
once. Unknown role, duplicate path/inode/content role, omitted edge, unordered edge, forward/same-
rank edge, digest mismatch, or cycle is terminal.

E1's transitive closure includes the exact Gate0 artifact and its attestations/inputs, interpreter
policy, interpreted guard, pinned interpreter, bootstrap, phase guard, run plan, this amendment and
consolidated design review, M_static, Gate1 attestations, and all their authority dependencies. It
binds S1 and I_static path/schema/bounds/absence/creation policies, never future S1/I_static hashes.

E2's closure includes every E1 row plus exact E1/M_static/S1/static trace/guard envelope/coordinator
receipt/raw streams/I_static, all I_static-bound artifacts, ordered pre-runtime phase receipts,
pre-runtime-finalize, M_completion, Gate2 attestations, and their dependencies. It binds S2 and Rv
path/schema/order/bounds/absence/creation policies, never future S2/Rv inode/content/hash.

The rank direction is immutable:

```text
design attestations -> consolidated design review -> Q/final L/R
Gate0 inputs -> Gate0 attestations -> Gate0 acceptance
run plan/M_static/Gate1 -> E1 -> S1/static -> post-static -> I_static
I_static/phase receipts -> pre-runtime-finalize -> M_completion/Gate2 -> E2
E2 -> S2/completion -> completion attestations -> Rv -> later gates
```

No later byte flows backward. E binds a future artifact's creation policy only; the future artifact
may bind the already-existing E hash. Rv binds completion attestations; attestations never bind
future Rv. Ranked closure parsing rejects accidental back-edges.

## Dedicated expected-hash delivery channels

Expected E and expected I_static hashes are delivered only over reviewed dedicated one-shot
coordinator-to-guard channels. They are not argv, environment, stdin, file, candidate output,
provider output, manifest field, envelope field, or candidate-readable FD.

The exact channel schema is fixed-length ASCII framing: role tag, one lowercase 64-hex hash,
epoch/phase, terminal LF, exact maximum bytes, then EOF. Coordinator creates the guard-only pipe or
socketpair under Gate0's exact FD topology, writes one complete frame, closes its endpoint, and
retains the expected value in trusted state. Guard bounded-reads exactly one frame, rejects extra/
short/reordered bytes or missing EOF, hashes the frame into its receipt, and closes every channel
endpoint before any candidate fork/spawn/exec. It then stable-opens the named artifact and verifies
the expected hash and full closure.

E1/E2 guard runs receive only their applicable expected E hash. Each inherited pre-runtime phase
guard receives only expected I_static hash and phase identity. No channel is reused. Leakage to a
candidate, inherited FD, or descendant is terminal. Gate0 reviews coordinator and guard endpoints,
close-on-exec state, close ordering, FD enumeration, and negative leakage tests.

## Atomic Gate0 acceptance

Gate0 follows design acceptance and precedes Q or any candidate/build/runtime action. Three blind
clean reviews with P0=0/P1=0, at least two harness/provider families, cover interpreted guard source,
pinned interpreter/policy, bootstrap, closure/envelope/manifest/spend/I_static/Rv parsers, expected-
hash channels, phase guard, deadlines, raw channels, process cleanup, typed audit-parent state, and
all authority postchecks.

Exactly one coordinator-issued `c2b2a-gate0-acceptance-v1` is created outside `_AUDIT_PARENT`.
The exact Gate0 review input set, frozen before those reviews and independent of the later run plan,
predeclares its owner-private parent identity, exact path/basename, schema/order/max bounds,
requested `0600`, absence, and creation policy, not future inode/content/hash. Canonical content
binds epoch/pass/scope, exact reviewed input-set digest, and full fixed-order identity rows for the
interpreter policy, pinned interpreter, guard source, bootstrap, every parser, phase guard,
coordinator enforcement code, and exactly three ordered semantic/guard/adversarial attestations.
Every row carries role/path/type/device/inode/UID/GID/mode/nlink/size/hash/counts and canonical
preimage digest. It has no self/future/run-plan hash. The later run plan repeats the Gate0 path/hash
as an already-existing input; Gate0 never depends on that future run plan.

Coordinator stable-opens/holds parent no-follow, verifies identity/ancestry/absence/no aliases,
calls `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)`, and requires initial regular/same-parent-device/
exact UID/GID/mode0600/nlink1/size0/stable inode. It performs bounded full canonical write, file
fsync, close, read-only no-follow reopen, `fstatat` pathname/FD same-object proof, bounded EOF
readback, canonical parse/hash/count, repeated file+parent fstat, close, and parent fsync. Existing/
partial/aliased output poisons the epoch. Coordinator retains expected Gate0 hash out of band.
E1/E2 bind exact Gate0. Gate0 alone authorizes nothing.

## Exact phase manifests and final-L invocation schema

Both manifests use canonical `c2b2a-phase-manifest-v3`: sequence/phase, one exact invocation,
complete independently observed semantic inputs including interpreter policy, exact output policy,
canonical serialization/max size/metadata/hash/counts/final LF. Missing/duplicate/reordered/optional/
placeholder/wildcard/prefix/environment-expanded/caller-selected/unresolved rows are terminal.

The post-Q invocation schema is named exactly `c2b2a-final-l-path-invocation-v1`. No active schema,
profile, manifest, envelope, receipt, or review calls final L "b5". B5 names only precursor history.
The schema binds final-L exact post-Q path/hash/metadata, pinned interpreter/policy, phase/mode/argv,
six-variable environment, cwd `/var/empty`, umask `0077`, visible FDs 0/1/2, direct guard parent,
semantic input rows, deadlines/resources/status, and typed filesystem transition.

M_static is frozen after Gate0/final L/R/run plan and before Gate1. It contains one `static`
invocation, existing complete semantic inputs, and first canonical audit-parent P. It excludes future
S1/static/I_static/pre-runtime/M_completion/E2/S2/completion/Rv/provider bytes. Static final L has
no filesystem output; only bounded FD1/FD2 captured externally.

M_completion cannot be drafted or frozen until I_static and every pre-runtime phase receipt/finalize
exist. It freshly inventories every then-existing semantic input for one
`completion-audit-publish` invocation, exact final L/C/R, referenced inputs, generated/build/source/
config/data/import rows, interpreter policy, and independently reproduced P. It excludes future
S2/C/Rv/later qualification/provider bytes.

Both manifests are immutable create-new external files. Any drift requires exact invalidation, not
repair, normalization, placeholder, or in-place edit.

## Canonical I_static and continuous phase validation

`I_static` is canonical `c2b2a-static-implementation-acceptance-v1`, sole coordinator-issued
necessary-but-not-sufficient acceptance for the exact ordered inherited pre-runtime phase list.
It authorizes no final-L, completion, provider, pilot, retry, unlisted/reordered phase, or input edit.

E1/run plan predeclare only its exact owner-private parent identity/path/basename/schema/order/max
bounds/absence/metadata/O_EXCL creation policy. Canonical length-delimited content binds epoch/pass/
scope, run-plan path/hash and literal phase list/digest, Gate1 input-set digest, and full identity
rows for the entire recursively expanded E1 closure, M_static, S1, static guard envelope,
coordinator receipt, raw FD1/FD2, plus exactly three ordered clean semantic/guard/adversarial post-
static attestations. Every identity row includes path/type/dev/inode/UID/GID/mode/nlink/size/hash/
counts and canonical preimage digest. It contains no self/future hash.

Only coordinator writes I_static using the exact Gate0 atomic protocol and retains expected hash in
trusted state. Existing/partial/aliased I_static poisons the epoch. Before each literal pre-runtime
phase, the Gate0-accepted phase guard receives expected hash once over a fresh dedicated channel,
closes every endpoint, retains the value only in guard-owned state, then stable-opens/holds I_static
and its full transitive closure and validates all canonical rows/metadata/preimages. After candidate
descendants drain/reap, that same guard uses the retained expected value and held artifacts for the
second validation; no second delivery channel is opened. Coordinator independently compares its
trusted value before and after and seals the equality receipt only after postcheck. E2 binds exact
I_static/ordered receipts. No I_static/hash/review/authority FD or content passes to candidate.

Every inherited pre-runtime phase must already have or separately receive accepted crash-durable
at-most-one-start control. If not, implementation is unimplementable. In-memory replay flags are
insufficient and inherited no-replay semantics cannot be weakened.

## Atomic S1/S2 spends and mechanical restart classification

S1/S2 are external owner-private exact paths. Run plan and E predeclare held parent identity,
basename/schema/mode/absence policy, not future hash. Claim content binds existing E/manifest hashes,
sequence/phase/attempt1/creation instant/final LF.

Before child admission, guard stable-opens parent, proves identity/ancestry/absence/no aliases, and
uses `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)`. Initial FD must be regular/same-parent-device/exact
UID/GID/mode0600/nlink1/size0/stable inode. Guard full-writes canonical bytes, fsyncs, closes,
reopens read-only no-follow, proves `fstatat` path/FD same object and exact size/metadata, bounded-
reads/parses/hashes/counts, repeats file+parent fstat, closes, and fsyncs parent. Child cannot start
before all steps finish.

O_EXCL is the live concurrency fence. Authorization becomes durably spent only after canonical file
fsync/readback and parent fsync. Under the honest-host/storage assumption, restart recovery freshly
stable-opens the exact parent and classifies exactly:

- basename absent: no durable authorization existed, therefore no child could have started; attempt
  1 may be recreated safely and this is not replay;
- any surviving canonical valid entry: conservatively spent; never start another child; or
- any surviving zero/partial/invalid/aliased entry: poisoned; never start a child.

No other state is inferred from volatile coordinator memory. A crash before durability starts no
child. A durable claim blocks post-crash replay. Surviving entries are never deleted/repaired/
renamed/reused. Receipts bind initial/repeated fstats, write/fsync/reopen/readback/path identity,
parent fsync, child admission ordering, and immutable poststate. E2 requires valid S1/receipt;
completion evidence contains S2/receipt.

## Trusted monotonic origins, admission, and reserves

Gate0 pins one monotonic clock API/semantics. Guard captures trusted `t0` once immediately before
authority precheck; no candidate supplies or resets it.

Static outer deadline is `t0+180s`. S1/precheck must finish and child admission occur by `t0+20s`
with at least 160 seconds remaining. Guard captures `t_child` immediately before the OS child-create
call; this is the normative trusted child-start origin and is conservatively no later than actual
execution. Child deadline is `min(t_child+120s, t0+140s)`. The final 40 seconds reserve exactly 20
seconds for descendant cleanup/reap/drain and 20 seconds for namespace/authority postcheck. A
reserve cannot be borrowed or reset.

Completion outer deadline is `t0+1200s`. S2/precheck must finish and final-L admission occur by
`t0+50s` with at least 1150 seconds remaining. Trusted guard captures `t_final` immediately before
child creation; this is the normative trusted final-L-start origin and is conservatively no later
than actual execution. Final-L whole deadline is `min(t_final+1100s, t0+1150s)`, leaving 50 seconds
for outer cleanup/reap/postcheck. Exact reviewed final L captures `t_scan` immediately before
scanner spawn; scanner must start by `t_final+100s`, has deadline
`min(t_scan+900s, t_final+1000s)`, and leaves 100 seconds for final-L capture/publication/internal
cleanup. The scanner origin is an internal blocking measurement; outer trusted deadlines remain
independent authority.

Admission/reserve miss, timeout, clock anomaly, incomplete drain/reap, or late postcheck makes
success impossible. A durable spend remains consumed and replay is forbidden. CPU/address-space/
file-size/process/FD/stream ceilings are frozen no later than Gate0/manifests and never weaken
inherited bounds.

## Typed audit-parent P/C/Rv state machine

`_AUDIT_PARENT` is the sole intentional mixed input/output direct parent. It is not held across
days. For each M/static/completion/Rv observation interval, guard/coordinator freshly stable-opens
one no-follow directory FD, compares it to the frozen parent path/device/inode/UID/GID/mode/nlink/
ancestry identity, holds it through that entire bounded interval, repeats fstat, then closes it.

Canonical `c2b2a-audit-parent-prestate-v1` P sorts direct entries by raw basename bytes and encodes
each basename as length plus lowercase hex, never locale/Unicode normalization. Supported rows are:

- `regular`: basename/type/dev/inode/UID/GID/mode/nlink/size/SHA-256/byte count/LF count or
  `not-applicable`; or
- `directory`: basename/type/dev/inode/UID/GID/mode/nlink plus canonical subtree digest and exact
  ordered child rows, recursing only for explicitly named directories without symlink/device crossing.

Symlink, socket, FIFO, device, unknown type, undeclared directory recursion, hardlink/inode alias,
case-fold/normalization alias, alternate ancestry/path, duplicate basename bytes, or unsupported row
is terminal. Every P leaf is immutable.

Let C be exact completion TSV/digest basenames and Rv exact completion-review basename:

```text
prestate:             P present; C union Rv absent
child transition:     P -> P union C
review transition:    P union C -> P union C union Rv
```

P is first frozen in M_static after final L/R, then freshly reobserved byte-identically for
M_completion. Child may create exactly TSV then digest with frozen formats/bounds/metadata/fsync
order. No temp/third/overwrite/rename/delete/P mutation/directory delta is allowed. TSV without
digest, digest without TSV, invalid/partial C, or any publication failure is terminal: preserve all
existing C bytes, create no Rv, and never repair/retry in this epoch. Reviewer transition permits
only exact Rv creation.

External E/M/S/I/Gate0/design/Gate1/post-static/Gate2/attestation/trace/receipt/capture namespaces
remain strictly parent/ancestry-disjoint from audit parent. Shared audit parent is the sole parent
exception; leaf aliasing is never allowed. Static has no filesystem delta and all unselected
namespaces remain unchanged.

## Design attestations and protected historical P entries

Three individual design attestations are sealed external artifacts outside `_AUDIT_PARENT`.
Only after all three are clean and blind does coordinator atomically create the single consolidated
design-review P leaf at this successor's prospective review path. It binds amendment hash and the
three attestation hashes but no future Q/L/E bytes. Final L sees only its basename/type/dev/inode
metadata projection, never content/hash/verdict/findings. This removes ambiguity between individual
attestations and consolidated review.

Protected pre-existing P entries include rejected 7d/f1/74c/bed960 amendment basenames. Their four
prospective review paths remain absent/uncreated/unauthorized:

```text
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_SUCCESSOR_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_REVIEW_2026-09-10.md
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_REVIEW_2026-09-10.md
```

Only this enforcement-successor amendment and eventual consolidated review metadata are newly
frozen before R0. Six inherited runtime anchors remain unchanged; there is no seventh. Later C/Rv
bytes never flow backward into Q/R0/D/L/H/R.

No new review content/hash/verdict/finding enters candidate argv/environment/stdin/FD/source,
Q/H/R, static vectors, provider input/payload, or runtime. Sole exceptions remain unchanged review
documents among the six inherited anchors and the declared amendment/consolidated-review metadata
projection.

## Completion attestations and atomic Rv construction

Before Gate2, only Rv's exact parent/path/basename, canonical schema/order/content policy, maximum
bounds, requested metadata, absence, and atomic creation protocol are frozen. No future Rv inode,
bytes, content hash, or instance identity is predicted or bound by M_completion/E2.

After successful E2/S2 completion postcheck and complete valid C, exactly three fresh clean sealed
completion attestations in semantic/guard/adversarial order bind C, E2/S2, trace/receipts, deadlines,
P transition, and inherited completion facts. They contain no future Rv hash.

Coordinator is sole Rv writer. For the bounded Rv interval it freshly opens/holds and identity-
compares audit parent; stable-opens/holds TSV, digest, all three attestations, E2/S2/receipts, P, and
every Rv-bound input; and prevalidates their full identity rows. It proves exact P union C/Rv absent,
then uses `openat(O_CREAT|O_EXCL|O_NOFOLLOW, 0600)` and the Gate0 atomic full-write/fsync/close/reopen/
readback/fstat/parent-fsync protocol. After parent fsync and before sealing, it revalidates every held
input and exact P union C union Rv. Concurrent authorized writers are forbidden. Malicious same-UID
precreation remains a nonclaim.

Existing/partial/aliased Rv or input drift poisons the epoch. Rv binds exact input rows and three
attestations, contains no self/future hash, is never child output, and is immutable. Later gates bind
the resulting Rv instance hash.

## Direct guard channels and authority postchecks

Guard is final L's direct parent under `c2b2a-final-l-path-invocation-v1`. Only FDs 0/1/2 reach
final L: read-only `/dev/null`, distinct nonseekable stdout/stderr capture pipes. Expected-hash and
all authority FDs are closed first.

Raw streams remain separate bounded external evidence. A third coordinator-owned channel carries
canonical guard facts: phase/E/M/Gate0/spend/invocation, trusted origins/deadlines/resources,
PID/PGID/SID/wait/cleanup/reap, stream hashes/counts, P/C/Rv/unselected transition, authority
pre/post rows, failures, validity.

Before dispatch and after complete drain/reap, coordinator/guard recursively expand, stable-open,
hold, and revalidate every selected E closure row plus applicable manifest/spend/parent state. The
canonical receipt includes pre/post full rows/preimages, held-E identity/hash, closure digest,
expected-hash channel receipt, exact invocation once, deadlines, spend, selected transition,
unselected unchanged, and terminal validity. Missing/extra/reordered/rebound/mutated/unreadable/
unvalidated objects are terminal. Raw evidence remains; success does not seal.

## Review gates and later provider boundary

All review attestations are create-new external canonical artifacts with exact input rows, harness/
provider/model/session/times, P0/P1/P2, verdict, and findings digest. Pass requires P0=P1=0. Writers
cannot review their artifacts; same-gate reviewers are blind until sealed; each three-review gate
uses at least two harness/provider families, at most two from one family. Correlated-model limits
remain explicit.

The exact gates are:

1. design attestations and atomic consolidated design review;
2. atomic Gate0 interpreted-enforcement acceptance;
3. Gate1 reviews of Gate0, final L/C/R, referenced inputs, M_static/P, run plan, S1/I_static policy,
   deadlines/channels/postchecks, then E1;
4. S1/static once, postcheck, three post-static reviews, then atomic I_static;
5. continuous I_static validation and durable replay control around every ordered pre-runtime phase,
   then pre-runtime-finalize;
6. M_completion, Gate2 review of full existing closure/P/C/Rv policies, then E2;
7. S2/completion once, exact P->P union C, three completion attestations, then atomic Rv;
8. build/source identity proposal+review;
9. provider-free Linux runner freeze+review;
10. runtime qualification+reviews and every remaining inherited gate; and
11. only eventually activation/evaluation via unchanged pilot runner.

E2 never authorizes provider work. Completion Rv is necessary but insufficient for later gates.

## Preserved inherited construction and exact invalidation

The inherited DAG remains:

```text
U -> A -> C -> HC -> L0/Q -> S -> W0/F0 -> components/B/V/O
  -> R0 -> D -> G_D -> L -> H -> G_H -> R
  -> authoritative W1/F1/replay
```

`Q(L)=L0`, `P(R)=R0`; H is raw only in sole `launcher_source_sha256` removed by P. Six anchors,
26 proof rows, carrier facts/codecs/dominance, non-source gates, and no-repair/no-replay semantics
remain. Final L after Q has a distinct exact identity from b5; no b5 hash/review transfers.

Any interpreter policy/Gate0/guard/bootstrap/phase guard/run plan/final L/Q/R/controller/referenced
provisioner/anchor/common bound/P pre-R0/amendment/review/M_static change invalidates Gate0/Gate1
as applicable. Static failure stops before I_static. I_static/phase receipt drift stops the current
and later phases. Post-static-only pre-runtime drift invalidates from affected phase/Gate2 without
rewriting historical static evidence. M_completion/P/C/Rv/Gate2/E2 drift invalidates Gate2.

A durable S1/S2 blocks phase retry. A pre-durability absent restart state proves no child admission
and permits the same attempt1 creation only under the exact recovery classifier. Any surviving
claim or partial C/Rv poisons or spends the epoch as defined. Alternate invocation, replay,
missing/duplicate/wildcard row, alias, unexpected delta, timeout, cleanup or authority failure is
terminal. Preserve all prior/partial bytes; never delete/repair/normalize/overwrite/replay. A new
attempt after a true start requires separately named successor and reviews.

Only external permission, interpreted Gate0/I_static enforcement, two manifests, durable spends,
typed mixed audit parent, exact deadlines/channels, and atomic Rv supersede matching inherited
clauses. All other inherited constraints remain.

## Acceptance boundary and current exact disposition

Before any candidate action: obtain three clean same-hash design attestations across at least two
families; atomically create consolidated design review; freeze TCB policy; implement/review/accept
Gate0; construct/review Q/final L/R; then proceed only through the exact gates above. Throughout
require pristine hashes/metadata, positive disk reserve, absent `target/debug`, combined output
absent before sole inherited create-new publication, and all no-replay/no-repair gates.

Design acceptance alone accepts no guard/Gate0/M/E/S/I_static/source/Q/static/pre-runtime/
completion/Rv/provider/pilot/flagship result. This amendment adds no provider/model/budget/VM/
adapter/daemon/datastore/connector/network/auth/product behavior/user data/runtime anchor/recovery
mode/source mutation, and authorizes no stage, commit, or live-setting change.

At present its prospective review is absent. There is no execution authority, Gate0, Q fill,
combined output, manifest, envelope, spend, I_static, pre-runtime run, completion output/Rv, or
controller/provisioner change. B5 `b5e50318...`, controller `eefa6eec...`, and provisioner
`aabe6b17...` remain unexecuted. Rejected 7d/f1/74c/bed960 bytes remain immutable evidence.
