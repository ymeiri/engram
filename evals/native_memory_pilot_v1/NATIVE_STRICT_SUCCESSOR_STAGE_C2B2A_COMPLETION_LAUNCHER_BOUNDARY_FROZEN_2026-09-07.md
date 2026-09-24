# Native strict successor Stage C2B2a — completion-launcher boundary amendment

## Disposition

This is a frozen candidate boundary amendment, not an accepted implementation or execution
record. It authorizes nothing unless independent review of its exact SHA-256 returns P0=0/P1=0.
No launcher source may be created and no controller, provisioner, Cargo, build, provider, VM,
adapter, daemon, datastore or payload path may run before that acceptance.

This amendment is subordinate to, and changes only the exact boundary conflict identified below
in, these accepted artifacts:

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
```

Everything not expressly superseded here remains authoritative.

## Rejected-candidate ratchet

The prior exact candidate is retained only as rejected design evidence:

```text
SHA-256 983c2bc0da52e46caeb82cfe51e8de50a935e03debc83269cfe5ca913f2c66e4
lines   196
bytes   12184
```

Three independent exact-SHA reviews returned `P0=0/P1=2`, `P0=0/P1=3` and `P0=0/P1=4`. Their
union identified four defects: the outer launcher entry did not supersede the controller-only entry
boundary; the launcher and preflight record formed an undefined mutual-hash trust loop; focused
behavioral checks had no legal executable checker; and published files did not prove successful
outer-process completion. That candidate is not accepted and cannot authorize implementation or
execution.

The next exact candidate is likewise retained only as rejected design evidence:

```text
SHA-256 c471e20422a52609bb5d21e3730d9569b3c8ef9e2767df3505cdbd6907056a27
lines   366
bytes   24030
```

Three independent exact-SHA reviews returned `P0=0/P1=1`, `P0=0/P1=2` and `P0=0/P1=2`. Their
union found three defects: the early static-test exception did not supersede the addendum's final
non-runnable-support clause; static-test bytes lacked bounded direct capture and a normal process-
completion witness; and the text simultaneously forbade controller/provisioner execution while
claiming to run their production functions. That candidate is not accepted and cannot authorize
implementation or execution.

The third exact candidate is also retained only as rejected design evidence:

```text
SHA-256 c0df59c3e3353f196e0781fd2f69594c677aa2049512920b83ac83702700eec5
lines   418
bytes   27785
```

Three independent exact-SHA reviews returned `P0=0/P1=1`, `P0=0/P1=2` and `P0=0/P1=2`. Their
union found three defects: the pure-AST execution was not expressly included in the early support-
execution supersession; control-flow dominance did not prove exclusive validated-data dependence
at effect adapters; and the outer production observer lacked a deadline, bounds and residual-
scanner cleanup. That candidate is not accepted and cannot authorize implementation or execution.

The fourth exact candidate is retained only as rejected design evidence:

```text
SHA-256 70ee6d152168397f47de5cfffe58cf984cfd5c52809d2e3a85bb45c89382604b
lines   471
bytes   31854
```

Three independent exact-SHA reviews returned `P0=0/P1=1`, `P0=0/P1=0` and `P0=0/P1=0`. The sole
defect was that pre-execution inspection began at dispatcher reachability and did not expressly
cover Python's earlier module initialization. That candidate is not accepted and cannot authorize
implementation or execution.

The fifth exact candidate is retained only as rejected design evidence:

```text
SHA-256 47e930320a15bc0a8fd6222d3f2dfb98c2d88546c4d0297b368f7ce72eb6d76b
lines   495
bytes   33316
```

All three independent exact-SHA reviews returned `P0=0/P1=1`. Their one shared defect was that the
bootstrap allowlist inspected the `__main__` edge but did not permit the comparison and direct call
needed to reach entry validation. That candidate is not accepted and cannot authorize
implementation or execution.

## Conflict being repaired

The accepted Darwin addendum requires a reviewed independent launcher/capture implementation to
start `completion-audit-scan`, capture its exact successful fd-1 bytes and publish the completion
TSV and digest. The base freeze maps every host-side launcher or checker, apart from a deliberately
non-authoritative invoker, to `build-support/controller.py`; it also permits only six payload
support files. The scanner is the sole new controller mode, must perform no write or subprocess,
and cannot review or launch itself. `provision.py` has a separate closed six-mode grammar, and all
existing Rust orchestration sources are protected inputs outside the edit boundary.

Consequently the required authoritative launcher has no legal implementation location. Treating
manual shell or agent orchestration as that launcher would leave its capture and publication bytes
unreviewed. Embedding executable source in an audit record or `python -c` argv would improperly turn
a non-input audit record or caller assertion into executable authority. This amendment adds one
explicit source location and no other implementation surface.

## Exact supersession and path boundary

For this launcher implementation preflight and the post-`pre-runtime-finalize` completion audit
only, this amendment supersedes:

- base lines 575–584 so that the terms **completion-audit launcher** and **implementation-preflight
  static-test checker** mean the two closed modes of the independent source below, not
  `controller.py`; every other controller, verifier, launcher and checker mapping is unchanged;
- base lines 586–615 only for the two exact launcher entries below. The original controller entry
  and its non-authoritative-invoker semantics remain unchanged for every controller invocation;
- base lines 177–230 only to append one host-only support source and the two non-input audit
  records listed below;
- Darwin-addendum lines 43–55 and 674–686 only to append this accepted boundary amendment and its
  accepted review as the scanner's two additional exact-hash trust-anchor reads; and
- Darwin-addendum lines 663–716 only to give the already-required reviewed launcher/capture and
  publication mechanism the exact source location and invocation boundary below;
- Darwin-addendum lines 764–768 only to add the directly observed outer-launcher completion facts
  to the already-authorized completion-audit review record; and
- Darwin-addendum lines 825–882 only to authorize the exact preflight-only static-test exception
  and finite implementation-preflight authority construction below, including execution of only
  the exact purity-proven controller/provisioner function ASTs in the isolated checker namespace.
  No module top level, mode, adapter or other support byte may execute and no other acceptance
  meaning is changed; and
- Darwin-addendum lines 890–893 only so that the newly added launcher's exact
  `implementation-preflight-static-test` mode may run under the sole effect-free exception below
  before final implementation-preflight acceptance and may execute only the exact allowlisted,
  purity-proven controller/provisioner function-definition ASTs in memory. Every prohibition on
  controller/provisioner module, top-level, mode or adapter execution and on production launcher,
  Cargo, build, provider, VM, daemon, adapter, datastore, payload and runtime execution remains
  unchanged.

The sole added implementation path is:

```text
evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
```

The two appended non-input audit records are exactly:

```text
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_REVIEW_2026-09-07.md
```

No directory, second source, test source, package, dependency, generated payload object or payload
path is added. The launcher is outside `engram-eval/native-c2b2a-payload`, so the accepted current
29-object and virtual post-manifest 30-object payload sidecars and their identities remain exactly
unchanged. Cargo, compilers, build scripts, provisioner children, policy probes, Linux guests and
payload runtime processes cannot read the launcher or either audit record. They cannot satisfy a
payload, source, build or runtime gate.

The launcher is a regular non-executable repository source: UID 502, GID 20, mode `0644`, link
count one, nonsparse, no extended ACL and the exact preflight-bound `provenance-fresh` xattr
profile. It is added to the descriptor-walked repository metadata authority and protected source
closure. Its complete content SHA-256, LF line count and byte count are later bound by the
implementation-preflight record and its independent review. It is never normalized after
creation. Any later byte or metadata change invalidates the implementation verdict.

## Finite implementation-preflight authority

The exact existing combined implementation-preflight path is fixed as:

```text
/Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IMPLEMENTATION_PREFLIGHT_REVIEW_2026-09-07.md
```

Its final bytes contain exactly one LF-terminated ASCII field whose prefix is
`launcher_source_sha256=` followed by exactly 64 lowercase hexadecimal bytes. No other line uses
that field name. Apart from source line and byte counts proven invariant under the same-width
constant substitution below, no other record field or expected evidence value is computed from the
launcher bytes, AST, embedded semantic digest or final launcher hash. Define the **preflight
semantic projection** by replacing only those 64 value bytes with exactly 64 ASCII `0` bytes and
leaving every other byte unchanged. A missing or duplicate field, wrong width, uppercase or non-
hex byte, missing final LF, or second possible replacement location is terminal. The SHA-256 of
the complete projected byte stream is the **preflight semantic digest**. The record contains
neither that digest nor its own raw SHA-256.

The launcher source contains one fixed `IMPLEMENTATION_PREFLIGHT_SEMANTIC_SHA256` constant equal
to the semantic digest. The finite construction order is exact:

1. freeze every combined-record byte, including expected static-test evidence, with only the
   launcher-hash field represented by the fixed 64-zero token;
2. compute the preflight semantic digest over those bytes;
3. replace the launcher's same-width 64-zero semantic-digest placeholder with that digest while
   preserving its device, inode, UID, GID, mode, link count, byte count, LF line count, xattr
   profile and ACL state;
4. freeze the complete launcher source and compute its raw SHA-256;
5. replace only the record field's 64-zero value with that launcher-source SHA-256; and
6. project the resulting final record again and require the same semantic digest.

These steps are not iterated. The static-test vector definitions, expected output and their
digests contain no launcher-source hash or value derived from it. Fixed-width projection leaves
the record's line and byte counts unchanged. Any drift in the stable launcher fields enumerated in
step 3 during the sole constant substitution aborts that candidate before execution; advancing
mtime or ctime is neither an equality witness nor a failure. No field is repaired or normalized.

At launcher entry both modes use fixed descriptor-walked repository paths, not caller-selected
paths, to stable-read the launcher source and combined record. The static-test mode checks the exact
candidate metadata captured by preflight; production requires the externally accepted metadata.
Both require the record field to equal the actual complete launcher-source SHA-256, project the
record exactly as above, and require its digest to equal the embedded semantic digest. They repeat
both stable reads and comparisons at their final trusted boundary. No argv, environment, pathname-
only match, caller assertion, record-supplied semantic digest or additional hash field can satisfy
or override either comparison. For the static mode this self-consistency remains candidate evidence
until external review; only after acceptance does the record supply authoritative controller,
interpreter, tool, trust-anchor, evidence-root, destination and scanner-argv identities.

The record may summarize rejected-candidate findings known before its final bytes are frozen. The
independent clean verdicts over its final exact SHA and the final launcher/support SHAs are external
acceptance observations and are never appended to or summarized inside those final bytes. Thus no
final reviewer output is asked to review a file containing that same output.

## Closed preflight-only static test

After this amendment and its review are accepted, and after the launcher and combined-record
candidate are frozen by the finite construction above, exactly one launcher mode may execute
before final implementation-preflight acceptance:

```text
/usr/bin/env -i LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=0 HOME=/var/empty \
  __CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0 \
  <exact-python> -I -B -S \
  <exact-native_c2b2a_completion_audit_launcher.py> implementation-preflight-static-test
