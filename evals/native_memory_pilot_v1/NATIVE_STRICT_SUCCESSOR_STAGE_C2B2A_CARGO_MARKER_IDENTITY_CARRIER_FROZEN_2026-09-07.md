# Native strict successor Stage C2B2a — Cargo marker identity-carrier amendment

## Status and authority

This is a frozen candidate design amendment, not an accepted implementation or execution record.
It authorizes nothing unless at least three independent reviews of its final exact SHA-256 each
return `P0=0/P1=0`. No controller, provisioner, launcher, Cargo, compiler, build, provider, VM,
adapter, daemon, datastore, payload or runtime mode may run on its authority.

This amendment is subordinate to, and changes only the exact cross-process Cargo marker identity
gap described below in, these accepted artifacts:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
SHA-256 4a4dec13b99c88694625b52538b246323f69465741300c7cb9b8394cbda83e02
lines   2653
bytes   179776

evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_FROZEN_2026-09-07.md
SHA-256 34ded5a82bba22ea741b8e896b15779a90893d858ed2826522b7a2b5df7d27d6
lines   893
bytes   56876

evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_REVIEW_2026-09-07.md
SHA-256 58b5e5324803cf8989a3368d111278defc4c828c67d9794e75b7660688bff3f5
lines   187
bytes   11527

evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
SHA-256 8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
lines   513
bytes   34249

evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_REVIEW_2026-09-07.md
SHA-256 352b47668388c8c8a7105e579597fcc41f0e11e202f7cb03f68b8e798b6af82e
lines   133
bytes   8512
```

Everything not expressly superseded here remains authoritative. In particular, this amendment
does not relax any xattr profile, ACL, ownership, mode, path, child, network, evidence, source,
build, completion-audit, finite semantic-projection or runtime gate.

## Rejected-candidate ratchet

The first exact draft is rejected before independent review:

```text
SHA-256 c4a0d53751781973a5b6894c5089a2743029c9edc15e3c4a78e0a0ea5a5f9dfc
lines   266
bytes   16568
```

That draft incorrectly carried the marker directory's link count and required its complete entry
inventory to remain stable. A Cargo target-triple directory legitimately gains descendants after
its marker inode is created, so directory link count and entries can change without inode or
profile substitution. The rejected bytes cannot authorize implementation or execution. This
candidate carries only the stable marker-directory fields required below while retaining exact
phase-specific logical-tree and sidecar delta validation.

The next exact candidate is also rejected before independent review:

```text
SHA-256 5d097dc21904e95d2784094b6a47dbe5b80b9c0b825c6d60e5bf59e6f5db4956
lines   290
bytes   17739
```

That candidate correctly excluded changing directory link count and entry inventory from stable
identity, but two textual contracts remained insufficiently exact. It named the first
host-validation-lints marker boundary only by descriptive child role instead of the frozen
`post-child-5` ordinal, and one later-boundary sentence could still be misread as requiring the
marker directory's earlier entry inventory to remain equal. Those bytes received no independent
review and cannot authorize implementation or execution. This candidate fixes only those two
ambiguities.

The next exact candidate is likewise rejected before independent review:

```text
SHA-256 d14619694dbe56973709d123cf226d687954a80c1b0421d49be81e9171f0455c
lines   308
bytes   18529
```

It fixed the host-validation ordinal and the later-inventory wording, but still described the
other three already-known first boundaries without their exact `post-child-0` ordinal. Those bytes
received no independent review and cannot authorize implementation or execution. This candidate
makes all four first-boundary ordinals explicit.

The next exact candidate is rejected after three independent exact-byte reviews:

```text
SHA-256 6bab9704edaa39718ba519092931856ada0c94de405ec97fa16fe7733df457c3
lines   322
bytes   19150
reviews P0=0/P1=1/P2=0, P0=0/P1=3/P2=0, and corrected P0=0/P1=1/P2=0
```

All reviewers ultimately agreed that marker-directory and tag modes cannot be immutable carrier
fields because the accepted host-lints seal changes those modes without replacing either inode.
The launcher-focused review found two additional P1 defects: the qualification classes omitted
their exact creating phase names, and the completion section did not close whether marker/tag
objects changed the already accepted completion TSV grammar. Those bytes cannot authorize
implementation or execution. This candidate removes modes from cross-process equality, freezes
both qualification phase names, and keeps marker validation a terminal scanner prerequisite that
adds no completion-TSV row.

## Impossibility being repaired

The accepted Darwin addendum requires each of the four exact Cargo-marked final directories to
retain the device/inode observed at its first post-child boundary at every later boundary. Within
one controller process that identity can remain in memory. Across ordinary phases, however, each
controller invocation is a new process.

The accepted `c2b2a-darwin-xattr-sidecar-v1` object row serializes only kind and relative path;
its attribute rows serialize only xattr names and values. The accepted logical content-tree
streams deliberately omit directory device/inode identity. The accepted
`darwin_metadata_checks` result array records only sidecar counts and hashes. None therefore
carries the first observed Cargo-marker device/inode into a later controller process.

Sidecar equality proves path, kind and exact xattr-profile continuity, but it does not prove inode
continuity. Reopening the same pathname in a later phase proves only its then-current inode. A
controller implementation cannot silently add a result observation because the addendum freezes
ordinary result schemas and permits only `darwin_metadata_checks` as their final added key.
Consequently the stable-later-inode requirement is unimplementable without one explicit carrier.

## Exact supersession and path boundary

This amendment supersedes only:

- Darwin-addendum lines 231–297, to append the closed marker-identity section below to the existing
  `darwin-metadata-v1.tsv` wrapper while retaining the exact xattr-sidecar grammar unchanged;
- Darwin-addendum lines 494–509, to define how the first post-child marker identity is carried and
  compared across controller processes;
- Darwin-addendum lines 528–639 only where they describe the wrapper framing, carrier order,
  cross-phase comparison source and final scanner source; ordinary `result.json` schemas,
  `darwin_metadata_checks` keys and the two existing metadata evidence-leaf basenames remain
  unchanged;
- Darwin-addendum lines 653–768, completion-launcher-amendment lines 135–147, 385–391 and 455–490,
  and the accepted launcher-boundary review's corresponding four-anchor summaries, only to add
  this amendment and its accepted review to the scanner, launcher and combined-preflight trust
  anchors and to require final marker-carrier comparison;
- Darwin-addendum lines 825–882 only to add exact implementation-preflight source/callsite tables
  and focused static vectors for this carrier; and
- Darwin-addendum lines 884–893 and completion-launcher-amendment lines 494–513 only so the exact
  effect-free static-test exception may exercise the pure marker-carrier codecs and comparisons.

The two appended non-input design records are exactly:

```text
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CARGO_MARKER_IDENTITY_CARRIER_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CARGO_MARKER_IDENTITY_CARRIER_REVIEW_2026-09-07.md
```

No support, payload, test, helper, package or dependency path is added. The bounded implementation
paths remain the existing controller and the single accepted completion launcher. The provisioner
does not create or consume this cross-process carrier. The two records are not payload inputs and
cannot be read by Cargo, a compiler, build script, guest, provider, adapter or runtime process.

After this amendment is accepted, the closed scanner/launcher trust-anchor set becomes exactly six
records in this order: the Darwin addendum and its accepted review, the completion-launcher
amendment and its accepted review, then this amendment and its accepted review. The entropy/build-
ID base freeze and its accepted predecessors remain separately required protected authorities;
they are not relabeled as members of this six-record addendum chain.

## Closed marker identity model

The only marker classes are the four already accepted classes, in this exact order:

1. `acquisition-registry` — `<cargo-home>/registry`, first present after `acquisition-fetch` at
   exact boundary `post-child-0`;
2. `qualification-A-target` — root A `target/aarch64-unknown-linux-musl`, first present in exact
   phase `qualification-A-supervisor` at exact boundary `post-child-0`;
3. `qualification-B-target` — root B `target/aarch64-unknown-linux-musl`, first present in exact
   phase `qualification-B-supervisor` at exact boundary `post-child-0`; and
4. `host-validation-lints-target` — host-validation
   `target/lints/aarch64-unknown-linux-musl`, first present after the first authorized cross-target
   lint child, at exact boundary `post-child-5`.

Implementation preflight freezes each exact absolute path, owning root, creating phase, first
boundary, every later applicable phase/boundary and predecessor carrier. No basename, prefix,
glob, regex, caller-supplied profile, observed attribute name or Cargo-looking path can select a
class. A fifth class or path requires another reviewed amendment.

Each accepted marker directory is observed through one held no-follow directory descriptor and
has exact current device, inode, UID 502, GID 20, directory type, flags, birthtime, absent extended
ACL and `provenance-plus-backup` profile. Its directory mode, link count, size, mtime, ctime and
entry inventory are not stable carrier fields because authorized sealing changes mode and Cargo
work adds descendants; their phase-specific states and deltas remain governed by the accepted
logical-tree and sidecar checks. Its exact child
`CACHEDIR.TAG` is opened no-follow through that held descriptor and has one current device/inode,
UID 502, GID 20, regular-file type, link count one, flags, birthtime, 177 bytes,
SHA-256 `6d9d1d216e0f83abc5e5662ca62c92b4f23009466b54fa27321a69acdb778bb2`
and `provenance-fresh`. Tag mode is phase-specific and is not a carrier field. Volatile atime,
mtime and ctime are not carrier equality fields for either object. Every observation separately
requires the exact current marker and tag modes selected by the frozen phase/boundary table.

The first retained identity is captured only after the accepted pre-child absence and successful
post-child final-path validation. A temporary candidate, rename fallback, pre-existing destination,
failed tag, residual sibling or observation before the exact first boundary cannot become
authority. Authority argv freezes only class/path/phase/boundary applicability; it never supplies
an observed device or inode.

## Canonical wrapper carrier

The accepted `c2b2a-darwin-xattr-sidecar-v1` stream is byte-for-byte unchanged. In particular, the
fixed 29-object and simulated 30-object payload sidecar identities remain unchanged.

`darwin-metadata-v1.tsv` retains its first three rows and adds exactly one fourth header row:

```text
schema=c2b2a-darwin-metadata-evidence-v1
phase=<exact-phase>
checks=<decimal-positive-count>
cargo-markers=<unsigned-decimal-count>
```

The existing `checks` check-header and full/identity sidecar blocks follow unchanged. After the
last check block, exactly `cargo-markers` LF-terminated rows follow, in the four-class order above:

```text
cargo-marker<TAB><sequence><TAB><class><TAB><absolute-path><TAB><first-phase><TAB><first-boundary><TAB><device><TAB><inode><TAB><uid><TAB><gid><TAB><flags><TAB><birthtime-ns><TAB><tag-device><TAB><tag-inode><TAB><tag-uid><TAB><tag-gid><TAB><tag-nlink><TAB><tag-flags><TAB><tag-birthtime-ns><TAB><tag-bytes><TAB><tag-sha256>
```

`sequence` begins at zero and is contiguous. Counts and metadata integers are unsigned base ten
without a leading zero except zero. Class, path, first phase and first boundary byte-equal the
preflight applicability row. The final tag fields are exactly `177` and the accepted lowercase
digest. UTF-8, control-byte, path, LF, field-count and checked-arithmetic bounds are the existing
wrapper/evidence bounds. A missing, extra, duplicate, reordered, malformed or inapplicable row is
terminal.

The carrier set is monotonic in phase order. Before a class's first boundary its path is proved
absent and it has no row. At that boundary the current process captures the first row. Every later
boundary in that process compares the current held-descriptor observation byte-for-byte to the
captured stable fields and separately revalidates the exact profile, tag, phase-specific
logical-tree state and absence of a residual candidate. No later comparison requires equality to
the marker directory's earlier changing descendant set. At phase completion the final equal row
is serialized.

Each later controller process loads the exact prior sealed wrapper/digest pair named by the
preflight predecessor table, parses the carrier rows, revalidates the pair and uses those rows as
the sole prior device/inode authority. Before and after every applicable policy probe,
substantive child, controller metadata transition and final boundary, it opens each marker through
its exact held parent/root chain and requires all carrier fields equal plus the independently
recomputed exact profile, tag and phase-specific logical-tree predicates. It then serializes the
same rows. Raw argv, path reopening, xattr equality alone or a newly observed replacement inode
cannot reset authority.

`graph-finalize` remains leaf-for-leaf unchanged and emits no metadata wrapper. Its process loads
the exact last preceding carrier, compares each applicable marker before and after its work, and
emits nothing. `source-audit` names that same preceding carrier as predecessor, combines its
successful graph evidence with fresh equal observations, and resumes the ordinary wrapper chain.
No missing carrier can be inferred through the graph gap.

The existing five-row `darwin-metadata-v1.sha256` digest grammar is unchanged; its `bytes` and
`sha256` cover the complete wrapper including the new fourth header and every marker row. The
ordinary `result.json` observation schema is unchanged. `darwin_metadata_checks` continues to
correspond bijectively only to check headers; marker rows are authenticated by the complete
wrapper/digest pair and independently validated through the closed applicability table.

## Pre-runtime and completion audit

`pre-runtime-finalize` loads the final predecessor carrier before creating its own phase path,
checks every applicable marker at its pre-runtime-entry and final boundaries, and emits the exact
four-row final carrier in its wrapper. Its custom observations remain exactly `gate_01` through
`gate_21`, followed by `darwin_metadata_checks`; no `cargo_marker_identities` key or other result
schema is added. Gate evidence covers the complete wrapper/digest bytes.

The completion scanner parses the sealed pre-runtime wrapper/digest pair and treats its four marker
rows as persisted serialized metadata authority. For each marker it opens the final path and tag
descriptor-relative/no-follow, compares every carrier field, independently revalidates the current
profile, ACL, phase-specific logical-tree state and tag content. Marker validation is a terminal
prerequisite to scanner success and emits no marker-directory or tag row. The completion TSV keeps
exactly the already accepted evidence-parent, phase-directory and evidence-leaf classes, object
count, order, per-object row/sidecar grammar and applicability table; it gains no class, row,
field, source label or sidecar. The separate marker applicability table names the exact carrier
row as each serialized comparison source and is bound by implementation preflight, but is not a
completion-object table. A replacement inode with identical pathname, xattrs and contents is
terminal before any successful scanner output. The scanner performs no write, child, network or
authority repair.

The completion launcher validates the scanner's complete dynamic TSV with the already accepted
whole-buffer parser and per-property applicability table. It does not re-open a marker path to
manufacture an independent source; the scanner's serialized comparison and the launcher's exact
TSV validation remain distinct layers. Publication and recursion rules are unchanged.

## Implementation-preflight additions

Before the effect-free launcher static test or any support/production execution, the combined
implementation-preflight candidate and non-executing source review must additionally freeze and
prove:

1. the accepted exact SHA/line/byte identities of this amendment and its review;
2. the exact six-record trust-anchor order and every scanner/launcher stable read;
3. the four marker applicability rows, exact first and later boundaries, predecessor carrier for
   every phase, and the special graph-finalize/source-audit edge;
4. the exact wrapper header/row grammar above, parser/serializer agreement, digest coverage and
   unchanged ordinary result schemas;
5. the source proof that no argv field, path pattern, fresh observation or xattr-only equality can
   establish or reset a persisted marker identity;
6. every producer and consumer callsite, with the accepted row assigned before any comparison or
   effect and every later decision exclusively dependent on that canonical row plus frozen
   literals;
7. descriptor continuity, no-follow opens, exact device/inode comparison, marker/tag metadata,
   profile/ACL and phase-specific logical-tree/mode revalidation and residual-sibling rejection at
   every applicable boundary, without treating either mode or directory link count or entries as
   stable;
8. static vectors for missing, extra, duplicate, reordered and malformed rows; a wrong class/path/
   first boundary; pre-first-boundary presence; post-first-boundary absence; same-path replacement
   inode; changed tag inode/content; xattr-equal inode substitution; authorized descendant growth
   with stable marker identity; authorized same-inode marker/tag mode sealing; wrong phase-specific
   mode; graph-gap loss; carrier reset from argv; unauthorized fifth marker; an attempted marker or
   tag completion row; and unchanged positive chains for one through four carrier rows;
9. exact pure-function extraction and launcher-local vector targets without executing support
   module top levels, adapters or modes; and
10. proof that the carrier adds no path, child, write, network, result key, payload sidecar change,
    recursive audit or authority beyond the exact stable marker identity requirement.

The combined preflight finite semantic-projection construction is unchanged. Its unique
`launcher_source_sha256=` field remains the only projected field. This amendment adds no launcher
hash, embedded semantic digest or record-self-hash field and creates no new hash loop. Final clean
implementation-review observations remain external.

Only after this amendment and its review are accepted may the controller and launcher candidates
be updated to implement the carrier and six-anchor closure. Those final exact source bytes and the
combined preflight candidate must then pass the existing non-executing source review, bounded
static-test observations and at least three independent final exact-SHA `P0=0/P1=0` reviews.

## Acceptance boundary

This amendment is acceptable only if three independent exact-SHA reviews each return
`P0=0/P1=0` and confirm that it supplies the missing cross-process inode authority without changing
the xattr-sidecar identities, ordinary result schemas, finite implementation-preflight closure or
execution boundary.

Even after acceptance, no support or production mode is authorized merely by this record. The
implementation-preflight, build/source identity, completion audit, Linux runner, runtime
qualification and final flagship criteria remain separate later gates. Repository `target/debug`
must remain absent; user-owned worktree changes must remain preserved; nothing is staged or
committed.
