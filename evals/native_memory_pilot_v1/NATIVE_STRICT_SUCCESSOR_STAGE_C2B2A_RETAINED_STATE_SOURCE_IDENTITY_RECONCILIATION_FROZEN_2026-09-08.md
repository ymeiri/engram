# Native strict successor Stage C2B2a — retained-state source-identity reconciliation

Date: 2026-09-08
Status: frozen candidate authority reconciliation; no implementation or execution authority

## Purpose and disposition

This reconciliation closes one source-identity cycle exposed while integrating the accepted
retained-state carrier source proof with the accepted completion-launcher boundary. The launcher
boundary permits one final raw launcher SHA-256 in one projected combined-record field and forbids
another launcher-derived value. The retained-state amendment requires 26 concrete source-proof
rows whose launcher identity and normalized AST/CFG facts are bound before the launcher semantic
digest is inserted. Repeating the final raw launcher hash in those rows would make the existing
finite construction cyclic.

The only authorized resolution is a dual launcher identity:

- `H` is the raw SHA-256 of the final launcher bytes. It remains present only in the existing sole
  top-level combined-record field `launcher_source_sha256` and remains the value removed by the
  accepted one-hole record projection.
- `S` is the SHA-256 of one exactly normalized launcher byte stream `Q(L)`. It is tagged by the
  exact identity-kind token `c2b2a-launcher-semantic-source-v1`. It may occur in the 26
  retained-state source-proof rows because it is invariant under the sole later semantic-digest
  substitution.
- Each of those 26 rows also carries the exact typed reference token
  `record-field-reference-v1:launcher_source_sha256`. That token is a non-`self` reference to the
  already verified sole top-level `H` field. It is not `H`, does not encode `H`, and cannot resolve
  to another field.

This document authorizes no implementation. Its exact final bytes must first receive at least
three independent exact-SHA reviews, each reporting `P0=0/P1=0`, and those outcomes must be bound
by a separately accepted review record at the prospective path below. No vote, finding, or hash
transfers after any byte or path change.

The earlier candidate with SHA-256
`d98a7ef0cc6c12815d44b5ca641c0df199b202194d9f1de37632880d1f9fa6f0` is rejected. Its review
outcomes do not transfer to this revision or to any later bytes.

## Controlling exact authorities and candidate context

This reconciliation is subordinate to all accepted C2B2a authority except for the explicit narrow
supersessions in this document. The immediately controlling accepted records are:

```text
SHA-256 8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
lines   513
bytes   34249

SHA-256 9cd1ec0211f53e3239bd36a3eb057bbf539e956727ef5f474dc41acc5ec8473f
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md
lines   1297
bytes   88630

SHA-256 26229d625abfe557d4a41c97d3809ce2f8333d8f249bba8151aee319c73168fd
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_REVIEW_2026-09-08.md
lines   182
bytes   13449
```

The following source identities are retained candidates only. They are useful starting points and
have no final implementation or execution authority:

```text
SHA-256 810051d1990df5875fa0dce6d5b1007ebc221c85c0a7e04050c60e33d782818e
path    evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
lines   12158
bytes   603827
state   IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256 is the 64-zero placeholder

SHA-256 b11ff52f4f4dc105dc9c146bfc07ee381826b295ecd5aaf7e730acf18487d60c
path    engram-eval/native-c2b2a-payload/build-support/controller.py
lines   27118
bytes   1067143
state   PREFLIGHT_AUTHORITY_SHA256 is the 64-zero placeholder expression

SHA-256 aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
path    engram-eval/native-c2b2a-payload/build-support/provision.py
lines   4349
bytes   160871
state   retained candidate; this reconciliation requires no provisioner edit
```

Neither candidate source hash is a substitute for the final identities constructed below. In
particular, `b11ff52f...` is not the final controller hash and `810051d...` is not `H` or `S`.

## Prospective exact audit-record paths

If this reconciliation is accepted, its final amendment and review records have exactly these
repository-relative paths:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_FROZEN_2026-09-08.md
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_SOURCE_IDENTITY_RECONCILIATION_REVIEW_2026-09-08.md
```

This narrowly supersedes the retained-state amendment's exact 20-path bounded non-input
audit-record list. The first 20 paths remain byte-for-byte and position-for-position unchanged.
The final reconciliation path is appended at position 21 and its later accepted review path is
appended at position 22. The new exact cardinality is 22. Missing, duplicate, reordered,
substituted, renamed, or extra paths are terminal.

Both new paths are preflight-only non-input audit records. Combined preflight and non-executing
exact-source review must bind their final path, SHA-256, LF count, byte count, metadata, and fixed
position. They are not controller, child, gate, completion-scanner, launcher-runtime, build,
provider, VM, pilot, payload, or runtime inputs. They do not enter or change the exact six-record
runtime trust-anchor set. The review record is created only after three independent clean reviews
of this reconciliation's final exact bytes; neither record contains its own SHA-256.

## Symbols and exact domains

All SHA-256 values below are lowercase 64-hex encodings of SHA-256 over exactly the stated bytes.
No textual path, Python object representation, Unicode normalization, newline conversion, AST
dump, record hash, or reviewer output may replace a stated byte domain.

The symbols are:

| Symbol | Exact meaning |
| --- | --- |
| `U` | the launcher-independent canonical runtime-authority semantic object after the exact controller support digest has been replaced by the existing literal `self` sentinel |
| `A` | `SHA256(canonical_json_line(U))`, in the already accepted `PREFLIGHT_AUTHORITY_SHA256` domain |
| `C0` | the fully frozen controller source template with its sole reviewed top-level `PREFLIGHT_AUTHORITY_SHA256` placeholder and no embedded `A` |
| `C` | final controller bytes obtained by replacing that sole placeholder assignment value with the exact lowercase-64-hex literal `A` once |
| `HC` | `SHA256(C)`, the final raw controller source identity |
| `L0` | final-form launcher bytes with the sole semantic-digest value span represented by 64 ASCII zero bytes and all other constants, including `HC`, final |
| `Q` | the exact byte normalization defined below, applicable only to accepted final-form launcher bytes |
| `S` | the candidate `SHA256(L0)`, accepted only after the authoritative verification epoch proves `Q(L) == L0` and therefore `S == SHA256(Q(L))` |
| `W0` | the ordered tuple of 26 candidate canonical proof-row byte strings constructed from the non-authoritative construction-epoch source models |
| `F0` | the candidate 27-LF proof record: the exact proof header followed by `W0` in order |
| `B` | the exact byte concatenation of the 200 final non-source vector-definition rows, serialized with their final indices and with no vector header or count row |
| `V` | the complete 226-vector definition stream: exact schema header, exact `vectors=226` row, and all 226 serialized vector rows in final order |
| `O` | the exact fixed-schema 229-LF static stdout whose only proof-dependent content is `V`'s SHA-256 in the summary row |
| `K` | the exact byte count contributed to `R0` by every fragment except the four classes of large hex value payloads whose lengths are added algebraically |
| `G_D` | the pure non-authoritative construction-transition predicate applied to candidate `D` before the sole D substitution |
| `G_H` | the pure non-authoritative construction-transition predicate applied to candidate `H` before the sole raw-H fill |
| `R0` | complete final-form combined-record bytes with only the sole `launcher_source_sha256` value represented by 64 ASCII zero bytes |
| `D` | `SHA256(R0)`, equivalently SHA-256 of the accepted one-hole semantic projection of final `R` |
| `L` | final launcher bytes obtained by substituting `D` once into `L0` |
| `H` | `SHA256(L)`, the final raw launcher source identity |
| `R` | final combined-record bytes obtained by filling only the sole raw launcher hash hole in `R0` with `H` |
| `W1` | the ordered tuple of 26 independently rebuilt canonical proof-row byte strings from the authoritative-verification source models |
| `F1` | the independently rebuilt 27-LF proof record: the exact proof header followed by `W1` in order |
| `J` | the fixed mutation-only lowercase-hex sentinel `1111111111111111111111111111111111111111111111111111111111111111` |
| `X186` | the exact 47-byte ASCII fixture descriptor `c2b2a-retained-state-over-1mib-bytes-fixture-v1`, whose SHA-256 is `e00b33112b85a250128ed5d7d7c6426b6ffa34b6eda909e1430b351d213963ee` |

The exact semantic source identity is the ordered pair:

```text
kind   c2b2a-launcher-semantic-source-v1
sha256 S
```

The tag is an identity-kind discriminator; it is not prepended to, appended to, or otherwise
included in the SHA-256 input. Thus the equation remains exactly `S = SHA256(Q(L))`.
`H` remains the sole primary raw launcher identity. `S` is only the supplemental normalized
preflight/static proof identity, is never embedded in launcher source, and cannot substitute for
`H` in a stable-source, record-projection, runtime-anchor, or completion comparison.

## Launcher-independent authority first

The authority ratchet begins before either source identity. Combined preflight first freezes the
launcher-independent authority semantic template, scanner semantic tables, exact retained-state
75-row table, exact six observation additions, exact paths, tools, policies, phase schedule, and
all other already accepted authority data. Call that complete parsed semantic input `U-input`.

To derive `U` and `A`:

1. put any grammar-valid lowercase 64-hex placeholder in the controller support source-digest
   input of `U-input`;
2. construct the exact top-level `_authority_object` under the accepted controller semantics;
3. require that the controller support entry is identified bijectively and that its source digest
   is replaced by the exact literal `self` in that object;
4. require that the placeholder has no other flow, alias, copy, observation, or output;
5. require that no controller bytes, `HC`, launcher bytes, `S`, `D`, `H`, combined-record raw hash,
   reviewer output, or final-review fact enters the object;
6. set `U` to that canonical object and set `A = SHA256(canonical_json_line(U))` in the existing
   `c2b2a-preflight-authority-v1` domain; and
7. insert `A` exactly once into `C0`, obtaining `C`, then compute `HC = SHA256(C)`.

Combined preflight then rebuilds the final controller support row with `HC`, reparses the same
frozen semantic inputs, reconstructs `_authority_object`, and requires byte-identical `U` and the
same `A`. The raw `HC` input must again be retired only through the `self` sentinel. A changed
object, another support entry, an `HC` occurrence in the canonical object, or an `A` mismatch is
terminal. `A` and `HC` must each be nonzero lowercase 64-hex and must be byte-distinct. There is no
iteration or fixed-point search.

This order makes the complete ratchet exact:

```text
freeze U-input and scanner semantic tables
  -> construct U with the controller support digest normalized to self
  -> compute A
  -> substitute A once into C0 to obtain C
  -> compute HC
  -> rebuild with HC and require identical U and A
  -> freeze final-form L0
  -> construction epoch: stable-read/parse C and L0 once each
  -> hash L0 to obtain candidate S; construct W0 and F0
  -> freeze R0 using HC, S, F0, and its injective static-vector transport
  -> compute D
  -> require G_D over the immutable pre-substitution domains
  -> substitute D once into L0 to obtain L
  -> compute H
  -> require G_H over the immutable pre-fill domains
  -> fill only the sole H hole in R0 to obtain R
  -> authoritative verification epoch: stable-read/parse C and L once each
  -> derive Q(L), W1, and F1 independently
  -> require Q(L) == L0, W1 == W0, and F1 == F0
  -> independently replay G_D/G_H and every final-binding gate
  -> accept only after all source, record, authority, static-vector, and final-gate predicates hold
