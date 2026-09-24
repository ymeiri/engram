# Native strict successor Stage C1 — frozen artifact-race repair

Date: 2026-09-06 (Asia/Jerusalem)

## Status and authority boundary

This addendum freezes the only authorized repair to the rejected C1 implementation described by
`NATIVE_STRICT_SUCCESSOR_STAGE_C1_FROZEN_2026-09-06.md`. It does not broaden C1. All original C1
privacy, provider-free, non-runnable, read-only, source, capability, error, and non-authority
boundaries remain normative. The only production file authorized to change is
`engram-eval/src/native_successor_artifact.rs`; test-only source in that same file may change only
as specified below.

The repaired classifier remains an inert observation. It does not prove a filesystem-wide atomic
snapshot, quiescence, current state after return, live execution, safe replay, or any authority.
A later stage must establish a separately proven quiescence, immutable-snapshot, or stronger
isolation boundary before treating an observed prefix as current or authorizing. C1 assumes a
trustworthy local macOS filesystem with non-ABA device, inode, size, nanosecond mtime, and
nanosecond ctime observations during its bounded two-sweep read. It makes no security claim
against a continuously active same-UID writer or a filesystem that can restore those identity
fields without detection.

## Finding 1: nonblocking entry open

The original classifier performs `fstatat` and then a blocking `openat(O_RDONLY, ...)`. A same-UID
racer can replace the checked regular file with a FIFO before `openat`, causing the read-only
classifier to hang before post-open validation.

The repaired entry-open flags are exactly:

```text
O_RDONLY | O_NONBLOCK | O_NOFOLLOW | O_CLOEXEC
```

`O_NONBLOCK` grants no accepted file type. The existing post-open identity, regular-file,
ownership, single-link, exact-mode, length, and ACL checks remain mandatory. A substituted FIFO,
device, socket, directory, symlink, or other non-regular entry returns `UnsafeEntry`. If `openat`
fails, the classifier performs the existing descriptor-relative non-following `fstatat` and uses
this exact precedence: observed unsafe entry metadata returns `UnsafeEntry`; unchanged safe
identity returns `Unreadable`; changed safe identity or an unavailable pathname returns
`IdentityChanged`. This makes `O_NOFOLLOW` symlink rejection categorical without ever following or
opening the symlink target.

## Finding 2: retained whole-set verification

The original classifier drops each entry descriptor after reading it and subsequently revalidates
only the directory. An in-place write, mode change, or ACL change to an already-read entry need not
change directory metadata, so cached bytes can otherwise form a mixed-time valid chain.

The repaired classifier uses this exact order:

1. Open and retain the directory as already frozen. Enumerate, validate, and retain the exact
   initial ASCII-sorted name set.
2. For every name in that order, use the original bounded double-read checks, but retain until the
   function returns or fails: the open entry descriptor, descriptor-relative `CString` name,
   original `FileIdentity`, original length, exact bytes, and SHA-256 of those bytes.
3. Run the existing pure structural classification over the captured bytes while every entry
   descriptor remains open. Save its result but do not return it.
4. Re-enumerate descriptor-relatively and require the exact initial name set. A changed name set
   returns `IdentityChanged`. Revalidate the retained directory.
5. In initial ASCII name order, verify every retained entry again. Both before and after its
   reread, obtain handle metadata and descriptor-relative `fstatat` metadata. At each boundary,
   check regular-file type, effective-UID ownership, single-link count, exact mode 0600, and absence
   of an extended ACL before comparing identity. An unsafe property returns `UnsafeEntry`; an
   identity, length, or timestamp mismatch returns `IdentityChanged`.
6. Seek the retained descriptor to byte zero, read exactly the original length plus an EOF probe,
   and require exact byte equality and SHA-256 equality with the first sweep. A byte, digest,
   length, timestamp, or identity difference returns `IdentityChanged`; a read failure with
   unchanged identity remains `Unreadable`.
7. After all entry rereads, re-enumerate descriptor-relatively once more and require the exact
   initial name set, then revalidate the retained directory. A namespace or directory identity
   change returns `IdentityChanged`; an unsafe directory property or ACL returns
   `UnsafeDirectory` as in the original design.
