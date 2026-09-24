# Native strict successor Stage C2B2a — Darwin host-metadata addendum

## Disposition

This is a frozen candidate addendum, not an accepted implementation or acceptance record. It must
receive an independent review bound to its exact SHA-256 before it can authorize any support-file
change, controller or provisioner invocation, Cargo command, build, probe, VM, provider, adapter,
daemon, datastore or payload execution.

This addendum narrowly supersedes only the Darwin-host ownership, extended-ACL and extended-
attribute assumptions named below in the accepted Stage C2B2a entropy/build-ID repair freeze:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
SHA-256 4a4dec13b99c88694625b52538b246323f69465741300c7cb9b8394cbda83e02
lines   2653
```

In that freeze, this addendum replaces:

- every requirement that a protected repository input or accepted APFS-created object have zero
  extended attributes;
- the extended-attribute rejection in the foundation subtree grammar at lines 726–746;
- the empty-xattr requirements for active ambient roots and rotation parents at lines 778–807;
- the fresh-root interpretation at lines 1081–1093 where the raw kernel-created direct child was
  implicitly expected already to have the invoking group and no inherited ACL;
- the zero-xattr requirements for qualification and host-validation `dylib-empty` at lines
  1847–1853 and 2232–2236; and
- the exact empty `xattrs` arrays and empty-xattr predicates in the foundation observations at
  lines 2366–2402.

For metadata evidence only, it also supersedes these exact base clauses:

- lines 647–712 only to add the two metadata basenames and result observation defined below;
  `graph-finalize` still has exactly its two graph-v2 leaves and no result;
- lines 764–768 only with the amended write, verification and seal order below;
- lines 2168–2173 only insofar as `source-audit` must write the two metadata leaves and carry the
  delegated `graph-finalize` metadata recomputation below; and
- lines 2548–2556, replacing both the `pre-runtime-finalize` sole-leaf rule and its exact
  observation-key inventory.

It additionally supersedes:

- the bounded non-input audit-record list at base lines 204–230 only to append the five exact
  Darwin metadata records listed below;
- the controller mode table at base lines 636–645 only to add the non-phase, read-only internal
  mode `completion-audit-scan` defined below; and
- the cross-root read closure at base lines 672–679 only for the exact controller-only
  `source-audit` reads, post-controller completion-data reads and accepted-addendum and
  addendum-review trust-anchor reads defined below; and
- base lines 605–614 only so that fd 1 from one successful `completion-audit-scan` invocation may
  become candidate completion-TSV bytes after the exact bounded capture and validation protocol
  below. Every other controller stdout or stderr byte, and scanner stderr in every outcome,
  remains diagnostic-only and cannot become acceptance authority.

Everything else in those ranges remains authoritative. In this addendum, an **ordinary phase** is
each of the 29 exact phases listed at base lines 650–658 except `graph-finalize`: exactly 28
phases. The exact replacement evidence and chronology are defined below.

The five appended non-input audit records are exactly:

```text
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_REVIEW_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT.tsv
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT.sha256
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT_REVIEW_2026-09-07.md
```

They inherit every base exclusion: Cargo, compilers, build scripts and runtime probes cannot read
them, and they cannot satisfy a payload, source, build or runtime gate except for the explicit
post-controller completion-audit gate below.

It also adds the separate host-metadata evidence necessary to preserve the original freeze's
pre/post stability, source independence, metadata immutability and terminal-audit claims. It does
not change any content-tree byte grammar or digest, relax any path, mode, ownership, hardlink,
symlink, sparse-file, special-file, ACL, write-closure, network, entropy, build, artifact, source,
query, runtime or evidence gate except as stated exactly here.

The existing tool/system-input stream at lines 976–1077 remains authoritative without
qualification: its complete raw xattr rows and values are preflight-bound, and no tool or system
input is removed, normalized or synthesized. This addendum does not accept a payload binary,
guest image, run plan, RocksDB result, persistence claim, C2B2b, harness behavior or the flagship
Engram goal.

## Why an addendum is mandatory

Read-only inspection and disposable-object diagnostics on the selected macOS host established all
of the following:

- the invoking identity is UID 502 and GID 20;
- `/private/tmp` is UID 0, GID 0 and mode `1777` and has no xattr, but it has one exact
  inherited-capable `_dd-agent` extended ACL entry;
- a directory created directly beneath `/private/tmp` by the selected Python APIs begins UID 502,
  GID 0 and mode `0700`, inherits that ACL, and receives `com.apple.provenance`;
- after descriptor-bound group and ACL normalization of that still-empty directory, subsequently
  created child directories and files are UID 502/GID 20, have no extended ACL, and retain the
  same exact provenance attribute;
- newly created regular files, directories and symlinks all receive the same provenance bytes;
- every current node in the 29-object payload tree carries that provenance attribute; and
- Cargo additionally writes one exact backup-exclusion attribute only at source-derived cache
  roots identified below.

The host also proved that provenance stripping is not a repair. `/usr/bin/xattr -d` and direct
`fremovexattr(fd, "com.apple.provenance", 0)` each returned success on disposable objects, but the
same attribute and value remained both immediately and after close/reopen. Return code zero is
therefore not evidence of removal. The controller and provisioner must not call an xattr removal
or synthesis API.

The original zero-xattr design is consequently non-runnable on this host. Globally ignoring
attributes, accepting an attribute by name, or hiding them outside evidence would be an unsafe
repair. The narrow repair is exact per-object classification, complete no-follow enumeration,
separate canonical evidence and fail-closed stability checks.

These attributes are host metadata only. `com.apple.provenance` does not establish source,
reviewer or human provenance, and the backup-exclusion marker does not establish Cargo integrity.
Content hashes, reviewed source, exact commands and the original gates remain the authority.

## Exact closed attribute profiles

The only two non-system attribute names introduced by this addendum are:

| Name | Bytes | Name SHA-256 | Lowercase name hex |
|---|---:|---|---|
| `com.apple.provenance` | 20 | `2ea77e12ee855235a7baf32be3f31ca8dfc5e3ea11abb9835d23fbfec9dfd420` | `636f6d2e6170706c652e70726f76656e616e6365` |
| `com.apple.metadata:com_apple_backup_excludeItem` | 47 | `8270a1c9680263b374e4a2ecbcd0985e796722a2dde54997521bbf405eb39853` | `636f6d2e6170706c652e6d657461646174613a636f6d5f6170706c655f6261636b75705f6578636c7564654974656d` |

The closed values are:

| Value class | Bytes | Value SHA-256 | Lowercase value hex |
|---|---:|---|---|
| fresh/payload provenance | 11 | `3ec9f86a978b1be491dd719a8593670b9f341ee45cf8190a5a097052cbd03ca1` | `010200c0ad31699ccf5a92` |
| repository-ancestor provenance | 11 | `790a83a1522d24e7c8ca8b949ee9c36f0ddb9da5c68513b90b8527f40d02a3e5` | `010200cf4e9166c4dec912` |
| Cargo backup exclusion | 61 | `8332208d45e5ce6a6e8fbce20032850ce228330125d59489170006e91384b7df` | `62706c69737430305f1011636f6d2e6170706c652e6261636b75706408000000000000010100000000000000010000000000000000000000000000001c` |

The implementation has exactly these non-system profiles:

1. `zero`: no xattr rows;
2. `provenance-fresh`: exactly the fresh/payload provenance row;
3. `provenance-ancestor`: exactly the repository-ancestor provenance row; and
4. `provenance-plus-backup`: exactly the Cargo backup-exclusion row followed by the
   fresh/payload provenance row in raw attribute-name byte order.

No profile is selected from a prefix, basename pattern, user-provided name, runtime attribute name
or caller assertion. The exact object path or closed object class selects the profile before the
object is opened. A missing, extra, duplicate, reordered or value-changed attribute is terminal.
`com.apple.quarantine`, Finder metadata and every other unlisted name are always terminal for
repository and generated objects. A new provenance value or backup-marker path requires a newly
reviewed addendum; preflight cannot authorize it by placing it in an opaque digest.

The tool/system-input stream is the sole exception to this four-profile table. It retains the
original freeze's complete preflight-bound arbitrary raw-name/value representation because those
objects are immutable inputs rather than repository or generated objects.

## Canonical Darwin xattr sidecar

Logical content trees retain their accepted schemas and digests. Every applicable repository,
payload, source, archive, target, ambient, canary, evidence, dylib, foundation or other generated
APFS root is represented by a separately computed complete sidecar stream. The sidecar includes
the selected root even when the logical content tree excludes that root. A sidecar is an in-memory
canonical stream until it is framed into the retained phase evidence defined below; it is never an
unlisted standalone pathname.

The sidecar begins with exactly:

```text
schema=c2b2a-darwin-xattr-sidecar-v1
```

The selected root is represented first with relative path `.`. Every descendant follows in
ascending raw UTF-8 relative-path byte order. Each object has exactly one LF-terminated row:

```text
object<TAB><directory|file|symlink><TAB>relative-path
```

`file` includes each regular-file pathname even when the content-tree grammar separately records
it as a hardlink member. Existing content-tree authority still proves hardlink topology, modes,
bytes and allowed object types. Devices, sockets, FIFOs, invalid paths and every object unsupported
by the corresponding content grammar remain terminal.

Each object's xattr rows immediately follow its object row in ascending raw attribute-name byte
order:

```text
xattr<TAB>decimal-name-bytes<TAB>name-sha256<TAB>lowercase-name-hex<TAB>decimal-value-bytes<TAB>value-sha256<TAB>lowercase-value-hex-or-dash
```

There is no path field in an xattr row; adjacency binds it to the immediately preceding object.
Absence is represented by zero xattr rows after the object. Decimal and hash encodings, raw-name
rules, final-LF requirements and per-name, per-value, aggregate-row, aggregate-value and complete-
stream bounds are exactly those of the accepted tool closure at lines 1022–1064. Sidecar row count
includes the schema, object and xattr rows.

Directories and regular files are opened descriptor-relative with `O_NOFOLLOW`; a symlink is
opened descriptor-relative with the reviewed Darwin `O_SYMLINK|O_NOFOLLOW` method and inspected
through its held descriptor. Collection binds one lstat/fstat identity, enumerates the complete
name-and-value snapshot twice, and rejects any identity, name-set or value drift. It never resolves
the symlink target.

Two independent read-only serializations reproduced the current payload identity:

```text
objects  29
rows     59
bytes    6937
SHA-256  6a85bb9c4c07f42e8def7868d312769a1cdbbecda4e2d304e51ec47013321742
```

Every one of those 29 objects has `provenance-fresh`. Simulating only the later authorized regular
file `build-support/archive-manifest-v1.tsv`, at its normal sorted location and with
`provenance-fresh`, yields the required post-publication sidecar identity:

```text
objects  30
rows     61
bytes    7193
SHA-256  686b3afc9567237fca0f0c329d5599f69d12773553fce3961afcc0cb1482c18f
```

The manifest simulation does not authorize its content, mode or creation. Those remain governed
by the original acquisition, publication and content-tree gates. Any other path-set or sidecar
delta is terminal.

### Retained sidecar evidence

Every phase except `graph-finalize` creates exactly two additional evidence leaves before its
`result.json`:

```text
darwin-metadata-v1.tsv
darwin-metadata-v1.sha256
```

These two basenames are added to, and are the only additions to, the original closed phase-leaf
inventory. `pre-runtime-finalize` therefore has exactly these two leaves plus `result.json`, rather
than the original sole result leaf. `graph-finalize` remains exactly the two graph-v2 leaves and
creates neither a metadata leaf nor a result.

`darwin-metadata-v1.tsv` begins with these three LF-terminated rows:

```text
schema=c2b2a-darwin-metadata-evidence-v1
phase=<exact-phase>
checks=<decimal-positive-count>
```

For each of exactly `checks` entries, in zero-based sequence order, it emits one LF-terminated
check-header row:

```text
check<TAB><sequence><TAB><boundary><TAB><label><TAB><retention><TAB><objects><TAB><rows><TAB><bytes><TAB><sha256>
```

`phase`, `boundary` and `label` satisfy the original evidence-field UTF-8/control-byte rules.
`sequence` is exactly the next unsigned integer with no leading zero; `retention` is exactly
`full` or `identity`. The three counts and digest describe one exact
`c2b2a-darwin-xattr-sidecar-v1` stream. When retention is `full`, the
header's LF is wrapper framing and is not part of `bytes`; it is followed immediately by exactly
`bytes` bytes containing that complete sidecar. When retention is `identity`, the header row is
the whole entry and no sidecar bytes follow; the identity must equal a previously retained `full`
check for the same root and label in this phase or an exact earlier sealed metadata leaf named by
the preflight applicability table. The parser advances by the declared byte length rather than
searching for a sentinel line. Every sidecar ends in LF, and the complete wrapper ends in LF.

Every initial state, final state and authorized path/profile delta listed for a substantive or
selected root receives `full` retention. A current phase's carrier leaves, result and post-seal
directory state follow the finite later-auditor delegation below and are not implicit in the word
“final.” Repeated intermediate checks whose required result is byte identity may use `identity`;
they are compared in memory to the named earlier full stream before the header is emitted. Runtime
output digests are observations and may not be caller-authorized. The exact ordered phase,
boundary, label, root, retention and comparison-source table is reviewed as semantic preflight
Authority, not represented only by its aggregate digest. The complete wrapper is bounded to
4,294,967,296 bytes with checked arithmetic.

The digest leaf has exactly these five LF-terminated ASCII rows:

```text
schema=c2b2a-darwin-metadata-evidence-digest-v1
phase=<exact-phase>
checks=<decimal-positive-count>
bytes=<decimal-wrapper-bytes>
sha256=<lowercase-64-hex-wrapper-sha256>
```

The controller writes, fsyncs, closes and reopens the wrapper before it creates this digest leaf;
it then writes, fsyncs, closes and reopens the digest leaf before it creates `result.json`. The
phase result's metadata-check identities must equal the wrapper headers exactly, including their
retention values. The wrapper and digest cannot recursively include their own metadata; their
post-creation state is assigned to the later audit boundary below.

## Repository and traversal metadata

Repository inputs are never normalized. Implementation preflight must publish and independently
review a complete descriptor-walked absolute-path metadata authority for every ancestor and
repository object on which the controller, provisioner or a child relies. Each record binds the
canonical path, type, device, inode, UID, GID, mode, exact xattr sidecar identity and exact ACL
state. It is revalidated before and after every applicable child.

The current closed traversal xattr facts are:

- `/Users` and `/Users/yuval.meiri` use the xattr profile `zero`;
- `/Users/yuval.meiri/projects` and the repository root use `provenance-ancestor`; and
- the observed deeper Engram directories, payload, protected host inputs and design records use
  `provenance-fresh`.

`zero` in that list names only an xattr profile; it makes no ACL assertion. The sole nonempty ACL
in the repository traversal closure is the restrictive entry present on
`/Users/yuval.meiri`:

```text
!#acl 1
group:ABCDEFAB-CDEF-ABCD-EFAB-CDEF0000000C:everyone:12:deny:delete
```

That LF-terminated canonical text is 75 bytes with SHA-256
`882b5793c6cf05dbee0154237359dd1c88022a2786b6d2451b56d761735eebd7`. Its portable
big-endian external representation is 68 bytes with SHA-256
`5fcce06fb1ed90e68bfaf7905c352f7e061b4f2ded7054649ca978f81fe7e0f8` and exact lowercase hex:

```text
012cc16d00000000000000000000000000000000000000000000000000000000000000000000000100000000abcdefabcdefabcdefabcdef0000000c0000000200000010
```

Its Darwin native external representation is 68 bytes with SHA-256
`4330d7ecf77e7ee68d333bd1416b2987e3a27be63723521c9ff58831cbd2c90f` and exact lowercase hex:

```text
6dc12c0100000000000000000000000000000000000000000000000000000000000000000100000000000000abcdefabcdefabcdefabcdef0000000c0200000010000000
```

It is an immutable traversal restriction, not an output template or permission grant. `/Users`
and every other relied-on repository or traversal path must have no extended ACL. A second ACL,
changed home ACL, permissive home entry or inherited copy in an output is terminal.

Preflight must enumerate every actual relied-on path; the three xattr bullets are not a prefix
rule. Only `zero`, `provenance-ancestor` and `provenance-fresh` are legal for this repository
closure, and the exact profile is fixed per canonical absolute path. Any attribute outside those
profiles, new path, changed ancestor, unexpected ACL, or pre/post drift blocks the run. No
repository xattr or ACL is copied as authority into an output; outputs receive only their own
observed, class-required host profile.

The repository closure and `/private/tmp` are additionally bound through held descriptors to the
same exact writable APFS data volume. Its stable `fstatfs` projection is `st_dev=16777233`,
`f_fsid={16777233,25}`, `f_owner=0`, `f_type=25`, `f_flags=0x04909080`, `f_fssubtype=1`,
`f_fstypename=apfs`, `f_mntonname=/System/Volumes/Data`, `f_mntfromname=/dev/disk3s5` and
`f_flags_ext=0x00000001`; `MNT_IGNORE_OWNERSHIP=0x00200000` is absent. Implementation preflight
must bind the reviewed Darwin `fstatfs` ABI, the exact stable projection and both descriptor
results. Capacity, free-block and free-file counters are deliberately excluded. Shell `df`,
`mount`, pathname text and a same-device integer alone are not runtime authority. A different
filesystem, mount, device, stable projection or ownership mode requires a new reviewed addendum.

The current 29-object and expected 30-object payload sidecars above are semantic Authority, not
merely reviewer-supplied digest fields. The preflight must reproduce their complete canonical rows
and embed or bind those rows so the controller and provisioner can independently enforce them.

## `/private/tmp` parent and direct-root transition

`/private/tmp` is a separate system-parent class, not a plain generated directory. Implementation
preflight binds and later revalidates its exact canonical path, device/inode, UID 0, GID 0, mode
`1777`, empty xattr set and exact extended ACL bytes. The current observed ACL has one entry:

```text
!#acl 1
user:94868CB2-91D3-4420-8743-D81FCEBE47A8:_dd-agent:450:allow,file_inherit,directory_inherit:read,execute,readattr,readextattr,readsecurity
```

Its current canonical text is 148 bytes with SHA-256
`c3e98c2e7ee1e6d6ddbcc0acbcba720ca43dad24b95348a530f406e3d857c8c0`.
Its portable big-endian external representation is 68 bytes with SHA-256
`6ea8817ad41fa89485410e631bfffc77c7c1146e6b447cec62eb484f3b73ef27` and exact lowercase hex:

```text
012cc16d0000000000000000000000000000000000000000000000000000000000000000000000010000000094868cb291d344208743d81fcebe47a80000006100000a8a
```

Its Darwin native external representation is 68 bytes with SHA-256
`7d6e8a8be3d7c7139a3ee4ee5d008cf775d3b22cb10d3d3284bfc4401a58561a` and exact lowercase hex:

```text
6dc12c010000000000000000000000000000000000000000000000000000000000000000010000000000000094868cb291d344208743d81fcebe47a8610000008a0a0000
```

The implementation-preflight record must bind the actual canonical text, portable and native
bytes, not rely on a fuzzy text or principal-name match. A changed parent blocks execution and
requires a new review.

Only the acquisition root and qualification roots A and B may use the following transition. Each
exact basename is already constrained by the accepted freeze. The controller:

1. holds and revalidates the exact `/private/tmp` descriptor and creates the previously absent
   direct child with exclusive, descriptor-relative, no-follow semantics and requested mode
   `0700` under recorded umask `0077`;
2. opens and binds the still-empty child descriptor and requires the same device as the parent, a
   new inode, UID 502, GID 0, mode `0700`, `provenance-fresh`, and the exact inherited ACL profile;
3. calls `fchown(fd, -1, 20)` on that same held descriptor;
4. creates an empty ACL with `acl_init(0)` and applies it to that descriptor with
   `acl_set_fd_np(fd, acl, ACL_TYPE_EXTENDED)`, checking return values and freeing the ACL;
5. reapplies mode `0700` with `fchmod`, fsyncs the child and parent, and re-observes the same
   device/inode, UID 502, GID 20, mode `0700`, `provenance-fresh`, no extended ACL and zero entries;
   and
6. only then accepts the root and creates a descendant.

The independently reproduced inherited child ACL portable representation is 68 bytes with
SHA-256 `296595cd60d8c0e4597e8dd124360802d7b23eb4869a0ce95ad046959338ef58` and exact lowercase
hex:

```text
012cc16d0000000000000000000000000000000000000000000000000000000000000000000000010000000094868cb291d344208743d81fcebe47a80000007100000a8a
```

Its native representation is 68 bytes with SHA-256
`e84b9fe9185ba46cf6c27be7ba3d7f8b59f3e49033bcab4eb5bb15cbb983c801` and exact lowercase hex:

```text
6dc12c010000000000000000000000000000000000000000000000000000000000000000010000000000000094868cb291d344208743d81fcebe47a8710000008a0a0000
```

Preflight embeds and runtime compares both complete representations before mutation, not merely
their digests. It also binds the ordered transition transcript. Between the raw-child
snapshot and the accepted post-transition snapshot, the only semantic changes are GID 0 to GID 20
and that exact inherited ACL to no extended ACL. Device, inode, type, UID 502, final mode `0700`,
link count, flags, birthtime, mtime, exact xattr bytes and the empty entry set must remain equal.
`ctime` may be equal or advance monotonically; equality is allowed because timestamp resolution is
not an operation witness. `atime`, allocated-block counts and filesystem-private accounting are
observed diagnostics but are not acceptance fields. No other stat or content delta is authorized.

Creating the direct child necessarily changes, and unrelated users may concurrently change,
`/private/tmp` directory accounting. Around the transition the controller revalidates the held
parent's device/inode/type, UID 0, GID 0, mode `1777`, flags, mount identity, empty xattr profile
and exact ACL bytes, plus the selected basename's absent-to-bound-child lookup. It does not require
the parent's mtime, ctime, atime, link count or unrelated entry inventory to be stable. This narrow
concurrency allowance does not permit replacement of the selected basename or either held
descriptor.

There is no recursive chown or ACL cleanup. Descendants must be born UID 502/GID 20, have no
extended ACL and satisfy their preselected exact xattr profile. The observed diagnostic proved
that directories, regular files and symlinks beneath the normalized root inherit GID 20 and no
ACL. Any contrary runtime observation blocks the run. No existing path, repository object,
tool/system input, Cargo output or post-child object may use this transition.

## Exact Cargo backup-marker closure

The backup marker is not generally allowed on a Cargo-looking path. Its complete reachable closure
is derived from pinned Cargo commit `083ac5135f967fd9dc906ab057a2315861c7a80d` and these exact
source files:

| Relative path in the sealed Cargo source root | SHA-256 |
|---|---|
| `Cargo.lock` | `7b2d20f9c6342e34e31ff1365e172e4ae38c9ffecb78fd5cfd32b2b3d6d851ee` |
| `crates/cargo-util/src/paths.rs` | `3c87a15cf681449d61b9452a4ebc4fb2f093d6dfa42bf4fdb64c5918b8efcb9e` |
| `src/cargo/core/compiler/layout.rs` | `9d510a5f31599924637b24ea6e9583c2e667dbb615f9be089aa96486426c81e9` |
| `src/cargo/core/workspace.rs` | `705583a26a33f02dae8c3948a0ef1db14da43e8ed5e11c1cf51631e0419fbc56` |
| `src/cargo/core/registry.rs` | `eb19a47c346ae27343549a103467c615bb5380532291bd53835b3192549dfa6e` |
| `src/cargo/core/source_id.rs` | `ffec1c7d3139108ec4b8f794d9a0f20432c9c98e172770cc0c30fe8596cb9726` |
| `src/cargo/sources/config.rs` | `f3cbe62254b8b4fb512c64abf8d435a8ed0e7e65397c2e9af4be1133884677c8` |
| `src/cargo/sources/directory.rs` | `33f64c40f376acd8c9cd0f6650a5449fcceeed9679fbaeed2e9c286ffb0639ab` |
| `src/cargo/sources/registry/mod.rs` | `ada2df03050816044a383b7b41b8c66656422a15aa2d3ca88b6fc92bb50d036c` |
| `src/cargo/sources/registry/remote.rs` | `90cce6d14b772001d97185cf4391e5e893fca9dfcc6176e4c86c050a3fe13a8f` |
| `src/cargo/sources/replaced.rs` | `137bd3f623985f85791d70d36bf5f3c17c84fa75a0c3eb51038ad0178480161d` |
| `src/cargo/sources/git/source.rs` | `fd0c49ea283a791be777c4bfdd2cc788135dff292fe70d894ec3a5799d8fed6f` |
| `src/cargo/util/context/mod.rs` | `84589b0311db9a6f39d1c027fb79e5c6a25f86dd7ba3c5d312a93fa3dfa6d323` |
| `src/cargo/ops/cargo_package/mod.rs` | `1bcd068fa3469e403eed3d29e296e8cb352a0635d33df2855d53e43406601536` |

The only required `provenance-plus-backup` path classes are:

1. acquisition `<cargo-home>/registry` after the one authorized online `cargo fetch`;
2. qualification A `<target>/aarch64-unknown-linux-musl`;
3. qualification B `<target>/aarch64-unknown-linux-musl`; and
4. host validation `<host-validation-target>/lints/aarch64-unknown-linux-musl` after the first
   authorized cross-target lint child.

For each class, preflight freezes the exact owning root, phase, creating Cargo callsite, first
applicable boundary and all later boundaries. The marked directory must contain exact regular file
`CACHEDIR.TAG`, 177 bytes, SHA-256
`6d9d1d216e0f83abc5e5662ca62c92b4f23009466b54fa27321a69acdb778bb2`.
The tag itself and every ordinary descendant use `provenance-fresh` unless another exact path in
this four-entry table selects `provenance-plus-backup`.

The acquisition registry case is marked directly at its final retained path and has no
transient-name exception. The other three cases are created through pinned Cargo's
`create_dir_all_excluded_from_backups_atomic`: while the exact final target-triple basename is
absent, Cargo 1.93.0 uses `TempFileBuilder::new().prefix(base).tempdir_in(parent)`, writes the
marker and tag into that nondeterministically named sibling directory, and renames it to the final
path. The addendum therefore withdraws only a claim about that one transient basename,
intermediate sidecar and complete in-flight entry history. The exception begins only after the
substantive Cargo child starts and ends before its post-child boundary. It does not withdraw the
pre-child absence claim or any final-state claim. At the post-child boundary the temporary sibling
is absent and the exact final path exists with `provenance-plus-backup` and the exact tag; that
observed final inode and profile must remain stable at every later boundary.

Implementation preflight must bind that call chain to the sealed `paths.rs`, the exact Cargo lock,
and registry package `tempfile` 3.23.0 with checksum
`2d31c77bdf42a745371d260a26ca7163f1e0924b64afa0b688e61b5a9fa02f16`. It must review and hash
the exact `tempfile::Builder::tempdir_in` implementation and filename-generation dependencies
rather than guessing a temporary-name pattern. For every accepted call it must also prove
`root == build_root`, no separate build directory and no unstable new-layout override, so
`layout.rs`'s conditional second helper call is unreachable. The accepted execution takes the
absent-destination successful-rename branch. A concurrent-Cargo rename fallback, pre-existing
final path, residual candidate, final inode substitution or marker failure is terminal.
That branch selection follows from pinned source plus the sole-writer and path-containment proof;
it is not represented as an observed syscall history. Runtime enforcement is the exact post-child
final object, tag/profile, stable later inode and absence of a residual sibling.

The controller-precreated acquisition target, qualification target parents, host-validation
`payload-tests` and `lints` roots remain `provenance-fresh`; pre-existence makes them distinct from
Cargo's marked descendants. Qualification and host validation use only the sealed directory-source
replacement. Pinned Cargo may construct the original crates.io `RegistrySource` solely for
replacement compatibility calls to `supports_checksums()` and `requires_precise()`; preflight
must bind that inert construction and prove it performs no filesystem or network operation.
`ReplacedSource` must retain and delegate preparation, readiness, query and download exclusively
to the sealed `DirectorySource`. No Git source may be constructed, and no filesystem-effecting
registry or Git preparation, readiness, query, download, cache or marker path may be reachable.
The proof binds the exact root configuration, command/environment closure, pinned source files
above and resolved source graph; all 443 locked source rows remain crates.io registry identities
and none is a Git row. Any qualification or host-validation `cargo-home/registry`, any
`cargo-home/git`, either `cargo package` marker callsite, a missing required marker/tag, or a backup
attribute at any other retained pre-child, post-child, seal or final-audit path is terminal. The
three explicitly unclaimed atomic in-flight basenames are not accepted objects and cannot satisfy
any gate.

## Generated-object and phase schema amendments

After the direct-root transition, every controller- or provisioner-created durable APFS directory,
regular file and symlink must have `provenance-fresh` unless it is one of the four exact
Cargo-marked directories. This includes seed and root-config copies, manifest authority, archive
staging and bundle, provisioned payload and source copies, active homes, rotated homes, target and
temporary roots, policy canaries, dylib sentinels, Cargo/compiler outputs, foundation roots and
outputs, evidence parents, phase directories and evidence leaves. Every controller- or
provisioner-retained temporary object is included: non-authoritative content does not exempt its
metadata from containment and terminal accounting. The only transient-name exception is pinned
Cargo's three atomic marker directories above, whose in-flight names are deliberately unclaimed
and whose exact final objects and absence of residual siblings are fully checked.

Every accepted generated object remains UID 502/GID 20 and ACL-free. The original rules for modes,
link counts, identities, mtimes, bytes, entry sets and allowed mutations remain unchanged. The
foundation content streams continue to reject an attribute that is missing from or inconsistent
with the selected exact sidecar profile, rather than rejecting their required provenance row.

Each prior foundation `xattrs: []` field becomes a canonical array of exact attribute objects in
raw-name order. An attribute object has exactly these ordered keys:

```text
name_bytes name_sha256 name_hex value_bytes value_sha256 value_hex
```

The value types are respectively unsigned integer, string, string, unsigned integer, string and
string. Foundation roots, parents and child directories use a one-element array for
`provenance-fresh`; a Cargo-marked directory uses the two exact rows only where the four-entry path
table makes that phase applicable. `extended_acl` remains false in every accepted post-transition
foundation state. This replacement applies only where an existing closed observation schema
already has an `xattrs` field.

In particular, the base `ambient_roots` element retains exactly its original twelve ordered
keys—`kind`, `active_path`, `archive_path`, `device`, `inode`, `uid`, `gid`, `pre_mode`,
`pre_entries`, `post_mode`, `post_entries`, `archive_mode`—and gains no inline xattr or ACL field.
Those existing ambient fields remain the identity, mode and entry-count authority. Xattr state is
authoritative only through the separate ordered sidecar checks below, while ACL absence remains
the explicit controller predicate required by the base and is not encoded in an xattr sidecar.
`dylib-empty` likewise gains no field in a schema that did not already contain one: its xattr state
is carried by the sidecar, and its ACL state remains the base predicate.

The existing invocation `pre_trees` and `post_trees` objects retain their exact four-key content-
tree schema. Every ordinary phase's existing `result.json` appends
`darwin_metadata_checks` as the final ordered key of its `observations` object; no earlier key or
order changes. Its value is an ordered array whose elements have exactly these ordered keys:

```text
sequence boundary label retention objects rows bytes sha256
```

`sequence`, `objects`, `rows` and `bytes` are unsigned 64-bit integers. `sequence` begins at zero
and increases by one. `boundary` and `label` are closed preflight-bound strings; `retention` is
exactly `full` or `identity`; and `sha256` is lowercase 64-hex. Array length, order and every field
must equal the wrapper's check headers bijectively and in the same order. `graph-finalize` is
exempt because it has no result. For `pre-runtime-finalize`, this addendum replaces the base
observation inventory with exactly `gate_01` through `gate_21` in numeric order followed by
`darwin_metadata_checks` as the final key.

The exact per-phase label, root, profile, boundary, sequence, retention and comparison-source
applicability table is semantic preflight Authority and is independently reviewed in full. For
substantive and selected roots it includes creation, pre/post policy probe, pre/post every
substantive child, pre/post authorized controller metadata transition, archive and applicable
final boundaries. A current phase's wrapper, digest, result and post-seal directory state are not
included in that phase's own table; they are assigned only through the finite delegation order
below. The table may not be represented only by aggregate rows, bytes or SHA-256.

At each listed boundary the controller recomputes the complete sidecar. Any root required stable
by the original freeze has byte-identical pre/post sidecars. An authorized new object or exact
Cargo marker changes only the preflight-declared path/profile rows and is verified against the
logical content-tree delta. Results record actual sidecar identities; runtime-generated output
digests are observations and are never caller-preauthorized.

For every ordinary phase other than `pre-runtime-finalize`, the finite carrier order is:

1. complete every substantive child and every applicable base evidence leaf other than
   `result.json`;
2. capture the final listed sidecars, then create, fsync, close and no-follow reopen
   `darwin-metadata-v1.tsv`;
3. create, fsync, close and no-follow reopen `darwin-metadata-v1.sha256`;
4. create, fsync, close and no-follow reopen `result.json`, whose check array exactly matches the
   wrapper; and
5. validate every then-present evidence leaf's content, `provenance-fresh`, absent ACL, exact
   ownership, mode and link count, then change the phase directory to mode `0500` and reopen it.

The phase's own wrapper, digest, result and post-seal directory state are expressly absent from
its wrapper. Later phases check their xattr state through retained full sidecars and recheck their
base stat, ACL and content predicates through the original result/content gates and controller
logic; `pre-runtime-finalize` performs the last in-run check. There is no self-inclusion.

`graph-finalize` keeps exactly the two graph-v2 leaves, creates no metadata leaf and creates no
result. It performs its stable-input checks before writing and checks its phase directory and both
graph leaves in memory after writing, reopening and sealing them. No retained serialization of the
actual graph-directory pre-write metadata state is claimed. Before `source-audit` creates any leaf,
it must recompute full sidecars for the sealed `graph-finalize` directory, its two leaves and every
still-stable graph input, then validate the graph content/digest pair. Where an input has an exact
earlier retained full check, the current sidecar must equal it. Where no earlier full check can
exist because an immediately preceding phase created its own wrapper, digest or result after its
last retained check—including `probe-build-B/result.json`—`source-audit` performs first retention:
it verifies the current object's exact content and identity against the graph's trusted input and
phase-completion evidence, proves from the frozen schedule, write closure and reviewed support
source that no authorized metadata mutation existed after sealing, and emits its current sidecar
as `full`. It then immediately recomputes the same root and emits an `identity` check against that
new full entry. This is an explicit containment argument under the base same-UID trust assumption,
not a claim of an observed syscall history or retained temporal equality at graph entry. The exact
applicability table names every first-retained object and prior authority. The graph-directory
recomputation is also retained as `full` in `source-audit/darwin-metadata-v1.tsv`.

Those are the sole new `source-audit` cross-root reads. The trusted controller may descriptor-
relative, no-follow read only the exact graph input roots, phase results and metadata
wrapper/digest leaves enumerated by the reviewed applicability table; it may not write them and
no child receives a descriptor or path permission. Every other base evidence-read restriction
remains unchanged.

Before the `pre-runtime-finalize` directory is created, that exact path must still be absent. The
lock-root and A/B evidence parents are already sealed mode `0500`; the acquisition evidence parent
is deliberately still mode `0700`; and every earlier phase directory and leaf is sealed. The
controller scans all of those exact states, validates all prior wrapper/digest/result
correspondences and the exact graph pair, and retains the required full streams and identities in
memory. Its acquisition-evidence-parent observation is named `pre-runtime-entry` and covers that
mode-`0700` state while its own phase path is absent. The controller then creates the phase
directory, writes and reopens its wrapper, digest and result in that order, validates the three
leaves, seals and reopens the phase directory, and finally changes the acquisition evidence parent
to mode `0500` and reopens it. Its wrapper makes no claim about its own three leaves, its post-seal
phase directory or the final acquisition evidence parent.

The completion scanner is the one newly authorized internal, non-phase mode
`completion-audit-scan` in the existing reviewed `controller.py`; no support path or payload object
is added. Implementation preflight binds its complete source and this shell-free argv grammar:

```text
/usr/bin/env -i LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=0 HOME=/var/empty \
  __CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0 \
  <exact-python> -I -B -S <exact-controller.py> completion-audit-scan \
  <exact-pre-runtime-finalize-authority-argv>