```

Final `R` must reconstruct exactly the same `U`, `A`, `C`, and `HC`; no launcher-derived value may
enter `U` or `A`. Any mismatch invalidates the complete candidate before static or production
execution.

## Exact launcher normalization `Q(L)`

`Q` is defined only for bytes that pass every condition in this section. It is not a general text
rewrite and has no caller-selected name or span.

The final launcher must be UTF-8 without BOM, contain no CR or NUL, end in exactly one LF, and have
the preflight-bound LF and byte counts. Its module source must contain exactly one raw line of the
following form, including one ASCII space on both sides of `=`, double quotes, no prefix, no escape,
no comment, and one terminal LF:

```text
IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256 = "<64-lowercase-hex>"
```

The line must be a direct top-level `ast.Assign` in `Module.body`, with exactly one target, that
target an `ast.Name` whose identifier is exactly
`IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256`, and the value exactly one `ast.Constant` of builtin
string value type and length 64. It has no annotation, tuple/list target, concatenation, f-string,
call, alias, default, delete, later store, shadow, import, reflective access, or alternate spelling.
The identifier has exactly one Store occurrence in the module and that Store is this assignment.
Every Load is in the source-closed semantic-record comparison flow.

The source checker converts the Constant node's one-based line and zero-based UTF-8 byte-column
coordinates into one absolute byte interval. The exact source slice at the Constant coordinates
must be one ASCII double quote, the exact 64 lowercase hex value bytes, and one ASCII double quote.
The exact replacement interval is the interior 64 bytes only. Its start is the Constant's absolute
start plus one; its end is the Constant's absolute end minus one; it is half-open. The raw source
must have exactly one assignment line, one matching AST owner, one matching Constant span, and one
candidate interior interval. Raw global search-and-replace is forbidden.

For accepted launcher bytes `X`, define:

```text
Q(X) = X[0:value_start] || (ASCII "0" repeated exactly 64 times) || X[value_end:]
```

No quote, identifier, spacing, LF, or other byte changes. `Q(X)` has exactly the same byte count and
LF count as `X`. Applying `Q` twice yields the same bytes. A missing/duplicate/relocated assignment,
uppercase or nonhex value, wrong width, alternate quoting, escape, line continuation, mismatched
AST span, second plausible span, or change outside the 64-byte interval is terminal.

`L0` has those 64 value bytes already zero. The construction epoch computes the candidate `S`
directly as `SHA256(L0)` and parses that same final-form `L0` exactly once for candidate proof
construction. `L` differs from `L0` only in that interval and contains exactly `D` there. The
authoritative verification epoch parses final `L` exactly once, proves the interval and
`Q(L) == L0`, independently rebuilds the proof from the normalized final-`L` model, and only then
accepts `S = SHA256(Q(L))` without a cycle.

## Two exact source-analysis epochs

There are exactly two distinct source-analysis epochs. Neither epoch imports or executes either
module top level, compiles the `Authority` class, `parse_authority`, `_authority_object`, either
production adapter, or compiles or executes a source mutation.

### Non-authoritative construction epoch

This epoch starts only after final `C` and final-form `L0` exist and their paths, expected identities,
and expected LF/byte counts are fixed by the construction protocol. The epoch then performs exactly
one stable source read and exactly one `ast.parse` of `C`, and exactly one stable source read and
exactly one `ast.parse` of `L0`. It checks `SHA256(C) == HC`, `SHA256(L0) == S`, all frozen counts,
and the exact zero-valued Q assignment/span. It builds one controller source model and one launcher
source model. Because `L0` already has zeros at the Q interval, its parsed tree is the construction
epoch's normalized launcher model; an immutable comparison also proves that changing the copied
assignment value to 64 zeros is a no-op.

For each of the exact 26 parent cases, the construction engine creates a fresh `copy.deepcopy` of
both retained epoch roots and executes only that case's exact Q-invariant row-construction recipe
from the partition table below. No case consumes a prior case's mutated root. It does not resolve
the raw-H reference, inspect final `L`, `H`, `R`, or `P(R)`, test a stale or wrong final H, scan for
future `D`/`H`/raw-record values as a case predicate, execute a final-binding recipe, or claim that
the complete parent case has passed. The later pure `G_D` and `G_H` construction-transition guards
are outside the 26-case engine: they exist only after their respective candidate digest exists,
emit no row or other artifact, and cannot retroactively change a recipe result. The engine derives
candidate `W0`, candidate `F0`, and the 26 source-vector expected values exclusively from the
Q-invariant results. The complete candidate vector-definition bytes combine those values with the
independently frozen 200 non-source components only after the component-length gate below succeeds.
These artifacts are inputs to `R0`; this epoch is expressly non-authoritative and cannot accept a
source, record, proof row, parent case, vector, static result, or run. A construction mismatch
discards the entire candidate.

### Authoritative verification epoch

This epoch starts only after `D` has produced final `L`, `H` has produced final `R`, and all final
paths, LF/byte counts, and raw candidate hashes are fixed. It performs exactly one stable source
read and exactly one `ast.parse` of final `C`, and exactly one stable source read and exactly one
`ast.parse` of final `L`. These reads and parses are independent of the construction epoch. They
prove `SHA256(C) == HC`, `SHA256(L) == H`, the exact raw assignment/span and value `D`, and all
frozen counts.

The verification engine applies `Q` to the retained final-`L` bytes and requires byte-for-byte
`Q(L) == L0`. It makes exactly one `copy.deepcopy` of the parsed final-`L` root for the retained
normalized launcher root and changes only the copied assignment Constant's builtin string value to
64 ASCII zeros. Coordinates, node kinds, contexts, owners, and every other value remain unchanged.
It independently retains the final-controller root. For each source case it then creates a fresh
deep-copy pair from these two verification roots, applies only the same exact Q-invariant
row-construction recipe, and independently rebuilds `W1`, `F1`, and all source-vector expected
values. Acceptance first requires element-for-element `W1 == W0`, byte-for-byte `F1 == F0`, and
byte-for-byte equality of the rebuilt vector definitions and static expected bytes to their `R0`
artifacts.

Only after those equalities and final `L/H/R/P(R)` validation does the verification engine execute
the partition table's final-binding-only gates. Each such gate uses fresh immutable final values,
executes every listed positive check and negative submutation, and returns only `None` on complete
success or raises the parent case's exact `result_token`; it emits no proof row, witness bytes,
digest, vector expected value, stdout field, or mutable carrier. A final gate cannot change `W1`,
`F1`, the canonical proof record, vector definitions, or static expected stdout. Completion of all
final gates must dominate `_proof_witness`, `_run_static_vectors`, combined-preflight success, and
every later success exit. A skipped/default gate or one executed during construction is terminal.
The applicable final gates independently recompute and replay the complete `G_D` and `G_H`
predicates from validated `Q(L) == L0`, `P(R) == R0`, and final `D/L/H/R`, then run their negative
mutations. A construction guard is only a necessary transition condition; it never supplies an
authoritative fact to a final gate.

Neither epoch parses `Q(L)`, a mutation, a reconstructed source string, or a second snapshot within
that epoch. There are exactly four proof-content parses in the complete ratchet: construction `C`,
construction `L0`, verification `C`, and verification `L`. There is no shared mutable AST between
epochs. A source read or parse outside those four, reuse of a mutated case root, use of construction
artifacts as verification facts instead of equality operands, or proof data derived from the raw
unnormalized final-`L` assignment value is terminal.

Only the three already accepted pure retained-state codec `FunctionDef` nodes may be deep-copied
from the authoritative-verification controller model into the one synthetic direct-vector module.
No other controller or launcher definition or module statement crosses that compile boundary.

The completion-launcher boundary's entry/final stable-source checks remain intact. An outer final
stable reread may only byte-compare a fresh stable snapshot to the retained accepted `L` snapshot
for liveness. It is not parsed, normalized, hashed into a new proof identity, or used to replace or
augment any proof fact. The corresponding existing controller liveness reread has the same
comparison-only restriction. A mismatch is terminal. These outer comparisons occur after the
authoritative verification epoch and do not create a third source-analysis epoch.

Unsupported AST, an unresolved owner or edge, an extra read or parse within either epoch, mutation
reuse, cross-epoch model reuse, or a fact derived from the unnormalized semantic-digest value is
terminal.

## Existing one-hole record projection remains exact

The accepted record projection is unchanged. Final `R` contains exactly one LF-terminated ASCII
field beginning at record start or immediately after LF:

```text
launcher_source_sha256=<64-lowercase-hex>
```

The value is exactly `H`. The existing projection `P(R)` replaces only those 64 value bytes with
64 ASCII zero bytes and leaves every other byte unchanged. Missing, duplicate, wrong-width,
uppercase, nonhex, non-LF-terminated, or second plausible locations are terminal.

`R0` is complete final record content with that value already zero. It contains neither `D` nor
its own raw SHA-256. `D = SHA256(R0)`. After the one substitution in `L0` and the one raw source
hash computation, final `R` is made by replacing only that zero value with `H`. Final validation
requires all of the following:

```text
P(R) == R0
SHA256(P(R)) == D
L differs from L0 only at the exact Q interval
Q(L) == L0
SHA256(Q(L)) == S
SHA256(L) == H
R.launcher_source_sha256 == H
```

`R`, `R0`, `L`, and `L0` have their frozen respective line and byte counts; each fixed-width fill
preserves its file's counts. No step is repeated, guessed, repaired, normalized after freeze, or
searched for a fixed point.

## Semantic identity in the 26 proof rows

Every retained-state source-proof row uses the semantic source identity, never the raw source hash.
Its exact launcher identity fields and values are:

```text
launcher_semantic_source_kind = c2b2a-launcher-semantic-source-v1
launcher_semantic_source_sha256 = S
launcher_raw_source_reference = record-field-reference-v1:launcher_source_sha256
```

The reference is a typed record-field reference. Resolution requires exactly one top-level scalar
field with the exact referenced name, requires that field already passed the raw stable-read
`SHA256(L) == H` comparison and the one-hole projection checks, and returns no serializable hash
value to the row. The literal `self`, a relative name, an indexed row, a generic binding, an alias,
a caller-supplied reference, a reference chain, or another field is terminal. The reference is not
a self-reference: it points from a proof row to the distinct sole top-level raw identity field.

Full positive parent-case acceptance ultimately requires both identities: the row's `S` binds its
Q-invariant AST/CFG facts, while the later final-only resolution of the typed reference proves those
facts were checked against the same final raw `L` whose `H` occupies the sole top-level field. The
row itself proves only the exact reference syntax and field name. A row with correct `S` but a
stale, unverified, ambiguous, or mismatching `H` fails at the final binding gate.

During the non-authoritative construction epoch, `W0` carries only the fixed reference token; `H`
does not yet exist and the token cannot be resolved or treated as an accepted identity. During the
authoritative verification epoch, the independently rebuilt identical token in `W1` is resolved
only after final `L`, `H`, `R`, and the one-hole projection are verified. Resolution does not alter,
refill, or reserialize either row tuple or proof record.

The Q-invariant wrong-reference mutation domain is exactly two values: the literal `self` and `J`.
Both are source-closed before `D` or `H` exists. `J` is a grammar-valid lowercase 64-hex string but
is never an identity, proof value, authority value, vector expected value, output, or accepted row
field. The authoritative final gate requires `J != H` before using that mutation result. No
Q-invariant recipe may insert, replace, scan, compare, or encode the actual candidate `H`. The sole
pre-authoritative handling of actual `H` is the positive `G_H` construction-transition scan outside
the 26-case engine; it emits nothing and accepts nothing. Every case-engine insertion, replacement,
scan, comparison, or bounded encoding of actual `H` is exclusively a final-binding operation.

The earlier retained-state wording that required the raw launcher SHA-256 value inside every proof
row is superseded. A proof row may not contain `H` as text, bytes, hex, tuple member, nested value,
digest input, evidence value, or derived value. All 26 rows contain the same `S`, tag, and typed
reference literal. Exact equality and cardinality are required.

## Canonical proof-row record

The 26 logical proof rows are serialized into one canonical LF-terminated UTF-8 proof record. Its
first line is exactly:

```text
schema=c2b2a-retained-state-source-proof-v1
```

It is followed by exactly 26 `canonical_json_line` rows. Each row has exactly these keys in this
order:

```text
kind sequence case source_role controller_source_sha256 launcher_semantic_source_kind
launcher_semantic_source_sha256 launcher_raw_source_reference lexical_owner node_kind spans
occurrence_count normalized_def_use polarity dominators success_exits failure_exits expected
result_token
```

`kind` is exactly `retained-state-source-proof`. `sequence` is a non-boolean u64 exactly `0..25`.
`case` follows the exact inventory below. `source_role` is exactly `controller-source`,
`launcher-source`, or `cross-source` according to the source-closed case table below; no shorter
spelling is accepted. It classifies only the Q-invariant recipe and facts actually serialized in
this row. A construction-transition guard, final-binding gate, or fact used only by either has no
row `source_role` and cannot add an owner, span, def-use edge, CFG node, or mutation site to the row's
role derivation. `controller_source_sha256` is exactly final `HC`.
The three launcher identity fields have the exact values above. `lexical_owner` and `node_kind` are
nonempty source-closed ASCII tokens.

`spans` is a nonempty ordered array of objects with exactly the keys
`source_role,start_line,start_utf8_byte_column,end_line,end_utf8_byte_column`. A span's
`source_role` is exactly `controller-source` or `launcher-source`; `cross-source` is legal only for
the containing case row. Lines are one-based; UTF-8 byte columns are zero-based; ends are
exclusive. Every coordinate is a non-boolean u64 and maps respectively to the accepted final `C`
controller model or accepted final `L` and same-coordinate normalized launcher model.
`occurrence_count` is a positive non-boolean u64 equal to the bijective occurrence inventory.

`normalized_def_use` is the exact ordered canonical def-use value emitted by the one proof engine
from the normalized model. `dominators`, `success_exits`, and `failure_exits` are exact ordered
arrays of the same canonical source-span values used by that engine. Empty is legal only when the
source-closed proof contract for that case requires empty; a missing reachable success or failure
exit is terminal. `polarity` is `positive` only for sequence 0 and `negative` otherwise.
`expected` is respectively `accepted` or `rejected`. `result_token` is exactly the token in the
inventory below. Those two fields state the frozen parent-case expectation and Q-invariant engine
token; they do not serialize, summarize, or attest a final-binding-only gate. The final gate is
accepted only as a later dominating control-flow fact and can never alter canonical row bytes.

Canonical JSON uses the already accepted canonical JSON scalar, escaping, integer, list, object,
and LF rules. Every proof-row byte string includes its one final LF. Re-encoding must be
byte-identical. Write `n_i = len(W_i)` and
`e_i = len(_canonical_value(W_i)) = 2*n_i + digits(n_i) + 3`, where `digits` is the minimal
unsigned decimal digit count. Each row must satisfy `1 <= n_i <= 524,283` and
`e_i <= 1,048,576`. The boundary is exact: `n_i=524,283` gives `e_i=1,048,575`, while
`n_i=524,284` gives `e_i=1,048,577` and is terminal. These checks occur before creating either
canonical-value bytes or a hex field.

The proof record has exactly 27 LF rows: the exact 44-byte header plus 26 proof rows. Therefore
`len(F) = 44 + sum(n_i)` and the row bound gives an exact maximum of `13,631,402` bytes. The
inherited `16,777,216` proof-record ceiling remains required but is weaker than this derived bound.
All per-row and aggregate bounds are checked before hex decoding, joining, or allocation in both
epochs.

### Exact combined-authority transport

The complete proof record is transported injectively through the existing combined-record
authority block with exactly four new scalar fields and 26 indexed fields:

```text
retained_state_source_proof_count=26
retained_state_source_proof_lf_count=27
retained_state_source_proof_bytes=<minimal unsigned decimal in 1..16777216>
retained_state_source_proof_sha256=<64-lowercase-hex>
retained_state_source_proof.000000.row_hex=<lowercase even hex>
...
retained_state_source_proof.000025.row_hex=<lowercase even hex>
```

The authority payload retains its existing strict raw-ASCII key sort, so these fields occur at
their unique lexicographically determined positions; no conceptual regrouping changes that order.
There is no count alias, generic binding, default, omitted row, alternate suffix, or extra field.
Each indexed value decodes to exactly one proof-row byte string, including its final LF, in
sequence order. The decoded row must contain no CR or NUL, must end in exactly one LF, must contain
no other LF, and must reparse and re-encode as the exact canonical JSON row for that index. Its
`sequence`, `case`, `source_role`, `expected`, and `result_token` must equal the independent closed
tables; all other fields must equal the independently derived proof facts.

The future `_parse_authority` implementation must add the four exact scalar names to
`scalar_keys`, parse `retained_state_source_proof_count` with `_authority_take_count`, parse the
indexed rows with exactly
`_authority_take_indexed(values, "retained_state_source_proof", count, ("row_hex",))`, add every
expanded key to `expected_keys`, and reject any missing, duplicate, reordered, or extra authority
key through the existing exact-set gate. It must length-check the hex before `_decode_hex`, decode
each row with the exact `524,283`-byte cap, compute and check `e_i` arithmetically before creating
`_canonical_value(W_i)`, and return only these immutable values:

```text
authority["retained_state_source_proof_rows"] = tuple(decoded row bytes in order)
authority["retained_state_source_proof_binding"] =
    (count, lf_count, byte_count, sha256)
