# Native strict successor Stage C2B2a — retained-state carrier amendment

Date: 2026-09-08
Status: frozen candidate design amendment; not an accepted implementation or execution record

## Purpose and controlling authority

This amendment repairs one proven cross-process gap in the accepted provider-free C2B2a design. Each
ordinary phase is a separate controller process. The current sealed evidence can prove a phase's
result and selected logical trees, but it does not carry the complete terminal state of mutable
launch-input roots from `lock-derive` into the entry of `graph-metadata`. It also cannot later
reconstruct the historical acquisition-fetch owner-tree claim by reading roots that later phases
legitimately changed.

This amendment is subordinate to, and preserves except for the explicit narrow supersession below,
these exact accepted records:

```text
4a4dec13b99c88694625b52538b246323f69465741300c7cb9b8394cbda83e02
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_FROZEN_2026-09-07.md
34ded5a82bba22ea741b8e896b15779a90893d858ed2826522b7a2b5df7d27d6
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_FROZEN_2026-09-07.md
8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
6180d5ffccc6f7ecfa26832c2c03d34538f74e9abb60bcaaf3d00621306b70c2
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CARGO_MARKER_IDENTITY_CARRIER_FROZEN_2026-09-07.md
```

The base same-UID trust assumption and every accepted nonclaim remain exact. This unprivileged
carrier does not defend against a malicious process running as the invoking UID, an administrator,
swap-and-restore between checked boundaries, or a compromised trusted host. A stronger claim still
requires the separately reviewed signed or privileged bootstrap named by the base freeze.

No statement in this amendment authorizes controller, provisioner, launcher, build, provider, VM,
pilot, adapter, daemon, datastore, connector, network, authentication, or user-data execution.

## Prospective exact audit-record paths

If this amendment is accepted, its final amendment and review records have exactly these stable paths:

```text
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_FROZEN_2026-09-08.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_RETAINED_STATE_CARRIER_REVIEW_2026-09-08.md
```

Those two paths are appended only to the base's bounded non-input audit-record list. They are
preflight-only design/review records: a non-executing combined implementation-preflight and exact
source review may open and rehash them. No controller child, Cargo process, compiler, build script,
policy probe, gate predicate, completion runtime scanner, launcher runtime, provider, VM, pilot,
adapter, daemon, datastore, connector, or payload/runtime process may read them or derive authority
from them. They are not added to the already exact six-record scanner/launcher runtime trust-anchor
set and cannot satisfy any build, evidence, semantic-gate, completion, or runtime predicate.

This is the only change to that bounded list. Its accepted cumulative cardinality becomes exactly
20 paths: the already accepted first 18 paths remain byte-for-byte and position-for-position
unchanged, followed by the exact final amendment path above and then the exact review-record path
above. No earlier path is removed, reordered, renamed, substituted, or counted again. The two new
paths have only the preflight-only role in the preceding paragraph; their append does not expand
any runtime input or runtime trust-anchor list.

The exact `_FROZEN_` path above is the sole candidate promotion target. Only its final path,
SHA-256, LF count, and byte count may receive the three independent clean design votes. No
`_DRAFT_`-path hash or vote transfers. The exact review-record path above is created only after
those outcomes are fixed; its own final bytes receive a separate exact-SHA review and acceptance
before combined preflight may bind either record.

## Proven gap

The accepted phase schedule requires `lock-derive` and `graph-metadata` to run as distinct
controller invocations. The lock-derive process can retain its final `home`, `cargo-home`,
`target`, and prior `tmp` inventory only in memory. The next process currently constructs a new
baseline from live paths. That proves only the graph-metadata entry state, not equality to the
successful lock-derive terminal state.

The existing content-tree leaves do not close the gap:

- their frozen kind set does not include the complete mutable owner roots;
- their rows omit the full Darwin identity, xattr, ACL, and named-absence state required here; and
- adding a new tree kind would change every invocation and evidence grammar.

The Darwin xattr sidecars do not close it because they serialize xattr state, not complete
device/inode, directory inventory, regular-file content, ACL, or named-absence state. The Cargo
marker carrier does not close it because it deliberately carries only four marker/tag stable
identities and explicitly does not require equality to an earlier changing descendant set.

Deleting or resetting a live tree would replace the state that must be proved and would turn a
continuity claim into a fresh self-baseline. A live reset is therefore terminal, not a repair.

Acquisition fetch has the related historical problem. Its sealed `owner_only_generated_trees`
array was produced from exact live walks, but later authorized phases add entries under the
acquisition `tmp` tree. A later replay cannot compare the historical four-row claim to the then-live
tree. The producer must persist a predicate over the state it actually observed, and the immediate
consumer must prove equality before making its first change.

## Minimal carrier decision

No new evidence pathname is required. The smallest faithful carrier is one exact
`observations.retained_state` predicate inside the already create-new, fsynced, reopened,
canonically parsed, and sealed phase `result.json`.

The predicate persists, for every closed root, exact object and byte counts plus the row count,
byte count, and SHA-256 of a complete canonical root stream. It also persists the row count, byte
count, and SHA-256 of the complete combined stream. The producer computes those values from one
descriptor-relative observation. The equality consumer independently recomputes the same complete
streams from current held descriptors before any mutation and requires every persisted scalar and
digest to match.

That is sufficient under the same SHA-256 collision-resistance assumption already used by every
C2B2a content, sidecar, graph, result, and completion identity. The cross-process requirement is
exact equality at two reviewed boundaries; it is not a requirement that a later human can recover
every historical row without the producer state. Persisting the full stream in a new leaf would
add a pathname, completion object, publication operation, and parser without strengthening that
boundary equality under the accepted hash assumption.

If a later accepted requirement demands independent row-level reconstruction of a historical
state after an authorized mutation, rather than equality at the immediate handoff plus an
authenticated historical predicate, this result-only design is insufficient. The smallest such
successor would be one new retained-state TSV leaf plus its digest leaf in each applicable producer
phase. That is not the present requirement and is not authorized by this amendment.

## Narrow supersession

This amendment, if and only if it is later accepted, supersedes the following accepted statements
and no others:

1. Base and implementation-preflight clauses that close the per-phase observation key set, solely
   to add one `retained_state` object to `acquisition-fetch`, `lock-derive`, and
   `graph-metadata`, and one `retained_state_acceptance` object to `acquisition-stage`,
   `graph-metadata`, and `archive-seal`.
2. Darwin-addendum and marker-carrier clauses saying ordinary result schemas are unchanged, solely
   for exactly those six observation additions across five distinct phase result schemas: three
   producer `retained_state` objects and three consumer `retained_state_acceptance` objects, with
   both additions in `graph-metadata`. `darwin_metadata_checks` remains the final observation key
   and remains bijective only with Darwin wrapper check headers.
3. The ordinary phase-completion order, solely to move the already-required sealing of the
   producer's substantive phase temp directory from after result publication to immediately after
   its final substantive child and before retained-state capture. The policy-probe temp preserves
   its accepted immediate post-probe same-inode `0700` to `0500` seal; producer finalization only
   revalidates that already sealed directory. No child, pathname, evidence leaf, mode, or additional
   metadata transition is introduced.
4. The ordinary completion chronology for exactly the three producer phases `acquisition-fetch`,
   `lock-derive`, and dual-role `graph-metadata`, solely to make
   the successful final held-descriptor phase-directory `0700` to `0500` `fchmod` the state-based
   completion commit, after all final checks and result publication, with exact `os._exit(0)` as
   the intended next source statement as specified below. For exactly those producers, this
   explicitly supersedes any
   implication that an independently observed exact zero worker-exit status is necessary to make
   an already visible, canonical `0500` commit authoritative. Consumer-only `acquisition-stage`
   and `archive-seal` retain the base finalization and exact-success requirements.
5. The bounded non-input audit-record list, solely to append the exact final amendment and review
   paths declared above after its unchanged first 18 entries, yielding exactly 20 entries. Their
   role is exclusively non-executing combined preflight and exact source review; the exact six
   runtime trust anchors and every child, gate, scanner, launcher-runtime, and payload input remain
   unchanged.
6. The closed controller CLI, `Authority`, and top-level `_authority_object` grammars, solely to add
   the one dedicated `--retained-state-table-binding` option, immutable
   `Authority.retained_state_table_binding` field, and exactly positioned
   `retained_state_table_binding` object key defined below. The generic preflight-binding option,
   name set, observation, and every other CLI, field, and object key remain unchanged.
7. Pre-runtime semantic replay, solely to consume the three closed producer predicates and three
   closed consumer acceptances as described below.

The top-level result schema string remains exactly `c2b2a-phase-result-v1`. Its top-level key order
remains exactly:

```text
schema phase valid failures observations
```

Every pre-existing observation keeps its accepted relative position. In a phase with only one new
key, that key occurs immediately before final `darwin_metadata_checks`. In `graph-metadata`, the
new suffix is exactly `retained_state_acceptance`, `retained_state`,
`darwin_metadata_checks`. `retained_state` is absent outside the three producers;
`retained_state_acceptance` is absent outside the three equality consumers; both are absent from
every policy-probe result.
The implementation-preflight parser must use this declared semantic order; it must not also demand
raw-UTF-8 lexical sorting of observation names, which is incompatible with the already accepted
final position of `darwin_metadata_checks`.

