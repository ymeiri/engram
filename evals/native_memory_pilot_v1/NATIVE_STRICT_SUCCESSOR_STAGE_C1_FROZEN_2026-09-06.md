# Native strict successor Stage C1 — frozen structural-policy slice

Date: 2026-09-06 (Asia/Jerusalem)

## Status and exact authority boundary

This repaired design is frozen for one more exact audit before implementation. It authorizes only
private, provider-free structural policy and read-only classification. It does not authorize a run
plan, daemon, listener, socket, HTTP or MCP client, real Engram call, request forwarding, semantic
claim, provider, authentication copy, secret generation or materialization, environment mutation,
process, thread, async task, VM, live adapter/settings change, or CLI/public run surface.

C1 contains only:

1. exact lexical newtypes and a pure family/phase-to-artifact policy table;
2. a test-only static launch-template framing over typed synthetic slot names;
3. exact minimal hash-chained artifact payloads and a conservative read-only classifier; and
4. a single-owner in-memory JSON-RPC parser/sequence automaton using deliberately synthetic calls.

Successor run-plan payloads, host bindings, real phase request/response contracts, domain semantic
projection, policy/relay digests tied to a plan, live collection, mutation writers, activation,
daemon ownership, and transport are deferred.

## Exact source boundary

The only authorized implementation files are:

```text
engram-eval/src/native_successor_policy.rs
engram-eval/src/native_successor_artifact.rs
engram-eval/src/native_successor_relay.rs
engram-eval/src/native_document.rs
engram-eval/src/lib.rs
```

Changes to `native_document.rs` are limited to crate-private reuse of its existing strict duplicate-
rejecting JSON byte parser, including a categorical crate-private error wrapper needed by the
synthetic automaton. Its public family/document-kind enums, accepted values, loader behavior, and
structural audit remain unchanged. C1 introduces no new public document kind.

The wrapper is a domain-neutral `strict_json_value_from_slice_categorized` returning a private
`StrictJsonReadError::{MalformedJson, DuplicateKey, TrailingData}`. It must preserve the existing
loader's behavior; the existing loader may continue mapping those categories to its current
`EvalError`. This wrapper is the only parser authority shared with C1.

`lib.rs` adds only private module declarations. `native_successor.rs` and the CLI remain unchanged.
No C1 type is re-exported. C1 artifact payloads must not implement or reference
`NativeSuccessorRunPlanPayload`, `PreparedExecution`, or any execution state.

The non-test C1 modules may use `native_document` and basic domain-independent standard-library
parsing, hashing, and filesystem-read types. They must not reference `native_successor_core`,
`native_execution`, `native_runner`, `native_pilot`, `native_correction`, `native_isolation`,
`native_vm`, CLI daemon/proxy code, network libraries, process APIs, async runtimes, CSPRNG APIs,
environment APIs, filesystem mutation APIs, callbacks, function pointers, trait executors, futures,
channels, or synchronization primitives.

Bidirectional whole-crate source tests require:

- no C1 file refers to an execution, historical runner, command, transport, or public facade; and
- `main.rs`, `native_successor.rs`, `native_successor_core.rs`, `native_execution.rs`, every
  historical native module, and every existing public module other than `lib.rs` contain no C1
  module or type name.

The `lib.rs` declarations are the only permitted reverse references.

### Exact test-fixture exception

The authorized files may contain `#[cfg(test)]` fixture modules. Only those test-only modules may
use `tempfile`, create/write/rename/remove temporary fixture entries, set fixture permissions,
create fixture symlinks or hardlinks, add an extended ACL to a temporary fixture, and use one
deterministic test-only hook coordinated by `std::thread` and `std::sync` to replace a path between
the classifier's retained-descriptor checkpoints. The hook cannot exist in a non-test build and
cannot alter classification logic or bypass a check; it only pauses at a checkpoint so the test
can mutate the temporary pathname before normal revalidation continues.