```

It accepts no further argv. This is the sole narrow exception to the no-support-execution gate.
Before it runs, independent non-executing exact-SHA inspection must confirm that its dispatcher and
complete reachable call graph cannot reach a child, network API, filesystem write, production
capture adapter or production publication adapter. The selected interpreter and fixed read inputs
also receive read-only host identity capture. Its result is candidate evidence only until the
combined record and exact implementation sources receive their required external reviews. Failure
or any later source or record edit invalidates every observation and requires a new candidate
ratchet. No controller or provisioner module or mode, Cargo, compiler, build, provider, VM, adapter,
daemon, datastore, payload or other launcher mode may run under this exception.

That inspection begins at Python module evaluation, not at the later dispatcher call. It freezes
and checks every launcher module-level AST statement, exact import and imported initialization
closure, decorator, default, annotation, class body, comprehension, assignment, registration hook,
call, `__main__` guard and dispatch edge. Before entry validation, only exact reviewed inert import,
immutable-literal binding and function-definition statements, the one exact
`__name__ == "__main__"` comparison and one direct call to the frozen zero-argument entry function
may execute. That entry function's first reachable operations perform the complete argv,
environment, cwd, umask and fd validation; argument construction, wrapper dispatch, alternate
entry calls and any earlier effect are forbidden. After the validated entry function returns, only
its direct `SystemExit` propagation may execute. No module-initialization path may write, spawn,
signal, network, register a deferred effect, load unlisted code, mutate an external path or rebind
a checked function. Every imported module and native image must be in the preflight-bound standard-
library/libSystem closure with reviewed initialization behavior. The dispatcher-reachable proof is
in addition to, not a substitute for, this complete bootstrap proof.

The mode validates the exact six-variable environment, cwd `/var/empty`, incoming umask `0077`,
read-only `/dev/null` fd 0, distinct nonseekable one-way fd 1 and fd 2 pipes, and absence of open fds
above 2. Apart from interpreter loading, it may stable-read only its own source, the combined
record, `controller.py` and `provision.py` through their four fixed no-follow repository paths. It
never imports either support module or executes module top-level code, starts no child, uses no
signal or network API, consults no ambient path, and performs no filesystem, xattr or ACL write.

Launcher-local vectors call the exact production authority parser, semantic projector, entry
validator, capture-transition machine, completion-TSV parser, digest serializer, publication-
transition machine and recursion guard already loaded in the launcher process. A test-only
duplicate implementation is forbidden. Static source checks require every production adapter to
feed observations into, consume the result of and be control-flow dominated by those exact
functions. For every acceptance branch and effectful adapter argument, an exact backward def-use
slice must prove exclusive variable-data dependence on the accepted canonical return plus frozen
literals. The original observation, its aliases and any second read or parse become unusable after
validation; using, rereading, reparsing or substituting them at an adapter is terminal. Rebinding,
alias substitution, ignored or no-op-consumed results, TOCTOU path reopening, exception swallowing
and a parallel unchecked path are likewise terminal. Descriptor effects must retain the same held
descriptor whose identity is carried by the accepted return.

Controller/provisioner vectors use one explicitly authorized, in-memory AST-extraction boundary.
The combined record freezes an exact qualified-function and literal-constant allowlist. Before any
extraction, the checker proves that every selected definition has no decorator, default expression,
annotation, closure, global/nonlocal declaration, import, dynamic lookup, I/O, process, signal,
network, metadata or adapter call; that every reachable name is a selected pure definition, an
immutable literal or an exact pure builtin in the closed allowlist; and that no selected name is
assigned, deleted, aliased or rebound elsewhere. Literal constants are constructed only with
`ast.literal_eval`. The checker compiles one synthetic `ast.Module` containing only copies of the
selected, purity-proven `FunctionDef` nodes and executes those definitions in an isolated namespace
with the exact minimal builtin table; it then invokes their bodies only through frozen vectors. No
other original module-level statement, import, class body or production adapter is compiled or
executed. This extracted-function execution is the sole dynamic-code exception and cannot escape
the static-test mode.

The checker builds exact control-flow and def-use tables for the controller and provisioner and
requires every applicable production callsite to invoke those same selected definitions, consume
their return values before any adapter effect, and have no bypass, alternate definition or hidden
exception path. Every acceptance decision and effect argument must have an exhaustive backward
slice whose only variable source is that canonical return; frozen literals are the only other
inputs. The source proof rejects reuse or aliasing of raw input, rereads, reparses, second path
lookups, second metadata observations and no-op consumption of validated data. Immutable in-memory
negative vectors then execute the extracted production pure functions themselves. Thus launcher-
local behavior is tested directly in the loaded launcher, controller/provisioner pure behavior is
tested from its exact production AST, and producer adapter effects remain source-proved and subject
to later runtime qualification rather than falsely treated as simulated.

Its closed vectors cover every focused static case required by Darwin-addendum lines 866–875 and
the malformed authority, semantic-hash, argv, environment, fd, inclusive stream threshold, EOF,
deadline, wait status, process-group, canonical TSV, destination race, short-write, reopen drift,
digest and recursion cases required here. The combined record freezes the exact ordered vector
names, positive count, canonical vector-definition stream and digest.

On success fd 2 is empty and fd 1 is one canonical LF-terminated TSV stream with schema
`c2b2a-completion-launcher-static-test-v1`, one ordered `case` row per frozen vector, a summary with
exact counts and vector digest, and final `valid=true`. The combined record freezes its exact row
grammar, complete expected bytes, line count, byte count and SHA-256. None depends on the launcher
self-hash. A failed assertion emits no summary or `valid=true`, exits nonzero and cannot become
evidence.

Each of at least three independent final exact-SHA reviewers directly starts the exact static-test
argv as its process parent; a second-hand transcript or caller-supplied byte buffer is insufficient.
For every observation the reviewer starts a fresh process group, concurrently drains both distinct
pipes through EOF, enforces an exact 120-second `CLOCK_MONOTONIC` deadline, an inclusive
16,777,216-byte fd-1 ceiling and 4,096-byte fd-2 ceiling, and completely terminates, drains and reaps
the group on timeout, overflow or error. Acceptance requires complete EOF, fd 1 byte-for-byte equal
to the record's expected stream, empty fd 2, and normal process exit zero with no signal, core,
stopped, continued or residual process-group state. A valid-looking stream followed by failure,
signal, hang or incomplete reap is terminal. Every direct capture/status observation remains
external and is not written back into the final record; all reviewers must report P0=0/P1=0 over
the same source, record and observed result.

## Closed production launcher role

The launcher is invoked only through this top-level environment-clearing shape, whose complete
absolute argv is frozen by implementation preflight:

```text
/usr/bin/env -i LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=0 HOME=/var/empty \
  __CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0 \
  <exact-python> -I -B -S \
  <exact-native_c2b2a_completion_audit_launcher.py> completion-audit-publish