These are exactly six additions across exactly five phase result schemas. The semantic suffixes
above are the sole observation ordering authority for those five schemas and explicitly supersede
any generic raw-UTF-8 observation-name sorting rule. The complete canonical `result.json`, from
its opening `{` through its single terminal LF, is at most exactly 16,777,216 bytes in exactly
those five amended schemas: `acquisition-fetch`, `acquisition-stage`, `lock-derive`,
`graph-metadata`, and `archive-seal`. The limit applies before allocation, read, parse,
construction, or acceptance, and truncation is terminal. This amendment adds or changes no
whole-result ceiling for any other ordinary phase, `policy-probe`, or `pre-runtime-finalize`.

No evidence basename changes. In particular there is no retained-state TSV or digest leaf, the
Darwin wrapper and digest grammars do not change, the Cargo marker rows do not change, and the
completion TSV receives no new row kind.

## Exact carrier edges

Let `A` be the exact preflight-bound acquisition root and `L` be exactly `A/lock-root`. These are
substitutions of authenticated authority values, not environment values, caller assertions, path
prefixes, or discovery results.

Exactly these three carrier rows exist, in this order in combined implementation preflight:

| Edge | Producer | Producer boundary | Equality consumer | Consumer boundary | Later semantic use |
| --- | --- | --- | --- | --- | --- |
| `acquisition-fetch-to-acquisition-stage-v1` | `acquisition-fetch` | `post-substantive-temp-seal-pre-carrier` | `acquisition-stage` | `phase-entry-pre-mutation` | gate 04 replays the sealed historical predicate; it does not compare later-mutated acquisition `tmp` to history |
| `lock-derive-to-graph-metadata-v1` | `lock-derive` | `post-substantive-temp-seal-pre-carrier` | `graph-metadata` | `phase-entry-pre-mutation` | graph-metadata success proves that the mandatory entry equality dominated its probe and child |
| `graph-metadata-to-archive-seal-v1` | `graph-metadata` | `post-substantive-temp-seal-pre-carrier` | `archive-seal` | `phase-entry-pre-mutation` | gates 09 and 10 later consume the producer predicate and archive-seal acceptance together with accepted metadata and graph evidence |

No generic predecessor, “latest result,” phase prefix, regular expression, basename-only match, or
caller-selected producer is legal. A fourth row requires another reviewed amendment.

## Exact retained-state authority binding

One canonical binding, and no family of caller-supplied table options, authenticates the complete
retained-state applicability table. Its binding name is exactly `retained-state-table`. Its
canonical stream is UTF-8 without BOM, uses only LF terminators, and is constructed by concatenating
these rows in order:

1. the literal header `schema=c2b2a-retained-state-table-v1` plus LF;
2. three edge rows in the exact edge-table order above;
3. 29 root rows: the seven acquisition rows, then the eleven lock rows for edge 1, then the same
   eleven path/class positions for edge 2, each in its exact root-table order below;
4. 26 parent-entry rows: the eight acquisition direct names, then the nine lock direct names for
   each lock edge, in the exact raw-UTF-8 name order below;
5. ten absence rows: the six acquisition absences followed by two absences for each lock edge, in
   their listed order; and
6. six observation rows in this order: acquisition-fetch `retained_state`, acquisition-stage
   `retained_state_acceptance`, lock-derive `retained_state`, graph-metadata
   `retained_state_acceptance`, graph-metadata `retained_state`, archive-seal
   `retained_state_acceptance`.

Every row after the header is one `canonical_json_line` object with exactly these ordered keys for
its row kind:

```text
edge:        kind sequence edge producer_phase producer_boundary consumer_phase consumer_boundary
root:        kind edge_sequence sequence class path scope
entry:       kind edge_sequence root_sequence sequence entry_type name disposition
absence:     kind edge_sequence root_sequence sequence scope parent_relative name
observation: kind sequence phase name value_type suffix_sequence suffix_length
```

`kind` is exactly the lowercase row-kind token. All sequence values are non-boolean u64 with these
only reset domains and exact values:

- `edge.sequence` is global across the table and is exactly `0,1,2` in the edge order above; it
  never resets.
- `root.sequence` resets for each edge: edge 0 is exactly `0..6`, edge 1 is exactly `0..10`, and
  edge 2 is exactly `0..10`.
- `entry.sequence` resets for each `(edge_sequence,root_sequence)` pair. Entries exist only for
  root 0: edge 0/root 0 is exactly `0..7`, edge 1/root 0 is exactly `0..8`, and edge 2/root 0 is
  exactly `0..8`.
- `absence.sequence` resets for each `(edge_sequence,root_sequence)` pair. The exact nonempty
  sequences are edge 0/root 0 `0..3`, edge 0/root 3 `0`, edge 0/root 4 `0`, edge 1/root 5 `0`,
  edge 1/root 6 `0`, edge 2/root 5 `0`, and edge 2/root 6 `0`; no other pair has an absence row.
- `observation.sequence` is global across the table and is exactly `0..5` in the six-observation
  order above; it never resets. The corresponding `(suffix_sequence,suffix_length)` pairs are
  exactly `(0,1),(0,1),(0,1),(0,2),(1,2),(0,1)`.

Each root `path` is the exact expanded absolute authority path; `scope` is `parent-named` only for
root zero and `recursive` otherwise. Every entry has `entry_type=directory`; `disposition` is its
exact `recursive-root:<sequence>`,
`excluded-self-evidence`, or `outside-edge` value from the direct-name tables. Every absence has
`parent_relative=.` and exact scope `direct-child` or `recursive-basename`. Every observation has
`value_type=object`. No prose, validator name, runtime digest, result value, or unlisted row is
encoded.

The stream therefore has exactly 75 LF-terminated rows:
`1 + 3 + 29 + 26 + 10 + 6 = 75`. Its bound byte count is the exact length
of the complete concatenation and its bound digest is SHA-256 over exactly those bytes. The parser
requires `rows=75`, a positive non-boolean u64 byte count, a lowercase 64-hex digest, the exact
header, exact row schemas/order/cardinality, byte-identical canonical re-encoding, and no prefix,
suffix, duplicate, omission, or alternate escape. The stream itself is at most 1,048,576 bytes.

The controller CLI adds exactly one dedicated option, with exact arity three:

```text
--retained-state-table-binding 75 <exact-bytes> <exact-sha256>
```

It occurs exactly once, immediately after the final existing `--preflight-binding
darwin-completion-audit-applicability ...` row and before the first `--phase-runtime-literal` row;
another position, repeated option, missing option, wrong arity, or generic preflight-binding row is
terminal. The exact Authority field is `Authority.retained_state_table_binding`, one immutable
`FrozenBinding` whose `name` is exactly `retained-state-table` and whose remaining fields are the
accepted `rows`, `bytes`, and `sha256` above.

`PREFLIGHT_BINDING_NAMES`, `_preflight_binding_observation`, and every pre-existing generic
`preflight_bindings` result observation remain unchanged and do not acquire this binding. In the
top-level `_authority_object`, the new key `retained_state_table_binding` occurs immediately after
the existing `preflight_bindings` key and before `phase_runtime_literals`; its value has exactly
ordered keys `name`, `rows`, `bytes`, `sha256`. No other `_authority_object` key moves.

After parsing all authority paths, the controller reconstructs the complete table from the finite
source-closed edge/root/entry/absence/observation tables, canonicalizes it, and requires its LF
count, byte length, and digest equal that sole binding before any phase effect. Combined preflight
and the launcher independently derive the same bytes and bind the same triple; comparing a label or
trusting the passed digest without reconstruction is terminal.

The canonical retained-state table never contains `PREFLIGHT_AUTHORITY_SHA256` or any
`authority_sha256` field. Its digest is included once in `_authority_object` through the dedicated
field; only then is `PREFLIGHT_AUTHORITY_SHA256` computed over that complete authority object. This
one-way inclusion has no self-reference or hash cycle.

## Exact acquisition root table

For `acquisition-fetch-to-acquisition-stage-v1`, root count is exactly seven. Root order is
semantic and is not sorted at runtime:

| Sequence | Class | Exact path | Scope and required state |
| ---: | --- | --- | --- |
| 0 | `acquisition-parent-named` | `A` | parent-only; exact root metadata and exact direct-name inventory; mode `0700`, UID 502, GID 20, `provenance-fresh`, no ACL |
| 1 | `seed-payload-sealed` | `A/seed-payload` | recursive; exact two-leaf sealed seed; root `0500`, leaves `0400`, mtime zero, source-closed seed bytes |
| 2 | `acquisition-owner-mutable` | `A/home` | recursive; owner-only mutable generated tree |
| 3 | `acquisition-owner-mutable` | `A/cargo-home` | recursive; owner-only mutable generated tree and the exact cache constraint below |
| 4 | `acquisition-owner-mutable` | `A/target` | recursive; owner-only mutable generated tree and recursive `.rustc_info.json` absence |
| 5 | `acquisition-temp-terminal` | `A/tmp` | recursive; root `0700`; the fetch and fetch-policy-probe child directories are sealed `0500` before capture |
| 6 | `acquisition-canary-terminal` | `A/policy-canaries` | recursive; root `0700`; the fetch canary subtree is in its accepted terminal sealed state |

The sequence-0 direct-name rows are exactly these raw-UTF-8-sorted names:

```text
cargo-home evidence home lock-root policy-canaries seed-payload target tmp
```

The `evidence` entry is marked `excluded-self-evidence`; only its direct name and directory type are
in the parent inventory. No evidence descendant or evidence-directory mode is part of this
carrier. The `lock-root` entry is marked `outside-edge`; its descendants are not silently included.
Every other named entry maps bijectively to the recursive root sequence shown above.

The acquisition absence rows are exactly:

```text
root 0 direct-child archive-staging
root 0 direct-child host-validation
root 0 direct-child manifest-authority
root 0 direct-child sealed-archives
root 3 direct-child git
root 4 recursive-basename .rustc_info.json
```