On macOS, the ACL fixture may invoke only the existing fixed test operation
`/bin/chmod +a "everyone allow write" <temporary-fixture-path>`; no provider-controlled value is
used. Test cleanup is limited to the temporary directory owned by that test. Even under
`#[cfg(test)]`, provider, network, daemon, authentication, secret, environment mutation, arbitrary
process execution, async runtime, and real Engram calls remain forbidden.

Source tests inspect the non-test source region and compile the library without `cfg(test)` to prove
the exception is absent from production. Separate fixture tests prove the hook has no non-test
symbol and that every retained-descriptor revalidation still runs after it.

## Exact validated newtypes

Every constructor rejects rather than normalizes invalid input. No type implements permissive
`From<String>` or `From<PathBuf>` conversions.

- `C1Sha256`: exactly 64 lowercase hexadecimal ASCII bytes. Canonical digest framing uses the
  decoded 32 bytes, while JSON fields use the 64-byte lowercase spelling.
- `C1ExecutionId`: 1-64 lowercase ASCII alphanumeric or internal hyphen bytes; first and last byte
  alphanumeric.
- `C1PolicyId`: 1-64 lowercase ASCII alphanumeric, dot, underscore, or internal hyphen bytes; first
  and last byte alphanumeric.
- Test-only `C1StaticToken`: 1-4096 valid UTF-8 bytes, no NUL or ASCII control byte.
- Test-only `C1EnvironmentKey`: 1-64 ASCII bytes matching `[A-Z_][A-Z0-9_]*`.
- `C1RequestStringId`: 1-64 ASCII letters, digits, dot, underscore, or hyphen.

These types confer no filesystem, process, execution, authentication, or replay authority.

## Exact phase/artifact policy

`C1PhaseArtifactPolicy::derive(family, phase)` is one exhaustive private match. No field-wise
constructor exists. It returns only structural labels: policy ID, policy digest, partition
presence, synthetic relay shape, mutation-journal presence, and exact artifact order.

The policy ID is `native-phase-artifact-policy-v1`. Its canonical bytes are UTF-8, include the
first domain line and final newline, and are exactly:

```text
engram-native-phase-artifact-policy-v1
native_correction_v1/correction_operator|partition=required|relay=none|mutation=required|artifacts=intent,dispatch,action,terminal
native_correction_v1/correction_proposal|partition=required|relay=synthetic_mutation_then_read_v1|mutation=required|artifacts=intent,dispatch,action,terminal
native_correction_v1/correction_retrieval|partition=required|relay=synthetic_single_read_v1|mutation=forbidden|artifacts=intent,action,terminal
native_stale_isolated_v1/stale_activation|partition=required|relay=none|mutation=required|artifacts=intent,dispatch,action,terminal
native_stale_isolated_v1/stale_evaluation|partition=required|relay=synthetic_single_read_v1|mutation=forbidden|artifacts=intent,action,terminal
native_stale_isolated_v1/stale_teaching_engram|partition=required|relay=synthetic_single_mutation_v1|mutation=required|artifacts=intent,dispatch,action,terminal
native_stale_isolated_v1/stale_teaching_native|partition=forbidden|relay=none|mutation=forbidden|artifacts=intent,action,terminal
```

The policy digest is SHA-256 of those exact bytes:
`70a8ddb1722e69009c05964df6f641b219f42c580180bd59dd8f9294f6533abb`. Family/phase substitutions
and every unlisted combination reject.

The synthetic relay labels deliberately do not name an Engram tool or action and cannot be mapped
to one by C1. The action artifact denotes only an inert observed-action fixture.

## Exact test-only static launch-template framing

`C1StaticLaunchTemplate`, its token/environment types, and its only constructor exist under
`#[cfg(test)]`. They are synthetic digest fixtures, not a production commitment or proof that
arbitrary literal text is non-secret. C1 has no production API accepting an argv or environment
literal. The test-only template contains:

