# Native strict successor Stage C1 — frozen post-read precedence repair

Date: 2026-09-06 (Asia/Jerusalem)

## Status and exact boundary

This addendum supplements the original C1 frozen design and the artifact-race repair frozen at
SHA-256 `3871b2bbb226060624fe1eb5e7c13268e6b0ca0c2c8a65ba593f78755e2be89c`.
It authorizes only the two rejected categorical-precedence repairs and their deterministic tests
inside `engram-eval/src/native_successor_artifact.rs`. Every prior provider-free, read-only,
non-runnable, private, inert, non-atomic, and non-authorizing boundary remains exact.

## Unconditional post-reread boundary

During the retained-entry second sweep, the implementation must not use `?`, compare bytes or
digests, or otherwise return the reread result before completing the post-reread metadata boundary.
The exact order is:

1. complete the pre-reread entry safety, ACL, and identity boundary;
2. seek to byte zero;
3. in tests only, pause at `before-second-sweep-read:<entry-name>`;
4. capture the exact-length-plus-EOF reread as a `Result` without propagating it;
5. unconditionally complete the post-reread handle/path safety, ACL, and identity boundary;
6. propagate any post-reread boundary error first;
7. only then propagate the captured reread error; and
8. only then compare bytes and SHA-256 with the first sweep.

Therefore a current unsafe entry property returns `UnsafeEntry` even when the same interleaving
also causes a byte mismatch, identity mismatch, or read error. If the post-boundary is safe but its
identity changed, `IdentityChanged` wins. If that boundary is unchanged, the captured read result
and then byte/digest equality decide as previously frozen.

The test-only hook pauses after the pre-boundary and seek but before the reread. A deterministic
test changes the exact same-length primary bytes and its mode to 0644 while paused; the result must
be `UnsafeEntry`, proving both that the post-boundary always runs and that safety precedes the
captured byte mismatch. A second deterministic test truncates the primary to one byte and changes
its mode to 0644 at the same checkpoint; it must return `UnsafeEntry`, proving the post-boundary
also precedes propagation of a captured short-read error. The hook remains absent from non-test
builds and cannot alter a result.

## Unconditional directory-safety precedence

No fallible entry or namespace operation may mask a concurrently unsafe retained directory. For
each first-sweep entry read, each second-sweep entry verification, and each namespace
re-enumeration, the implementation captures the operation result without propagating it, then
unconditionally calls `RetainedDirectory::revalidate`.

The exact combination precedence is:

1. `UnsafeDirectory` from directory revalidation returns first;
2. otherwise the entry or namespace operation error returns;
3. otherwise any remaining directory-revalidation error returns; and
4. only then may the captured success value be used.

This preserves `UnsafeEntry` for FIFO, symlink, and other unsafe entry substitutions even when
their namespace mutation also changes directory identity; it preserves `UnsafeDirectory` when an
entry operation fails because the retained directory became unsafe; and it preserves ordinary
directory `IdentityChanged` when the entry or namespace operation itself succeeded.

A deterministic `before-open:execution-intent.json` test changes the retained directory from mode
0700 to mode 000. It must return `UnsafeDirectory`, even though both entry `openat` and fallback
`fstatat` may fail with access denied. The test restores mode 0700 after the classifier joins so
temporary-directory cleanup remains reliable. A second `after-first-sweep` mode-000 test proves
namespace re-enumeration cannot mask the same category.

## Exact regression-strengthening requirements

The production source firewall must remove ASCII whitespace from the inspected non-test source and
assert the resulting source contains exactly the entry call spelling
`libc::openat(self.file.as_raw_fd(),name_c.as_ptr(),ENTRY_OPEN_FLAGS)`, in addition to the exact
constant-value test. Counting identifier occurrences alone is insufficient.

The overwrite-then-restore test must restore the original mtime seconds and nanoseconds before
resuming, using only a test-owned path and fixed `utimensat` arguments. It must assert size and mtime
equal their originally captured values and ctime differs before resuming. The subsequent
`IdentityChanged` result therefore proves ctime detection rather than an incidental mtime change.

No other code, test, API, error, state, or design change is authorized.

## Acceptance gate

The repaired exact source must pass every original and repair-focused test repeatedly, the full
provider-free library and integration suites, strict Clippy, format, whitespace, external-target,
and repository-`target/debug` gates. Fresh independent security, conformance, and adversarial
reviews of the new exact source must each report P0=0 and P1=0 before C1 can be accepted.