```

The exact reviewed completion-reviewer launcher/capture implementation, its source and executable
identities, the absolute `/usr/bin/env`, Python and controller paths, and every authority argv are
bound by implementation preflight. It invokes no shell. It gives the scanner the same exact base
controller entry state: cwd `/var/empty`; incoming umask `0077`; fd 0 as read-only `/dev/null`;
fds 1 and 2 as distinct, nonseekable, one-way pipes; every fd greater than 2 closed or `CLOEXEC`;
and exactly the six environment assignments shown above after `/usr/bin/env -i`. The controller
validates the exact argv, environment, cwd, umask and fd state before resetting its working umask,
and performs the same exact entry and exit source/tool/root rehashes required by the base contract.
Any mismatch or drift is terminal.

The scanner is accepted only after a successful `pre-runtime-finalize` exit and the final
acquisition-evidence-parent seal. Its dispatcher cannot enter `run_phase`, create an evidence
directory or emit `controller_valid=true`. Apart from the exact base entry/exit source, tool and
root trust closure, its completion-data reads are limited to the exact final evidence roots
authorized below. Its only additional repository reads are the accepted addendum and accepted
addendum-review paths listed above, opened descriptor-relative and no-follow through held verified
repository directories and checked as read-only regular, link-count-one files with their accepted
exact hashes. Every other repository path is forbidden. It performs no write, subprocess or
network operation, emits exactly the canonical completion TSV to fd 1 and emits zero bytes to fd 2
on success. The accepted addendum and its review must be present with their exact hashes; the TSV,
digest and completion-review paths must all be absent during the scan. These completion-data and
two trust-anchor reads are the sole new post-controller cross-root reads; no child is involved and
every other base restriction remains unchanged.

Only after successful controller exit, a separate independent completion reviewer launches that
exact scanner with the interpreter, libSystem binding, absolute argv and root closure accepted by
implementation preflight. The launcher starts the scanner in a new process group and concurrently
drains both pipes completely through EOF into separate byte buffers. The interval from immediately
before process creation through complete process-group reaping has an exact 900-second
`CLOCK_MONOTONIC` deadline. The inclusive fd-1 ceiling is 4,294,967,296 bytes and the inclusive
fd-2 ceiling is 4,096 bytes. Timeout, a byte beyond either ceiling, an incomplete or failed read,
failure to observe EOF on both pipes, failure to reap the complete process group, non-normal exit,
nonzero exit status, signal or core termination, any fd-2 byte, or any bytes not consumed as the
one complete canonical TSV is terminal. On timeout or overflow the launcher terminates the entire
scanner process group, drains and reaps it, and publishes nothing.

Only after all process checks pass does the complete fd-1 buffer become candidate TSV authority.
No other controller stdout and no stderr can become authority. The reviewer validates the whole
buffer against the frozen canonical TSV grammar, including its terminal LF, exact row count,
ordering and byte/count bounds; truncation, an unconsumed prefix or suffix, trailing data, NUL or
noncanonical encoding is terminal. It computes the digest record only from those validated bytes.

The reviewed launcher then publishes under held, no-follow repository-directory descriptors. It
requires both destinations absent, opens each basename with create-new `O_EXCL|O_NOFOLLOW` regular-
file semantics and mode `0600` under umask `0077`, and never replaces, truncates or repairs a
pre-existing path. In order, it writes all TSV bytes, syncs and closes the fd, reopens no-follow and
rehashes the complete TSV; derives the digest from that same validated buffer; writes all digest
bytes, syncs and closes the fd; reopens no-follow and rehashes and reparses the complete digest;
then syncs the held parent directory. At every open and reopen it verifies the expected device,
parent containment, regular-file type, link count one, current-UID ownership and exact mode `0600`.
Short write, metadata mismatch, content mismatch, sync or close failure, parent drift, or any
partial publication is terminal and requires a separately reviewed recovery freeze; this protocol
never deletes or repairs it. The two create-new paths, in publication order, are:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT.tsv
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT.sha256
```