- one already-validated 32-byte executable-binding digest;
- zero to 64 ordered `C1StaticToken` argv values;
- one already-validated 32-byte working-directory-binding digest; and
- zero to 64 environment entries sorted by key.

An environment value is exactly `Literal(C1StaticToken)` or one `C1SyntheticSlot`:

```text
c1_synthetic_daemon_slot
c1_synthetic_relay_slot
```

`C1_SYNTHETIC_DAEMON_SLOT` accepts only `c1_synthetic_daemon_slot`;
`C1_SYNTHETIC_RELAY_SLOT` accepts only `c1_synthetic_relay_slot`. Those keys reject literals and
the wrong slot. A slot rejects every other key. Duplicate keys reject. C1 has no type or method for
a runtime secret value.

The digest input is exactly:

```text
ASCII "engram-native-static-launch-v1" followed by NUL
32 raw executable-binding digest bytes
argv count as unsigned 64-bit big-endian
for each argv: unsigned 64-bit big-endian byte length, then UTF-8 bytes
32 raw working-directory-binding digest bytes
environment count as unsigned 64-bit big-endian
for each entry in ASCII-key order:
  unsigned 64-bit big-endian key length, then key bytes
  one tag byte: 0 for literal, 1 for slot
  unsigned 64-bit big-endian value length, then literal UTF-8 or slot spelling bytes
```

The result is lowercase SHA-256. Literal and slot encodings are domain-separated by the tag byte.
No runtime value can affect this synthetic vector because no such input exists. Arbitrary literals
could themselves contain sensitive text, so C1 claims only framing correctness; the tests use
fixed non-secret fixtures and scan their serialized values for a canary. Actual secret-independent
launch authority remains entirely deferred.

The normative static-template golden vector uses an executable-binding digest of 32 zero bytes,
argv `c1`, `--fixture`, a working-directory-binding digest of 32 `0x11` bytes, and these two
environment entries supplied in either order:

```text
C1_LITERAL = Literal("fixture")
C1_SYNTHETIC_DAEMON_SLOT = Slot(c1_synthetic_daemon_slot)
```

Its framed byte length is 237 and its SHA-256 is
`2ce018e48ec3c82b9ff3855fa072815785adc3f200fcef1624d0a058f4477e41`.

## Exact artifact envelopes and payloads

C1 uses only existing successor outer kinds:

- intent: `document_kind=execution_intent`;
- dispatch and action: `document_kind=audit` plus a private exact `artifact_kind`;
- terminal: `document_kind=terminal_receipt`.

The existing public `NativeSuccessorDocumentKind` is unchanged. Each primary is one strict
duplicate-rejecting UTF-8 JSON object followed by one final newline; no other trailing bytes are
accepted. C1 does not claim, expose, or require a JSON writer or one canonical member order:
insignificant JSON whitespace and object-member order may vary, and the sidecar always hashes the
exact primary bytes observed through the retained descriptor. Each payload uses
`serde(deny_unknown_fields)` and must contain exactly the fields listed below; it does not repeat
the outer successor `family` or `schema_version`. Each sidecar is exactly the primary's 64-byte
lowercase SHA-256 plus one newline. Primary size is 1-1,048,576 bytes; sidecar size is exactly 65
bytes.

Fixed pair names are:

```text
execution-intent.json
execution-intent.json.sha256
mutation-dispatch-001.json
mutation-dispatch-001.json.sha256
phase-action-receipt.json
phase-action-receipt.json.sha256
terminal-receipt.json
terminal-receipt.json.sha256
```

The exact payload field sets are shown in their normative declaration order below. Declaration
order is not a byte-order requirement. There is no additional field.

