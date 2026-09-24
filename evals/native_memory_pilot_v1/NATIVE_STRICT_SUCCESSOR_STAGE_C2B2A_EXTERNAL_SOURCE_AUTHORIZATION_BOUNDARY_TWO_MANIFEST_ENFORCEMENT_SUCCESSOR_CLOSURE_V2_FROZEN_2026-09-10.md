# Native strict successor Stage C2B2a — enforcement closure V2

Date: 2026-09-10
Status: frozen candidate design; no implementation or execution authority

## Disposition

This is an append-only successor to the rejected enforcement-closure epoch:

```text
SHA-256 bbdb5af7f55f5ee6c504f3ccbf3cf02c0711e5404fc33bc51366daa72700aaec
lines   515
bytes   32801
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_SUCCESSOR_CLOSURE_FROZEN_2026-09-10.md
state   rejected; no authority transfers
```

Its blind reviews were `FAIL 0/5/2` (Codex normative), `PASS 0/0/3` (Codex guard), and
`PASS 0/0/4` (isolated Claude); therefore the aggregate gate failed. Exact bbdb left external
review observations, DAG roots/types, coordinator continuity, and scanner control insufficiently
mechanical. The two passes cannot override the fail and no finding or verdict transfers.

Rejected predecessors `5fac7622...`, `bed96069...`, `74c883fe...`, `f1d7ddc8...`, `7d0ae76f...`,
and unavailable historical `1cd2c842...` remain immutable raw evidence and confer no authority.
The six inherited boundary/reconciliation/carrier records retain the exact hashes frozen in bbdb.
Current source candidates remain unchanged and unexecuted:

```text
b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530  launcher
eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9  controller
aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261  provisioner
```

Recursive candidate self-checks remain rejection-only. Success never grants permission. This V2
document supersedes bbdb only for external-review serialization, authority-DAG serialization,
coordinator continuity, completion scanner control, deadline wording, typed rows, and the matching
invalidation clauses below. Every other inherited constraint remains.

This candidate authorizes no import, compilation, AST parse, Q fill, source/candidate/static/build/
runtime/provider/pilot execution, publication, VM, adapter, daemon, datastore, or setting change.

## Exact paths and bootstrap parent

This amendment path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_SUCCESSOR_CLOSURE_V2_FROZEN_2026-09-10.md
```

Its prospective consolidated review path is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_EXTERNAL_SOURCE_AUTHORIZATION_BOUNDARY_TWO_MANIFEST_ENFORCEMENT_SUCCESSOR_CLOSURE_V2_REVIEW_2026-09-10.md
```

The review parent is frozen as absolute path
`/Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1`, device `16777233`, inode
`700150912`, UID `502`, GID `20`, mode `0755`, nlink `128`. Parent size, mtime, and ctime are
deliberately not identity fields. Any listed parent field change before record creation is terminal
and requires a newly named successor. The exact review basename is the prospective basename above;
it must be absent. The candidate record is one regular file, mode `0644`, at most 262144 bytes and
6000 LF bytes, final byte LF. Any pre-existing, overwritten, aliased, partial, invalid, or multiply
created leaf is rejected and never repaired.

## Trust model

The trust root is an honest coordinator and independent exact-byte reviewers. Assumptions remain
collision-resistant SHA-256, honest reviewed enforcement code/interpreter, and honest kernel/
filesystem/durable storage. Compromised coordinator/reviewer/host, same-UID injection or pathname
swap, malicious same-UID precreation, lying storage, and ambient same-UID access without an OS
sandbox remain explicit nonclaims. Owner-private paths do not authenticate against same UID/root.

## Canonical external review observations

Every design-stage reviewer output uses `c2b2a-external-review-observation-v1`, maximum 65536 bytes,
1000 LF bytes, 32 findings, final LF. It uses one encoding only:

- text bytes are UTF-8 and encoded as unsigned-decimal raw-byte length, colon, then lowercase hex of
  exactly those bytes; decimal has no leading zero except `0`;
- integer and count fields are unsigned decimal plus LF;
- fields occur in this order: schema, stage, ordinal, role, reviewed-target tuple, harness, provider,
  model, session, UTC start, UTC end, P0, P1, P2, verdict, findings count, findings block, findings
  digest;
- `unavailable` is the sole literal for an unavailable identity field; UTC is RFC3339 with seconds
  and `Z`; verdict is `PASS` or `FAIL`; pass requires P0=0 and P1=0; and
- each finding occurs by ascending ordinal and contains severity, title, exact section, exact line
  reference, evidence, impact, and proposed repair, each in the same length/hex encoding.