The TSV schema is `c2b2a-darwin-metadata-completion-audit-v1`; its complete closed row grammar,
bounds and applicability table are frozen in implementation preflight. It must retain full
sidecars and exact stat, ACL, entry-inventory and content-hash bindings for every final evidence
parent, every phase directory and every leaf. It specifically verifies the final
`pre-runtime-finalize` entry set of exactly `darwin-metadata-v1.tsv`,
`darwin-metadata-v1.sha256`, `result.json`; the wrapper digest; canonical valid result; exact gate
and check-array order; all three leaf contents and metadata; the sealed phase-directory mode; and
the final mode-`0500` acquisition evidence parent.

For every earlier object, the scanner byte-compares each persistently serialized property against
its exact applicable pre-runtime wrapper, result, graph or earlier phase authority: this includes
the retained full xattr sidecars, contents and hashes, entry inventories and any stat, identity or
ACL property actually serialized by that authority. The reviewed applicability table classifies
every final property as an exact serialized comparison or as the bounded containment case below
and names its comparison source. It is terminal to claim a temporal equality from a field that was
retained only in the exited controller's memory.

For a stat or ACL property whose pre-runtime raw value was checked but not serialized, the scanner
records and validates its exact current final value but uses the same bounded containment argument
as `source-audit`: the canonical successful pre-runtime result proves the earlier check completed;
the frozen schedule, sealed states, write closure and exact reviewed support source prove that no
authorized metadata mutation existed between that check and this scan; and the base same-UID trust
assumption closes untrusted concurrent mutation. This is not a claim of retained raw temporal
equality or an observed syscall history. Under that rule, every earlier phase directory and leaf
must have its exact accepted current final state. For the acquisition evidence parent, the prior
successful `pre-runtime-entry` check establishes mode `0700` with the phase path absent, while the
scanner verifies the current mode `0500` and addition of exactly the sealed `pre-runtime-finalize`
directory; those are the only scheduled deltas, without inventing an unserialized prior inode or
ACL value. The digest file has exactly these five LF-terminated ASCII rows:

```text
schema=c2b2a-darwin-metadata-completion-audit-digest-v1
kind=completion-audit
rows=<decimal-TSV-row-count>
bytes=<decimal-TSV-bytes>
sha256=<lowercase-64-hex-TSV-sha256>
```

The TSV row count includes its schema and all embedded full-sidecar rows. Counts are unsigned base
ten with no leading zero except zero.

Both completion-audit files require an independent exact-SHA P0=0/P1=0 review, recorded only at
`evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_COMPLETION_AUDIT_REVIEW_2026-09-07.md`,
before a build/source identity record may be proposed. The TSV, digest and review are outside
controller and run evidence, are never scanned by themselves, and make no claim about their own
filesystem metadata; their integrity is only their exact bytes, digest and independent review. No
recursive metadata audit follows. A content hash without the required metadata check cannot
satisfy an evidence gate.

## Darwin API and race closure

Both selected Homebrew Python 3.14 and Apple Python 3.9 lack Python `os.listxattr`, `os.getxattr`
and `os.removexattr`. The reviewed controller and provisioner use
`ctypes.CDLL(None, use_errno=True)` and bind these Darwin signatures exactly:

```text
flistxattr(int, void *, size_t, int) -> ssize_t
fgetxattr(int, const char *, void *, size_t, uint32_t, int) -> ssize_t
fstatfs(int, struct statfs *) -> int
acl_get_fd_np(int, acl_type_t) -> acl_t
acl_init(int) -> acl_t
acl_set_fd_np(int, acl_t, acl_type_t) -> int
acl_size(acl_t) -> ssize_t
acl_copy_ext(void *, acl_t, ssize_t) -> ssize_t
acl_copy_ext_native(void *, acl_t, ssize_t) -> ssize_t
acl_free(void *) -> int
```