```text
intent:
  document_kind = execution_intent
  artifact_schema_version = 1
  execution_id
  plan_sha256
  phase
  sequence
  previous_artifact_sha256
  phase_policy_id = native-phase-artifact-policy-v1
  phase_policy_sha256
  static_launch_template_sha256
  declared_started_unix_ms
  declared_terminal_unix_ms

dispatch:
  document_kind = audit
  artifact_kind = mutation_dispatch
  artifact_schema_version = 1
  execution_id
  plan_sha256
  phase
  sequence
  previous_artifact_sha256
  ordinal = 1
  request_sha256
  created_unix_ms

action:
  document_kind = audit
  artifact_kind = phase_action_receipt
  artifact_schema_version = 1
  execution_id
  plan_sha256
  phase
  sequence
  previous_artifact_sha256
  ordinal = 1
  request_sha256
  response_projection_sha256
  outcome = success | failure
  started_unix_ms
  completed_unix_ms

terminal:
  document_kind = terminal_receipt
  artifact_schema_version = 1
  execution_id
  plan_sha256
  phase
  sequence
  previous_artifact_sha256
  intent_sha256
  dispatch_sha256 = null | exact digest as required by policy
  action_sha256
  outcome = success | failure
  timing_claim = timely | late
  completed_unix_ms
```

`phase` is exactly one of `correction_operator`, `correction_proposal`,
`correction_retrieval`, `stale_activation`, `stale_evaluation`,
`stale_teaching_engram`, or `stale_teaching_native`; aliases, case changes, and every other value
reject. The outer successor family and phase must be one of the seven policy-table pairs. The outer
envelope is the existing exact successor envelope and remains subject to the Stage-B document
firewall.

`artifact_schema_version`, `sequence`, and `ordinal` are exact nonnegative JSON integer literals,
not strings or floating-point values. All timestamp fields are JSON integers in
0..=9,007,199,254,740,991. Digest and ID fields use their validated lexical newtypes; nullable
digest fields accept only JSON null or the exact digest string.

Intent requires sequence 1 and a null previous digest. Dispatch, when required, has sequence 2 and
points to the intent primary. Action has sequence 2 without dispatch or 3 with dispatch and points
to the immediately preceding primary. Terminal has sequence 3 without dispatch or 4 with dispatch
and points to the action primary. Its `intent_sha256`, optional `dispatch_sha256`, and
`action_sha256` bind those exact observed primary bytes. All artifacts must have identical
execution ID, plan digest, and phase. Every intent policy ID and digest must equal the derived
policy. Dispatch is present exactly when the policy requires a mutation journal. Action ordinal is
always 1; dispatch ordinal is always 1; on mutation phases their request digests are equal.

The chronology rules are exact: intent declared start is strictly less than intent declared
terminal; dispatch creation is at or after intent declared start and at or before action start;
action completion is at or after action start; and terminal completion is at or after action
completion. For a phase without dispatch, action start is at or after intent declared start.
Terminal outcome equals action outcome. `timing_claim=late` if and only if terminal completion is
strictly greater than intent declared terminal; `timing_claim=timely` if and only if terminal
completion is less than or equal to intent declared terminal. These wall-clock fields are audit
data and never reconstruct a monotonic deadline. No timestamp or directory metadata infers file
creation order.

## Conservative read-only classifier

The production classifier is macOS-only and performs only bounded reads. On every other target it
returns `UnsupportedPlatform` before attempting path or filesystem I/O. On macOS it may reuse only
the existing strict duplicate-rejecting JSON byte parser and macOS extended-ACL check after their
crate-private visibility is narrowly exposed. It must not use the existing path-based successor
loader or path-based owned-file reader: every entry is parsed from the exact bytes read through its
retained directory-relative descriptor. It performs no create, write, remove, rename, permission,
lock, repair, retry, resume, or cleanup operation.

The directory must be canonical, owner-owned, mode 0700, free of extended ACLs, and opened with
`O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC`. Entries are enumerated descriptor-relatively with a fresh open
file description and a cap of eight. Every primary and sidecar must be a regular owner-owned
single-link mode-0600 file opened descriptor-relatively with `O_NOFOLLOW|O_CLOEXEC`. Identity,
length, ACL, bytes, and SHA-256 are checked before and after each bounded read. A pathname or
directory identity change rejects.