The findings-block preimage begins with the canonical findings-count field and ends with the last
finding field. `findings_sha256` is lowercase SHA-256 of exactly that byte string and is outside its
own preimage. The observation identity is `(byte_count, LF_count, SHA256(complete observation))`.
Unknown/duplicate/reordered fields, normalization, truncation, prose outside the grammar, mismatched
count/digest, or an unavailable required target identity is terminal.

`stage` is exactly `amendment-review` for the first three observations and `record-review` for the
later three. Roles/ordinals are exactly `(1,semantic)`, `(2,guard)`, `(3,adversarial)`. The reviewed-
target tuple is fixed-order absolute path raw bytes, type, device, inode, UID, GID, mode, nlink,
size, LF count, byte count, final-byte value, and content SHA-256.

## Non-recursive consolidated review bootstrap

After three clean blind amendment-review observations across at least two harness/provider families,
the coordinator may create one `c2b2a-design-review-v2` candidate at the prospective path. Its
fixed-order content is schema, this amendment's complete target tuple, then the complete canonical
bytes and observation identity of the three amendment observations in ordinal order. It contains no
self tuple/hash, record-review observation, Gate0/Q/L/E/future identity, or caller extension.

The coordinator first revalidates the frozen parent and absence, then performs one create-new
workspace-file operation. This candidate has no authority. It is full-read and independently
path/FD/stat/hash/count verified. Three fresh blind `record-review` observations must then review
that exact completed tuple. Those three full canonical observation bytes and identities remain
external coordinator trust-root state; no recursive second review record is created.

Only after all six observations are clean does the coordinator retain the complete accepted review
tuple `(path,device,inode,UID,GID,mode,nlink,size,LF_count,byte_count,SHA256)` out of band. Gate0 must
bind that identical tuple and embed the complete three record-review observation bytes/identities.
A byte-identical inode/path/metadata replacement is terminal. Existence or content hash alone never
accepts the record.

## Interpreter policy without self-identity

There is no native enforcement build. Exact interpreted enforcement uses a pinned interpreter and
`c2b2a-interpreter-tcb-policy-v2`, frozen before Gate0 review begins. Policy bytes contain only their
schema and closed interpreter/version/ABI/feature, loader/shared-cache, import isolation, empty site-
package/plugin, and literal ordered unpinned-stdlib semantic policy. They contain no own path,
metadata, counts, content hash, identity row, or internal digest. External Gate0/manifest/DAG rows
alone bind the policy file tuple. No glob/prefix/optional/caller expansion is allowed.

Every manifest inventories every candidate-controlled semantic repository/generated/writable/
plugin/import/config/data/executable/environment/argv/cwd/FD/path/absence/ordering input. Only the
honest kernel/syscall/filesystem, dyld/shared cache, OS CSPRNG, and exact policy-listed stdlib are
ambient TCB. An ambient category cannot launder any candidate-controlled input.

## Canonical authority DAG union V2

E1, I_static, and E2 use only `c2b2a-authority-dag-union-v2`. A serialized stream is exact header,
root table, artifact-row table, and terminal LF. `dag_stream_sha256` is over the complete header/root/
row bytes excluding only that digest field. Parser reconstructs and compares it.

An artifact key is `(length-prefixed raw absolute path bytes, st_dev, st_ino)`. A second path for one
device/inode, path rebinding, hardlink, symlink resolution, case/normalization/ancestry alias is
terminal. Content-equal distinct keys are allowed. Contextual roles exist only in root/edge labels,
never in artifact rows.

The root table is included in the digest and contains exact root count followed by entries
`(ordinal, length-prefixed label bytes, row_index)`. Ordinals begin at 1; labels are unique and occur
only in the schema-defined order. No implicit, optional, caller-selected, or unlisted root exists.

Rows contain `dag_depth`, type tag, artifact key, UID/GID/mode/nlink, and ordered edge table.
`regular` additionally requires size, byte count, LF count, final-byte value, and content SHA-256.
`directory` requires literal `not-applicable` for those five content fields and instead contains the
complete direct-entry count/table, its SHA-256, and child edges for every declared descendant. Direct
entries sort by raw basename bytes and bind basename/type/device/inode/UID/GID/mode/nlink; selected
recursion is closed and device-bound. Symlink/socket/FIFO/device/unknown types are terminal.