8. Only after steps 1–7 succeed may the saved pure classification result be returned. Descriptors
   remain retained through classification and the complete verification sweep and are dropped only
   on return or failure.

Safety checks intentionally precede identity comparisons during the second sweep, so a current
mode, type, owner, link-count, or ACL violation is categorized as `UnsafeEntry`. This ordering does
not weaken identity checks. No retry, lock, sleep, mutation, repair, or unbounded loop is added to
production.

## Finding 3: directory error precedence

The original `RetainedDirectory::revalidate` combines retained/path identity comparisons and
unsafe directory properties in one branch returning `IdentityChanged`. Because a directory mode,
owner, type, or ACL change can also change timestamps or pathname identity, that ordering can mask
the more specific unsafe state and contradict the original categorical contract.

At initial retained-directory validation and every later revalidation boundary, the repaired
classifier obtains retained-handle metadata and pathname metadata, plus the canonical pathname as
already frozen. It then applies this exact precedence:

1. if either metadata view is not an owner-owned directory with exact mode 0700, return
   `UnsafeDirectory`;
2. if the retained directory handle has an extended ACL, return `UnsafeDirectory`;
3. only then compare the original identity against both metadata views and compare the canonical
   pathname; any mismatch returns `IdentityChanged`.

A pathname that cannot be observed or canonicalized after the directory was retained remains
`IdentityChanged`. A safe replacement directory remains `IdentityChanged`. The classifier does
not open an untrusted replacement pathname merely to inspect its ACL.

## Exact test-only additions

The original C1 test-fixture exception is extended only to allow:

- one deterministic `before-open:<entry-name>` checkpoint after the initial `fstatat` and before
  entry `openat`;
- one deterministic `after-first-sweep` checkpoint after all descriptors and captured bytes are
  retained and the pure classification result is saved, but before the second enumeration;
- temporary FIFO creation with `libc::mkfifo` inside a test-owned temporary directory; and
- a fixed nonblocking test-only FIFO keeper descriptor preloaded with the one-byte ASCII sentinel
  `X`, so the race test cannot hang even if its behavioral failure is not the one under assertion.

These hooks remain `#[cfg(test)]`, only pause normal checks, and cannot bypass or alter a
classification result. They add no production symbol or capability.

The repair must add deterministic tests proving:

1. the exact entry-open flag constant contains `O_NONBLOCK`, and a regular-to-FIFO substitution at
   `before-open:execution-intent.json` returns `UnsafeEntry`; the keeper must still read the exact
   sentinel `X` afterward, proving the classifier did not consume FIFO payload bytes;
2. a regular-to-symlink substitution at that same pre-open checkpoint returns `UnsafeEntry`
   without following or opening the target;
3. a same-length in-place overwrite of an already-read primary at `after-first-sweep` returns
   `IdentityChanged`;
4. changing an already-read sidecar to mode 0644 at that checkpoint returns `UnsafeEntry`;
5. adding the existing fixed extended ACL to an already-read primary at that checkpoint returns
   `UnsafeEntry`; and
6. overwriting and then restoring the exact original primary bytes before resuming returns
   `IdentityChanged` through changed ctime;
7. changing the retained directory to mode 0755 at `after-first-sweep` returns
   `UnsafeDirectory`; and
8. adding the existing fixed extended ACL to the retained directory at that checkpoint returns
   `UnsafeDirectory`.

The existing path-replacement, directory-replacement, permission, ACL, symlink, hardlink, size,
digest, grammar, prefix, cross-binding, chronology, source-firewall, and non-macOS tests remain.
The non-test source firewall must assert that the entry `openat` uses the exact flag constant and
that the constant includes `O_NONBLOCK`.

## Acceptance gate

Acceptance requires all focused artifact tests, all C1/core tests, the full provider-free
`engram-eval` library and integration suites, strict Clippy, format, and whitespace checks to pass
using an external `CARGO_TARGET_DIR`; repository `target/debug` must remain absent. Fresh
independent conformance, security, and adversarial reviews of the exact repaired source must each
report P0=0 and P1=0. Any P0/P1 finding rejects the snapshot and requires another frozen repair;
no provider, authentication copy, daemon, listener, network, process, or live Engram mutation is
authorized by this addendum.