The classifier signature returns `Result<C1OfflineState, C1ArtifactReadError>`. The only state
values are:

```text
Pristine
IntentOnlyFrozen
MutationDispatchOutcomeUnknown
IncompleteFrozenAfterAction
TerminalPairTimingUnproven
TerminalPairClaimsLate
```

The stable categorical read-error values are:

```text
UnsupportedPlatform
Unreadable
UnsafeDirectory
UnsafeEntry
IdentityChanged
InvalidResidue
```

No error contains or wraps a raw operating-system error, path, file name, or input bytes. Failure
to open, enumerate, or read is `Unreadable`; directory ownership, mode, type, link, or ACL failure
is `UnsafeDirectory`; entry ownership, mode, type, link, or ACL failure is `UnsafeEntry`; a
retained directory or entry identity, length, or timestamp change is `IdentityChanged`. After all
bytes have been read safely, every name-set, pair, digest, JSON, payload, policy, chain, or
chronology failure is the `InvalidResidue` error. `InvalidResidue` is never returned as a
successful state.

An empty valid directory is `Pristine`, but that label confers no permission or replay authority.
A complete valid intent-only prefix is `IntentOnlyFrozen`. A valid required dispatch without action
is `MutationDispatchOutcomeUnknown`. A valid action without terminal is
`IncompleteFrozenAfterAction`. A complete valid terminal chain is timing-unproven unless it claims
late. Partial pairs, non-prefix sets, forbidden dispatch, chain mismatch, a second/extra entry,
malformed envelope, wrong family/kind/phase/sequence, or digest mismatch return the
`InvalidResidue` error. Unsafe files and replacements return their categorical read errors.

Every result is inert and non-authorizing. No result mints a seal or claims live execution,
timeliness, provider outcome, mutation attribution, semantic completeness, or safe replay. Every
nonempty state remains no-replay evidence for any later boundary.

## Synthetic in-memory JSON-RPC automaton

The automaton is synchronous, single-owner, and explicitly `!Send + !Sync` through an `Rc` marker.
It has no `Clone`, Debug, Serde, or production constructor. Only `#[cfg(test)]` fixtures can bind a
synthetic program. It contains no URL, transport, header, token, client, listener, socket, callback,
executor, future, channel, thread, process, retry, reconnect, or forwarding code.

It accepts at most 65,536 injected bytes per message and requires exactly one JSON value that is an
object with `jsonrpc="2.0"`. Input object order and insignificant whitespace do not matter. It
rejects recursive duplicate keys, arrays/batches, trailing non-whitespace bytes, and `_meta` at any
depth. Every recognized object has exactly the fields shown below; every nested object also has
exactly the shown fields.

IDs are either JSON integers 1 through 9,007,199,254,740,991 or JSON strings whose decoded contents
satisfy `C1RequestStringId`. Boolean, null, floating-point, empty, control-bearing, out-of-range,
and repeated semantic IDs reject. A local response echoes the same integer-or-string value using
its canonical compact JSON encoding; allowed string contents therefore serialize without escapes.

The exact control sequence is:

```text
initialize request
initialized notification
tools/list request
zero or more tools/call requests required by the bound synthetic program
finish()
```

`finish()` is an internal zero-input Rust method; it is never an injected JSON-RPC message. It
returns `SequenceAccepted` only after the exact program is complete and otherwise rejects with
`incomplete_program` and poisons the instance. Successful finish seals the instance; no further
input method is available from the accepted value.

The accepted request structures are exactly these compact fixtures, where `<id>` is one allowed ID
token and `<tool>` is the exact next tool required by the bound program:

```json
{"jsonrpc":"2.0","id":<id>,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"engram-c1-synthetic","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":<id>,"method":"tools/list"}
{"jsonrpc":"2.0","id":<id>,"method":"tools/call","params":{"name":"<tool>","arguments":{"nonce":"fixture"}}}
```

The only tools are `c1_synthetic_read` and `c1_synthetic_mutation`. `tools/list` always exposes
both, in that order, regardless of the bound program. Their exact definitions are:

```json
{"name":"c1_synthetic_read","description":"Synthetic read fixture; performs no operation.","inputSchema":{"type":"object","properties":{"nonce":{"type":"string","const":"fixture"}},"required":["nonce"],"additionalProperties":false}}
{"name":"c1_synthetic_mutation","description":"Synthetic mutation fixture; performs no operation.","inputSchema":{"type":"object","properties":{"nonce":{"type":"string","const":"fixture"}},"required":["nonce"],"additionalProperties":false}}
```

For an accepted initialize or tools-list request, `LocalResponse` contains exactly one of the
following compact UTF-8 objects with `<id>` replaced by the canonical encoding of the request ID
value and with no trailing newline:

```json
{"jsonrpc":"2.0","id":<id>,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"engram-c1-synthetic","version":"1"}}}
{"jsonrpc":"2.0","id":<id>,"result":{"tools":[{"name":"c1_synthetic_read","description":"Synthetic read fixture; performs no operation.","inputSchema":{"type":"object","properties":{"nonce":{"type":"string","const":"fixture"}},"required":["nonce"],"additionalProperties":false}},{"name":"c1_synthetic_mutation","description":"Synthetic mutation fixture; performs no operation.","inputSchema":{"type":"object","properties":{"nonce":{"type":"string","const":"fixture"}},"required":["nonce"],"additionalProperties":false}}]}}
```

Initialized returns `NotificationAccepted`. An accepted tools/call returns only
`SyntheticCallAccepted`; it emits no response bytes and performs no operation. The synthetic
program is exactly one of `none`, `single_read`, `single_mutation`, or `mutation_then_read`, and its
call order and cardinality are exact.

Decision variants are only:

```text
LocalResponse { bytes }
NotificationAccepted
SyntheticCallAccepted { request_id, ordinal, tool, arguments_sha256 }
SequenceAccepted { ledger_sha256 }
Rejected { code }
```

`ordinal` is the one-based call ordinal in the selected synthetic program. `tool` is a closed
private two-variant enum, `request_id` is a closed private integer-or-string enum, and both digest
fields are `C1Sha256`. No decision carries injected input bytes or a generic JSON value.

`SyntheticCallAccepted` is inert syntax evidence, not an upstream request. `SequenceAccepted`
proves only that injected downstream bytes followed the synthetic grammar; it does not prove a call
was executed, forwarded, answered, successful, or authorized.

Every rejection permanently poisons the instance. Stable rejection codes are:

```text
already_poisoned
oversized
malformed_json
duplicate_key
trailing_data
batch_forbidden
meta_forbidden
invalid_jsonrpc
invalid_id
reused_id
unexpected_message
extra_field
contract_mismatch
incomplete_program
```

For an input with multiple violations, rejection precedence is exact: an already-poisoned instance;
oversized input; then the first left-to-right strict parse failure (`duplicate_key` for a duplicate
at that point, otherwise `malformed_json`); trailing data after one valid first value;
`batch_forbidden`; `_meta` at any depth; missing or non-exact `jsonrpc`; a field outside the exact
top-level or nested field set; invalid or missing/present-in-the-wrong-kind ID; reused ID; message
kind or order not accepted by the current state; then any remaining exact-value, nested-shape,
tool-name, argument, order, or cardinality mismatch. These map respectively to the listed stable
codes, with top-level non-object and wrong message/order mapping to `unexpected_message`, and the
last group mapping to `contract_mismatch`. `incomplete_program` arises only from `finish()`.