```

The binding tuple uses exact builtin non-boolean integers and lowercase-hex digest text. It is a
launcher preflight/static authority value, not a controller `Authority` field, controller CLI
option, runtime scanner argument, generic preflight binding, result key, evidence path, completion
row, or seventh runtime anchor. The rows container must have exact builtin type `tuple`; every row
must have exact builtin type `bytes`; no subclass, list, generator, lazy decode, mutable alias, or
post-validation store is accepted.

`_bind_record` reconstructs the source-closed header followed by the 26 accepted decoded rows,
requires exact 26/27 counts, `len(F)`, `SHA256(F)`, ordered case names, source roles, identities,
spans, proof facts, polarities, exits, expected values, and result tokens, and retains that verified
immutable record for static mode. Combined preflight independently reconstructs the same `F0` and
all four scalar values. The authoritative verification epoch independently reconstructs `F1` and
requires `F1 == F0` before static execution. A digest-only comparison, caller-supplied row table,
or row reconstructed from a vector's expected bytes is insufficient.

After final source verification, `_static_mode` supplies the exact authoritative `W1` tuple under
the new `proof.retained-state-source` key when it builds `proofs`; it does not carry `W0`, raw `C`,
raw `L`, a parsed AST, or the authority row hex into `_run_static_vectors`. The latter requires the
new exact four-key proof-key set, rechecks the source-row/vector bijection before its case loop, and
passes the selected immutable row to `_proof_witness`. Every successful exit from `_bind_record`,
`_static_mode`, `_run_static_vectors`, and the final static comparison is dominated by the exact
proof binding, independent source rebuild, all final-binding-only gates, vector mapping, and output
checks. Construction-epoch results cannot enter `proofs`.

### Exact source-row to static-vector mapping

The future `_REQUIRED_VECTOR_CASES` contains one contiguous retained-state block at its unique
source-closed position immediately after `recursion-review-present` and immediately before
`runtime-six-anchor-order`: the exact 19 direct cases in their order below, followed immediately by
the exact 26 source cases in their order below. The combined count remains 45 and no case appears
elsewhere. In the held 181-row launcher inventory, `recursion-review-present` has zero-based index
172. Therefore the direct cases have exact final zero-based vector indices `173..191`, source case
`i` has exact final vector index `192 + i` (`192..217`), `runtime-six-anchor-order` moves to exact
index 218, and the final complete vector count is exactly 226. Those numbers, the two boundary case
names, and every case at every index are independently required; inserting another case anywhere
is terminal. The combined authority's existing `static_vector_count` is therefore exactly `226`.
Every source case has target exactly `proof.retained-state-source`; every source
vector has outcome exactly `return` and argument bytes that decode by the existing literal parser
to exactly the empty tuple `()`.

The future `_static_target_table` adds exactly
`"proof.retained-state-source": _proof_witness`. The `proofs` exact-key gate adds that same fourth
key and requires the exact sorted tuple
`("proof.controller-source", "proof.launcher-source", "proof.provisioner-source",
"proof.retained-state-source")`. Its immutable value is the already independently
verified ordered tuple of the 26 proof-row byte strings; it is not a construction artifact, digest
tuple, mutable collection, or raw source. For this exact target only, `_proof_witness` requires one
case-name match at the closed index and returns that exact verified proof-row byte string `W_i`.
It does not return the existing four-field digest witness used by other `proof.*` targets and does
not validate a row by consulting vector expected bytes. Control flow reaching this branch must be
dominated by successful completion of all final-binding gates; `_proof_witness` has no default,
skip, or construction-epoch path.

For source vector index `i`, the authority vector row is exact:

```text
global vector index = 192 + i
name     = exact source case i
target   = proof.retained-state-source
outcome  = return
args     = ASCII ()
expected = _canonical_value(W_i)
```

The mapping is injective because `_canonical_value` uses its existing type tag, byte length,
lowercase byte hex, and terminal delimiter for builtin `bytes`; distinct canonical row bytes yield
distinct expected bytes. For `type(W_i) is bytes`, the exact formula is
`b"y" || minimal_ascii_decimal(len(W_i)) || b":" || ascii_lower_hex(W_i) || b";"`; subclasses and
alternate encodings are terminal. `_parse_authority` requires every source vector's exact index, name,
target, outcome, empty-tuple arguments, and `expected == _canonical_value(decoded proof row i)`.
`_vector_definition_stream` then serializes those existing five vector fields and their existing
length/hash/hex framing. `_run_static_vectors` first revalidates the complete proof record and exact
mapping, obtains `W_i` only from the fourth verified proof authority, and uses its existing
`_canonical_value(observed) == expected` comparison. Thus neither a self-consistent substituted
row/vector pair nor a proof digest without its row bytes can pass.

### Exact empty `expected_hex` authority exception

The raw combined-authority payload retains its universal nonempty-value rule except for exactly one
syntactic key family. A raw key qualifies only when its bytes are precisely:

```text
ASCII "static_vector." || exactly six ASCII decimal digits || ASCII ".expected_hex"
```

Only such a key may have zero raw value bytes after its sole `=`. Its canonical empty encoding is
therefore a line ending immediately after `expected_hex=` with one LF; for index 186 the line is
exactly `static_vector.000186.expected_hex=` followed by LF. There is no `-`, sentinel, space,
quoted empty string, alternate hex spelling, wildcard suffix, or general allow-empty flag.
`_decode_hex("", 1_048_576)` returns exact builtin `b""`; the serializer and all byte/length/hash
formulas remain unchanged.

The raw syntactic exception cannot accept a field. After the exact authority key set,
`static_vector_count`, six-digit index, suffix, and indexed-row order are bound, every permitted
empty key must be an in-range member of that exact set. An empty scalar, an empty value under any
other indexed suffix or prefix, a malformed/short/long/nondigit index, an out-of-range six-digit
index, or an extra key is terminal. `args_hex` is always nonempty, including index 186. Every
nonempty hex field retains the exact even-length lowercase `[0-9a-f]+` grammar and its existing
decoded-byte cap; `-`, odd length, uppercase, nonlowercase, and nonhex are terminal.

After every vector row is decoded but before vector-definition comparison, target selection,
dispatch, or the index-186 materializer, require the bidirectional invariant:

```text
(expected == b"") iff (outcome == "ValueError")
```

Thus every `return` vector has nonempty expected bytes and every `ValueError` vector has exactly
empty expected bytes. The check uses the independently accepted exact outcome token and decoded
expected bytes, not the raw key's emptiness alone, and dominates every static success exit.

This parser exception and iff dominance are added to the existing Q-invariant case-24
`q-runtime-reconstruction` proof facts and its unchanged `launcher-source` role. Combined preflight
independently derives the exact raw-key matcher, indexed-key set, row-decode flow, iff predicate,
dominators, and failure exits; the row cannot derive them from carried vector values. The separate
global proof requires the raw syntactic exception to be the sole empty-value branch and the iff gate
to dominate vector-definition comparison, materialization, dispatch, and static success. The
authoritative `final-record-reconstruction` gate independently binds all outcomes and expected bytes
from final `R`. No changed vector/proof pair can self-authenticate, and no case, role, target, or
stdout row is added.

### Inherited vector and record caps

No cap is raised. Let `d(x)` be the minimal unsigned decimal digit count, let `c_i` be source case
`i`'s exact ASCII name bytes, let `a_i=2` for the exact argument bytes `()`, and retain `e_i` from
the row bound above. Before any vector row is serialized or joined, each epoch freezes one exact
builtin immutable component tuple
`(global_index,name,target,outcome,arguments,expected)` for every non-source vector and one exact
source prefix tuple `(global_index,name,target,outcome,arguments)` plus its retained `W_i/n_i/e_i`
for every source vector. The fields have the exact types and individual bounds already required by
the accepted vector parser; `global_index` is the independently derived final index, not a stored
vector field. The existing
`_vector_definition_stream` serializes such a component with exact row length:

```text
145 + d(global_index) + len(name) + len(target) + len(outcome)
    + d(len(arguments)) + 2*len(arguments)
    + d(len(expected)) + 2*len(expected)