For the selected 64-bit Darwin ABI, `struct statfs` has size 2168, alignment 8 and these exact
field offsets in bytes:

```text
f_bsize=0 f_iosize=4 f_blocks=8 f_bfree=16 f_bavail=24 f_files=32 f_ffree=40
f_fsid=48 f_owner=56 f_type=60 f_flags=64 f_fssubtype=68 f_fstypename=72
f_mntonname=88 f_mntfromname=1112 f_flags_ext=2136 f_reserved=2140
```

Implementation preflight compiles or independently validates that layout against the exact SDK
headers and selected Python/libSystem ABI before either support script may use it. Runtime compares
only the stable projection declared above; volatile capacity and free-space fields cannot cause or
satisfy acceptance.

FD xattr calls use options zero because the descriptor was already obtained no-follow. Every size
is checked before allocation. Attribute names are nonempty, at most 255 bytes, NUL-delimited and
unique; counts, individual values, aggregate values and stream bytes use the accepted bounds. The
collector takes two complete sorted name-and-value snapshots around a stable fstat identity. It
fails on growth, shrinkage, deletion, unreadability, invalid termination, duplicate names, short
reads, overflow or any value drift.

ACL collection uses an exact bounded external representation, verifies the selected type
`ACL_TYPE_EXTENDED = 0x100`, checks every return code, and frees every allocated ACL. A null ACL is
accepted as absence only for the preflight-reviewed Darwin errno semantics. On success, stale
`errno` is ignored and the return value is authoritative. The empty-ACL write occurs only in the
direct-root transition above. Runtime compares both `acl_copy_ext` portable bytes and
`acl_copy_ext_native` native bytes; the canonical text in this addendum is reviewer-readable
corroboration and is not parsed as authority.