`A/cargo-home/registry/cache` must contain exactly one source directory. That held, validated source
directory and all of its regular single-link `.crate` descendants are included in root 3. Its
absolute path must equal the existing `cache_directory` result observation. The root 2 through 5
summary counts must equal, in exact path order `home`, `cargo-home`, `target`, `tmp`, the existing
`owner_only_generated_trees` rows' `directories`, `regular_files`, and `bytes` fields. The existing
historical array gains no key and is not recomputed from a later-mutated live `tmp`.

## Exact lock-root tables

Both lock-root edges use the same exact eleven root paths and order. They carry different observed
bytes because graph metadata may make its accepted changes to `home`, `cargo-home`, `target`, and
its own temp subtree.

| Sequence | Class | Exact path | Scope and required state |
| ---: | --- | --- | --- |
| 0 | `lock-parent-named` | `L` | parent-only; exact root metadata and exact direct-name inventory; mode `0700`, UID 502, GID 20, `provenance-fresh`, no ACL |
| 1 | `lock-config-sealed` | `L/.cargo` | recursive; exactly `config.toml`, root `0555`, file `0444`, mtime zero, bytes equal exact root configuration authority |
| 2 | `lock-payload-sealed` | `L/payload` | recursive; exact sealed payload; final `Cargo.lock` is `0444`, mtime zero, and matches lock/manifest authority |
| 3 | `lock-source-sealed` | `L/cargo-source` | recursive; exact authenticated postpatch source tree in its sealed modes |
| 4 | `lock-owner-mutable` | `L/home` | recursive; owner-only generated tree |
| 5 | `lock-owner-mutable` | `L/cargo-home` | recursive; owner-only generated tree |
| 6 | `lock-owner-mutable` | `L/target` | recursive; owner-only generated tree |
| 7 | `lock-temp-terminal` | `L/tmp` | recursive; root `0700`; every completed lock-phase temp and policy-probe temp child is sealed `0500` before capture |
| 8 | `lock-canary-terminal` | `L/policy-canaries` | recursive; root `0700`; every completed lock-phase canary is in its accepted terminal sealed state |
| 9 | `archive-staging-sealed` | `A/archive-staging` | recursive; exact manifest-covered sealed staging tree |
| 10 | `manifest-authority-sealed` | `A/manifest-authority` | recursive; exactly the accepted detached `archive-manifest-v1.tsv` authority |

The sequence-0 direct-name rows are exactly these raw-UTF-8-sorted names:

```text
.cargo cargo-home cargo-source evidence home payload policy-canaries target tmp
```

The `evidence` entry is marked `excluded-self-evidence`; only its direct name and directory type are
included. It is separately governed by the accepted evidence protocol. Every other name maps
bijectively to the recursive sequence above. Roots 9 and 10 are opened through the exact held `A`
authority chain; their presence does not turn `A` into a general traversal root.

The lock-root absence rows are exactly:

```text
root 5 direct-child git
root 6 recursive-basename .rustc_info.json
```

For the lock-derive carrier, roots 4 through 8 are the terminal state after the lock-derive probe,
Cargo metadata child, exact lock seal, canary seal, and temp seals. For the graph-metadata carrier,
they are a new terminal snapshot after its probe and `--locked` Cargo metadata child. The second
carrier is not required to equal the first: graph metadata has exact authorized mutable-root
deltas. Within graph metadata, the retained lock-derive entry snapshot, every pre/post launch
snapshot, and the graph-metadata terminal snapshot form one explicit continuity chain.

## Closed object classes

Every root row names one immutable source-closed traversal capability before its first stat or
open. Every descendant capability is derived before its stat/open from its parent's retained
capability and the exact descriptor-relative entry name observed in the stable inventory. No path
prefix, basename, current xattr, caller profile string, fallback, or post-open observation may
select an object class or xattr profile.

For every recursive root:

- the root is a directory;
- every descendant is either a no-follow directory or a link-count-one regular file;
- symlinks, hard links, shared `(device,inode)` pairs, sockets, devices, FIFOs, and unknown types
  are terminal;
- generated objects have UID 502, GID 20, no set-ID/sticky bits, no group/other access while
  mutable, and the exact already accepted class-specific sealed modes where sealed;
- each object has the exact per-path profile selected by the closed capability table, including
  exact Cargo-marker/tag exceptions; extra, missing, reordered, or changed xattrs are terminal;
- every generated object has empty portable and Darwin-native extended ACL bytes; and
- every class-specific content, inventory, cache, manifest, payload, source, marker, and mode
  predicate from the accepted base and Darwin/marker amendments remains independently required.

The carrier records exact observed metadata; it does not convert an observed value into permission.
A value must first pass the applicable class predicate. Consumer equality is in addition to, not a
substitute for, current class, xattr, ACL, content-tree, marker, mount, and path-authority checks.

The parent-only roots are deliberately nonrecursive. They serialize the root object and the exact
direct names, types, and scopes listed above. The excluded evidence child prevents result
self-inclusion. No other uncarried present name is allowed.

## Descriptor-relative observation

Each root is reached from its already held, exact, preflight-bound parent/root chain. The observer
uses no-follow descriptor-relative stat/open for every component and object. It never reconstructs
authority from an absolute path after opening.

For each directory, it:

1. validates the held descriptor's exact class, APFS mount, UID/GID, mode, flags, xattrs, and ACL;
2. captures the raw-UTF-8-sorted entry-name tuple and rejects duplicates or invalid names;
3. preselects the exact child capability before `fstatat`/`openat`;
4. compares the no-follow directory entry identity to the opened descriptor;
5. recursively observes the child, hashing each regular file from the held descriptor;
6. revalidates each child after hashing;
7. recaptures and byte-compares the directory identity, xattrs, ACL, and complete name tuple; and
8. reopens the root name through its held parent and compares the complete root identity before
   accepting the stream.

An absence row is checked twice around the applicable parent inventory. `direct-child` means two
descriptor-relative `fstatat(..., no-follow)` observations returning absence for that exact name.
`recursive-basename` means a complete traversal with a zero occurrence count for that exact raw
basename, bracketed by the same directory stability checks. An error other than absence is
terminal. Pathname `exists`, `lexists`, globbing, or a caller-supplied negative result is not
authority.

Regular-file content is read to EOF from the held no-follow descriptor in bounded chunks. The
observer compares pre/post device, inode, type, mode, link count, UID, GID, size, mtime, ctime,
flags, birthtime, xattrs, and ACL, and requires the read byte count to equal `st_size`. It does not
serialize atime, block allocation, filesystem-private accounting, or a claimed syscall history.

## Canonical root stream

Every stream is UTF-8 without BOM and ends in exactly one LF. Tabs are field delimiters. Arbitrary
path, xattr, ACL, and content bytes never appear raw: paths/names are strict UTF-8 and are encoded as
lowercase hex plus their byte count; other arbitrary bytes are lowercase hex plus byte count and
SHA-256. Zero bytes are encoded with count `0`, the SHA-256 of empty bytes
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, and hex token `-`.
There is no Unicode normalization, case folding, locale collation, or alternate escape form.

Unsigned integers use base ten with no leading zero except zero and are exact non-boolean u64.
Modes use exactly four lowercase octal digits. SHA-256 uses exactly 64 lowercase hex digits. A
relative path is `.` for the root or a nonempty slash-separated descendant with no empty, `.`,
`..`, backslash, NUL, C0, or DEL component. Its UTF-8 length is at most 4,096 bytes and it has at
most 128 components. Object order is root first, then strict raw-relative-UTF-8 byte order. Xattrs
immediately follow their owning object in strict raw-name-byte order. All
object-with-immediate-xattrs groups come first. For a parent-only root, entry rows then follow in
strict raw-name byte order; recursive roots have no entry rows. Absence rows then follow in their
already-fixed source-closed table order. No row kind may be interleaved with, moved before, or moved
after another body section.

Each root stream begins with exactly these twelve LF-terminated lines:

```text
schema=c2b2a-retained-state-root-v1
edge=<exact-edge>
root-sequence=<u64>
root-class=<exact-class>
root-path-bytes=<u64>
root-path-hex=<lowercase-hex>
objects=<u64>
directories=<u64>
regular-files=<u64>
content-bytes=<u64>
xattrs=<u64>
absences=<u64>
```

The header is followed by only these row forms:

```text
object<TAB><root-sequence><TAB><object-sequence><TAB><directory|regular><TAB><relative-bytes><TAB><relative-hex><TAB><device><TAB><inode><TAB><uid><TAB><gid><TAB><mode><TAB><nlink><TAB><size><TAB><mtime-ns><TAB><ctime-ns><TAB><flags><TAB><birthtime-ns><TAB><content-bytes><TAB><content-sha256><TAB><profile><TAB><xattr-count><TAB><acl-portable-bytes><TAB><acl-portable-sha256><TAB><acl-portable-hex><TAB><acl-native-bytes><TAB><acl-native-sha256><TAB><acl-native-hex>
xattr<TAB><root-sequence><TAB><object-sequence><TAB><xattr-sequence><TAB><name-bytes><TAB><name-hex><TAB><value-bytes><TAB><value-sha256><TAB><value-hex>
entry<TAB><root-sequence><TAB><entry-sequence><TAB><directory|regular><TAB><name-bytes><TAB><name-hex><TAB><recursive-root:<sequence>|excluded-self-evidence|outside-edge>
absent-child<TAB><root-sequence><TAB><absence-sequence><TAB><parent-object-sequence><TAB><name-bytes><TAB><name-hex>
absent-basename<TAB><root-sequence><TAB><absence-sequence><TAB><name-bytes><TAB><name-hex><TAB>0
```

