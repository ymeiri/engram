# Native strict successor Stage C1 — frozen handle-first boundary repair

Date: 2026-09-06 (Asia/Jerusalem)

## Status and exact boundary

This addendum supplements the original C1 design, race-repair SHA-256
`3871b2bbb226060624fe1eb5e7c13268e6b0ca0c2c8a65ba593f78755e2be89c`, and
post-read-repair SHA-256 `9d969461cad6c5ea7b4b1a35d46b737882194628e88101845383da6af5f35e43`.
It authorizes only a shared handle-first read boundary, directory handle-first ordering, and exact
deterministic tests in `engram-eval/src/native_successor_artifact.rs`. Every earlier private,
provider-free, bounded-read, non-runnable, inert, non-atomic, and non-authorizing boundary remains.

## One entry boundary at every observation point

Production must use one private entry-boundary function for post-open validation, both first-sweep
post-read boundaries, and both second-sweep pre/post-reread boundaries. Given the retained entry
descriptor, retained directory descriptor, descriptor-relative name, effective UID, and original
`FileIdentity`, it performs this exact order:

1. obtain retained-handle metadata; failure is `Unreadable`;
2. check the handle is a regular effective-UID-owned single-link exact-mode-0600 file; failure is
   `UnsafeEntry`;
3. check the retained handle has no extended ACL; failure is `UnsafeEntry`;
4. obtain descriptor-relative non-following `fstatat` metadata; failure is `IdentityChanged`;
5. check the pathname metadata is a regular effective-UID-owned single-link exact-mode-0600 file;
   failure is `UnsafeEntry`; and
6. compare both handle and pathname identity with the original identity; mismatch is
   `IdentityChanged`.

No fallible pathname observation may run before classifying already-observed retained-handle
safety and ACL state. Thus an unsafe retained inode renamed away remains `UnsafeEntry`; a safe
retained inode renamed away is `IdentityChanged`.

## First-sweep read completion

The original first-sweep double read is repaired to use the same unconditional completion rule as
the second sweep. For each bounded read:

1. capture the exact-length-plus-EOF read as a `Result` without propagating it;
2. unconditionally run the entry-boundary function;
3. return the boundary error first;
4. only then propagate the captured read error; and
5. only after both reads and boundaries succeed compare exact bytes and SHA-256.

The seek between reads is also captured without propagation, followed by the entry boundary; the
boundary error returns before the seek error. The existing first-sweep checkpoint named exactly as
the entry remains after the second read and before its post-read boundary.

A new test-only `before-first-sweep-read:<entry-name>` checkpoint occurs after post-open boundary
success and immediately before the first read. A deterministic test truncates the primary to one
byte and changes its mode to 0644 at that checkpoint; the first read is captured as an error, the
mandatory boundary observes unsafe mode, and the result is `UnsafeEntry`.

A test-only `before-second-first-sweep-read:<entry-name>` checkpoint occurs after the captured
inter-read seek and its boundary both succeed, immediately before the second read. A deterministic
test truncates the primary to one byte and changes its mode to 0644 there; the second read is
captured as an error and its mandatory post-read boundary must return `UnsafeEntry` first.

A second deterministic test pauses at the existing `execution-intent.json` checkpoint after the
second read, renames that retained inode outside the artifact directory, changes its mode to 0644,
and resumes. It must return `UnsafeEntry`, proving first-sweep post-read handle safety precedes the
now-missing pathname.

## Second-sweep and directory handle-first completion

The second-sweep pre/post-reread checks call the same entry-boundary function. A deterministic
`before-second-sweep-read:execution-intent.json` test renames the retained primary outside the
artifact directory, changes the retained inode to mode 0644, and resumes. The mandatory post-read
boundary must return `UnsafeEntry` before pathname absence or directory identity change.

For categorical symmetry, the second-sweep seek is captured without propagation and is followed
immediately by the shared entry boundary. That boundary error returns before the captured seek
error. Only after both succeed may the `before-second-sweep-read:<entry-name>` hook and reread run.

`RetainedDirectory::revalidate` must likewise classify its already-observed retained handle before
any fallible pathname operation:

1. obtain retained-handle metadata; failure is `Unreadable`;
2. check owner, directory type, and exact mode 0700; failure is `UnsafeDirectory`;
3. check the retained handle has no extended ACL; failure is `UnsafeDirectory`;
4. obtain pathname metadata; failure is `IdentityChanged`;
5. check pathname owner, directory type, and exact mode 0700; failure is `UnsafeDirectory`;
6. canonicalize the pathname; failure is `IdentityChanged`; and
7. compare both identities and canonical pathname; mismatch is `IdentityChanged`.

A `before-open:execution-intent.json` test and an `after-first-sweep` test each change the retained
directory to mode 000 and rename it to a sibling path before resuming. Both must return
`UnsafeDirectory`; each restores mode 0700 on the moved directory after the classifier joins.

## Exact source and scope rule

The non-test source must contain exactly one definition of the entry-boundary function, and all
post-open/first-sweep/second-sweep entry checks must call it rather than duplicate safety or
identity orderings. The whitespace-normalized source firewall must also assert the second-sweep
sequence captures the seek result, calls the shared boundary, propagates the boundary result, and
only then propagates the seek result. Tests may use only the existing deterministic hook mechanism
and test-owned temporary filesystem mutations already authorized by the prior designs. No new
production type, API, I/O capability, retry, lock, thread, process, network, provider, daemon,
authentication, writer, or authority is authorized.

## Acceptance gate

The exact repaired source must pass every original and repair-focused test repeatedly, full
provider-free library and integration suites, strict Clippy, format, whitespace, external-target,
and repository-`target/debug` gates. Fresh independent security, conformance, and adversarial
reviews must each report P0=0 and P1=0 before C1 acceptance.