`fremovexattr`, `removexattr`, `setxattr`, `fsetxattr`, path-based ACL mutation and subprocess ACL
or xattr tools must not occur in controller or provisioner source. Pinned Cargo's exact
backup-marker behavior and the host's provenance stamping are observed effects, not controller
authority to synthesize them.

## Implementation-preflight requirements

Before any support file becomes executable or any support mode runs, the implementation-preflight
record and its independent exact-SHA review must prove all of the following:

1. the accepted entropy/build-ID freeze identity and this addendum's separately accepted exact
   identity;
2. the selected Python executables, libSystem image, every ctypes symbol/signature and the exact
   no-follow descriptor implementation used by both support scripts, including the reviewed
   `statfs` layout and stable `fstatfs` projection;
3. the exact same-volume APFS binding; complete repository/traversal path authority; the sole
   restrictive home ACL and zero-ACL state of every other relied path; and immutable pre/post
   repository checks;
4. the complete `/private/tmp` parent metadata/ACL authority, disposable root-transition
   transcript, complete raw-child ACL bytes and same-inode postcondition, including the
   nondecreasing-ctime rule, excluded atime/accounting fields and narrowed parent-stability fields,
   with no retained diagnostic object;
5. the exact four profiles as embedded semantic constants, all canonical encodings, and static
   proof that no wildcard, prefix, name-only or caller-selected profile path exists;