For a directory, `content-bytes` is zero and `content-sha256` is the empty-byte digest. For a
regular file they are the exact held-descriptor byte count and digest. `profile` is the exact
closed profile name selected before open. The ACL triples contain the complete portable and native
external representations, not just a digest. Empty ACL hex is `-`. An `entry` row exists only for
a parent-only root; recursive roots use object rows for every present object. `object-sequence`
begins at zero and is contiguous within one root stream. For every
`(root-sequence, object-sequence)` independently, `xattr-sequence` resets to zero; when the owning
object's `xattr-count` is `n`, its xattr rows use exactly `0..n-1` in strict raw xattr-name byte
order, and `n = 0` admits no xattr row. An xattr sequence never continues from a prior object and
is never a root-global counter. `entry-sequence` and `absence-sequence` each begin at zero and are
contiguous within their already-fixed per-root scopes.

The body order after the twelve header lines is exactly:

1. every `object` row in the object order above, with all and only that object's `xattr` rows
   immediately after it, independently numbered from zero through `xattr-count - 1` in strict raw
   xattr-name byte order;
2. for a parent-only root, every `entry` row in strict raw-name byte order, or zero entry rows for a
   recursive root; and
3. every `absent-child` or `absent-basename` row in the exact already-fixed absence-table order.

The source-closed root class fixes whether section 2 exists and its exact cardinality; no header
field or runtime observation selects it. The encoder emits only that concatenation, initializes
`xattr-sequence` to zero for each object, and increments it exactly once for each emitted xattr row.
The parser requires the same section boundaries, entry cardinality/name order, immediate xattr ownership,
per-object xattr-sequence reset, exact `0..xattr-count-1` numbering, raw xattr-name byte order, and
absence order before accepting or re-encoding. An entry before completion of all object/xattr
groups, between an object and its xattrs, after the first absence, or in any alternate name order,
and any root-global or continued xattr numbering is noncanonical and terminal. Header, root-summary,
and combined-stream `xattrs` values are aggregate cardinalities only; they do not create or alter an
xattr-sequence domain. Root `rows`, `bytes`, and SHA-256, every root-summary triple, and every
combined-stream count and digest are computed over this one ordering and numbering only; fixtures
contain the exact ordered bytes rather than a set or separately sorted row families.

The root stream's `rows` is its LF count, including the twelve header lines. Its `bytes` is the
complete encoded byte length. Its digest is over exactly those bytes. Header counts and summary
counts are independently recomputed and must agree with the parsed body.

## Canonical combined stream

The combined stream begins with exactly these nine lines:

```text
schema=c2b2a-retained-state-combined-v1
edge=<exact-edge>
producer-phase=<exact-producer>
producer-boundary=<exact-producer-boundary>
consumer-phase=<exact-equality-consumer>
consumer-boundary=<exact-consumer-boundary>
authority-sha256=<exact-nonzero-combined-preflight-authority-sha256>
retained-state-table-sha256=<exact-retained-state-table-binding-sha256>
roots=<exact-root-count>
```

For each root in source-closed sequence order, the stream then contains one summary line followed
immediately by the complete root stream bytes:

```text
root-stream<TAB><sequence><TAB><class><TAB><objects><TAB><directories><TAB><regular-files><TAB><content-bytes><TAB><xattrs><TAB><absences><TAB><rows><TAB><bytes><TAB><sha256>
<complete-root-stream>
```

The combined row count includes its nine header lines, every root-summary line, and every root
stream row. Combined bytes and SHA-256 cover the complete concatenation. The explicit root-stream
byte count makes the framing unique; a parser consumes exactly that many bytes and still requires
the inner terminal LF and digest. An unconsumed prefix, suffix, partial root, duplicate root,
reordered root, or count disagreement is terminal.

The canonicalizer uses checked u64 addition for every counter. Per root, directories are at most
1,048,576, regular files at most 4,194,304, and regular-file content bytes at most
536,870,912,000. Each xattr name is 1 through 255 bytes, each value is at most 16,777,216 bytes,
there are at most 1,048,576 xattr rows, and aggregate xattr value bytes are at most
1,073,741,824. Each ACL external representation is at most 1,048,576 bytes. Each root stream is at
most 268,435,456 bytes; the combined stream is at most 536,870,912 bytes. The canonical result JSON
is at most exactly 16,777,216 bytes including its single terminal LF for exactly the five amended
schemas named above. That whole-result ceiling is identical for their producer-only,
consumer-only, and graph-metadata producer-plus-consumer results, and this amendment imposes no
ceiling on any other phase result. A count, multiplication, hex expansion, addition, read,
allocation, or encoded byte beyond a limit is terminal before acceptance; truncation is never
permitted.

## Exact result predicate

`retained_state` has exactly these ordered keys:

```text
schema edge producer_phase producer_boundary consumer_phase consumer_boundary
authority_sha256 retained_state_table_sha256 roots combined_rows combined_bytes combined_sha256
```

Its values are:

```json
{"schema":"c2b2a-retained-state-predicate-v1","edge":"<exact-edge>","producer_phase":"<exact-producer>","producer_boundary":"<exact-boundary>","consumer_phase":"<exact-consumer>","consumer_boundary":"<exact-boundary>","authority_sha256":"<lowercase-64-hex>","retained_state_table_sha256":"<lowercase-64-hex>","roots":[],"combined_rows":0,"combined_bytes":0,"combined_sha256":"<lowercase-64-hex>"}
```

The illustrative zero values above are grammar examples only and cannot satisfy a real edge.
`roots` has exactly seven or eleven elements according to the edge. Each root object has exactly
these ordered keys:

```text
sequence class path objects directories regular_files content_bytes xattrs absences rows bytes sha256
```

`path` byte-equals the expanded authority-table path. Every count is a non-boolean u64. Root order,
class, path, counts, and digest equal the canonical stream. `authority_sha256` is the exact nonzero
`PREFLIGHT_AUTHORITY_SHA256` already required by the controller. Its digest domain is exactly
`sha256(canonical_json_line(_authority_object(authority)))` over the existing canonical runtime
authority object whose schema is `c2b2a-preflight-authority-v1`, after the closed parser has bound
all of its finite tables. In that existing object, and only there, the controller support entry's
source digest is represented by the finite literal sentinel `self` rather than the controller's
recursive file hash. The result copies this already verified digest; neither argv nor the result
may set, reset, or recompute it.

This field is not the raw combined implementation-preflight record hash, launcher semantic-record
hash, scanner-argv hash, controller source hash, amendment/review hash, result hash, carrier hash,
or a self-hash. Non-executing canonical fixture and source-flow witnesses use the exact finite
synthetic expected-authority value
`0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef`; it must occur identically in
the synthetic expected-authority input, combined stream, and predicate. That value is test-only,
does not claim to be the digest of production authority, and cannot reach a production parser or
runtime invocation. A production zero value, the synthetic value, or any value unequal to the
embedded reviewed `PREFLIGHT_AUTHORITY_SHA256` is terminal. Unknown, missing, duplicate,
differently ordered, null, signed, floating, noncanonical escaped, or out-of-range values are
terminal.

`retained_state_table_sha256` is exactly
`Authority.retained_state_table_binding.sha256`. It must equal the independently reconstructed
75-row table digest and the combined-stream header. It is not accepted from the result itself.

The predicate contains no raw full stream, timestamp, random identifier, runtime-selected path,
or claimed human approval. Its summary is accepted only after the full producer stream has been
constructed and parsed back canonically in memory.

## Exact consumer acceptance

Every equality consumer records the successful handoff in its own existing sealed `result.json`.
An implicit source-flow claim is insufficient. `retained_state_acceptance` has exactly these
ordered keys:

```text
schema edge producer_phase producer_result_sha256 consumer_phase consumer_boundary
retained_state_table_sha256 roots
producer_combined_rows producer_combined_bytes producer_combined_sha256
recomputed_combined_rows recomputed_combined_bytes recomputed_combined_sha256 matched
```

Its `schema` is exactly `c2b2a-retained-state-acceptance-v1`; `edge`, phases, and boundary equal the
immutable edge row; `producer_result_sha256` is the digest of the complete accepted canonical
producer `result.json`; `retained_state_table_sha256` equals the current globally bound and
independently reconstructed 75-row table digest; and `matched` is exactly `true`. `roots` has the
edge's exact root count.
Each root acceptance object has exactly these ordered keys:

```text
sequence producer_rows producer_bytes producer_sha256
recomputed_rows recomputed_bytes recomputed_sha256 matched
```

Every scalar comes from the accepted producer predicate or the consumer's independently accepted
entry recomputation. Each per-root and combined producer/recomputed triple must be equal, and every
root `matched` and the outer `matched` must be exactly `true`. The acceptance object is constructed
once at entry from immutable canonical values, retained without mutation through the consumer, and
written only if the phase otherwise succeeds. It carries no path or authority selected by the
producer result; those come solely from the current immutable edge/root table.

The exact result-observation suffixes are therefore:

```text
acquisition-fetch:  retained_state, darwin_metadata_checks
acquisition-stage:  retained_state_acceptance, darwin_metadata_checks
lock-derive:         retained_state, darwin_metadata_checks
graph-metadata:      retained_state_acceptance, retained_state, darwin_metadata_checks
archive-seal:        retained_state_acceptance, darwin_metadata_checks
```