After exact `jsonrpc` validation, an unknown or absent method is immediately
`unexpected_message`. For one of the four recognized methods, allowed fields and ID rules are
checked against that method's fixture before current-state order is checked. Thus a recognized but
out-of-order request can still reject as `extra_field`, `invalid_id`, or `reused_id` first; nested
value validation waits until after the order check and maps to `contract_mismatch`.

The canonical arguments digest is exactly:

```text
SHA-256(ASCII "engram-c1-synthetic-arguments-v1" || NUL ||
        UTF-8 "{\"nonce\":\"fixture\"}")
= 75d3234c0ad7a748c1fc9cb107bd1714a9e66a239c77ec82e50ec2de044e1e20
```

The accepted-sequence ledger is exactly:

```text
ASCII "engram-c1-synthetic-relay-v1" followed by NUL
program spelling as unsigned-64-bit-big-endian length, then ASCII bytes
record count as unsigned 64-bit big-endian
for each accepted record in observation order:
  one record tag byte: initialize=0, initialized=1, tools/list=2,
                       synthetic-read=3, synthetic-mutation=4
  one ID tag byte: absent=0, integer=1, string=2
  if integer: unsigned 64-bit big-endian value
  if string: unsigned-64-bit-big-endian length, then literal ASCII bytes
  method as unsigned-64-bit-big-endian length, then ASCII bytes
  tool name as unsigned-64-bit-big-endian length, then ASCII bytes; zero length if absent
  one argument-digest tag byte: absent=0, present=1
  if present: 32 raw canonical-arguments digest bytes
```

The normative ledger golden vector binds program `mutation_then_read` and records: initialize with
integer ID 1; initialized with no ID; tools/list with string ID `list-1`; synthetic mutation with
integer ID 2; and synthetic read with string ID `read-1`. Its framed byte length is 369 and its
SHA-256 is `cbbfc4a6780d0d0866b815d4626cff06b20f7e15cfd820f05b710db8d7f689a0`.

## Required C1 tests

- every lexical newtype boundary and invalid normalization attempt;
- exact phase-policy golden bytes/digest and every family/phase substitution;
- static-template golden digest, environment sorting, duplicate/reserved-key failure, literal/slot
  domain separation, and proof that no runtime value is accepted;
- every valid artifact prefix, partial pair, non-prefix set, forbidden dispatch, cross-binding,
  sequence/hash-chain/timing failure, timely/late terminal claim, extra file, symlink, hardlink,
  permissions, ACL, pathname/directory replacement, and classification result;
- every automaton transition, exact initialize and schema golden, all rejection codes, numeric and
  string ID boundaries/reuse, recursive duplicate, batch, `_meta`, trailing data, extra field,
  alias/case/hyphen, order, cardinality, poisoning, incomplete finish, and accepted-sequence digest;
- compile/source firewalls proving private and reverse isolation, `!Send + !Sync`, and absence of
  forbidden dependencies/APIs;
- a secret-marker scan over every serialized fixture/result;
- full provider-free tests, strict Clippy, format, and whitespace checks using an external
  `CARGO_TARGET_DIR`; and
- repository `target/debug` absent before and after validation.

## Deferred gates

C2 must separately choose and freeze exact successor plans and host bindings, mutable Engram-home
state, descriptor-relative partition creation, embedding cache/offline startup, inherited
subprocess environment scrubbing, activation, exact reachable semantic coverage, observer writes
and time authority, post-intent secret materialization, artifact writers, daemon ownership, and
terminal hygiene.

A later transport slice must freeze real phase calls and minimized results, listener/authentication,
HTTP framing, MCP session lifecycle, nested success/error semantics, and host-specific non-durable
capability delivery. No same-UID provider evidence may claim relay enforcement while daemon secrets
can reach Git or other inherited subprocesses. Strong evidence additionally requires a mount-free
VM, different UID, namespace/cgroup-equivalent teardown, or another independently proven boundary.