```

The constant 145 is exactly six ASCII bytes for `vector`, ten tabs, one LF, and the two 64-byte
SHA-256 fields. The length gate derives both SHA-256 field lengths as the fixed literal 64 and does
not allocate either row or its hex fields. For source row `i`, global index `192+i` has three digits,
target
`proof.retained-state-source` has 27 bytes, outcome `return` has six bytes, and `a_i=2` has one
decimal digit. Its exact addition is therefore:

```text
r_i = 186 + len(c_i) + d(e_i) + 2*e_i
```

The first phase freezes, without serializing, exactly the 200 non-source component tuples using
their final indices: rows `0..191` followed by rows `218..225`. This includes all 19 direct retained
codec components at final indices `173..191`; no direct result may be discovered after this phase.
For each non-source component `j`, independently compute the formula above as `b_j`, require every
operand and arithmetic result to be a bounded non-boolean unsigned integer, and set
`B_len = sum(b_j)` in that exact 200-component order. The same phase freezes the 26 source prefix
tuples together with their already bounded immutable `W_i/n_i/e_i`, computes every `r_i`
algebraically without allocating `_canonical_value(W_i)` or a complete source component, and
computes:

```text
V_len = 63 + B_len + sum(r_i for i in 0..25)
```

For global index 186 specifically, the fixed lengths are: index digits 3, name 36, target 43,
outcome 10, arguments 47 with two length digits and 94 hex bytes, and expected 0 with one length
digit and zero hex bytes. Therefore `b_186 = 145+3+36+43+10+2+94+1 = 334`. Neither the
1,048,577-byte materialized payload nor its tuple is an authority component or a term in `b_186`,
`B_len`, `V_len`, `K`, or `len(R0)`.

The exact 63 bytes are the 51-byte vector schema header and the 12-byte `vectors=226` row. Before
serializing any vector row, joining any vector bytes, creating any definition hex, or allocating
`V`, require `V_len <= 16,777,216` and all inherited individual argument/expected bounds. A missing,
late, mutable, caller-supplied, or post-serialization component, including any of the 19 direct
components, is terminal.

Only after that complete component-and-length gate passes may the second phase materialize each
source expected value as `_canonical_value(W_i)`, form its complete immutable component, and
serialize every component exactly once with the held serializer. It requires each actual row length
to equal its precomputed `b_j` or `r_i`. `B_pre` is then the byte concatenation of serialized rows `0..191`,
`B_post` is the byte concatenation of serialized rows `218..225`, and `B = B_pre || B_post`; `B`
contains no schema header, `vectors=` row, or source row. The split is derived from those row counts
and is not caller supplied. The only legal complete stream is:

```text
V = header_51 || count_row_12 || B_pre || source_rows_0_through_25 || B_post
len(B) = B_len
len(V) = V_len = 63 + B_len + sum(r_i for i in 0..25)
```

After serialization, require actual `len(B) == B_len`, actual `len(V) == V_len`, and exact component
round-trip equality before freezing `B`, `V`, their byte counts, and SHA-256 values. Combined
preflight and both source-analysis epochs independently derive the same immutable 200 non-source
components, 26 source components, arithmetic lengths, and final bytes. Actual `V` and its SHA-256
must match the combined-authority definition binding. No byte stream produced before the length
gate, alternative serialization, or digest-only `B` comparison is accepted.

The complete record cap is checked before any large record hex value or `R0` allocation. Define
`K` as the exact byte count of final `R0` with only these value payloads omitted while retaining
every corresponding key, `=`, and LF: the 26 proof `row_hex` values; the 26 source-vector
`expected_hex` values at indices `192..217`; `static_vector_definitions_hex`; and
`static_expected_hex`. `K` includes every other byte of the record, including both authority block
delimiters, every non-source vector value, all decimal lengths and SHA-256 fields, and the exact
64-zero `launcher_source_sha256` hole. It is derived by summing the exact source-closed fragments,
not by allocating a record skeleton. For index 186, `K` includes the exact expected-field key, `=`,
and LF with zero value bytes, plus the descriptor's 94 `args_hex` bytes; it never includes expanded
payload bytes. If `O` is the exact 229-LF static stdout, then the only legal total is:

```text
len(R0) = K + 2*sum(n_i) + 2*sum(e_i) + 2*len(V) + 2*len(O)
```

The four terms respectively restore proof `row_hex`, source-vector `expected_hex`,
`static_vector_definitions_hex`, and `static_expected_hex`; no term is already included in `K`, and
no raw `F`, `V`, or `O` blob is stored a second time. Before any of those hex encodings or `R0` is
allocated, require `1 <= len(R0) <= 67,108,864`, every individual authority value satisfies its
existing bound, and `2*len(V) <= 33,554,432`. After construction, actual field lengths and actual
`len(R0)` must equal the formula. The fixed-width D and H substitutions preserve source and record
sizes, so final `R` has the same bound. Construction and authoritative replay each perform this
arithmetic independently before their corresponding allocations; equality of final counts and
bytes is mandatory.

The case-name streams remain exactly:

```text
schema=c2b2a-retained-state-direct-case-names-v1
<minimal-unsigned-decimal-sequence><TAB><case-name><LF>

schema=c2b2a-retained-state-source-case-names-v1
<minimal-unsigned-decimal-sequence><TAB><case-name><LF>
```

Sequences are respectively the minimal ASCII decimal values `0..18` and `0..25`, with no sign,
leading zero, whitespace, or alternate spelling. Their SHA-256 values are the existing
`retained_state_direct_case_names_sha256` and
`retained_state_source_case_names_sha256`. The combined inventory is exactly the complete direct
tuple followed by the complete source tuple. Its count is 45 and is bijectively determined by the
two counts, two ordered-name digests, disjointness, and concatenation rule; no third caller-chosen
order or digest is introduced.

## Exact 19 direct codec cases

The direct inventory remains exactly the following 19 cases in order. The negative must raise
inside the named extracted codec function, never in a harness precheck. The positive must traverse
all three functions in the listed order and byte-compare the accepted canonical return.

```text
00 retained-state-table-positive
   c2b2a_encode_retained_state_table -> c2b2a_parse_retained_state_table -> c2b2a_validate_retained_state_table_binding
   return
01 retained-state-table-missing-row
   c2b2a_parse_retained_state_table
   ValueError
02 retained-state-table-duplicate-row
   c2b2a_parse_retained_state_table
   ValueError
03 retained-state-table-reordered-row
   c2b2a_parse_retained_state_table
   ValueError
04 retained-state-table-extra-row
   c2b2a_parse_retained_state_table
   ValueError
05 retained-state-table-wrong-header
   c2b2a_parse_retained_state_table
   ValueError
06 retained-state-table-wrong-schema
   c2b2a_parse_retained_state_table
   ValueError
07 retained-state-table-wrong-row-schema
   c2b2a_parse_retained_state_table
   ValueError
08 retained-state-table-wrong-edge
   c2b2a_parse_retained_state_table
   ValueError
09 retained-state-table-wrong-root
   c2b2a_parse_retained_state_table
   ValueError
10 retained-state-table-wrong-name
   c2b2a_parse_retained_state_table
   ValueError
11 retained-state-table-wrong-absence
   c2b2a_parse_retained_state_table
   ValueError
12 retained-state-table-wrong-observation
   c2b2a_parse_retained_state_table
   ValueError
13 retained-state-table-over-1mib-bytes
   c2b2a_parse_retained_state_table
   ValueError
14 retained-state-table-noncanonical-bytes
   c2b2a_parse_retained_state_table
   ValueError
15 retained-state-table-wrong-binding-name
   c2b2a_validate_retained_state_table_binding
   ValueError
16 retained-state-table-wrong-binding-rows
   c2b2a_validate_retained_state_table_binding
   ValueError
17 retained-state-table-wrong-binding-bytes
   c2b2a_validate_retained_state_table_binding
   ValueError
18 retained-state-table-wrong-binding-sha256
   c2b2a_validate_retained_state_table_binding
   ValueError