All keys before these suffixes remain in their previously accepted phase-specific order. A
consumer result missing the acceptance, containing a false match, naming another producer, or
hashing different producer-result bytes is terminal and cannot prove the handoff.

The dedicated table binding is present globally in parsed `Authority` for source/table validation,
but its digest is carried into result bytes only inside the three `retained_state` objects and three
`retained_state_acceptance` objects above. These six objects occur in exactly five ordinary phase
schemas. `policy-probe`, `pre-runtime-finalize`, every other phase result, every gate object, and
every generic `preflight_bindings` observation omit the dedicated table digest and both new object
keys. Pre-runtime must authenticate the five sealed amended results as inputs; it does not copy the
table digest into its own result.

## Producer protocol and capture boundary

For each producer, the controller performs this exact order:

1. Authenticate current authority, phase schedule, predecessor marker carrier, and every existing
   input required by the phase.
2. Complete the policy probe and its evidence. Immediately after the probe and before any
   substantive child, seal the policy-probe temp on its held descriptor using the already accepted
   same-inode `0700` to `0500` transition; fsync, reopen no-follow, and revalidate its exact content,
   metadata, xattrs, ACL, path link, and parent inventory. This accepted timing does not move.
3. Complete every substantive child, class-specific result validation, canary seal, and every
   pre-existing evidence leaf other than the Darwin wrapper/digest and `result.json`. Immediately
   after the final substantive child, revalidate the policy-probe temp is still the accepted sealed
   `0500` object, then seal only the substantive phase temp from `0700` to `0500` on its held
   descriptor and perform the same fsync, reopen, metadata/content, path-link, and inventory checks.
4. At boundary `post-substantive-temp-seal-pre-carrier`, derive the exact edge/root/absence table
   from immutable preflight authority and construct every complete root stream and the combined
   stream through held no-follow descriptors.
5. Parse the constructed bytes with the independent canonical parser, require byte-identical
   re-encoding, and derive the exact result predicate only from that accepted parse.
6. Perform the existing final Darwin `pre-carrier` capture, then create, fsync, close, reopen, and
   validate `darwin-metadata-v1.tsv` and `darwin-metadata-v1.sha256` in their accepted order.
7. Construct canonical `result.json` with `retained_state` immediately before the final
   `darwin_metadata_checks` and, for graph metadata, its earlier immutable
   `retained_state_acceptance`, create/fsync/close/reopen it, parse it, and require the accepted
   predicate, acceptance, and Darwin checks to match the in-memory canonical values.
8. Still before completion commit, recompute the complete retained state and require byte-for-byte
   equality to the accepted in-memory root and combined streams. Rehash source, tool, canonical
   runtime authority, and every applicable immutable input; revalidate the result, wrapper,
   digest, complete evidence-leaf ledger and inventory; perform the final marker-carrier comparison;
   perform any already authorized last-writer evidence-parent seal; and complete every other
   fallible trusted check and cleanup. The precommit seal preparation requires the source-closed
   complete leaf inventory and every leaf's exact regular/link-count-one identity, mode, UID/GID,
   xattr profile, ACL, size, and content digest; syncs all leaves and the held directory; reopens the
   still-`0700` phase directory no-follow through the held parent; revalidates the same identity,
   exact `0700` mode, inventory, and leaves; then closes every descriptor except the one held
   no-follow phase-directory commit fd. A last-writer parent sealed here does not commit the phase
   because its child phase directory remains `0700` and is consumer-invalid.
9. Immediately revalidate the held commit fd is that same directory with exact mode `0700`, then
   call exactly `os.fchmod(commit_fd, 0o500)`. Successful return of that syscall and an actually
   visible canonical `0500` directory is, by definition, the transaction's sole authoritative
   phase-success commit. There is no explicit controller branch, handler, `finally` edge, fsync,
   reopen, stat, leaf/ledger check, source/tool/authority validation, descriptor close, result
   publication, marker observation, buffered output, or cleanup after this source statement.
10. The immediately following explicit source statement is exactly `os._exit(0)`. It is the
    intended termination path and emits no diagnostic success line. It is not an evidence fact:
    CPython may execute periodic scheduling, audit, or pending-call machinery after the builtin
    `os.fchmod` call returns and before the next Python call. This amendment makes no claim that
    those interpreter internals are absent or introspectable, and it does not use signal/handler,
    trace/profile, GC, timer, wakeup-fd, thread, or callback normalization as a proof. Worker exit
    status and any postcommit scheduler activity are non-authoritative and cannot satisfy a gate.
    Static source proof requires only that no explicit controller path continues after the commit
    other than this exact intended `_exit` statement.

Any interpreter-internal activity that changes carrier-visible state after `fchmod` invalidates the
carrier when the sequencer, consumer, resume path, pre-runtime replay, or completion parser next
recomputes it. Such activity is never authorized by this amendment and cannot create, preserve, or
repair authority merely because the phase directory is `0500`.

There is no result self-inclusion. Writing/sealing evidence may change only the explicitly excluded
evidence subtree. The existing marker boundary label `post-phase-seal` is retained for grammar
compatibility, but in these three producer implementations its check occurs in step 8 after all
phase-owned non-evidence seals and before the phase-evidence-directory completion commit; there is
no postcommit marker call. Any retained root, parent direct-name inventory, absence, or stream
change during steps 4 through 9 is terminal. A precommit producer failure may leave create-new
precommit bytes, but cannot yield a completed carrier. A kill or nonzero observed worker status
after an exact visible commit is not a producer failure and cannot undo that committed state.

This commit is atomic enough for the accepted threat model without a new leaf. Before the
successful step-9 `fchmod`, the phase directory is `0700`, which every consumer rejects. After
every one of the three producer-worker terminations, regardless of observed exit status, the
sequencer, resume path, and
equality consumer inspect the exact phase path descriptor-relative and no-follow. If the directory
is visibly `0500` and its canonical inventory, leaf identities/content, result/wrapper/digest,
predecessor, and current authority all validate, the phase is committed: it must never be replayed,
repaired, or reclassified as producer failure. A kill after successful `fchmod` and before
`os._exit(0)` is therefore a committed phase. If the path is absent, remains `0700`, or any
canonical check fails, it is terminal and cannot be consumed or replayed. A malformed `0500` state
is rejected rather than using exit status or postcommit activity to repair its authority.
The accepted same-UID/nonmalicious-host assumption closes unauthorized change between commit and
inspection.

Crash and power-loss outcomes are state-based. A later observer accepts only an exact `0500`
commit actually visible through the held-parent capability. An absent or rolled-back `0700` state
is uncommitted; a torn or otherwise invalid `0500` state is terminally corrupt and is neither
consumed nor repaired. This adds neither repair nor recovery: every outcome is final for that run.

## Consumer protocol

At each equality-consumer boundary, before creating the current phase evidence directory, temp
directory, policy-probe temp, canary, durable output, pipe, listener, or child, the controller:

1. resolves the one exact producer result path from the immutable edge table;
2. opens every ancestor, producer phase directory, Darwin wrapper/digest, and `result.json`
   descriptor-relative and no-follow through the exact preflight capability chain;
3. requires the producer directory sealed `0500`, every leaf regular/link-count-one/mode `0600`,
   exact UID/GID/profile/ACL, stable inventory, and canonical wrapper/digest/result correspondence;
4. requires `valid=true`, empty `failures`, exact producer phase, exact current authority including
   the dedicated table binding, unchanged existing `preflight_bindings`, exact marker predecessor,
   the exact retained-state schema/edge/boundaries, and no unknown key;
5. parses and canonicalizes the predicate once, retires the raw JSON aliases, and retains the
   accepted immutable predicate;
6. independently recomputes the complete current root and combined streams from held descriptors;
7. requires every persisted root scalar and every root/combined row count, byte count, and digest
   equal to the independently recomputed complete streams; and
8. revalidates authority and the complete root closure immediately before the first current-phase
   mutation.

No consumer may use current observations to replace a producer digest or create a new baseline.
The accepted predecessor value must dominate every current-phase effect and launch. A mismatch is
terminal before any current-phase artifact exists.

After the equality check, the consumer retains the accepted predicate, complete producer-result
digest, and recomputed identities as immutable values. Its successful `result.json` records the
exact `retained_state_acceptance` object above. Failure to carry that value to the result, or any
route that emits a valid result without it, is terminal.

Consumer-only `acquisition-stage` and `archive-seal` do not use the producer commit protocol or its
exit-status supersession. After their entry equality, they retain the immutable acceptance, may
perform only their existing exact authorized deltas, and use the unchanged base result/evidence
finalization and exact-success rule. They never compare their terminal live roots to the predecessor
carrier: acquisition-stage necessarily creates its authorized staging/manifest state, while
archive-seal performs only its existing archive-seal write set. Acquisition-stage may create only
the exact previously absent `A/archive-staging` and `A/manifest-authority` outputs plus its existing
temp/canary/evidence state. Archive-seal may create and seal only the exact `A/sealed-archives`
output plus its existing temp/canary/evidence state; it reads but cannot mutate the carried lock
roots, archive staging, or manifest authority. Their result validator proves that the entry-derived
acceptance object was carried unchanged; it does not recast a later state as the predecessor.

`graph-metadata` is dual-role. It first consumes lock-derive exactly at entry and retains that
immutable acceptance, then may perform only its graph-metadata-authorized deltas. It never requires
its terminal state to equal lock-derive. After its final child and temp seal, it separately captures
a new graph-metadata terminal `retained_state`, writes both distinct objects into one canonical
result, and uses the special producer commit protocol because it is one of the three producers.