6. the exact 29-object payload sidecar and deterministic expected 30-object post-publication
   sidecar, including the manifest's exact `build-support/archive-manifest-v1.tsv` path;
7. an exhaustive support-code callsite-to-root/profile/boundary map, including repository,
   provisioned, ambient, target, temporary, canary, dylib, foundation and evidence objects;
8. the exact Cargo source commit and file hashes above, an exhaustive marker-callsite proof, the
   four reachable marked final paths, exact tag bytes, directory-source replacement proof,
   `root == build_root`, the pinned `tempfile` source/call chain, the three bounded in-flight
   exceptions and post-child residual-sibling checks;
9. the complete wrapper and digest serializers/parsers; the exact ordinary-phase leaf inventory;
   graph-to-source-audit delegation; pre-runtime-entry scan; finite carrier DAG; and proof that no
   phase must include its own carrier, result or post-seal directory metadata;
10. closed nested result schemas for every amended foundation xattr array and
    `darwin_metadata_checks`; unchanged ambient/dylib inline schemas; and an exact per-phase
    applicability, retention, comparison-source and order table rather than only an aggregate
    digest;
11. the completion-audit scanner and reviewed launcher/capture implementation; their fixed
    read-only argv, environment, entry-state, tool and root closure; the exact 900-second deadline,
    pipe ceilings, concurrent complete-drain and process-group-reaping behavior; the complete
    closed TSV row grammar and digest grammar; the descriptor-relative create-new mode-`0600`
    publication and reopen/rehash/sync protocol; the exact per-property serialized-comparison or
    containment classification and authority; final coverage; and the non-recursive acceptance
    gate;