Each edge is `(length-prefixed unique label, child_row_index)`, sorted by raw label then child key.
Leaves have `dag_depth=0`; a parent has exactly `1+max(child dag_depth)`. Rows sort by increasing
depth, type tag, raw path, device, inode. Edges point only to an earlier lower-depth row. Each row's
preimage digest hashes its preceding exact fields excluding its digest. Repeated discovery of one key
reuses one row index only if its entire row and outgoing edges are byte-identical; otherwise terminal.
Every row is reachable from a root; extra/unreachable/missing/duplicate/inconsistent/forward/cyclic
content is terminal.

The E1 root table is exactly, in order:

```text
AMENDMENT
CONSOLIDATED_DESIGN_REVIEW
INTERPRETER_POLICY
PINNED_INTERPRETER
GUARD_SOURCE
BOOTSTRAP_SOURCE
PHASE_GUARD_SOURCE
COORDINATOR_SOURCE
GATE0_ACCEPTANCE
RUN_PLAN
M_STATIC
GATE1_SEMANTIC
GATE1_GUARD
GATE1_ADVERSARIAL
```

E1 is the externally authenticated envelope and is not its own row. Its dependency rows recursively
include Gate0 inputs/attestations and the three record-review observations embedded by Gate0.

I_static's sole representation of its acceptance inputs is this exact root table:

```text
E1_ROOT
S1_ROOT
STATIC_GUARD_ENVELOPE_ROOT
STATIC_COORDINATOR_RECEIPT_ROOT
RAW_FD1_ROOT
RAW_FD2_ROOT
POST_STATIC_SEMANTIC_ROOT
POST_STATIC_GUARD_ROOT
POST_STATIC_ADVERSARIAL_ROOT
```

`E1_ROOT` is one regular row for the now-existing E1 file; its edges reproduce E1's embedded root
table. No separate M_static root or second attestation list exists. Header counts/verdicts are
derived from these roots, never independently supplied. `e1_dag_stream_sha256` hashes E1's exact
embedded DAG stream and must recompute equal.

E2's root table is exactly E1_ROOT, I_STATIC_ROOT, one
`PRE_RUNTIME_RECEIPT/<length-encoded-exact-phase-id>` root per literal run-plan phase in that exact
order, PRE_RUNTIME_FINALIZE_ROOT, M_COMPLETION_ROOT, GATE2_SEMANTIC_ROOT, GATE2_GUARD_ROOT, and
GATE2_ADVERSARIAL_ROOT. `I_STATIC_ROOT`, `PRE_RUNTIME_FINALIZE_ROOT`, and `M_COMPLETION_ROOT` are
regular rows for those exact existing files. `I_STATIC_ROOT` edges reproduce I_static's root table.
No wildcard/prefix is expanded: the frozen run-plan literal phase list determines the exact finite
receipt labels before E2 review. Union traversal serializes every repeated dependency once. E2 is
not its own row.

## Lifecycle stages, not DAG depths

`lifecycle_stage` is never serialized as `dag_depth`. Exact order, including within-stage order, is:

```text
0  freeze amendment -> three amendment-review observations
1  create consolidated candidate -> three record-review observations -> accept full tuple
2  freeze policy/enforcement inputs -> three Gate0 attestations -> atomic Gate0 acceptance
3  construct inherited Q/R0/D/L/H/R -> final L and R
4  freeze run plan/M_static -> three Gate1 attestations -> E1
5  durable S1 -> guarded static -> three post-static attestations -> I_static
6  each ordered guarded pre-runtime phase/receipt -> pre-runtime-finalize
7  freeze M_completion -> three Gate2 attestations -> E2
8  durable S2 -> guarded completion -> O_pub
9  three completion attestations -> atomic Rv
10 build/source identity -> Linux runner -> runtime qualification -> remaining gates -> pilot
```

Exactly two phase manifests/envelopes exist: M_static/E1 and M_completion/E2. E2 authorizes only one
completion-audit-publish invocation. It never authorizes provider, model, network, activation,
evaluation, pilot runner, or budget spend. No higher-stage bytes flow backward.

## Coordinator capability and all-stage continuity

At the start of stage 2, the coordinator obtains exactly 32 random bytes from the OS CSPRNG and keeps
them only in locked process memory. The commitment is
`SHA256(ASCII("c2b2a-coordinator-cap-v1") || 0x00 || capability)`. Gate0, run plan, E1, S1,
I_static, every phase receipt, E2, S2, and guard receipts bind the commitment. The preimage is never
persisted, logged, placed in argv/environment/stdin/candidate/provider bytes, or inherited by a
candidate.