For `graph-metadata`, the lock-derive accepted stream is retained through the process. Every probe
and substantive-child prestate must equal the immediately preceding accepted poststate for every
root without an authorized delta. Only the exact graph-metadata write set may differ. The terminal
graph-metadata carrier is produced from the final post-child state after temp sealing; it is not a
copy of the lock-derive predicate.

## Historical acquisition replay

Gate 04 consumes the canonical acquisition-fetch result, invocation/policy evidence, exact cache
selection, transcript, retained-state predicate, and the canonical sealed acquisition-stage result.
It requires:

- the predicate's root 2 through 5 counts equal the four existing historical owner-tree rows;
- the unique cache source and archive inventory in root 3 equal the existing `cache_directory` and
  accepted cache/lock authority;
- the `git` and `.rustc_info.json` absence rows are present and valid;
- the acquisition-stage result contains its mandatory `retained_state_acceptance` for exactly
  `acquisition-fetch-to-acquisition-stage-v1`, authenticated under the current reconstructed table;
- that acceptance's `producer_result_sha256` equals SHA-256 of the exact accepted canonical
  acquisition-fetch `result.json`, its producer root and combined triples equal the producer's
  `retained_state`, its recomputed triples are pairwise equal, and every match flag is true;
- the acquisition-stage entry comparison that produced that immutable acceptance dominated its
  first mutation, and the exact acceptance was carried unchanged through its base-success
  finalization; and
- every gate-evidence path/digest binding is unchanged.

Gate 04 does not compare the acquisition-fetch `tmp` digest to the later final acquisition `tmp`,
because later scheduled phases legitimately add sealed temp children. Instead it validates the
persisted predicate, its producer source-flow, and its mandatory immediate handoff. Gate 04 cannot
pass from the acquisition-fetch predicate alone or from a source-flow assertion about a consumer;
the exact sealed acquisition-stage acceptance object is a mandatory authenticated input. For any
root that the no-writer schedule proves terminal, a later live equality is an additional check,
not a substitute for the historical predicate.

## Graph-metadata terminal replay

The sealed graph-metadata result must contain both distinct objects: its
`retained_state_acceptance` for `lock-derive-to-graph-metadata-v1` and its new terminal
`retained_state` for `graph-metadata-to-archive-seal-v1`. The acceptance's
`producer_result_sha256` must equal SHA-256 of the exact accepted canonical lock-derive
`result.json`; its table digest, edge, phases, boundaries, root triples, combined triples, and match
flags must cross-bind exactly to that lock-derive producer predicate and the graph entry
recomputation.

At archive-seal `phase-entry-pre-mutation`, the consumer authenticates that complete graph-metadata
result and records the exact successful equality in its own `retained_state_acceptance`. Its
`producer_result_sha256` must equal SHA-256 of those exact canonical graph-metadata result bytes,
and its table digest, edge, phases, boundaries, root triples, combined triples, and match flags must
cross-bind exactly to the graph-metadata terminal predicate and archive-seal entry recomputation.

At `pre-runtime-entry-pre-mutation`, pre-runtime finalization authenticates the sealed lock-derive,
graph-metadata, and archive-seal results, requires both consumer acceptances above, and independently
recomputes the graph-metadata eleven-root state. Since archive seal and no later phase have write
authority to those roots, the complete stream must equal the graph-metadata terminal predicate.
Gates 09 and 10 each consume the graph result's mandatory lock-derive acceptance, graph terminal
predicate, and archive-seal acceptance together with the exact graph-metadata stdout, invocation,
payload/source trees, final lock, root configuration, reconstructed graph v2, and graph digest.
Generic well-hashed result bytes or a self-consistent retained-state digest cannot satisfy either
gate without these cross-relations.

The completion parser later authenticates all five exact sealed amended results and validates all
six nested objects, with no omission or inferred success:

```text
acquisition-fetch retained_state
acquisition-stage retained_state_acceptance
lock-derive retained_state
graph-metadata retained_state_acceptance
graph-metadata retained_state
archive-seal retained_state_acceptance
```

It requires each result's exact phase schema and semantic key order, the current reconstructed table
digest, every exact edge, and each consumer `producer_result_sha256` against the full canonical
bytes of its exact producer result. It also requires the existing pre-runtime evidence bindings.
For acquisition history and the superseded lock-derive state it does not pretend that current
later-mutated roots are historical bytes; it validates the exact persisted predicate and the
authenticated acceptance/source/schedule/consumer-dominance chain. For the terminal graph-metadata
predicate it also requires the pre-runtime final live recomputation. The completion TSV grammar
stays unchanged: the six objects are properties of already listed `result.json` contents, so their
exact bytes are compared through those existing result-content rows rather than emitted as new
rows.

## Exact no-authorized-writer schedule

The implementation-preflight callgraph and policy tables must prove these exact intervals:

- After acquisition-fetch terminal capture and before acquisition-stage entry comparison, only the
  producer's precommit excluded-evidence writes, final trusted checks, sole phase-directory commit,
  intended `os._exit(0)` edge, non-authoritative CPython internal activity, kernel fd teardown,
  state-based commit inspection, and the next
  controller's read-only entry checks may occur. After successful commit there is no authorized or
  explicit controller effect other than the intended `_exit` edge. No writer to any of the seven
  roots or six absences exists.
- After lock-derive terminal capture and before graph-metadata entry comparison, only the same
  precommit excluded-evidence/final-source operations, sole phase-directory commit, immediate
  intended `os._exit(0)`, non-authoritative CPython internal activity, kernel fd teardown, commit
  inspection, and next-process read-only checks may occur.
  No writer to any of the eleven roots or two absences exists.
- After graph-metadata terminal capture and before archive-seal entry comparison, only the excluded
  precommit graph-metadata evidence/final-source operations, sole phase-directory commit,
  intended `os._exit(0)`, non-authoritative CPython internal activity, kernel fd teardown, commit
  inspection, and archive-seal read-only entry checks may occur. After that recorded equality and
  before pre-runtime entry recomputation,
  archive seal,
  all qualification A/B payload/source/build phases, both probe-build phases, `graph-finalize`,
  `source-audit`, both artifact audits, and all six host-validation phases have no controller,
  child, policy, inherited descriptor, or helper write authority to any carried lock-root,
  archive-staging, or manifest-authority object. Their allowed writes are confined to their already
  accepted roots. Archive seal may read but not mutate archive staging or manifest authority;
  graph-finalize and source-audit retain only their accepted read-only evidence access.

The source proof must enumerate every `open`, create, write, chmod, chown, utime, xattr, ACL,
rename, link, unlink, and directory mutation adapter reachable in those intervals and show no
unlisted path capability reaches a carried root. Absence of a sandbox permission alone is not a
controller write-closure proof.

“No writer” and “no producer effect” in this amendment mean no authorized child, policy,
descriptor, helper, or explicit controller mutation. They do not assert that CPython performs no
internal scheduler, audit, or pending-call activity before the intended `_exit`. Any
carrier-visible mutation from such non-authoritative activity makes the exact later recomputation
fail and therefore invalidates the carrier; it can never satisfy a gate or completion check.

This schedule plus exact state-based commits and the explicit same-UID trust assumption closes the
between-process interval. Worker exit status is observed for diagnostics but cannot invalidate an
already canonical visible commit or validate an absent/`0700`/malformed one. This does not claim
continuous kernel monitoring, a retained syscall trace, or resistance to a same-UID
swap-and-restore race.

## Failure, invalidation, and replay rules

Every malformed stream/result, failed open/read/stat/xattr/ACL/hash/fsync/close, changed identity,
unexpected entry, missing or newly present absence, profile mismatch, limit overflow, predecessor
mismatch, incomplete revalidation, or timing/order deviation is terminal.

A failure invalidates the current run. The controller must not:

- catch and downgrade the failure;
- retry against a new observation;
- regenerate, reset, repair, delete, replace, or overwrite a root or evidence leaf;
- select another predecessor or accept an earlier/later result;
- treat a missing predicate as inapplicable;
- accept a digest from argv or a caller;
- continue after creating a partial current-phase path; or
- publish `controller_valid=true`.

A stale result from another run fails exact root identity, authority digest, edge, phase schedule,
marker predecessor, and current-state recomputation. A valid predicate is single-run evidence, not
a reusable cache key or recovery token. Recovery after any partial publication requires a new
reviewed run plan; this amendment adds no recovery mode.

## Implementation-preflight and launcher obligations

Before any implementation, direct codec vector, or non-executing source-mutation witness may run,
combined implementation preflight must freeze and independently review:

1. the exact accepted SHA/line/byte identity of this amendment and its accepted review, while
   binding their exact prospective paths named above only in the bounded non-input audit-record
   list and preserving the existing exact six runtime trust anchors;
2. the exact three edge rows, producer/consumer boundaries, seven/eleven root tables, direct-name
   rows, absence rows, class predicates, xattr-profile selector, and no-writer schedule above;
3. exactly six ordered observation rows across five distinct phase result schemas: one
   `--phase-observation <phase> retained_state object` row for each of the three producers and one
   `--phase-observation <phase> retained_state_acceptance object` row for each of the three
   equality consumers, with both rows in `graph-metadata`, the exact semantic suffix order above,
   and no such row for another phase;
4. the one exact `--retained-state-table-binding 75 <bytes> <sha256>` option, its closed CLI
   position/arity, canonical 75-row table bytes, dedicated Authority and `_authority_object` field,
   unchanged `PREFLIGHT_BINDING_NAMES`/`_preflight_binding_observation`, and complete coverage
   equality across independently derived combined-preflight, launcher, controller, and
   edge/root/direct-name/absence/observation source tables;