12. focused static tests for unknown, missing, extra, value-changed or reordered xattrs; wrong path
    class; repository immutability; home-ACL drift; failed root transition; ctime equality and
    monotonic advance; volatile parent fields; inherited ACL; group mismatch; symlink no-follow;
    Cargo marker/tag mismatch; `root != build_root`; transient residuals; wrapper/result mismatch;
    graph delegation; pre-runtime absence/order; evidence drift; and audit recursion termination;
13. proof that every former plain/zero-xattr validator callsite was deliberately classified and
    that tool/system-input handling retained its original exact arbitrary-row seal; and
14. a final static source scan proving no xattr removal/synthesis API, recursive chown/ACL cleanup,
    path-based metadata mutation, subprocess-based ACL/xattr inspection or mutation, or hidden
    metadata exception exists.

The exact hand-authored support files then require a fresh independent P0/P1 review bound to their
complete hashes. Any support edit invalidates that verdict. The implementation-preflight may bind
runtime paths, device/inode values and generated observation values only where this addendum
explicitly declares them later-bound. It may complete only the external audit row grammar expressly
delegated above; it may not manufacture a new result schema, attribute value, path class, Cargo
callsite, normalization operation or acceptance meaning.

## Acceptance boundary

This addendum is acceptable only if an independent exact-SHA review returns P0=0 and P1=0 and
confirms that it converts an impossible zero-xattr assumption into stricter, visible and
recomputable evidence without relaxing content or execution gates.

Even after that review, the current support files remain non-runnable until they implement every
requirement above and pass their own exact-SHA implementation-preflight review. Repository
metadata remains untouched. No build, provider, VM, daemon, adapter, datastore or runtime phase is
authorized by this design artifact. Repository `target/debug` must remain absent.