```

### Sole compact materializer for direct case 13

Direct ordinal 13 has global vector index exactly `186`. Its complete accepted vector identity is:

```text
index      186
name       retained-state-table-over-1mib-bytes
target     controller.c2b2a_parse_retained_state_table
outcome    ValueError
arguments  X186
expected   b""
```

`X186` is the exact descriptor bytes defined above, not a Python literal and not the malformed
payload. It is 47 bytes, satisfies every generic authority argument-byte cap, and has only the one
stated spelling, length, and SHA-256. The authority vector row, the component tuple used by the
`B_len/V_len` gate, `B`, `V`, `K`, `R0`, and every length/hash/hex field serialize, count, and hash
only those 47 descriptor bytes. The expanded fixture is absent from authority, `B`, `V`, `R0`, proof
rows, source-vector values, and static stdout.

At `_run_static_vectors`, the special branch is reachable only after complete authority
parse/binding, exact required-case ordering, vector-definition byte and digest equality, target-table
closure, proof-authority validation, and the global source-proof dominance predicate have all
succeeded. At loop index 186, but outside the outcome-catching `try`, it requires the simultaneous
exact identity above: index, name, target, outcome, `arguments_data == X186`, and
`expected == b""`. It also requires no other vector uses `X186`. Any mismatch raises terminally
outside the outcome catcher.

Final launcher source contains the exact 47-byte `X186` literal exactly once, as the comparison
operand in this branch; it has no assignment, alias, concatenation, decoder, environment source,
caller override, or second Load. The case-24 occurrence inventory and accepted authority component
must agree on that one literal without deriving either from the other.

Only after those comparisons, the branch executes the one literal materialization expression
`payload = b"x" * 1_048_577` and immediately constructs exact `arguments = (payload,)`. The size and
byte value are source literals, never descriptor-controlled. There is no payload wrapper, target
adapter, size/content precheck, synthetic exception, fallback, alternate materializer, cache, or
reuse by another vector. Allocation, tuple construction, identity-check, dispatch-selection, or
other non-target failure occurs outside the outcome catcher and cannot satisfy `ValueError`.

The branch then enters the ordinary outcome `try` and performs the same direct
`target(*arguments)` call as every vector. For index 186, source proof requires `target` is the exact
already extracted `c2b2a_parse_retained_state_table`, the tuple is not rebuilt or inspected, and the
sole payload object reaches argument zero unchanged. A counted failure requires exact builtin
`ValueError` whose traceback originates inside that extracted function after the direct call;
another exception, a launcher/harness raise, a pre-call rejection, or a swallowed allocation failure
is terminal. The function's own `len(data) > 1_048_576` check therefore remains the sole expected
rejection and is actually traversed.

Every other vector, including the other 18 direct cases and all 26 source cases, retains the generic
bounded `ast.literal_eval(arguments_data)` route and its existing caps. `X186` cannot reach that
route, and no other descriptor or materializer branch exists.

This structure is carried by the existing Q-invariant case 24
`retained-state-source-wrong-runtime-reconstruction`, whose role remains `launcher-source`; it adds
no public case or role. The independently source-closed case-24 proof facts cover the unique index
literal, exact six-field identity comparisons, descriptor occurrence and retirement, dominating
validation predecessors, literal allocation expression, one-element tuple def-use, direct target
dispatch, ordinary-try boundary, exception provenance, unchanged payload flow, and generic else
route. Combined preflight derives those expected facts without reading vector expected bytes, and
the authoritative `final-record-reconstruction` gate separately binds the final index-186 component
to `R`; changing the row and proof together cannot self-authenticate. The separate global predicate
proves the branch's predecessors and success/failure dominance. Neither case 24 nor this branch
interacts with `G_D`, `G_H`, or another final-binding gate.

Each negative fixture reaches the extracted function with the malformed input intact. Except for
the exact nonrejecting index-186 fixture materialization above, a test-only adapter, expected-failure
label, caller-side type/size/hash check, or exception manufactured by the harness is not coverage.
The exception permits fixture construction only; it does not relax target selection, direct-call,
exception-origin, or result semantics. The direct vector definitions and fixture-generation rules
contain no `S`, `H`, `D`, raw record hash, future reviewer output, or unnormalized AST digest.

## Exact 26 non-executing source cases

The source inventory, exact `result_token`, and exact `expected` values are:

```text
00 retained-state-source-positive-authority-flow                 positive-authority-flow accepted
01 retained-state-source-identity-mismatch                       identity                rejected
02 retained-state-source-reread-or-reparse                       single-parse            rejected
03 retained-state-source-unsupported-ast                         supported-ast           rejected
04 retained-state-source-occurrence-inventory-mismatch           occurrence-bijection    rejected
05 retained-state-source-missing-option                          option-presence         rejected
06 retained-state-source-duplicate-option                        option-uniqueness       rejected
07 retained-state-source-generic-binding-substitution            dedicated-option        rejected
08 retained-state-source-wrong-cli-arity                         cli-arity               rejected
09 retained-state-source-wrong-cli-order                         cli-order               rejected
10 retained-state-source-wrong-cli-final-position                cli-position            rejected
11 retained-state-source-wrong-cli-row-literal                   row-literal             rejected
12 retained-state-source-wrong-cli-byte-bound                    byte-bound              rejected
13 retained-state-source-wrong-cli-digest-grammar                digest-grammar          rejected
14 retained-state-source-wrong-frozen-binding-name               binding-name            rejected
15 retained-state-source-cli-default-reset-or-alias              cli-retirement          rejected
16 retained-state-source-wrong-authority-field-order-or-default  authority-field         rejected
17 retained-state-source-wrong-authority-constructor-assignment  authority-constructor   rejected
18 retained-state-source-authority-alias-reflection-or-mutation  authority-immutability  rejected
19 retained-state-source-wrong-authority-object-key-position     authority-object-position rejected
20 retained-state-source-wrong-authority-object-nested-order     authority-object-order  rejected
21 retained-state-source-wrong-authority-object-value-flow       authority-object-flow   rejected
22 retained-state-source-missing-validation-dominance            validation-dominance    rejected
23 retained-state-source-wrong-source-role                       source-role             rejected
24 retained-state-source-wrong-runtime-reconstruction            runtime-reconstruction rejected
25 retained-state-source-controller-launcher-divergence          adapter-agreement       rejected
```

The exact ordered `source_role` table is:

```text
00 retained-state-source-positive-authority-flow                 cross-source
01 retained-state-source-identity-mismatch                       cross-source
02 retained-state-source-reread-or-reparse                       cross-source
03 retained-state-source-unsupported-ast                         cross-source
04 retained-state-source-occurrence-inventory-mismatch           cross-source
05 retained-state-source-missing-option                          controller-source
06 retained-state-source-duplicate-option                        controller-source
07 retained-state-source-generic-binding-substitution            controller-source
08 retained-state-source-wrong-cli-arity                         controller-source
09 retained-state-source-wrong-cli-order                         controller-source
10 retained-state-source-wrong-cli-final-position                controller-source
11 retained-state-source-wrong-cli-row-literal                   controller-source
12 retained-state-source-wrong-cli-byte-bound                    controller-source
13 retained-state-source-wrong-cli-digest-grammar                controller-source
14 retained-state-source-wrong-frozen-binding-name               controller-source
15 retained-state-source-cli-default-reset-or-alias              controller-source
16 retained-state-source-wrong-authority-field-order-or-default  controller-source
17 retained-state-source-wrong-authority-constructor-assignment  controller-source
18 retained-state-source-authority-alias-reflection-or-mutation  controller-source
19 retained-state-source-wrong-authority-object-key-position     controller-source
20 retained-state-source-wrong-authority-object-nested-order     controller-source
21 retained-state-source-wrong-authority-object-value-flow       controller-source
22 retained-state-source-missing-validation-dominance            controller-source
23 retained-state-source-wrong-source-role                       cross-source
24 retained-state-source-wrong-runtime-reconstruction            launcher-source
25 retained-state-source-controller-launcher-divergence          cross-source
```

This is the only source-role token grammar. Combined preflight derives this table directly from
the accepted case contracts. The launcher contains an independent exact literal table. The proof
engine derives a role without consulting either carried table: `controller-source` means every
primary owner, occurrence, def-use edge, CFG node, and mutation site used by the row's Q-invariant
recipe is in the controller model; `launcher-source` means every such row-construction item is in
the normalized launcher model; `cross-source` means the Q-invariant row recipe's required equality
or exhaustive submatrix has primary proof owners in both models.
Empty ownership, a shorter token, an unknown token, or disagreement among the independently
derived role, exact launcher table, combined-preflight table, and canonical row is terminal.

The transition guards and final-binding gates are source-proved separately and globally, outside
this table and outside all 26 row-role calculations. Their implementation and dispatcher nodes must
have exact launcher-source lexical ownership; any controller-model or final-record fact they read is
an immutable explicitly named input, not an implementation owner. The global proof bijectively
inventories those nodes and their allowed immutable inputs, proves the two guards dominate their
respective substitutions, proves every final gate dominates `_proof_witness`, static execution, and
preflight success, and forbids another owner, dispatcher, default, bypass, or mutable carrier. This
global proof emits no row, vector, record, or stdout bytes. Case 22 remains `controller-source`
because its serialized Q-invariant recipe proves controller-side validation dominance; the distinct
global proof of final-gate dominance does not change that role.

Case 23 cannot prove itself by changing all copies together. Its closed Q-invariant submutations
independently change exactly one of: the carried canonical-row role, the launcher literal-table
role, or one Q-recipe model-owner/mutation-domain fact used by derivation, while the other two
channels and the exact table above remain fixed; each must reject with `source-role`. Its later
`final-source-role-equality` gate compares those already derived Q-invariant roles and tables only;
it never classifies itself or another final gate. Case 25 independently changes first
the controller adapter derivation and then the launcher adapter derivation while the opposite
source, 75-row authority, proof-role table, and expected record remain fixed; each must reject with
`adapter-agreement`. This is zero-based case 25, the 26th source case. A caller-selected or
proof-row-derived role is forbidden.

The exact per-parent partition is below. These recipe and gate tokens are internal source-closed
proof-engine selectors only; they add no public case, vector, proof-row key, authority field, or
stdout value. `none` means that every invariant for that parent is Q-invariant and there is no
final-only operation for that row.

```text
00 positive-q-authority-flow       final-positive-raw-h-binding
01 q-identity                      final-identity-binding
02 q-single-parse                  final-epoch-read-parse-binding
03 q-supported-ast                 final-q-assignment-binding
04 q-occurrence-bijection          final-occurrence-binding
05 q-option-presence               none
06 q-option-uniqueness             none
07 q-dedicated-option              none
08 q-cli-arity                     none
09 q-cli-order                     none
10 q-cli-position                  none
11 q-row-literal                   none
12 q-byte-bound                    none
13 q-digest-grammar                none
14 q-binding-name                  none
15 q-cli-retirement                none
16 q-authority-field               none
17 q-authority-constructor         none
18 q-authority-immutability        none
19 q-authority-object-position     none
20 q-authority-object-order        none
21 q-authority-object-flow         none
22 q-validation-dominance          final-gate-dominance
23 q-source-role                   final-source-role-equality
24 q-runtime-reconstruction        final-record-reconstruction
25 q-adapter-agreement             final-epoch-adapter-agreement
```

The Q-invariant recipes may inspect only stable `C`, final-form `L0` or normalized final-`L` model
facts, `HC`, `S`, fixed names/tokens, and source-closed authority/adapter structure. In particular,
`q-identity` covers mutations of `S`, its kind token, the syntax and exact field name of the typed
raw-H reference, replacement of that reference only with `self` or `J`, the normalized Q interval,
`HC`, and the controller identity field, but never an actual, guessed, or resolved H.
`q-single-parse` covers the source structure that enforces one read/parse in the current epoch, not
an observation from the other epoch. `q-runtime-reconstruction` covers the Q-invariant launcher
reconstruction algorithm and field flow without supplying final record values.

The final gates are exact. `final-positive-raw-h-binding` validates final `L/H/R/P(R)` and resolves
the sole typed reference. `final-identity-binding` runs the fresh wrong/stale sole-H, changed-final-L
with stale H, wrong Q/D value, premature/wrong reference resolution, and D/H/raw-record occurrence
submutations. `final-epoch-read-parse-binding` checks the exact two-epoch read/parse inventory and
forbids any extra proof-content read or parse. `final-q-assignment-binding` validates the sole raw
final-`L` assignment and Q span. `final-occurrence-binding` requires construction and verification
owner/span/occurrence bijection plus the unique final D slot. `final-gate-dominance` proves every
other final gate dominates static/preflight success. `final-source-role-equality` compares the
independently derived Q-invariant verification roles to both fixed tables and `W0`, without
classifying any final gate. `final-record-reconstruction`
authenticates and reconstructs the proof transport from final `R` without using vector expectations
as authority. `final-epoch-adapter-agreement` requires final controller/launcher adapter results,
`W1/F1`, vector definitions, and static expected output to equal their construction artifacts.

Sequence 0 runs the Q-invariant positive recipe over one fresh unmodified
controller/normalized-launcher model pair. Each negative Q recipe runs the same source-proof engine
over its own fresh deep-copy pair with only its exact Q-invariant mutation. Harness logic may select
and apply a mutation, but it may not decide its result. The row's `expected` and `result_token` are
the frozen parent-case disposition and token; during construction they do not claim that a listed
final gate ran. Final parent success is the conjunction of the identical authoritative Q-invariant
row result and successful completion of its exact final gate, if any. A mutation that fails in
mutation construction, a harness precheck, serialization, expected-output comparison, or an
unrelated invariant does not satisfy the case.

Case 01 is one closed identity-mismatch parent with a partitioned exhaustive submatrix. Its
Q-invariant fresh copies separately mutate `S`, the semantic identity-kind token, typed raw-H
reference kind or field-name syntax, replace that reference with exactly `self` or `J`, mutate the
normalized Q value interval or copied Constant, mutate `HC`, and mutate the controller SHA field.
Only `final-identity-binding` separately inserts or mutates the actual verified sole global `H`,
changes final `L` while retaining stale or wrong `H`, changes final Q/D binding or reference
resolution order, and attacks D/H/raw-record occurrence closure. It also first proves `J != H`.
Every submutation must reject with `identity`; none emits or changes a row; all submutations together
remain one parent case and do not change the 26-case count.

Cases 02 through 25 retain the exact accepted mutations. Their closed variants additionally cover,
without adding cases:

- a Q-recipe source shape that authorizes an extra current-epoch read/parse, and, only in
  `final-epoch-read-parse-binding`, an actual extra read or parse in either epoch, parsing `Q(L)`,
  parsing a mutation or reconstructed source, using final `L` in construction, using `L0` as the
  verification input, reusing a mutated model, sharing a model between epochs, or deriving proof
  facts from an unlisted snapshot;
- missing, duplicate, aliased, nested, relocated, wrong-width, uppercase, nonhex, escaped,
  concatenated, or non-top-level semantic-digest assignments and any second Q candidate span;
- changing a byte outside Q's 64-byte interval, changing line/byte counts, applying global
  replacement, or failing `Q(L) == L0`;
- omitting, duplicating, reordering, or adding proof rows; wrong case sequence/name/count/name
  digest, wrong proof-row key order, wrong proof-record bytes/hash, unsupported proof atom, or an
  unproved reachable exit;
- replacing the typed reference with exactly `self` or `J` is Q-invariant; inserting actual
  candidate `H`, resolving the reference before the sole raw-H comparison, resolving it to another
  field, or permitting a reference chain is tested only by the final binding gates;
- placing future reviewer output or a digest of the unnormalized AST in a proof row, vector
  definition, expected evidence value, or launcher constant is Q-invariant; the unmutated `D` and
  `H` occurrence predicates are first required by `G_D` and `G_H` after those values exist, while
  their negative mutations, actual-H insertion, raw record hash, and derived/encoded attacks are
  tested only by the authoritative final binding gates;
- making the 26 proof-row `S` values disagree, changing the domain tag, or changing proof facts
  before and after `D` substitution;
- changing source role, Authority/CLI/object flow, validation dominance, controller/launcher
  table reconstruction, direct/source inventory disjointness, or the exact 19/26/45 counts; and
- iterating `S`, searching for a fixed point, accepting a caller value, executing a mutation,
  importing module top level, or compiling a non-codec definition is Q-invariant; any `D`/`H`
  iteration or refill is tested only after candidate `D`/`H` exist by the final binding gates.

Each variant is bound as a named submutation in the source-case definition bytes and all variants
must reject for their parent case to pass. A Q-invariant variant runs in both source-analysis epochs;
a final-binding-only variant runs only in the authoritative epoch after its required final value
exists. This preserves the exact 26 source cases and 45 total cases while preventing one weak
mutation from standing for a broader invariant.

## Q-invariant static proof and expected-output independence

The narrow permitted launcher-derived content is exactly:

1. the same `S` and exact identity-kind token in each of the 26 source-proof rows;
2. Q-invariant source coordinates, lexical owners, node kinds, occurrence counts, normalized
   def-use facts, dominators, and exits computed from the normalized model; and
3. the complete proof-record bytes/counts/digest and vector-definition bytes/count/digest
   necessarily derived from items 1 and 2; and
4. the existing static expected-output SHA-256, which may depend transitively on the
   vector-definition digest printed by the fixed summary row.

Those values may exist in `R0` and final `R` only in the exact authority fields defined above. They
may not be embedded as final expected-value literals or digests in `L0` or `L`; the launcher must
compute them from its authoritative-verification source models and compare them to the already
semantically authenticated record. The literal case names, mutation recipes, identity-kind token,
typed reference token, serializer, and proof algorithm may be in `L0`; no final `S` or S-derived
digest may be.

The direct-vector definition bytes remain fully launcher-identity independent. The source-case
definition bytes contain only case names, invariant tokens, and mutation recipes; they contain no
derived or accepted identity value. The fixed literal `J` is permitted only as the mutation recipe's
nonidentity wrong-reference sentinel and the final gate must prove `J != H`. The existing
`_vector_definition_stream` may include the source vectors'
`_canonical_value(W_i)` expected bytes and their length/hash/hex framing, so that stream and its
digest may depend on `S`. No other vector field may carry a source identity or proof fact.

Static subprocess stdout remains byte-for-byte in the existing schema and contains only:

```text
schema=c2b2a-completion-launcher-static-test-v1
case<TAB><minimal-index><TAB><exact-case-name><TAB>pass
...
summary<TAB>cases=<exact-total><TAB>vector-sha256=<definitions-sha256>
valid=true
```

With the exact 226-vector inventory, `<exact-total>` is `226` and stdout has exactly 229 LF rows:
one schema row, 226 case rows, one summary row, and one `valid=true` row.

The case rows contain no observed return, expected value, proof row, source role, span, identity, or
result token. No literal `S`, raw-H reference token, `HC`, normalized proof fact, or proof-record
digest is permitted in stdout. Only the summary's existing `vector-sha256` value and, therefore,
the combined record's existing `static_expected_sha256` may depend transitively on proof rows or
`S`. `static_expected_bytes`, `static_expected_lines`, and the output bytes are recomputed from the
fixed schema/case names and exact vector-definition digest; they may not carry an alternate proof
transport. A proof fact printed directly, an added summary field, or a successful source vector
whose proof row was not matched before the `pass` row is terminal.

Outside that closed allowance:

- raw `H` exists only in the sole top-level `launcher_source_sha256` field;
- `D` exists only as the sole value of the final launcher's exact top-level
  `IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256` assignment;
- no raw combined-record SHA-256 is recorded or used;
- no future reviewer output or value derived from it enters construction, source proof, vectors,
  expected output, launcher source, controller authority, scanner arguments, or runtime; and
- no fact or digest derived from the unnormalized AST may appear in a proof row, vector expected
  value, or static expected-output dependency.

Accepted pre-existing review-record identities retain only their already authorized audit-record
or six-anchor roles. This reconciliation does not derive proof expectations from their verdict
text and does not add future reviewer output to any source, record, gate, or runtime input.

## Exact construction and final revalidation

The proof record is an ordinary immutable input to `R0`; it has no digest hole. The only legal
construction procedure is:

1. freeze the launcher-independent `U-input`, scanner semantic tables, accepted paths, all
   non-launcher source candidates, the exact 75-row retained-state table, exact 19/26/45 case
   definitions, exact 26-entry source-role table, proof serializer, proof engine, vector mapping,
   and all fixed literal algorithms;
2. derive `U` with a grammar-valid controller-digest placeholder retired to `self`, derive `A`,
   fill `C0` once, compute `HC`, rebuild with `HC`, and require byte-identical `U` and the same `A`;
3. freeze final-form `L0` with exact final `HC`, all algorithms, inventories, roles, constants,
   imports, LF count, and byte count, leaving only the exact semantic-digest value interval as 64
   zeros; no proof row, proof digest, vector expected value, `S`, `D`, or `H` is embedded in `L0`;
4. perform the exact construction epoch over stable `C` and `L0`, compute candidate
   `S = SHA256(L0)`, build `W0`, enforce every `n_i/e_i` and `F0` bound before canonical-value or
   hex allocation, and build `F0`; freeze all 200 non-source component tuples, including the 19
   direct tuples, plus the 26 source prefix tuples and retained `W0/n_i/e_i`; compute every `b_j`,
   `r_i`, `B_len`, and `V_len`, and require the complete vector cap before any vector-row
   serialization; only then materialize source expected values, serialize every component once,
   freeze exact `B` and `V`, verify their actual lengths and hashes against the arithmetic and
   components, and build exact static expected stdout from `V`'s digest;
5. construct complete `R0` once, including `HC`, `S`, every proof scalar/indexed field, `F0`, the
   exact vector-definition bytes/digest, exact static expected bytes/counts/digest, and only the
   existing raw launcher hash field represented by 64 zeros, but first derive `K`, apply the exact
   nested-hex size formula, and require the `67,108,864`-byte cap before allocating any large hex
   payload or `R0`; require the symbolic uniqueness and pre-hash absence conditions below;
6. compute `D = SHA256(R0)` exactly once, run pure non-authoritative `G_D` over the exact immutable
   pre-substitution domains below, and, only if it passes, replace the exact Q value interval in
   `L0` once with `D` to obtain `L`;
7. require the byte diff between `L0` and `L` is exactly that 64-byte interval, then compute
   `H = SHA256(L)` exactly once, run pure non-authoritative `G_H` over the exact immutable pre-fill
   domains below, and require it passes;
8. replace only `R0`'s exact sole `launcher_source_sha256` zero value once with `H` to obtain `R`;
9. perform the authoritative verification epoch over stable final `C` and final `L`; independently
   derive `Q(L)`, `S`, `W1`, and `F1`; independently rebuild the same 200 non-source components and
   26 source prefixes, apply the same pre-serialization `n_i/e_i/F1/B_len/V_len` arithmetic gates,
   then rebuild source expected values, vector definitions, static expected stdout, and the
   pre-allocation `R0` cap; require `Q(L) == L0`, `W1 == W0`, `F1 == F0`, and byte-identical equality
   of every rebuilt record-carried artifact;
10. independently parse final `R`, resolve each typed raw-H field reference only after final
    `SHA256(L) == H`, reconstruct `P(R)`, `R0`, `D`, every proof row and vector mapping, and the
    exact same `U/A/C/HC`; require all equations, counts, key sets, roles, inventories, spans,
    proof facts, output bytes, and hashes; then execute every final-binding-only gate, including
    independent replay of `G_D` and `G_H` plus their negative mutations from the reconstructed
    pre-transition values, without modifying an artifact; and
11. perform only the accepted comparison-only final source/record liveness rereads, then permit
    separately authorized static execution. A liveness reread is never parsed or used to rebuild a
    fact.

The resulting finite DAG is exactly:

```text
U-input -> U -> A -> C -> HC
(C, HC, L0) -> construction epoch -> S -> W0 -> F0
(fixed 200 non-source components, 26 source prefixes, W0)
    -> arithmetic component/length gate -> B -> V -> O