5. the three selected pure retained-state-table codec FunctionDefs and their byte-identical
   parse/re-encode properties, plus non-executing exact-source proof of every root/combined-stream,
   predicate, acceptance, authority, and production-adapter encoder/parser/validator callsite and
   bound; no other production definition crosses the extraction boundary;
6. descriptor-relative producer and consumer callsites, with object class selected before open,
   raw observations retired after canonicalization, and only accepted immutable values reaching
   comparison/result effects;
7. exact producer ordering, including the unchanged immediate post-probe seal, the moved
   post-final-child substantive-temp seal, final retained-state recomputation and every other
   fallible trusted check/sync/reopen/cleanup before commit, the held-fd `fchmod` as sole state-based
   commit, exact `os._exit(0)` as the immediately following explicit source statement, no other
   explicit controller continuation, and the narrowed CPython scheduler/audit/pending-call and
   exit-status nonclaims, plus proof that excluded evidence writes cannot affect a carried field;
8. exact consumer dominance before every evidence/temp/canary/create/listener/pipe/child effect,
   exact immutable acceptance carry to the consumer result, and retained post-to-next-pre
   continuity inside graph metadata;
9. exact historical acquisition result-field relations and gate 04 dispatch requiring the sealed
   acquisition-stage acceptance, exact lock/metadata relations and gate 09/10 dispatch requiring
   both the graph-metadata lock-derive acceptance and archive-seal acceptance, and pre-runtime
   authentication of that same chain plus final live equality;
10. consumer authentication of the exact sealed phase directory, complete inventory, leaf modes
    and identities/content, and canonical result/wrapper/digest correspondence; completion parsing
    of all six objects in all five amended sealed results, including every producer-result SHA,
    table digest, and edge cross-binding, with no completion-row or gate-result schema change;
11. the source-closed absence checks and the explicit rejection of path inference, prefix rules,
    caller-selected class/profile, post-open classification, mutable canonical carriers, and
    self-baselining; and
12. exact use of the existing `PREFLIGHT_AUTHORITY_SHA256` digest domain and `self` sentinel, plus
    rejection of every other digest domain and exact isolation of the finite synthetic
    source/fixture witness; and
13. a source inventory proving there is no provider, VM, pilot, adapter, daemon, datastore,
    connector, network, authentication, user-data, or recovery entrypoint added by this work.

The implementation should expose one finite immutable edge table and one finite immutable root and
absence table. This amendment fixes only the table codec/validator/adapter names explicitly listed
below; eventual exact source review must freeze every other helper name and the complete selected
pure-function allowlist, which under this amendment contains only the three table codec FunctionDefs.
Launcher proof must reject a table label that is merely compared as a string without selecting the
corresponding substantive producer, parser, relation, and consumer predicate.

The combined preflight binds the exact final amendment and review paths externally and only during
non-executing preflight/source review. Neither record is read by a child, semantic gate, completion
runtime scanner, launcher runtime, or payload/runtime process. This amendment does not add either path
to the controller/scanner six-anchor runtime set, does not add another runtime source-hash loop,
and does not make a reviewer's vote a runtime input.

The later source candidate must expose exactly these three selected pure table codec FunctionDefs,
with no decorators, defaults, annotations, closures, global/nonlocal declarations, imports, ambient
authority, effects, dynamic lookup, or alternate codec:

```text
c2b2a_encode_retained_state_table(edge_rows, root_rows, entry_rows, absence_rows, observation_rows) -> bytes
c2b2a_parse_retained_state_table(data) -> (edge_rows, root_rows, entry_rows, absence_rows, observation_rows)
c2b2a_validate_retained_state_table_binding(data, name, rows, byte_count, sha256) -> bytes
```

The validator returns only the accepted byte-identical canonical stream. Under the accepted launcher
boundary, the checker may compile one synthetic `ast.Module` containing copies of only these three
purity-proven FunctionDefs and invoke only those extracted functions through the direct codec
vectors below. Before extraction it proves every reachable name is one of those three definitions,
an immutable literal constructed only with `ast.literal_eval`, or an exact pure builtin from the
closed minimal table, and that no selected name is assigned, deleted, aliased, or rebound elsewhere.
It does not compile or execute the original module top level, the `Authority` class body,
`parse_authority`, `_authority_object`, `_retained_state_table_stream`,
`_retained_state_table_stream_from_combined`, or any other production adapter or definition.

The production controller adapter remains exactly `_retained_state_table_stream(authority)`: exact
source proof must show that it supplies only source-closed rows and Authority-expanded paths to the
selected encoder, then passes those canonical bytes and the dedicated binding to the selected
validator before returning only the accepted bytes. The launcher independently implements
`_retained_state_table_stream_from_combined(record)` from accepted combined-preflight rows. The two
adapters may share no imported implementation. Their behavior, callsites, dominance, and
controller/launcher agreement are proved statically and remain subject to later runtime
qualification; neither adapter is invoked by this source-only proof.

The closed direct extracted-function codec inventory is exactly 19 cases: one positive and 18
negative. Its names and only executable targets are:

```text
retained-state-table-positive                         c2b2a_encode_retained_state_table -> c2b2a_parse_retained_state_table -> c2b2a_validate_retained_state_table_binding
retained-state-table-missing-row                      c2b2a_parse_retained_state_table
retained-state-table-duplicate-row                    c2b2a_parse_retained_state_table
retained-state-table-reordered-row                    c2b2a_parse_retained_state_table
retained-state-table-extra-row                        c2b2a_parse_retained_state_table
retained-state-table-wrong-header                     c2b2a_parse_retained_state_table
retained-state-table-wrong-schema                     c2b2a_parse_retained_state_table
retained-state-table-wrong-row-schema                 c2b2a_parse_retained_state_table
retained-state-table-wrong-edge                       c2b2a_parse_retained_state_table
retained-state-table-wrong-root                       c2b2a_parse_retained_state_table
retained-state-table-wrong-name                       c2b2a_parse_retained_state_table
retained-state-table-wrong-absence                    c2b2a_parse_retained_state_table
retained-state-table-wrong-observation                c2b2a_parse_retained_state_table
retained-state-table-over-1mib-bytes                   c2b2a_parse_retained_state_table
retained-state-table-noncanonical-bytes                c2b2a_parse_retained_state_table
retained-state-table-wrong-binding-name               c2b2a_validate_retained_state_table_binding
retained-state-table-wrong-binding-rows               c2b2a_validate_retained_state_table_binding
retained-state-table-wrong-binding-bytes              c2b2a_validate_retained_state_table_binding
retained-state-table-wrong-binding-sha256             c2b2a_validate_retained_state_table_binding
```

The over-1MiB direct vector supplies at least 1,048,577 bytes and the extracted parser must reject
it before table decoding or allocation beyond the bound. The noncanonical-byte direct vector has
valid semantics and counts but bytes unequal to the one canonical encoding. Every direct negative
must reject inside its named extracted codec FunctionDef; a caller-side precheck, test-only adapter,
or expected-failure label is not coverage. No direct vector constructs a production Authority or
invokes a production parser or adapter.

### Non-executing authority and adapter source proof

For each exact controller and launcher candidate, the checker rehashes and recounts the source once,
requires its combined-preflight identity, and calls `ast.parse` on those accepted bytes once. It
then constructs exact AST, control-flow, dominance, exit, occurrence, lexical-owner, and def-use
tables without importing the module, compiling any node from these authority/adapter surfaces, or
executing any definition. A source reread, second parse, unsupported AST form, unresolved owner,
dynamic lookup, or incomplete control-flow/def-use edge is terminal to the static proof.

The positive exact-source proof must establish all of the following without claiming that a real
Authority object was constructed:

1. A bijective occurrence inventory covers the exact option literal, Authority field, constructor
   keyword, `_authority_object` key, three selected codec definitions, controller adapter, and
   launcher adapter. Every occurrence has one lexical owner and reviewed role; no alias, duplicate,
   shadow, reassignment, deletion, reflection, or alternate definition exists.
2. `parse_authority` consumes exactly one literal `--retained-state-table-binding` at the exact
   cursor after the final accepted generic preflight binding and before the first phase-runtime
   literal, consumes exactly three following tokens in order, requires the first to equal literal
   non-boolean integer `75`, requires byte count in `1..1,048,576`, requires exactly 64 lowercase
   hex digest characters, constructs the fixed name `retained-state-table`, and has no generic-row,
   default, reset, fallback, alias, or caller-selected path.
3. The `Authority` dataclass declares `retained_state_table_binding` exactly once, immediately after
   `preflight_bindings` and before `phase_runtime_literals`, with no default. Every constructor has
   exactly one keyword assignment in that same semantic position from the accepted parser locals,
   and no later store, alias, mutation, reflection, or replacement can reach the field.
4. `_authority_object` contains exactly one `retained_state_table_binding` key immediately after
   `preflight_bindings` and before `phase_runtime_literals`. Its nested key order is exactly
   `name,rows,bytes,sha256`, and exclusive def-use flow reaches each value from the corresponding
   immutable Authority field member without a raw CLI token, generic binding, fallback, or reset.
5. `_retained_state_table_stream(authority)` derives the exact source-closed 75-row bytes, calls the
   selected binding validator with those bytes and the four exact Authority field values, and uses
   only its accepted return. That validation, the 1,048,576-byte bound, canonical byte-equality,
   row/count/digest checks, and current authority-object digest all dominate every phase effect and
   every success exit; no bypass or exception downgrade exists.