Every authority guard receives the capability preimage and applicable expected accepted tuple/hash
over a dedicated private one-shot channel, recomputes the commitment, authenticates the tuple/hash,
then closes every endpoint before candidate creation. The same live coordinator instance and
capability must persist from stage-2 start through completion postcheck. Capability loss, coordinator
death/replacement/exec/restart, or commitment mismatch permanently terminates the epoch. No accepted
review, Gate0, E1, I_static, or E2 expected tuple/hash may be reconstructed, adopted, or replaced from
target files, candidate bytes, receipts, or filesystem hints. Continuation requires a separately
named successor.

The channel uses exact `c2b2a-expected-hash-frame-v2`, maximum 2048 bytes, final LF then EOF. Each
field is `name=unsigned-byte-length:lowercase-hex(raw bytes)\n` in fixed order: schema, purpose,
epoch, phase, coordinator commitment, capability, target path, target device, target inode, target
size, target LF count, target byte count, target SHA-256. Names are literal ASCII; numeric values are
encoded as their minimal unsigned-decimal ASCII bytes before hex encoding. Unknown/reordered/
duplicate/short/extra/missing-EOF data is terminal. The configured maximum is not serialized.

## Gate0 and phase authority

Gate0 begins only after the accepted consolidated tuple and policy are final. Three clean blind
attestations across at least two families review exact interpreted guard, pinned interpreter/policy,
bootstrap/coordinator/phase guard, DAG/manifests/spend/I_static/Rv parsers, capability/hash and
scanner-control channels, watchdog/resources, typed audit parent, cleanup, and authority postchecks.

One external `c2b2a-gate0-acceptance-v2` binds the coordinator commitment, accepted consolidated
tuple, complete three record-review observations, exact reviewed input DAG, and three Gate0
attestations. It contains no self/future hash. Its exact owner-private parent/path/schema/bounds/0600/
absence policy is frozen before Gate0 review. Coordinator creates it with no-follow parent hold,
`openat(O_CREAT|O_EXCL|O_NOFOLLOW,0600)`, initial regular/same-device/owner/mode/nlink1/size0 checks,
bounded full write/file fsync, close/reopen no-follow, path/FD same-inode proof, bounded readback/
parse/hash/count, repeated file/parent fstat, close, and parent fsync. Partial/existing/invalid output
poisons the epoch. Gate0 alone authorizes no candidate action.

M_static and M_completion remain create-new complete semantic manifests under
`c2b2a-phase-manifest-v5`. The final-L schema is `c2b2a-final-l-path-invocation-v3`; final L is the
post-Q identity, never b5. M_static is frozen after final L/run plan and before Gate1, has embedded P,
one static invocation, zero filesystem output, and excludes future evidence. M_completion is frozen
only after I_static and all pre-runtime receipts/finalize exist, reconstructs identical embedded P,
includes final L, `C_inherited`, R, and every then-existing semantic input, and excludes future S2,
O_pub, Rv, qualification, and provider bytes.

`I_static` is `c2b2a-static-implementation-acceptance-v3`; its root table above is its sole input
identity representation. It contains epoch/pass/scope, literal run-plan phase list/digest,
`e1_dag_stream_sha256`, and that DAG stream, but no self/future hash or duplicate attestation list.
Coordinator creates it with Gate0-grade atomic protocol. Before/after each literal pre-runtime phase,
the same guard instance receives/authenticates the capability and expected I_static tuple, holds and
revalidates the entire DAG, drains/reaps descendants, and seals one receipt. Every phase separately
requires crash-durable at-most-one-start control; otherwise implementation is unimplementable.

## Durable spends and timeout classification

S1/S2 retain the exact O_EXCL/write/fsync/readback/parent-fsync protocol and child-before-durability
prohibition. Missing/rebound/inaccessible parent is terminal. O_EXCL is a live concurrency fence;
authorization is durably spent only after canonical readback and parent fsync.

The absent pre-durability classifier is permitted only to the same still-capable coordinator after a
caught non-timeout, non-process-fatal write/fsync failure, on the unchanged held/reopened parent. An
absent basename then proves no durable authorization and no child admission, so attempt 1 may be
recreated. Any surviving entry blocks/poisons. Coordinator/capability loss, timeout, cancellation,
or uncertain outcome is terminal and never eligible for absent recovery. If S is durable it remains
consumed; if timeout occurs before durable S, no child was admitted but the epoch is still terminal.

## Independent watchdog and completion scanner control