(HC, S, F0, V, O, all other fixed record fields)
    -> K/R0 size gate -> R0 -> D -> G_D -> L -> H -> G_H -> R
(C, L, R, D, H, W0, F0, V, O)
    -> authoritative C/L/R verification
    -> Q(L), P(R), W1, F1, rebuilt components/B/V/O
    -> exact equality with L0, R0, W0, F0, B, V, and O
    -> independent G_D/G_H replay and all final-binding-only gates
    -> proof witness and static/preflight success
```

`F0` is exactly 27 rows and is complete before `R0`; its SHA-256 is computed once over those final
bytes and is stored without a placeholder. Neither `F0`, `W0`, the vector definitions, nor static
expected stdout depends on `D`, `L`, `H`, final `R`, raw record hash, a final-binding gate result,
or reviewer output. `F1` is a verification result only and is never refilled into `R0`. Final gates
produce control-flow acceptance only after `F1 == F0`; they emit no bytes into the DAG. There is no
proof-record placeholder, post-`D` proof repair, fixed-point search, digest iteration, or retry. Any
mismatch discards the complete candidate; a successor restarts from step 1 with newly frozen
inputs.

### Exact staged uniqueness and absence checks

Checks are performed only after the value they name exists. Before `D` or `H` is computed,
combined preflight proves only symbolic structure: exactly one Q assignment and byte interval,
exactly one projected raw-H field and zero hole, exact proof-field names and cardinalities, exact
typed reference literal, no reference chain or alias, no generic proof transport, no second
semantic-digest or raw-H owner, and no field or algorithm that can later copy either digest. These
symbolic facts may enter Q-invariant rows. Concrete candidate-D and candidate-H checks occur twice:
first as the pure necessary construction-transition guards before their respective substitutions,
then as independently recomputed authoritative final-binding predicates. The later predicates alone
authorize success and execute the negative mutations. A raw-record-hash scan has no construction
transition and remains final-binding-only. No guard emits or alters a proof row, proof record,
vector, static output, binding, digest, count, or other carried byte, and no guard accepts a case.

For these checks, `hex^0(X)` is the 64 ASCII bytes of lowercase digest text `X`, and
`hex^(n+1)(X)` is the lowercase ASCII hex encoding of `hex^n(X)`. The exact authority/row/vector
framing has maximum reachable encoding depth three: canonical row text, `row_hex` or
`_canonical_value(bytes)`, vector `expected_hex`, and authority `static_vector_definitions_hex`.
The checker scans `hex^0..hex^3` of the lowercase value and, when byte-distinct, its uppercase
spelling. Another encoding layer or a decoder not in this exact chain is forbidden.

After candidate `S` exists but before `R0` is hashed, require `S` is lowercase 64-hex and nonzero,
`S != A`, and `S != HC`. The only authorized `S` spans are exact: `hex^0(S)` once in the
`launcher_semantic_source_sha256` field of each decoded canonical row and therefore its `F0`
placement; `hex^1(S)` only inside the corresponding `row_hex` and
`_canonical_value(W0[i])` field payloads; `hex^2(S)` only inside the corresponding vector
`expected_hex`; and `hex^3(S)` only inside the authority's exact
`static_vector_definitions_hex`. The parser derives those span whitelists from independent
field boundaries, not substring search. Any candidate encoding occurrence outside its exact
whitelisted span is terminal. `S` may additionally affect only the permitted proof-record,
vector-definition, and static-expected digests transitively. Every `hex^0..hex^3(S)` form is absent
from `L0`, direct-vector definitions, case-definition literals, mutation recipes, and static
stdout.

After `D = SHA256(R0)` exists and before substitution, `G_D` requires `D` is lowercase 64-hex,
nonzero, and pairwise distinct from `A`, `HC`, and `S`. It scans exact immutable stable `C`, `L0`,
`R0`, `W0`, `F0`, all vector arguments and expected values, vector definitions, and static expected
stdout for every `hex^0..hex^3(D)` lowercase/uppercase form; every occurrence is terminal. It emits
nothing and merely permits or discards the transition. During authoritative verification,
`final-identity-binding` reconstructs the same pre-substitution domains from validated
`Q(L) == L0` and `P(R) == R0`, recomputes the complete `G_D` predicate, runs each wrong/omitted-scan
negative mutation, and also requires `hex^0(D)` occurs exactly once in `L` at Q's value interval and
remains absent from every other domain; uppercase and `hex^1..hex^3(D)` remain forbidden.

After `H = SHA256(L)` exists and before the raw-H fill, `G_H` requires `H` is lowercase 64-hex,
nonzero, pairwise distinct from `A`, `HC`, `S`, `D`, and `J`, and absent in every
`hex^0..hex^3(H)` lowercase/uppercase form from exact immutable stable `C`, `L`, `R0`, `W0`, `F0`,
all vector arguments and expected values, vector definitions, and static expected stdout. It emits
nothing and merely permits or discards the fill. During authoritative verification,
`final-positive-raw-h-binding` and `final-identity-binding` reconstruct the same pre-fill domains
from validated `P(R) == R0` and final `L/H`, recompute the complete `G_H` predicate, run each
wrong/omitted-scan and actual-H-insertion negative mutation, and require `hex^0(H)` occurs exactly
once in final `R` as the sole value of `launcher_source_sha256`, remaining absent from every other
domain; uppercase and `hex^1..hex^3(H)` remain forbidden. The 26 rows retain only the exact typed
field reference.

After final `R` exists, its raw SHA-256 may be computed only for external file identity and review.
Every `hex^0..hex^3` lowercase/uppercase form of that value must be absent from `R`, `L`, all
proof/vector/static artifacts, and all source constants. It is never stored in `R` and never
participates in the ratchet. These scans use independently derived field-span whitelists plus
direct byte scans over already stable immutable artifacts; they do not parse a source again,
normalize, rewrite, or authorize it.

## Fail-closed source and projection witnesses

Before any extracted codec vector can run, non-executing preflight must prove all of these negative
families reject. They are submutations of the exact 26-case inventory, not extra direct or source
cases:

- `Q` assignment absent, duplicated, moved out of module scope, aliased, rebound, deleted,
  annotated, concatenated, escaped, single-quoted, wrong-width, uppercase, nonhex, or followed by
  an alternate candidate;
- AST coordinate, raw byte interval, target context, owner, line ending, source LF count, or source
  byte count mismatch;
- zeroing a quote, identifier, spacing, newline, another constant, fewer/more than 64 bytes, or all
  occurrences of the digest text;
- stale/wrong `S`, identity-kind tag, `HC`, Q interval, typed reference, or replacement of the
  reference with either `self` or `J` in a Q-invariant recipe; and, only in a final-binding gate,
  stale/wrong `D`, actual `H`, global reference target, or reference resolution order;
- a second raw `H` occurrence anywhere in record proof/static bytes, a raw `H` copied into any of
  the 26 rows, or a row reference whose resolved field was not already verified against final `L`;
- a proof or static value derived from `D`, `H`, raw record hash, future reviewer output, or the
  unnormalized AST;
- a missing, duplicate, reordered, extra, over-bound, odd-hex, uppercase-hex, noncanonical, or
  mismatched proof scalar/indexed authority field; wrong 26/27 count, row bytes, record bytes, or
  proof SHA-256; or failure to retire raw row hex after canonical acceptance;
- an empty scalar, empty non-`expected_hex` indexed field, empty `args_hex`, malformed
  `static_vector` index or suffix, out-of-range empty expected key, wildcard empty-value matcher,
  empty expected on `return`, nonempty expected on `ValueError`, or expected hex encoded as `-`,
  odd length, uppercase, nonlowercase, or nonhex;
- a source vector at any index other than `192 + i`, wrong case/target/outcome/arguments/expected,
  missing or extra fourth proof authority, `_proof_witness` returning a digest tuple instead of the
  accepted row bytes, or a self-consistent row/vector substitution not equal to the independently
  rebuilt proof;
- index 186 missing or moved; wrong name, target, outcome, expected, descriptor spelling, descriptor
  length, or descriptor digest; `X186` reused by another vector; expanded fixture bytes entering
  authority/B/V/R0; materialization before a dominating validation; materialization inside the
  outcome catcher; descriptor-controlled size/content; size/content precheck; wrapper or target
  adapter; synthetic launcher exception; nonliteral payload construction; payload alias, rewrite,
  second use, or changed call object; generic literal parsing at index 186; compact materialization
  at another index; or a counted `ValueError` without exact extracted-target traceback provenance;
- any proof identity, reference, role, span, fact, result token, observed value, or expected value
  in static stdout, or any stdout shape other than the fixed 229-LF schema;
- a mutation of any non-Q launcher byte after `S`, any proof fact changing across L0-to-L, or any
  failure to recompute Q/S from final L;
- construction artifacts used as authoritative facts, verification artifacts copied from
  construction instead of rebuilt, a missing `W1/F1` equality, an epoch-swapped source, or an extra
  source read/parse in either epoch;
- record projection missing, duplicated, widened, nested, renamed, reordered, applied to S, or
  applied to a second field;
- fixed-point iteration, retry, repair, fallback, caller override, path inference, environment
  input, runtime-discovered span, or hash substitution in an unreviewed order;
- stale controller `A`, controller support digest not retired to `self`, launcher data entering U/A,
  changed scanner semantics after A, or final R failing to reconstruct byte-identical U/A; and
- missing, duplicate, reordered, cross-inventory, harness-rejected, or wrong-target cases in the
  exact 19/26/45 closure.

No mutation is written to disk, reparsed, imported, or executed. Every Q-invariant source mutation
is an in-memory change to one member of a fresh controller/normalized-launcher model pair in each
source-analysis epoch and is evaluated by the same fail-closed source-proof engine as the positive
case. Every final-binding-only mutation is instead applied only to fresh immutable final values in
the authoritative epoch, after the named value exists; it emits no proof row, vector definition, or
static-output byte. The verification Q-invariant results, not the construction results, are the
authoritative row proof; exact equality between them is additionally mandatory, and all final gates
must then succeed before that proof can authorize anything. Unsupported syntax or analysis is
rejection, not an exemption.

## Narrow supersessions

This reconciliation supersedes exactly these earlier clauses and no others:

1. The completion-launcher boundary statement that no combined-record field, expected evidence
   value, or static digest other than source line/byte counts may be computed from launcher bytes
   or AST is narrowed solely to permit `S`, its exact identity-kind tag, Q-invariant normalized
   AST/CFG proof facts, and the proof/static bytes/counts/digests necessarily derived from them.
2. The retained-state amendment's requirement that each of the 26 proof rows contain controller
   and launcher raw source SHA-256 identities is narrowed so the controller field remains raw
   final `HC`, while the launcher fields are exactly the S identity pair plus the typed non-`self`
   reference to the sole globally verified raw `H` field. Raw `H` is not duplicated.
3. The retained-state amendment's 20-path bounded non-input audit-record list is extended only by
   appending this reconciliation and its later review at positions 21 and 22, for cardinality 22.
4. The retained-state amendment's absolute one-read/one-parse wording and this reconciliation's
   rejected earlier no-`L0`-parse wording are superseded only by the two exact epochs above: one
   stable read and parse of final `C` plus final-form `L0` for non-authoritative construction, then
   one independent stable read and parse of final `C` plus final `L` for authoritative
   verification. No other proof-content read or parse is allowed.
5. The earlier implication that source-proof rows were bound only through existing aggregate
   static-definition/expected-output fields is superseded by the exact four scalar and 26 indexed
   combined-authority fields above. This is a preflight/static launcher-authority transport only;
   the prohibition on a new controller CLI option, controller `Authority` field, runtime scanner
   argument, evidence/result field, or runtime anchor remains intact.
6. Any earlier implication that construction executes every negative mutation or completes a
   source parent case is superseded by the exact 26-row partition above. Construction executes only
   Q-invariant row recipes; final-value mutations and final parent-case acceptance occur only in the
   authoritative epoch and contribute no canonical row, vector, record, or stdout bytes.
7. Any earlier implication that concrete candidate-D or candidate-H scans occur only during final
   binding is superseded by the two pure non-authoritative construction-transition guards. Those
   guards are necessary before substitution/fill but cannot accept a case; authoritative final
   gates independently replay them and alone authorize success.
8. Any earlier implication that a row's `source_role` includes construction-transition or
   final-binding ownership is superseded by the Q-invariant-only role calculus. Guard/gate ownership
   and dominance are proved once by the separate global source predicate and never enter a row.
9. Any earlier implication that serialized `B`, a direct-vector row, or a source expected value may
   be built before the complete vector-size gate is superseded by the exact immutable-component,
   arithmetic-length, then single-serialization order above.
10. The inherited universal `ast.literal_eval(arguments_data)` route and blanket test-only fixture
    adapter prohibition are narrowed only for global vector index 186's exact nonrejecting `X186`
    materialization. The exception creates one fixed builtin payload and tuple before the ordinary
    target-call `try`; it is not a target adapter, result decision, precheck, synthetic failure, or
    reusable argument codec. Every other vector and every result-semantic prohibition is unchanged.
11. The raw authority payload's universal nonempty-value rule is narrowed only for exact
    `static_vector.` plus six ASCII digits plus `.expected_hex` keys. Exact key-set/range binding and
    the decoded `(expected == b"") iff (outcome == "ValueError")` gate are mandatory before any
    definition comparison or dispatch. No serializer, hex grammar, cap, other field, or result
    semantic is changed.

The source-normalization permission applies only to the exact Q interval and exact semantic source
identity in this document. It does not authorize normalization of controller, provisioner,
combined-record, evidence, scanner output, runtime input, another launcher constant, metadata,
xattr, ACL, path, newline, Unicode, or review bytes.

## Untouched authority

Everything not explicitly superseded remains unchanged, including:

- the original combined-record one-hole projection, its exact field name, and its sole raw `H`;
- the prohibition on placing `D` or a raw combined-record SHA-256 in the combined record;
- the finite noniterative semantic-record construction and exact final stable-read comparisons;
- the exact six runtime trust anchors and their order, roles, hashes, and runtime scope;
- all accepted authority-object, scanner-argument, stable-path, source, interpreter, tool, metadata,
  xattr, ACL, process, deadline, output-bound, and fail-closed requirements;
- the retained-state 75-row table, three edges, 29 roots, 26 parent entries, ten absences, six
  observations, five amended schemas, six objects, 16 MiB scoped result bound, producer/consumer
  semantics, gates, pre-runtime, completion, state-based commits, and no-replay behavior;
- the exact direct extraction boundary containing only the three pure retained-state codec
  `FunctionDef` nodes;
- the exact 19 direct, 26 non-executing source, and 45 combined case counts and ordered names;
- the rule that a negative must fail inside its named extracted codec or the common source-proof
  engine rather than through a harness precheck;
- the original SHA-256 collision-resistance and trusted-host assumptions; and
- every accepted same-UID, administrator, compromised-host, CPython scheduler/audit/pending-call,
  crash, power-loss, and stronger signed/privileged-bootstrap nonclaim.

The Q-invariant proof is preflight/static evidence only. `S` is not a seventh runtime trust anchor,
not a controller authority digest, not a runtime source hash, not an evidence identity, and not a
completion-gate value. Runtime continues to use only the already accepted raw identities and six
anchors.

## Non-expansion and execution prohibition

This reconciliation adds no provider, VM, pilot, adapter, daemon, datastore, connector, product,
network, authentication, user-data, recovery, build, or runtime scope. It adds no evidence path,
phase result key, retained-state row, carrier edge, gate, completion row, controller CLI option,
runtime launcher mode, scanner argument, controller `Authority` field, generic binding, runtime
source-role capability, process, child, or filesystem mutation. The four scalar and 26 indexed
combined-authority fields are only the exact preflight/static proof transport defined here; the
three source-role tokens only close the already required proof-row `source_role` field.
The sole index-186 compact fixture materializer is the exact closed static-test exception above; it
does not add a general adapter, runtime launcher mode, target wrapper, codec, or production input.

Before this reconciliation and its review are separately accepted, nobody may use it to execute,
import, compile, or dispatch the controller, provisioner, launcher, scanner, static suite, build,
Cargo, compiler, provider, VM, pilot, payload, or runtime. Direct extraction of the three codec
functions and all 45 cases remain unauthorized until the full ratchet is accepted. Creating
`target/debug` is forbidden.

The only work authorized while this document is a candidate is read-only source/document review,
in-memory reasoning, and edits to this document followed by exact Markdown identity checks. The
prospective review record is not created until this document's final exact bytes have three clean
independent reviews.

## Acceptance gates

This reconciliation can become accepted authority only after all of the following occur in order:

1. freeze this exact path and its SHA-256, LF count, byte count, metadata, and final LF;
2. obtain at least three independent reviews of those exact bytes, each with
   `P0=0/P1=0`, no verdict transfer, and explicit review of the Q span, A/HC order, S/H dual
   identity, two source-analysis epochs, 27-row proof transport, 26-entry source-role table,
   26-row Q-invariant/final-binding partition, two non-authoritative transition guards with
   authoritative replay, Q-only row roles plus global gate ownership, one-hole projection, staged
   absence scans, finite DAG, immutable component-first vector sizing, proof-row and complete-vector
   pre-allocation bounds, exact index-186 compact descriptor/materializer and exception provenance,
   exact empty-`expected_hex` key grammar and outcome iff gate, 67,108,864-byte record cap,
   19/26/45 inventories, fixed static stdout, and non-expansion boundary;
3. create the exact prospective review record with the reviewed identity and independent outcomes,
   without a self-SHA or any runtime authority claim;
4. obtain separate exact-SHA acceptance of that review record;
5. append the two accepted paths to the bounded non-input preflight list in the fixed order above;
6. implement the reconciliation in a new controller/launcher/combined-record candidate without
   touching the provisioner or widening scope;
7. perform new exact-source reviews of every changed byte and the complete A/HC/Q/S/R0/D/L/H/R
   ratchet; and only then
8. run the separately authorized combined preflight and, after its acceptance, any later permitted
   static or runtime stage.

A clean review of the current `b11ff52f...` controller or `810051d...` launcher does not satisfy a
review of the later filled sources. Any edit restarts the applicable exact-byte review gate.

## Candidate self-review

This document's authoring review checks only its design surface:

- one raw launcher identity `H` remains in one projected field;
- one normalized identity `S` derives from exactly one AST-cross-checked 64-byte Q interval;
- `S` is acyclic because no S value or S-derived digest is embedded in `L0`;
- `U/A` precede controller `C/HC`, and final `HC` reconstructs identical `U/A` through `self`;
- `R0` may contain S-derived proof/expected bytes but no `D`, raw `H`, or record hash;
- all 26 proof rows use the exact S identity pair and typed non-`self` raw-H reference;
- the original record projection remains the sole H hole;
- the non-authoritative construction and authoritative verification epochs each read and parse
  exactly one `C` and one epoch-specific launcher source, use fresh case copies, and compare their
  complete proof/vector artifacts byte-for-byte;
- the exact 26-row partition keeps every Q-invariant row recipe reproducible in both epochs while
  deferring all H/D/R/final-L mutations and acceptance gates to the authoritative epoch without
  adding any canonical bytes;
- pure `G_D` and `G_H` discard unsafe candidates before substitution/fill but accept no case, while
  authoritative final gates independently reconstruct and replay both predicates and their negative
  mutations before success;
- Q-invariant wrong-reference recipes use only `self` or fixed sentinel `J`; the sole earlier
  handling of actual candidate `H` is nonaccepting `G_H`, every case-engine use of actual `H` is
  final-binding-only, and final verification requires `J != H`;
- the exact four scalar plus 26 indexed authority fields injectively carry the 27-row proof record,
  and all source vectors map one-to-one to authoritative row bytes;
- each proof row satisfies `n_i <= 524,283` and
  `2*n_i + digits(n_i) + 3 <= 1,048,576`, including the exact accept/reject boundary; exact `B`
  is serialized only after all 200 immutable non-source components, including the 19 direct rows,
  are frozen and the complete 226-row `V_len` passes its cap; the exact `K` formula then bounds `R0`
  at 67,108,864 bytes before any nested hex allocation;
- global vector index 186 alone carries exact 47-byte `X186`, contributes `b_186=334` to `B_len`,
  materializes literal `b"x" * 1_048_577` only after every authority/vector/proof predecessor, and
  can count only an exact `ValueError` originating inside the unchanged extracted parser;
- only keys composed byte-for-byte as `static_vector.`, six ASCII decimal digits, and
  `.expected_hex` may have zero raw value bytes; decoded expected bytes are empty exactly for
  `ValueError` vectors and nonempty exactly for `return` vectors, with the iff check preceding
  definitions, materialization, dispatch, and success;
- the exact 26-entry role table uses only `controller-source`, `launcher-source`, and
  `cross-source`, independently derived and checked solely from each row's Q-invariant recipe, while
  guard/gate ownership and dominance are proved by a separate non-row global predicate;
- static stdout carries no literal source identity, typed reference, or proof fact; only its fixed
  vector-definition digest field and final digest depend transitively on `S`;
- `D` and `H` literal/uppercase/encoded absence scans occur only after their values exist;
- the exact 19 direct, 26 source, and 45 combined inventories remain disjoint and exhaustive;
- the bounded audit list changes only from 20 to 22 with two preflight-only non-input paths;
- the six runtime anchors and all carrier/runtime semantics remain unchanged; and
- no implementation or execution is authorized by these candidate bytes.

Any independent reviewer finding against one of those statements is at least P1 until repaired and
the changed exact bytes are reviewed again.