6. The independently owned launcher derivation covers the same exact edge/root/entry/absence/
   observation rows and canonical bytes without importing controller implementation. Exact static
   source comparison and the combined record require controller bytes, launcher bytes, row count,
   byte count, and digest to agree; any divergent row, role, ordering, or adapter flow is rejected.

Combined preflight freezes an ordered non-executing source-proof inventory of exactly 26 cases: one
positive source-shape proof and 25 negative in-memory source-mutation witnesses. The exact order is:

```text
00 retained-state-source-positive-authority-flow
01 retained-state-source-identity-mismatch
02 retained-state-source-reread-or-reparse
03 retained-state-source-unsupported-ast
04 retained-state-source-occurrence-inventory-mismatch
05 retained-state-source-missing-option
06 retained-state-source-duplicate-option
07 retained-state-source-generic-binding-substitution
08 retained-state-source-wrong-cli-arity
09 retained-state-source-wrong-cli-order
10 retained-state-source-wrong-cli-final-position
11 retained-state-source-wrong-cli-row-literal
12 retained-state-source-wrong-cli-byte-bound
13 retained-state-source-wrong-cli-digest-grammar
14 retained-state-source-wrong-frozen-binding-name
15 retained-state-source-cli-default-reset-or-alias
16 retained-state-source-wrong-authority-field-order-or-default
17 retained-state-source-wrong-authority-constructor-assignment
18 retained-state-source-authority-alias-reflection-or-mutation
19 retained-state-source-wrong-authority-object-key-position
20 retained-state-source-wrong-authority-object-nested-order
21 retained-state-source-wrong-authority-object-value-flow
22 retained-state-source-missing-validation-dominance
23 retained-state-source-wrong-source-role
24 retained-state-source-wrong-runtime-reconstruction
25 retained-state-source-controller-launcher-divergence
```

Each negative is a separate in-memory mutation of an already accepted AST/source-model copy and
must make the static proof reject the named invariant; no mutated source is written, reparsed,
imported, compiled, or executed. Missing/duplicate dedicated options and substitution as a generic
`--preflight-binding` are distinct mutations. Wrong order and wrong final position are distinct.
The malformed field, constructor, object key/order/value-flow, dominance, source-role,
reconstruction, and divergence cases mutate those exact AST/CFG facts rather than simulating a
runtime object.

For each of the 26 cases, combined preflight freezes one canonical proof row with exactly: global
sequence and case name; controller and launcher source SHA-256 identities; lexical owner and node
kind; AST spans encoded as one-based start/end line numbers plus zero-based UTF-8 byte columns with
the end column exclusive; bijective occurrence count; normalized def-use flow; positive or negative
polarity; ordered dominator spans; ordered success and failure exits; and expected accepted/rejected
result. It also freezes exact case count `26`, total LF row count, total byte count, ordered case-name
digest, and complete proof-record digest. Unknown fields, missing or duplicate spans, an unsupported
node, an unproved exit, or a count/digest mismatch is terminal. These proof rows are
preflight/source-review records only and add no runtime CLI option, Authority field, result key,
evidence path, or completion row.

The inventories are disjoint and exhaustive for this amendment: exactly 19 direct codec cases plus
exactly 26 non-executing source-proof cases, for exactly 45 combined preflight cases. The report
binds all three counts and separate ordered name digests; a case may not move between inventories.

## Deterministic tests and mutation witnesses

The later implementation candidate must include the following closed obligations. Only the 19
table-codec cases above are direct extracted-function vectors, and they execute only the three
purity-proven codec FunctionDefs. Authority, parser, object, and adapter items use only the 26-case
non-executing AST/CFG/semantic proof. Root/combined-stream and filesystem-state items below are
canonical fixture/source obligations or later runtime-qualification witnesses, not an additional
AST-extraction allowlist. The xattr-reset fixture and witness below therefore do not change the
exact 19 direct, 26 non-executing, or 45 combined preflight case counts or their ordered-name
digests. This amendment authorizes none of that later runtime execution.

- exact positive acquisition, lock-derive, and graph-metadata root and combined streams;
- zero-file mutable roots, the maximum legal counters, zero-length regular content, empty ACLs,
  exact Cargo-marker profiles, and the unique cache source;
- parse/re-encode byte equality and exact root/combined rows, bytes, and SHA-256;
- an exact multi-object xattr fixture in which each `(root-sequence, object-sequence)` restarts at
  `xattr-sequence = 0` and then uses exactly `0..xattr-count-1` in raw xattr-name byte order, plus a
  distinct rejection witness whose otherwise canonical second object continues the prior object's
  numbering as a root-global sequence;
- an exact parent-only root fixture ordered as all object/immediate-xattr groups, then raw-name-ordered
  entries, then fixed-order absences, plus distinct rejection witnesses for an entry before object
  completion, between an object and its xattrs, after an absence, or in alternate entry-name order;
- non-executing exact-source proof of the production authority-object digest binding, literal
  `self` controller sentinel, and finite synthetic authority-value dataflow, with a static
  production-rejection witness and no constructed Authority instance;
- the positive static `parse_authority` field-assignment and ordered `_authority_object` value-flow
  proof, plus the distinct non-executing option, field, key, dominance, reconstruction, and
  divergence mutations in the 26-case inventory; the over-1MiB and noncanonical-table cases remain
  direct codec vectors because they invoke only the extracted pure parser/validator definitions;
- wrong schema, edge, phase, boundary, authority digest, root count, sequence, class, or path;
- missing, extra, duplicated, or reordered roots, objects, xattrs, entries, and absences;
- invalid UTF-8, control bytes, slash/dot/backslash components, uppercase/odd/noncanonical hex,
  malformed octal/u64/hash, bool-as-int, overflow, truncation, suffix, or missing terminal LF;
- root or descendant replacement inode, device/type/mode/UID/GID/nlink/size/mtime/ctime/flags/
  birthtime change;
- same-size content mutation, xattr name/value mutation, ACL mutation, hardlink/shared-inode,
  symlink, special entry, directory insertion/deletion, cache-source change, Git presence, and
  `.rustc_info.json` presence;
- a well-formed per-root digest with a wrong combined digest and the inverse;
- policy-probe temp capture before its immediate post-probe seal, substantive-temp capture before
  its post-final-child seal, retained-root change during result/evidence publication, and a
  post-result/precommit final recomputation mismatch;
- failure before directory chmod, a kill after successful `fchmod` but before `os._exit(0)`, normal
  `os._exit(0)`, and power-loss simulations whose later visible state is respectively exact `0500`,
  absent, `0700`, or malformed; require exact `0500` to stay committed regardless worker status,
  absent/`0700` to be terminal uncommitted, and malformed state to be terminally corrupt;
- an attempted replay/repair of an exact committed phase, an attempted consumer of an uncommitted
  phase, any explicit controller statement/control path other than the intended `os._exit(0)` after
  successful commit, reliance on exit status or CPython scheduler/audit/pending-call activity, and
  any accepted phase directory with mismatched inventory/result/wrapper/digest;
- acquisition-stage or archive-seal selecting the special producer commit/exit-status rule, or any
  of the three producers falling back to base exit-status-only completion;
- consumer mutation before comparison, current-state self-baseline, argv reset, stale predecessor,
  skipped immediate consumer, and wrong-phase replay;
- an acquisition-stage or archive-seal validator that incorrectly compares terminal state to its
  entry predecessor, a consumer that mutates before entry equality, and a consumer that changes or
  drops its retained immutable acceptance before normal base finalization;
- authorized graph-metadata mutable-root change producing a new valid terminal predicate while
  preserving its distinct lock-derive entry acceptance and leaving protected
  payload/source/config/manifest state exact;
- later authorized acquisition `tmp` growth that preserves the acquisition historical predicate
  but correctly prevents a false live-history equality claim; and
- an attempted new evidence leaf, Darwin-check row, marker row, completion row, result `direct`
  key, or gate-result key.

Host mutation witnesses, when a later accepted implementation-preflight authorizes them, must use
fresh disposable roots and restore nothing in place. At minimum they independently mutate one
field from each identity, content, xattr, ACL, inventory, and absence category between the producer
and consumer observations and require terminal rejection. No such witness is authorized by this
amendment.

## Non-expansion and nonclaims

This amendment adds no provider, VM, pilot, Linux guest, datastore, adapter, daemon, connector,
product, network, authentication, correction, deletion, user-data, runtime, or recovery scope. It
adds no child, executable, environment variable, descriptor inheritance, policy permission,
repository input, generated root, broader traversal, wildcard, prefix rule, evidence pathname,
completion pathname, completion row, gate key, Darwin sidecar field, Cargo marker field, or payload
schema.

It does not prove build success, graph correctness, artifact equality, VM containment, RocksDB
behavior, persistence, runtime configuration, cleanup, resource ceilings, or flagship completion.
It proves only that a later accepted implementation can carry and compare the exact bounded host
state across the three named boundaries under the already accepted threat model.

Repository `target/debug` must remain absent. User-owned worktree changes remain preserved.
Nothing is staged, committed, published, or executed by this amendment.

## Acceptance boundary

This file is a frozen candidate and cannot by itself authorize a source edit, static test, support
import, build, provider, VM, pilot, or runtime action.

Before implementation, its exact final bytes must receive at least three independent exact-SHA
reviews with `P0=0/P1=0`, followed by one accepted exact review record. The resulting controller,
launcher, provisioner if affected, and combined preflight must then receive their existing separate
source-only, three-FunctionDef effect-free codec-vector, non-executing exact-source proof, exact-SHA,
build/source-identity, completion-audit, runtime, and flagship gates. A review finding changes the
bytes, invalidates every earlier vote, and restarts the exact-SHA review count.