```

It accepts no further argv. The semantically authenticated combined record supplies the one exact
`completion-audit-scan` argv array; the non-authoritative invoker cannot supply or alter that array.

The non-authoritative host invoker supplies cwd `/var/empty`, umask `0077`, fd 0 as read-only
`/dev/null`, fds 1 and 2 as distinct nonseekable one-way diagnostic pipes, and every fd greater
than 2 closed or `CLOEXEC`. At its first trusted boundary the launcher validates the exact argv,
six-variable environment, cwd, incoming umask and complete fd table, resets umask `0077`, and
verifies its exact interpreter, source, accepted design/review records, implementation-preflight
authority and repository/root closure. It repeats every source, tool, audit-record and root hash or
identity check at exit. The repository audit parent retains its device/inode, ownership, mode,
xattr and ACL identity while its sole permitted entry delta is the exact absent-to-published TSV
and digest pair; volatile directory timestamps and accounting fields are not equality witnesses.
A mismatch is terminal. On success its own stdout and stderr are both empty; on failure they remain
diagnostic-only and cannot satisfy a gate.

The launcher has exactly the two closed modes defined here: the preflight-only static test and
production `completion-audit-publish`. It cannot be imported as authority by the controller or
provisioner. Neither mode can invoke a shell, resolve a program through `PATH`, use the network,
load a third-party package, run Cargo or another support mode, remove or synthesize an xattr, alter
an ACL, or inspect an unlisted repository path. The static-test mode cannot call any production
filesystem, child-process, capture-adapter or publication entry point and cannot satisfy a
completion gate.

`completion-audit-publish` is the sole production mode. Its sole child is the exact
`completion-audit-scan` invocation frozen by the Darwin addendum and semantically authenticated
from the combined record. Its sole writes are the exact completion TSV and digest through the
already-frozen descriptor-relative create-new publication protocol. It never creates or writes the
later independent completion-review record.

Before spawning, it requires a successful sealed `pre-runtime-finalize` result and final
acquisition evidence-parent state through the exact scanner-authority root set, requires the four
accepted design/review trust anchors at their exact hashes, requires the finite semantic
implementation-preflight authority above, and requires the completion TSV, digest and completion-
review destinations absent. These checks do not duplicate or weaken the scanner: the scanner
independently revalidates its own complete trust and evidence closure after exec.

The launcher implements exactly the accepted Darwin-addendum capture contract: no shell; one new
scanner process group; distinct bounded fd-1/fd-2 pipes drained concurrently through EOF; the exact
900-second `CLOCK_MONOTONIC` deadline; inclusive 4,294,967,296-byte stdout and 4,096-byte stderr
ceilings; complete process-group termination and reaping on failure; normal exit zero and empty
stderr on success; whole-buffer canonical TSV parsing before authority; and rejection of timeout,
overflow, partial read, truncation, trailing data, signal, core, nonzero status or reap failure.

Independently of that child interval, production enforces an exact 1,100-second monotonic deadline
from its first trusted entry check through its final source/root/parent recheck and exit-ready state.
On an overall deadline or catchable termination it prevents further child creation, terminates,
drains and reaps the exact scanner group if one exists, and exits nonzero. It never treats a
partially or fully published pair as success after such a failure.

Only the complete validated fd-1 buffer may become candidate TSV authority. Publication inherits
the accepted create-new `O_EXCL|O_NOFOLLOW`, mode-`0600`, held-parent-descriptor, full-write, sync,
close, reopen, reparse, rehash and parent-sync order. The digest is derived only from the validated
buffer. A pre-existing destination, short write, metadata drift or partial publication is terminal
and requires a separately reviewed recovery freeze; the launcher never repairs or deletes it.

## Outer-launcher completion witness

The process that starts `completion-audit-publish` is mechanically non-authoritative: its source,
ancestry or assertions cannot satisfy a gate. The already-authorized independent completion-audit
review must nevertheless be performed by a reviewer that directly starts the one exact outer argv,
requires all three completion paths absent before start, and places the launcher alone in a fresh
owned process session and process group. It concurrently drains both outer pipes through EOF,
enforces an exact 1,200-second `CLOCK_MONOTONIC` deadline and inclusive 1,048,576-byte ceiling on
each outer stream, and observes the outer process's complete wait status. The reviewer cannot rely
on a shell transcript, an agent summary, a pre-existing pair or a status reported by the launcher
itself.

Implementation preflight freezes the exact Darwin process-table API and identities used by the
reviewer to enumerate the fresh session as `(pid, process-start identity, ppid, pgid, sid, argv)`.
The reviewed launcher is the sole initial member and its exact scanner group is the only permitted
descendant group. On outer timeout, overflow, read error or abnormal status, the reviewer first
prevents the outer group from creating another child, enumerates the complete fresh session,
terminates both the outer and every observed scanner group with the frozen TERM/grace/KILL
sequence, continues draining, reaps its direct child and polls until no process with that session
identity remains. Failure to enumerate, terminate, drain, reap or prove both groups absent is
terminal; it never triggers a retry or permits either completion file to pass.

The exact completion-audit review record already authorized by the Darwin addendum must bind the
outer launcher's final source SHA-256, embedded semantic digest, combined-record raw SHA-256, exact
argv/interpreter/environment/cwd/umask/fd entry, start and reap observations, empty successful fd-1
and fd-2 captures, and normal outer exit zero with no signal, core, stopped or continued terminal
state. It also binds the two deadlines, stream ceilings, fresh session/group identities and the
final empty session inventory. It must confirm from the reviewed production control flow that zero
exit is reachable only after complete scanner-group reaping, both published files have been
reopened and validated, and every final launcher source, tool, trust-anchor, root and repository-
parent check has passed.

The TSV and digest pair cannot pass independent review without that direct outer completion
witness. A missing, second-hand, inconsistent, nonzero, signaled, cored, stopped, continued or
unreaped observation, outer overflow/timeout or residual outer/scanner session member is terminal
even if both files contain valid bytes. A failure after either publication leaves the existing
fail-closed partial-publication state and requires a separately reviewed recovery freeze; no rerun
or inferred success is allowed. Outer stdout or stderr never becomes completion data. The launcher
still cannot create or write the review record.

## Implementation-preflight additions and sole early-execution exception

Before `completion-audit-publish`, any controller or provisioner mode, or any Cargo, build, probe,
VM, provider or runtime command may run, the existing combined implementation-preflight record and
its external independent exact-SHA P0=0/P1=0 reviews must additionally prove:

1. the exact accepted hash, line and byte identities of this amendment and its review;
2. the launcher's complete source identity, mode, ownership, profile, ACL and descriptor-walked
   repository/traversal metadata before and after every check;
3. the exact finite semantic-authority construction above, including field uniqueness, fixed-width
   projection, embedded digest, actual-self-hash comparison and entry/exit repetition;
4. the exact Python, `/usr/bin/env`, `/dev/null`, `/var/empty`, libSystem symbols, both argv
   grammars, cwd, umask, environment and fd-entry contracts;
5. a closed parser for the authenticated record and scanner argv, including exact four-
   trust-anchor hashes, controller/interpreter identities, final evidence roots, destination paths
   and expected repository parent identity;
6. static whole-source and pre-dispatch module-bootstrap proof of exactly one production scanner
   child, only inert pre-entry initialization, no shell, `PATH`, network, third-party import or
   unlisted read, no existing-path mutation or xattr/ACL mutation API, and exclusive validated-
   return data dependence at every acceptance branch and effect adapter;
7. exact process-group/session, pipe, nonblocking/concurrent drain, internal 900-second, overall
   1,100-second and reviewer 1,200-second deadlines, all byte ceilings, Darwin process enumeration,
   wait-status, EOF, termination and complete-reaping logic for every success and failure edge;
8. exact whole-TSV validator and digest serializer agreement with the scanner's preflight-frozen
   row grammar, bounds and per-property authority table;
9. exact held-descriptor publication order, create-new flags/modes, inode and containment checks,
   sync/close/reopen/rehash/reparse checks and fail-closed partial-publication state;
10. the exact static-test vector table, launcher-local production-function map, controller and
    provisioner extracted-pure-function allowlist, AST/control-flow/def-use invariants, canonical
    success output, bounded direct-reviewer captures, statuses and digests above; and
11. proof that neither launcher mode can be reached by any Cargo, build, policy-probe, guest,
    provider, adapter, daemon, datastore or payload command and that the source changes neither
    accepted payload sidecar identity.

The one already-authorized path
`NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IMPLEMENTATION_PREFLIGHT_REVIEW_2026-09-07.md`
is the combined implementation-preflight evidence and acceptance candidate; no second preflight or
test path is added. It contains complete preflight facts, expected static-test evidence and the
rejected-candidate ratchet. Its final immutable bytes receive the required external independent
exact-SHA review; clean final reviewer verdicts are not written back into it.

Only the exact `implementation-preflight-static-test` invocation above may run before those final
reviews, and only under its complete exception. After its clean captured result and the external
reviews return P0=0/P1=0, production launcher, controller, provisioner and other already-authorized
implementation modes become eligible under their existing later gates. The launcher source
requires the same final independent exact-SHA implementation review as controller, provisioner and
the other hand-authored support files. Any source or combined-record edit invalidates the static-
test observation and every verdict.

## Acceptance boundary

This amendment is acceptable only if independent exact-SHA review returns P0=0/P1=0 and confirms
that it supplies the one missing implementation location without relaxing the accepted scanner,
capture, publication, source, metadata or execution gates.

Even after acceptance, the launcher does not yet exist and no implementation or production mode is
authorized. After the launcher and combined-record candidate are constructed, only the exact
effect-free `implementation-preflight-static-test` exception above may run before final preflight
acceptance. Build/source identity, production completion audit, Linux runner and runtime remain
separate gates. Repository `target/debug` must remain absent, user-owned changes must remain
preserved, and no stage may claim C2B2a, the pilot or flagship goal completion early.