The independent guard watchdog captures `t0` before capability/hash-channel read, authority check,
spend, or other blocking operation and remains armed through postcheck. Static admission must return
by `t0+20s`, child terminates by `t0+140s`, cleanup/drain/reap by `t0+160s`, postcheck by `t0+180s`.
Completion final-L admission must return by `t0+50s`; trusted pre-spawn time is `t_final`; final L
terminates by both `t_final+1100s` and `t0+1150s`, cleanup by `t0+1175s`, postcheck by `t0+1200s`.
Every blocking call has trusted pre/post timestamps. Late admission is killed/reaped and terminal.

Static final L receives only FDs 0/1/2. Completion final L receives FDs 0/1/2 plus exactly one
write-only nonseekable guard control pipe at FD3, an explicit narrow supersession. M_completion binds
its identity and absence of every other FD. FD3 never carries authority/expected hashes.

Immediately after spawning its sole scanner child, final L writes one
`c2b2a-scanner-admission-v1` frame, maximum 2048 bytes, through FD3 and closes FD3. It uses the same
fixed length/hex field encoding and contains schema, epoch, final-L PID, scanner PID, scanner parent
PID, scanner process-birth tuple, PGID, SID, executable path/device/inode/hash, and candidate-reported
pre/post-spawn times. FD3 is close-on-exec for the scanner and never inherited by it.

Guard holds FD3's read end. By `t_final+100s` it must receive EOF, parse one frame, independently
observe the same live scanner PID/birth/executable/direct-parent/PGID/SID through reviewed host APIs,
and prove no extra descendant. Candidate timestamps are evidence only; guard receipt plus OS
observation is the admission authority. Guard continuously tracks the scanner/session. If the
scanner remains at `t_final+1000s`, or the frame is missing/late/invalid/mismatched, guard kills the
scanner and final-L session, proves session empty, and reaps final L inside outer cutoffs. Scanner
success permits final L publication/cleanup only until `t_final+1100s`. Any scanner/control failure
consumes durable S2 and cannot publish valid O_pub.

## Typed audit parent, O_pub, and Rv

P remains embedded in M_static and M_completion; no standalone P exists. For each interval the guard
freshly no-follow opens the frozen audit parent, holds it through the interval, and repeats identity
checks. P direct entries sort by raw basename bytes. Regular entries always bind basename/type/dev/
inode/UID/GID/mode/nlink/size/content SHA/byte count/LF count/final byte. Directory entries use
literal not-applicable content fields and bind metadata plus complete sorted direct-entry table,
closed selected subtree digest, and typed child edges. Symlink/socket/FIFO/device/unknown, undeclared
recursion, cross-device, hardlink/path/case/normalization alias, or duplicate name is terminal.

Publication output is only `O_pub={completion TSV, completion digest}`; the inherited construction
node is only `C_inherited`. State transitions are exactly `P -> P union O_pub -> P union O_pub union
Rv`. Static has no audit-parent delta. Partial/invalid O_pub is retained, terminal, and forbids Rv.

Before Gate2, only Rv path/schema/bounds/absence/creation policy exists. After valid completion and
three clean completion attestations, coordinator alone holds M_static, M_completion, audit-parent FD,
both O_pub files, E2/S2/traces/receipts/attestations and every bound input; verifies both embedded P
copies and live P; atomically creates Rv with Gate0-grade protocol; then revalidates all held inputs
and exact final state. Rv contains no self/future hash. Later gates bind its actual tuple.

## Invalidation, later gates, and current state

Any policy/Gate0/guard/bootstrap/coordinator/capability/run-plan/final-L/Q/R/C_inherited/referenced
source/anchor/P/amendment/review/M_static drift invalidates from its earliest dependency. Static
failure stops before I_static. I_static/coordinator/phase drift stops all later phases. M_completion/
Gate2/E2 drift invalidates Gate2. O_pub/Rv or completion-postcheck drift revokes downstream
completion acceptance beginning after Gate2; it does not make future O_pub/Rv a Gate2 input or
rewrite Gate2 history. Durable spend prevents retry. Every failure preserves bytes and requires the
named successor/recovery rules; no delete, repair, normalization, overwrite, or replay.

After Rv, unchanged separate gates remain: build/source identity review, provider-free Linux runner
review, runtime qualification/reviews, every inherited gate, and only eventually unchanged-pilot
provider activation/evaluation. E2 and Rv are never sufficient provider authority.

Design acceptance permits only creation/review of the consolidated record. Record acceptance permits
only policy/enforcement construction and Gate0 review. Gate0 acceptance permits only later gated Q
construction. At present the prospective review is absent; there is no Gate0, Q, manifest, envelope,
spend, I_static, pre-runtime execution, O_pub/Rv, combined output, or source change. `target/debug`
must remain absent. User-owned worktree changes remain preserved; nothing is staged or committed.
